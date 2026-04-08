// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//! LLM-driven memory consolidation.
//!
//! After each conversation turn, extracts structured information:
//! - `history_entry`: A timestamped summary for the daily conversation log.
//! - `memory_update`: New facts, preferences, or decisions worth remembering
//!   long-term (or `null` if nothing new was learned).
//! - `facts`: Atomic facts with categories and confidence scores (DeerFlow-style).
//!
//! This two-phase approach replaces the naive raw-message auto-save with
//! semantic extraction, similar to Nanobot's `save_memory` tool call pattern.

use crate::memory::conflict;
use crate::memory::importance;
use crate::memory::traits::{Memory, MemoryCategory};
use crate::providers::traits::Provider;
use parking_lot::Mutex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configuration for the memory consolidation system.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConsolidationConfig {
    /// Enable consolidation after each turn. Default: true.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Extract atomic facts with categories and confidence scores.
    /// When enabled, consolidation produces structured facts instead of (or in
    /// addition to) raw memory_update text. Default: true.
    #[serde(default = "default_fact_extraction")]
    pub fact_extraction_enabled: bool,

    /// Maximum facts to extract per turn. Default: 10.
    #[serde(default = "default_max_facts")]
    pub max_facts_per_turn: usize,

    /// Minimum confidence score (0.0-1.0) for a fact to be stored.
    /// Facts below this threshold are discarded. Default: 0.7.
    #[serde(default = "default_confidence_threshold")]
    pub fact_confidence_threshold: f64,

    /// Maximum facts to store in Core memory (oldest below threshold are purged).
    /// Default: 100.
    #[serde(default = "default_max_facts")]
    pub max_core_facts: usize,

    /// Debounce window in seconds — consolidation is delayed by this duration
    /// to batch rapid successive turns. Set to 0 to disable debouncing. Default: 30.
    #[serde(default = "default_debounce_secs")]
    pub debounce_secs: u64,
}

fn default_enabled() -> bool {
    true
}
fn default_fact_extraction() -> bool {
    true
}
fn default_max_facts() -> usize {
    100
}
fn default_confidence_threshold() -> f64 {
    0.7
}
fn default_debounce_secs() -> u64 {
    30
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            fact_extraction_enabled: default_fact_extraction(),
            max_facts_per_turn: default_max_facts(),
            fact_confidence_threshold: default_confidence_threshold(),
            max_core_facts: default_max_facts(),
            debounce_secs: default_debounce_secs(),
        }
    }
}

/// Output of consolidation extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    /// Brief timestamped summary for the conversation history log.
    pub history_entry: String,
    /// New facts/preferences/decisions to store long-term, or None.
    pub memory_update: Option<String>,
    /// Atomic facts extracted from the turn (when fact_extraction_enabled).
    #[serde(default)]
    pub facts: Vec<ExtractedFact>,
    /// Observed trend or pattern (when fact_extraction_enabled).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trend: Option<String>,
}

/// A single atomic fact extracted from a conversation turn.
/// Matches DeerFlow's fact model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFact {
    /// Unique stable ID (SHA-256 of content, truncated).
    pub id: String,
    /// The fact content.
    pub content: String,
    /// Category: context, preference, behavior, correction.
    pub category: FactCategory,
    /// Confidence score (0.0-1.0) assigned by the LLM.
    pub confidence: f64,
}

impl ExtractedFact {
    /// Compute a stable ID from content.
    pub fn compute_id(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        content.hash(&mut h);
        format!("fact_{:x}", h.finish())
    }
}

/// Category of an extracted fact. Matches DeerFlow's taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FactCategory {
    /// General factual information about the user or their work.
    Context,
    /// User preference or stated liking/disliking.
    Preference,
    /// Observed behavioral pattern.
    Behavior,
    /// Correction of a previously incorrect assumption.
    Correction,
}

const CONSOLIDATION_SYSTEM_PROMPT: &str = r#"You are a memory consolidation engine. Given a conversation turn, extract:
1. "history_entry": A brief summary of what happened in this turn (1-2 sentences). Include the key topic or action.
2. "memory_update": Any NEW facts, preferences, decisions, or commitments worth remembering long-term. Return null if nothing new was learned.

Respond ONLY with valid JSON: {"history_entry": "...", "memory_update": "..." or null}
Do not include any text outside the JSON object."#;

/// Enhanced fact-extraction prompt — produces atomic facts with categories and scores.
const FACT_EXTRACTION_SYSTEM_PROMPT: &str = r#"You are a memory fact extraction engine. Given a conversation turn, extract atomic facts about the user.

For each fact you identify, assign:
- A category: "context" (general info), "preference" (stated likes/dislikes), "behavior" (observed patterns), "correction" (corrected errors)
- A confidence score from 0.0 to 1.0 (how certain are you this fact is correct and worth remembering)

Rules:
- Each fact must be a single, self-contained statement.
- Discard facts below 0.6 confidence.
- Extract at most 10 facts per turn.
- Ignore trivial statements like greetings or small talk.
- Focus on user identity, preferences, work context, and behavioral patterns.

Respond ONLY with valid JSON:
{"facts": [{"content": "...", "category": "context|preference|behavior|correction", "confidence": 0.0-1.0}], "trend": "optional observed trend or null"}

Respond ONLY with valid JSON. No markdown, no explanation."#;

const CONSOLIDATION_USER_PROMPT_TEMPLATE: &str =
    "## Conversation Turn\nUser: {user}\nAssistant: {assistant}";

const FACT_EXTRACTION_USER_PROMPT_TEMPLATE: &str =
    "## Conversation Turn (analyze for facts)\nUser: {user}\nAssistant: {assistant}\n\nOnly respond with the JSON object.";

/// Run two-phase LLM-driven consolidation on a conversation turn.
///
/// Phase 1: Write a history entry to the Daily memory category.
/// Phase 2: Write a memory update to the Core category (if the LLM identified new facts).
///
/// This function is designed to be called fire-and-forget via `tokio::spawn`.
/// Strip channel media markers (e.g. `[IMAGE:/local/path]`, `[DOCUMENT:...]`)
/// that contain local filesystem paths.  These must never be forwarded to
/// upstream provider APIs — they would leak local paths and cause API errors.
fn strip_media_markers(text: &str) -> String {
    // Matches [IMAGE:...], [DOCUMENT:...], [FILE:...], [VIDEO:...], [VOICE:...], [AUDIO:...]
    static RE: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\[(?:IMAGE|DOCUMENT|FILE|VIDEO|VOICE|AUDIO):[^\]]*\]").unwrap()
    });
    RE.replace_all(text, "[media attachment]").into_owned()
}

pub async fn consolidate_turn(
    provider: &dyn Provider,
    model: &str,
    memory: &dyn Memory,
    user_message: &str,
    assistant_response: &str,
) -> anyhow::Result<()> {
    let config = ConsolidationConfig::default();
    consolidate_turn_with_config(provider, model, memory, user_message, assistant_response, &config)
        .await
}

/// Same as [`consolidate_turn`] but with explicit config.
pub async fn consolidate_turn_with_config(
    provider: &dyn Provider,
    model: &str,
    memory: &dyn Memory,
    user_message: &str,
    assistant_response: &str,
    config: &ConsolidationConfig,
) -> anyhow::Result<()> {
    if !config.enabled {
        return Ok(());
    }

    let turn_text = format!(
        "User: {}\nAssistant: {}",
        strip_media_markers(user_message),
        strip_media_markers(assistant_response),
    );

    // Truncate very long turns to avoid wasting tokens on consolidation.
    // Use char-boundary-safe slicing to prevent panic on multi-byte UTF-8 (e.g. CJK text).
    let truncated = if turn_text.len() > 4000 {
        let end = turn_text
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i <= 4000)
            .last()
            .unwrap_or(0);
        format!("{}…", &turn_text[..end])
    } else {
        turn_text.clone()
    };

    // Phase 1: Basic consolidation (history_entry + memory_update)
    let raw = provider
        .chat_with_system(Some(CONSOLIDATION_SYSTEM_PROMPT), &truncated, model, 0.1)
        .await?;

    let result: ConsolidationResult = parse_consolidation_response(&raw, &turn_text);

    // Phase 2 (optional): Fact extraction
    let facts = if config.fact_extraction_enabled {
        extract_facts_with_config(provider, model, &truncated, config).await
    } else {
        Vec::new()
    };

    // Phase 3: Write history entry to Daily category.
    let date = chrono::Local::now().format("%Y-%m-%d").to_string();
    let history_key = format!("daily_{date}_{}", uuid::Uuid::new_v4());
    memory
        .store(
            &history_key,
            &result.history_entry,
            MemoryCategory::Daily,
            None,
        )
        .await?;

    // Phase 4: Write memory update to Core category (if present).
    if let Some(ref update) = result.memory_update {
        if !update.trim().is_empty() {
            let mem_key = format!("core_{}", uuid::Uuid::new_v4());

            // Compute importance score heuristically.
            let imp = importance::compute_importance(update, &MemoryCategory::Core);

            // Check for conflicts with existing Core memories.
            match conflict::check_and_resolve_conflicts(
                memory,
                &mem_key,
                update,
                &MemoryCategory::Core,
                0.85,
            )
            .await
            {
                Ok(superseded_ids) if !superseded_ids.is_empty() => {
                    tracing::debug!(
                        "conflict resolution superseded {} existing entries",
                        superseded_ids.len()
                    );
                }
                Ok(_) => {}
                Err(e) => {
                    tracing::debug!("conflict check skipped: {e}");
                }
            }

            // Store with importance metadata.
            memory
                .store_with_metadata(
                    &mem_key,
                    update,
                    MemoryCategory::Core,
                    None,
                    None,
                    Some(imp),
                )
                .await?;
        }
    }

    // Phase 5: Store extracted facts
    for fact in &facts {
        if fact.confidence < config.fact_confidence_threshold {
            continue;
        }
        store_fact(memory, fact, config).await?;
    }

    // Phase 6: Enforce max_core_facts cap
    if let Err(e) = enforce_fact_cap(memory, config.max_core_facts).await {
        tracing::warn!("failed to enforce fact cap: {e}");
    }

    Ok(())
}

/// Extract structured facts from a conversation turn using the LLM.
async fn extract_facts_with_config(
    provider: &dyn Provider,
    model: &str,
    turn_text: &str,
    config: &ConsolidationConfig,
) -> Vec<ExtractedFact> {
    // Parse user and assistant parts
    let (user_part, assistant_part) = if let Some(idx) = turn_text.find("\nAssistant: ") {
        let u = &turn_text[..idx];
        let a = &turn_text[idx + "\nAssistant: ".len()..];
        (u.trim(), a.trim())
    } else {
        (turn_text.trim(), "")
    };

    let user_prompt = FACT_EXTRACTION_USER_PROMPT_TEMPLATE
        .replace("{user}", user_part)
        .replace("{assistant}", assistant_part);

    let raw = match provider
        .chat_with_system(Some(FACT_EXTRACTION_SYSTEM_PROMPT), &user_prompt, model, 0.2)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("fact extraction LLM call failed: {e}");
            return Vec::new();
        }
    };

    // Parse JSON facts
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    #[derive(Debug, Deserialize)]
    struct RawFactsResponse {
        #[serde(default)]
        facts: Vec<RawFact>,
        #[serde(default)]
        trend: Option<String>,
    }

    #[derive(Debug, Deserialize)]
    struct RawFact {
        content: String,
        #[serde(default)]
        category: Option<String>,
        #[serde(default)]
        confidence: Option<f64>,
    }

    let response: RawFactsResponse = match serde_json::from_str(cleaned) {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!(error = %e, "failed to parse fact extraction JSON");
            return Vec::new();
        }
    };

    response
        .facts
        .into_iter()
        .take(config.max_facts_per_turn)
        .map(|f| {
            let category = match f.category.as_deref() {
                Some("context") => FactCategory::Context,
                Some("preference") => FactCategory::Preference,
                Some("behavior") => FactCategory::Behavior,
                Some("correction") => FactCategory::Correction,
                _ => FactCategory::Context,
            };
            let confidence = f.confidence.unwrap_or(0.7).clamp(0.0, 1.0);
            ExtractedFact {
                id: ExtractedFact::compute_id(&f.content),
                content: f.content,
                category,
                confidence,
            }
        })
        .collect()
}

/// Store a single fact to memory, checking for duplicates.
async fn store_fact(memory: &dyn Memory, fact: &ExtractedFact, _config: &ConsolidationConfig) -> anyhow::Result<()> {
    // Check for near-duplicate by checking existing Core memories.
    // This is a simple heuristic: skip if content substring match exists.
    let existing = memory
        .recall(&fact.content, 3, None, None, None)
        .await
        .unwrap_or_default();

    for entry in existing {
        let similarity = compute_text_similarity(&fact.content, &entry.content);
        if similarity > 0.85 {
            tracing::debug!(fact_id = %fact.id, existing_key = %entry.key, similarity = similarity, "fact skipped — near-duplicate found");
            return Ok(());
        }
    }

    let key = format!("fact_{}", fact.id);
    let content = format!(
        "[{}|{:.1}] {}",
        fact.category_to_string(),
        fact.confidence,
        fact.content
    );
    let imp = fact.confidence * importance::compute_importance(&fact.content, &MemoryCategory::Core);

    memory
        .store_with_metadata(
            &key,
            &content,
            MemoryCategory::Core,
            None,
            None,
            Some(imp),
        )
        .await?;

    tracing::debug!(
        fact_id = %fact.id,
        category = ?fact.category,
        confidence = fact.confidence,
        "extracted fact stored"
    );

    Ok(())
}

impl ExtractedFact {
    fn category_to_string(&self) -> &'static str {
        match self.category {
            FactCategory::Context => "context",
            FactCategory::Preference => "preference",
            FactCategory::Behavior => "behavior",
            FactCategory::Correction => "correction",
        }
    }
}

/// Simple word-overlap similarity (0.0-1.0).
fn compute_text_similarity(a: &str, b: &str) -> f64 {
    let a_words: HashSet<String> = a
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| !w.is_empty() && w.len() > 2)
        .collect();
    let b_words: HashSet<String> = b
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
        .filter(|w| !w.is_empty() && w.len() > 2)
        .collect();

    if a_words.is_empty() && b_words.is_empty() {
        return 1.0;
    }
    if a_words.is_empty() || b_words.is_empty() {
        return 0.0;
    }

    let intersection: usize = a_words.intersection(&b_words).count();
    let union = a_words.union(&b_words).count();
    intersection as f64 / union as f64
}

/// Enforce the max_core_facts cap by removing lowest-importance facts.
async fn enforce_fact_cap(memory: &dyn Memory, max_facts: usize) -> anyhow::Result<()> {
    let all_entries = memory.list(Some(&MemoryCategory::Core), None).await?;

    // Count existing facts
    let fact_entries: Vec<_> = all_entries
        .iter()
        .filter(|e| e.key.starts_with("fact_"))
        .collect();

    if fact_entries.len() <= max_facts {
        return Ok(());
    }

    // Sort by importance descending, keep top max_facts
    let mut sorted: Vec<_> = fact_entries
        .iter()
        .map(|e| (e.key.clone(), e.importance.unwrap_or(0.0)))
        .collect();
    sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    let to_remove: Vec<String> = sorted
        .into_iter()
        .skip(max_facts)
        .map(|(k, _)| k)
        .collect();

    for key in &to_remove {
        let _ = memory.forget(key).await;
        tracing::debug!(key = %key, "fact evicted due to max_core_facts cap");
    }

    Ok(())
}

// ── Debounced consolidation queue ──────────────────────────────────────

/// Per-session consolidation queue with debouncing.
/// Multiple turns within the debounce window are batched into a single consolidation.
pub struct ConsolidationQueue {
    config: ConsolidationConfig,
    pending: Arc<Mutex<PendingQueue>>,
    runtime: tokio::runtime::Handle,
}

struct PendingQueue {
    turns: Vec<(String, String)>, // (user, assistant)
    timer: Option<tokio::task::JoinHandle<()>>,
    scheduled_at: Option<Instant>,
}

impl ConsolidationQueue {
    /// Create a new queue backed by the given runtime.
    pub fn new(config: ConsolidationConfig) -> Self {
        Self {
            config,
            pending: Arc::new(Mutex::new(PendingQueue {
                turns: Vec::new(),
                timer: None,
                scheduled_at: None,
            })),
            runtime: tokio::runtime::Handle::current(),
        }
    }

    /// Enqueue a turn for debounced consolidation.
    /// If `debounce_secs == 0`, consolidation runs immediately.
    pub fn enqueue(
        self: Arc<Self>,
        user_message: String,
        assistant_response: String,
        provider: Box<dyn Provider>,
        model: String,
        memory: Arc<dyn Memory>,
    ) {
        let debounce = Duration::from_secs(self.config.debounce_secs);

        let mut pending = self.pending.lock();

        if debounce.is_zero() {
            // Immediate consolidation — spawn and return
            let cfg = self.config.clone();
            let u = user_message;
            let a = assistant_response;
            drop(pending);
            self.runtime.spawn(async move {
                let _ = consolidate_turn_with_config(
                    provider.as_ref(),
                    &model,
                    memory.as_ref(),
                    &u,
                    &a,
                    &cfg,
                )
                .await;
            });
            return;
        }

        // Add turn to batch
        pending.turns.push((user_message, assistant_response));

        // If already scheduled, just extend the batch
        if pending.timer.is_some() {
            return;
        }

        // Schedule consolidation after debounce window
        let pending_ref = Arc::clone(&self.pending);
        let cfg = self.config.clone();
        let provider_ref = provider;
        let model_ref = model;
        let memory_ref = memory;

        let handle = self.runtime.spawn(async move {
            tokio::time::sleep(debounce).await;

            let turns = {
                let mut p = pending_ref.lock();
                p.timer = None;
                p.scheduled_at = None;
                std::mem::take(&mut p.turns)
            };

            if turns.is_empty() {
                return;
            }

            // Consolidate each turn
            for (user, assistant) in turns {
                let _ = consolidate_turn_with_config(
                    provider_ref.as_ref(),
                    &model_ref,
                    memory_ref.as_ref(),
                    &user,
                    &assistant,
                    &cfg,
                )
                .await;
            }
        });

        pending.timer = Some(handle);
        pending.scheduled_at = Some(Instant::now());
    }

    /// Drain pending turns immediately without waiting for debounce.
    /// Useful for shutdown.
    pub fn flush(&self) {
        let turns = {
            let mut p = self.pending.lock();
            p.timer = None;
            p.scheduled_at = None;
            std::mem::take(&mut p.turns)
        };
        if !turns.is_empty() {
            tracing::debug!(pending = turns.len(), "consolidation queue drained");
        }
    }
}

/// Parse the LLM's consolidation response, with fallback for malformed JSON.
fn parse_consolidation_response(raw: &str, fallback_text: &str) -> ConsolidationResult {
    // Try to extract JSON from the response (LLM may wrap in markdown code blocks).
    let cleaned = raw
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    serde_json::from_str(cleaned).unwrap_or_else(|_| {
        // Fallback: use truncated turn text as history entry.
        // Use char-boundary-safe slicing to prevent panic on multi-byte UTF-8.
        let summary = if fallback_text.len() > 200 {
            let end = fallback_text
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|&i| i <= 200)
                .last()
                .unwrap_or(0);
            format!("{}…", &fallback_text[..end])
        } else {
            fallback_text.to_string()
        };
        ConsolidationResult {
            history_entry: summary,
            memory_update: None,
            facts: Vec::new(),
            trend: None,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_json_response() {
        let raw = r#"{"history_entry": "User asked about Rust.", "memory_update": "User prefers Rust over Go."}"#;
        let result = parse_consolidation_response(raw, "fallback");
        assert_eq!(result.history_entry, "User asked about Rust.");
        assert_eq!(
            result.memory_update.as_deref(),
            Some("User prefers Rust over Go.")
        );
    }

    #[test]
    fn parse_json_with_null_memory() {
        let raw = r#"{"history_entry": "Routine greeting.", "memory_update": null}"#;
        let result = parse_consolidation_response(raw, "fallback");
        assert_eq!(result.history_entry, "Routine greeting.");
        assert!(result.memory_update.is_none());
    }

    #[test]
    fn parse_json_wrapped_in_code_block() {
        let raw =
            "```json\n{\"history_entry\": \"Discussed deployment.\", \"memory_update\": null}\n```";
        let result = parse_consolidation_response(raw, "fallback");
        assert_eq!(result.history_entry, "Discussed deployment.");
    }

    #[test]
    fn fallback_on_malformed_response() {
        let raw = "I'm sorry, I can't do that.";
        let result = parse_consolidation_response(raw, "User: hello\nAssistant: hi");
        assert_eq!(result.history_entry, "User: hello\nAssistant: hi");
        assert!(result.memory_update.is_none());
    }

    #[test]
    fn fallback_truncates_long_text() {
        let long_text = "x".repeat(500);
        let result = parse_consolidation_response("invalid", &long_text);
        // 200 bytes + "…" (3 bytes in UTF-8) = 203
        assert!(result.history_entry.len() <= 203);
    }

    #[test]
    fn fallback_truncates_cjk_text_without_panic() {
        // Each CJK character is 3 bytes in UTF-8; byte index 200 may land
        // inside a character. This must not panic.
        let cjk_text = "二手书项目".repeat(50); // 250 chars = 750 bytes
        let result = parse_consolidation_response("invalid", &cjk_text);
        assert!(
            result
                .history_entry
                .is_char_boundary(result.history_entry.len())
        );
        assert!(result.history_entry.ends_with('…'));
    }

    #[test]
    fn fact_id_is_deterministic() {
        let id1 = ExtractedFact::compute_id("User prefers Rust");
        let id2 = ExtractedFact::compute_id("User prefers Rust");
        assert_eq!(id1, id2);
    }

    #[test]
    fn fact_id_differs_for_different_content() {
        let id1 = ExtractedFact::compute_id("User prefers Rust");
        let id2 = ExtractedFact::compute_id("User prefers Go");
        assert_ne!(id1, id2);
    }

    #[test]
    fn similarity_exact_match() {
        let sim = compute_text_similarity("hello world", "hello world");
        assert!((sim - 1.0).abs() < 0.001);
    }

    #[test]
    fn similarity_no_overlap() {
        let sim = compute_text_similarity("cat dog bird", "zebra lion elephant");
        assert!((sim - 0.0).abs() < 0.001);
    }

    #[test]
    fn similarity_partial_overlap() {
        let sim = compute_text_similarity("User prefers Rust programming", "User prefers Rust over Go");
        assert!(sim > 0.3);
        assert!(sim < 1.0);
    }

    #[test]
    fn similarity_ignores_short_words() {
        // Words <= 2 chars are filtered
        let sim = compute_text_similarity("I am great", "I am bad");
        assert!((sim - 0.0).abs() < 0.001);
    }

    #[test]
    fn config_defaults() {
        let cfg = ConsolidationConfig::default();
        assert!(cfg.enabled);
        assert!(cfg.fact_extraction_enabled);
        assert_eq!(cfg.max_core_facts, 100);
        assert!((cfg.fact_confidence_threshold - 0.7).abs() < 0.001);
        assert_eq!(cfg.debounce_secs, 30);
    }

    #[test]
    fn fact_category_serialization() {
        let fact = ExtractedFact {
            id: "test".into(),
            content: "User likes coffee".into(),
            category: FactCategory::Preference,
            confidence: 0.9,
        };
        let json = serde_json::to_string(&fact).unwrap();
        assert!(json.contains("preference"));
        assert!(json.contains("0.9"));
    }

    #[test]
    fn consolidation_result_serialization() {
        let result = ConsolidationResult {
            history_entry: "Discussed project".into(),
            memory_update: Some("User works on Rust".into()),
            facts: vec![ExtractedFact {
                id: "f1".into(),
                content: "User works on Rust".into(),
                category: FactCategory::Context,
                confidence: 0.8,
            }],
            trend: None,
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("history_entry"));
        assert!(json.contains("facts"));
        assert!(json.contains("context"));
    }
}

