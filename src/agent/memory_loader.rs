// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//! Memory context loader with token-budgeted injection.
//!
//! Implements the [`MemoryLoader`] trait, retrieving relevant memories and formatting
//! them for injection into the system prompt. Memory entries are filtered by
//! relevance score and limited by both entry count and total token budget,
//! matching DeerFlow's `MemoryMiddleware` token capping.

use crate::memory::{self, Memory, decay};
use async_trait::async_trait;

/// Default maximum tokens for memory context injection (≈2000 tokens ≈ 8000 chars).
/// This is the per-turn budget for retrieved memories in the system prompt.
pub const DEFAULT_MAX_INJECTION_TOKENS: usize = 2000;

/// Rough chars-per-token ratio used for estimation (≈4 chars/token).
const CHARS_PER_TOKEN: usize = 4;

#[async_trait]
pub trait MemoryLoader: Send + Sync {
    async fn load_context(
        &self,
        memory: &dyn Memory,
        user_message: &str,
        session_id: Option<&str>,
    ) -> anyhow::Result<String>;
}

pub struct DefaultMemoryLoader {
    limit: usize,
    min_relevance_score: f64,
    /// Maximum tokens for the injected memory context.
    /// When set, entries are truncated/removed to stay within budget.
    max_injection_tokens: usize,
}

impl Default for DefaultMemoryLoader {
    fn default() -> Self {
        Self {
            limit: 5,
            min_relevance_score: 0.4,
            max_injection_tokens: DEFAULT_MAX_INJECTION_TOKENS,
        }
    }
}

impl DefaultMemoryLoader {
    pub fn new(limit: usize, min_relevance_score: f64) -> Self {
        Self {
            limit: limit.max(1),
            min_relevance_score,
            max_injection_tokens: DEFAULT_MAX_INJECTION_TOKENS,
        }
    }

    /// Set the maximum injection token budget. Use `0` for unlimited.
    pub fn with_max_injection_tokens(mut self, tokens: usize) -> Self {
        self.max_injection_tokens = tokens;
        self
    }
}

#[async_trait]
impl MemoryLoader for DefaultMemoryLoader {
    async fn load_context(
        &self,
        memory: &dyn Memory,
        user_message: &str,
        session_id: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut entries = memory
            .recall(user_message, self.limit, session_id, None, None)
            .await?;
        if entries.is_empty() {
            return Ok(String::new());
        }

        // Apply time decay: older non-Core memories score lower
        decay::apply_time_decay(&mut entries, decay::DEFAULT_HALF_LIFE_DAYS);

        let mut context = String::from("[Memory context]\n");
        let budget_chars = self.max_injection_tokens * CHARS_PER_TOKEN;
        let header_footer_chars = "[Memory context]\n[/Memory context]\n".len();

        // Iterate in reverse order (least relevant first) so we can remove items
        // from the front without breaking iteration.
        let mut selected_entries = Vec::new();

        for entry in entries.into_iter().rev() {
            if memory::is_assistant_autosave_key(&entry.key) {
                continue;
            }
            if memory::should_skip_autosave_content(&entry.content) {
                continue;
            }
            if let Some(score) = entry.score {
                if score < self.min_relevance_score {
                    continue;
                }
            }

            let entry_line = format!("- {}: {}\n", entry.key, entry.content);
            let projected_len = context.len() + entry_line.len() + header_footer_chars;

            if self.max_injection_tokens > 0 && projected_len > budget_chars {
                // Would exceed budget. Try truncating the entry.
                let available = budget_chars
                    .saturating_sub(context.len() + header_footer_chars + 4); // +4 for ellipsis
                if available >= entry.key.len() + 8 {
                    // At least "key: ..." can fit
                    let truncated = if entry.content.len() > available - entry.key.len() - 6 {
                        format!("{}: {}…", entry.key, &entry.content[..available.saturating_sub(entry.key.len() + 6)])
                    } else {
                        format!("{}: {}", entry.key, entry.content)
                    };
                    selected_entries.push(truncated);
                    break; // Budget exhausted
                } else {
                    break; // Not enough space even for key
                }
            } else {
                selected_entries.push(entry_line);
            }
        }

        // Add entries in forward order (most relevant first)
        for entry_line in selected_entries.into_iter().rev() {
            context.push_str(&entry_line);
        }

        // If all entries were below threshold, return empty
        if context == "[Memory context]\n" {
            return Ok(String::new());
        }

        context.push_str("[/Memory context]\n\n");
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{Memory, MemoryCategory, MemoryEntry};
    use std::sync::Arc;

    struct MockMemory;
    struct MockMemoryWithEntries {
        entries: Arc<Vec<MemoryEntry>>,
    }

    #[async_trait]
    impl Memory for MockMemory {
        async fn store(
            &self,
            _key: &str,
            _content: &str,
            _category: MemoryCategory,
            _session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn recall(
            &self,
            _query: &str,
            limit: usize,
            _session_id: Option<&str>,
            _since: Option<&str>,
            _until: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            if limit == 0 {
                return Ok(vec![]);
            }
            Ok(vec![MemoryEntry {
                id: "1".into(),
                key: "k".into(),
                content: "v".into(),
                category: MemoryCategory::Conversation,
                timestamp: "now".into(),
                session_id: None,
                score: None,
                namespace: "default".into(),
                importance: None,
                superseded_by: None,
            }])
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(vec![])
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(true)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(0)
        }

        async fn health_check(&self) -> bool {
            true
        }

        fn name(&self) -> &str {
            "mock"
        }
    }

    #[async_trait]
    impl Memory for MockMemoryWithEntries {
        async fn store(
            &self,
            _key: &str,
            _content: &str,
            _category: MemoryCategory,
            _session_id: Option<&str>,
        ) -> anyhow::Result<()> {
            Ok(())
        }

        async fn recall(
            &self,
            _query: &str,
            _limit: usize,
            _session_id: Option<&str>,
            _since: Option<&str>,
            _until: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(self.entries.as_ref().clone())
        }

        async fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn list(
            &self,
            _category: Option<&MemoryCategory>,
            _session_id: Option<&str>,
        ) -> anyhow::Result<Vec<MemoryEntry>> {
            Ok(vec![])
        }

        async fn forget(&self, _key: &str) -> anyhow::Result<bool> {
            Ok(true)
        }

        async fn count(&self) -> anyhow::Result<usize> {
            Ok(self.entries.len())
        }

        async fn health_check(&self) -> bool {
            true
        }

        fn name(&self) -> &str {
            "mock-with-entries"
        }
    }

    #[tokio::test]
    async fn default_loader_formats_context() {
        let loader = DefaultMemoryLoader::default();
        let context = loader
            .load_context(&MockMemory, "hello", None)
            .await
            .unwrap();
        assert!(context.contains("[Memory context]"));
        assert!(context.contains("- k: v"));
    }

    #[tokio::test]
    async fn default_loader_skips_legacy_assistant_autosave_entries() {
        let loader = DefaultMemoryLoader::new(5, 0.0);
        let memory = MockMemoryWithEntries {
            entries: Arc::new(vec![
                MemoryEntry {
                    id: "1".into(),
                    key: "assistant_resp_legacy".into(),
                    content: "fabricated detail".into(),
                    category: MemoryCategory::Daily,
                    timestamp: "now".into(),
                    session_id: None,
                    score: Some(0.95),
                    namespace: "default".into(),
                    importance: None,
                    superseded_by: None,
                },
                MemoryEntry {
                    id: "2".into(),
                    key: "user_fact".into(),
                    content: "User prefers concise answers".into(),
                    category: MemoryCategory::Conversation,
                    timestamp: "now".into(),
                    session_id: None,
                    score: Some(0.9),
                    namespace: "default".into(),
                    importance: None,
                    superseded_by: None,
                },
            ]),
        };

        let context = loader
            .load_context(&memory, "answer style", None)
            .await
            .unwrap();
        assert!(context.contains("user_fact"));
        assert!(!context.contains("assistant_resp_legacy"));
        assert!(!context.contains("fabricated detail"));
    }

    #[tokio::test]
    async fn token_budget_truncates_large_context() {
        // Very small token budget: 10 tokens ≈ 40 chars
        let loader = DefaultMemoryLoader::new(10, 0.0)
            .with_max_injection_tokens(10);

        let big_content = "x".repeat(200);
        let memory = MockMemoryWithEntries {
            entries: Arc::new(vec![
                MemoryEntry {
                    id: "1".into(),
                    key: "big_fact".into(),
                    content: big_content.clone(),
                    category: MemoryCategory::Core,
                    timestamp: "now".into(),
                    session_id: None,
                    score: Some(0.9),
                    namespace: "default".into(),
                    importance: None,
                    superseded_by: None,
                },
            ]),
        };

        let context = loader
            .load_context(&memory, "test", None)
            .await
            .unwrap();

        // The content should be truncated within budget
        assert!(context.contains("big_fact"));
        assert!(context.len() < big_content.len() + 100,
            "context ({}) should be much smaller than original content ({})",
            context.len(),
            big_content.len()
        );
    }

    #[tokio::test]
    async fn unlimited_budget_keeps_all_entries() {
        // Unlimited budget (0 tokens = no limit)
        let loader = DefaultMemoryLoader::new(10, 0.0)
            .with_max_injection_tokens(0);

        let entries: Vec<MemoryEntry> = (0..5)
            .map(|i| MemoryEntry {
                id: format!("{}", i),
                key: format!("fact_{}", i),
                content: format!("content {}", i),
                category: MemoryCategory::Core,
                timestamp: "now".into(),
                session_id: None,
                score: Some(0.9),
                namespace: "default".into(),
                importance: None,
                superseded_by: None,
            })
            .collect();

        let memory = MockMemoryWithEntries {
            entries: Arc::new(entries),
        };

        let context = loader
            .load_context(&memory, "test", None)
            .await
            .unwrap();

        // All 5 entries should be present
        for i in 0..5 {
            assert!(context.contains(&format!("fact_{}", i)), "fact_{} should be present", i);
        }
    }
}
