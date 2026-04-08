// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//! Suggestions Engine - contextual next-action suggestions for agent sessions.
//!
//! Analyzes conversation history and available tools to suggest
//! relevant follow-up actions the user might want to take.
//!
//! Supports two modes:
//! - **Rule-based**: Fast pattern-matching on the latest turn.
//! - **LLM-driven**: Uses the model to generate richer, context-aware suggestions
//!   from full conversation history ( DeerFlow-inspired).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Suggestions configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SuggestionsConfig {
    /// Enable suggestions generation. Default: true.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Maximum number of suggestions to generate. Default: 4.
    #[serde(default = "default_max_suggestions")]
    pub max_suggestions: usize,
    /// Use LLM to generate suggestions (more intelligent, slightly slower).
    /// When false, falls back to rule-based suggestions. Default: true.
    #[serde(default = "default_llm_enabled")]
    pub llm_enabled: bool,
    /// Model to use for LLM suggestions. Default: empty (uses provider default).
    #[serde(default)]
    pub llm_model: String,
    /// Timeout for LLM suggestion call in seconds. Default: 10.
    #[serde(default = "default_llm_timeout_secs")]
    pub llm_timeout_secs: u64,
}

fn default_enabled() -> bool {
    true
}
fn default_max_suggestions() -> usize {
    4
}
fn default_llm_enabled() -> bool {
    true
}
fn default_llm_timeout_secs() -> u64 {
    10
}

impl Default for SuggestionsConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            max_suggestions: default_max_suggestions(),
            llm_enabled: default_llm_enabled(),
            llm_model: String::new(),
            llm_timeout_secs: default_llm_timeout_secs(),
        }
    }
}

/// A single suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Short label for the suggestion.
    pub label: String,
    /// Full prompt text to execute if selected.
    pub prompt: String,
    /// Category of the suggestion.
    pub category: SuggestionCategory,
    /// Relevance score (0.0 - 1.0).
    pub relevance: f64,
}

/// Categories of suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionCategory {
    /// Follow-up question.
    FollowUp,
    /// Deeper exploration.
    Explore,
    /// Related task.
    Action,
    /// Correction or refinement.
    Refine,
}

impl SuggestionCategory {
    fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "follow_up" | "followup" | "follow-up" => SuggestionCategory::FollowUp,
            "explore" => SuggestionCategory::Explore,
            "action" => SuggestionCategory::Action,
            "refine" | "correction" => SuggestionCategory::Refine,
            _ => SuggestionCategory::FollowUp,
        }
    }
}

impl std::fmt::Display for SuggestionCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FollowUp => write!(f, "follow_up"),
            Self::Explore => write!(f, "explore"),
            Self::Action => write!(f, "action"),
            Self::Refine => write!(f, "refine"),
        }
    }
}

/// A single message in a conversation for LLM suggestions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: String,
    pub content: String,
}

/// Generate suggestions based on conversation context.
pub fn generate_rule_based_suggestions(
    user_message: &str,
    assistant_response: &str,
    available_tools: &[String],
    config: &SuggestionsConfig,
) -> Vec<Suggestion> {
    if !config.enabled {
        return Vec::new();
    }

    let mut suggestions = Vec::new();
    let msg_lower = user_message.to_lowercase();
    let resp_lower = assistant_response.to_lowercase();

    if resp_lower.contains("error") || resp_lower.contains("failed") || resp_lower.contains("issue")
    {
        suggestions.push(Suggestion {
            label: "Debug further".to_string(),
            prompt: "Can you investigate the error in more detail and suggest a fix?".to_string(),
            category: SuggestionCategory::Explore,
            relevance: 0.9,
        });
    }

    if resp_lower.contains("file") || resp_lower.contains("created") || resp_lower.contains("wrote")
    {
        suggestions.push(Suggestion {
            label: "Review changes".to_string(),
            prompt: "Show me a summary of all the changes that were made.".to_string(),
            category: SuggestionCategory::FollowUp,
            relevance: 0.8,
        });
    }

    if msg_lower.contains("search") || msg_lower.contains("find") {
        suggestions.push(Suggestion {
            label: "Refine search".to_string(),
            prompt: "Can you narrow down the search with more specific criteria?".to_string(),
            category: SuggestionCategory::Refine,
            relevance: 0.7,
        });
    }

    if available_tools.iter().any(|t| t.contains("memory")) && msg_lower.len() > 100 {
        suggestions.push(Suggestion {
            label: "Save to memory".to_string(),
            prompt: "Please save the key points from this conversation to memory.".to_string(),
            category: SuggestionCategory::Action,
            relevance: 0.6,
        });
    }

    if resp_lower.contains("code") || resp_lower.contains("function") || resp_lower.contains("impl")
    {
        suggestions.push(Suggestion {
            label: "Add tests".to_string(),
            prompt: "Can you write tests for the code we just discussed?".to_string(),
            category: SuggestionCategory::Action,
            relevance: 0.7,
        });
    }

    if resp_lower.contains("todo")
        || resp_lower.contains("next step")
        || resp_lower.contains("remaining")
    {
        suggestions.push(Suggestion {
            label: "Continue work".to_string(),
            prompt: "Please continue with the next pending task.".to_string(),
            category: SuggestionCategory::FollowUp,
            relevance: 0.85,
        });
    }

    suggestions.sort_by(|a, b| {
        b.relevance
            .partial_cmp(&a.relevance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    suggestions.truncate(config.max_suggestions);
    suggestions
}

/// Build the system prompt for LLM-driven suggestions ( DeerFlow-inspired).
fn build_llm_system_prompt(n: usize) -> String {
    format!(
        r#"You are an expert assistant that generates short follow-up questions to help the user continue the conversation.

Based on the conversation below, generate EXACTLY {n} short questions the user might ask next.

Requirements:
- Questions must be relevant to the preceding conversation.
- Questions must be written in the same language as the user's last message.
- Keep each question concise (ideally <= 25 words).
- Output MUST be a valid JSON array of objects, each with fields: "prompt" (the question text) and "category" (one of: follow_up, explore, action, refine).
- Do NOT include numbering, markdown code fences, or any extra text.
- Output only the JSON array, nothing else."#
    )
}

/// Format conversation messages for the LLM prompt.
fn format_conversation_for_llm(messages: &[ConversationMessage]) -> String {
    messages
        .iter()
        .map(|m| format!("{}: {}\n", m.role, m.content))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse the LLM JSON response into Suggestion structs.
fn parse_llm_suggestions(response: &str, max: usize) -> Vec<Suggestion> {
    let trimmed = response.trim();

    // Try to extract JSON array from markdown code blocks or raw text
    let json_str = if trimmed.starts_with("```json") {
        trimmed
            .trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if trimmed.starts_with("```") {
        trimmed
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        trimmed
    };

    // Try to parse as array of objects with prompt + category
    if let Ok(items) = serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
        items
            .into_iter()
            .filter_map(|item| {
                let prompt = item.get("prompt")?.as_str()?.trim().to_string();
                if prompt.is_empty() {
                    return None;
                }
                let category_str = item
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("follow_up");
                let category = SuggestionCategory::from_str(category_str);
                // Label is first ~30 chars of prompt
                let label = if prompt.len() > 30 {
                    format!("{}...", &prompt[..30])
                } else {
                    prompt.clone()
                };
                Some(Suggestion {
                    label,
                    prompt,
                    category,
                    relevance: 0.85,
                })
            })
            .take(max)
            .collect()
    } else {
        // Fallback: try to parse as array of plain strings
        if let Ok(strings) = serde_json::from_str::<Vec<String>>(json_str) {
            strings
                .into_iter()
                .filter_map(|s| {
                    let prompt = s.trim().to_string();
                    if prompt.is_empty() {
                        return None;
                    }
                    let label = if prompt.len() > 30 {
                        format!("{}...", &prompt[..30])
                    } else {
                        prompt.clone()
                    };
                    Some(Suggestion {
                        label,
                        prompt,
                        category: SuggestionCategory::FollowUp,
                        relevance: 0.85,
                    })
                })
                .take(max)
                .collect()
        } else {
            Vec::new()
        }
    }
}

/// Generate suggestions using an LLM, based on full conversation history.
/// This is the DeerFlow-inspired approach: richer and more context-aware than
/// rule-based suggestions, at the cost of an extra model call.
///
/// Falls back to an empty vector on any error (timeout, parse failure, etc.).
pub async fn generate_llm_suggestions(
    provider: &dyn crate::providers::Provider,
    model: &str,
    messages: &[ConversationMessage],
    config: &SuggestionsConfig,
) -> Vec<Suggestion> {
    if !config.enabled || !config.llm_enabled {
        return Vec::new();
    }
    if messages.is_empty() {
        return Vec::new();
    }

    let system_prompt = build_llm_system_prompt(config.max_suggestions);
    let conversation = format_conversation_for_llm(messages);
    let user_prompt = format!("Conversation:\n{}\n\nGenerate {} follow-up questions as a JSON array.", conversation, config.max_suggestions);

    let effective_model = if config.llm_model.is_empty() {
        model
    } else {
        &config.llm_model
    };

    let timeout = Duration::from_secs(config.llm_timeout_secs);

    let result = tokio::time::timeout(
        timeout,
        provider.chat_with_system(Some(&system_prompt), &user_prompt, effective_model, 0.7),
    )
    .await;

    match result {
        Ok(Ok(response)) => {
            let suggestions = parse_llm_suggestions(&response, config.max_suggestions);
            tracing::debug!(count = suggestions.len(), "LLM-generated suggestions");
            suggestions
        }
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "LLM suggestions call failed — falling back");
            Vec::new()
        }
        Err(_) => {
            tracing::warn!(
                "LLM suggestions timed out after {}s — falling back",
                config.llm_timeout_secs
            );
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Fake provider for testing LLM suggestions
    struct FakeProvider {
        response: String,
        should_fail: bool,
    }

    impl FakeProvider {
        fn new(response: String) -> Self {
            Self { response, should_fail: false }
        }
        fn failing() -> Self {
            Self { response: String::new(), should_fail: true }
        }
    }

    #[async_trait::async_trait]
    impl crate::providers::Provider for FakeProvider {
        async fn chat_with_system(
            &self,
            _system_prompt: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            if self.should_fail {
                anyhow::bail!("fake error")
            }
            Ok(self.response.clone())
        }
    }

    #[test]
    fn test_error_suggestions() {
        let suggestions = generate_rule_based_suggestions(
            "fix the bug",
            "I found an error in the code: undefined variable",
            &[],
            &SuggestionsConfig::default(),
        );
        assert!(!suggestions.is_empty());
        assert!(suggestions.iter().any(|s| s.label.contains("Debug")));
    }

    #[test]
    fn test_memory_suggestions() {
        let long_msg = "a".repeat(200);
        let suggestions = generate_rule_based_suggestions(
            &long_msg,
            "Here is the result",
            &["memory_store".to_string(), "memory_recall".to_string()],
            &SuggestionsConfig::default(),
        );
        assert!(suggestions.iter().any(|s| s.label.contains("memory")));
    }

    #[test]
    fn test_disabled() {
        let config = SuggestionsConfig {
            enabled: false,
            ..Default::default()
        };
        let suggestions = generate_rule_based_suggestions("hello", "world", &[], &config);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_max_suggestions() {
        let config = SuggestionsConfig {
            max_suggestions: 1,
            ..Default::default()
        };
        let suggestions = generate_rule_based_suggestions(
            "search for files",
            "I found an error in the file and created a new function with code",
            &["memory_store".to_string()],
            &config,
        );
        assert!(suggestions.len() <= 1);
    }

    // ── LLM-driven suggestion tests ─────────────────────────────────

    #[tokio::test]
    async fn test_llm_suggestions_disabled() {
        let config = SuggestionsConfig {
            llm_enabled: false,
            ..Default::default()
        };
        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        let provider = FakeProvider::new(r#"[{"prompt": "test", "category": "follow_up"}]"#.to_string());
        let suggestions = generate_llm_suggestions(&provider, "gpt-4", &messages, &config).await;
        assert!(suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_llm_suggestions_fallback_on_llm_failure() {
        let provider = FakeProvider::failing();
        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        let suggestions =
            generate_llm_suggestions(&provider, "gpt-4", &messages, &SuggestionsConfig::default())
                .await;
        assert!(suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_llm_suggestions_empty_messages() {
        let provider = FakeProvider::new(r#"[{"prompt": "test", "category": "follow_up"}]"#.to_string());
        let suggestions =
            generate_llm_suggestions(&provider, "gpt-4", &[], &SuggestionsConfig::default()).await;
        assert!(suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_llm_suggestions_parses_json_objects() {
        let provider = FakeProvider::new(
            r#"[{"prompt": "What files were created?", "category": "explore"}, {"prompt": "Run the tests", "category": "action"}]"#
                .to_string(),
        );
        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: "Create a new API endpoint".to_string(),
        }];
        let suggestions =
            generate_llm_suggestions(&provider, "gpt-4", &messages, &SuggestionsConfig::default())
                .await;
        assert_eq!(suggestions.len(), 2);
        assert_eq!(suggestions[0].category, SuggestionCategory::Explore);
        assert_eq!(suggestions[1].category, SuggestionCategory::Action);
        assert!(suggestions[0].prompt.contains("files"));
    }

    #[tokio::test]
    async fn test_llm_suggestions_parses_plain_strings() {
        let provider = FakeProvider::new(
            r#"["What does this code do?", "Can you add comments?", "Show me the tests"]"#
                .to_string(),
        );
        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: "Write a function".to_string(),
        }];
        let suggestions =
            generate_llm_suggestions(&provider, "gpt-4", &messages, &SuggestionsConfig::default())
                .await;
        assert_eq!(suggestions.len(), 3);
        assert!(suggestions.iter().all(|s| s.category == SuggestionCategory::FollowUp));
    }

    #[tokio::test]
    async fn test_llm_suggestions_max_truncation() {
        let provider = FakeProvider::new(
            r#"["q1", "q2", "q3", "q4", "q5", "q6"]"#.to_string(),
        );
        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        let config = SuggestionsConfig {
            max_suggestions: 3,
            ..Default::default()
        };
        let suggestions = generate_llm_suggestions(&provider, "gpt-4", &messages, &config).await;
        assert_eq!(suggestions.len(), 3);
    }

    #[tokio::test]
    async fn test_llm_suggestions_handles_markdown_code_block() {
        let provider = FakeProvider::new(
            "```json\n[\"question one\", \"question two\"]\n```".to_string(),
        );
        let messages = vec![ConversationMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        let suggestions =
            generate_llm_suggestions(&provider, "gpt-4", &messages, &SuggestionsConfig::default())
                .await;
        assert_eq!(suggestions.len(), 2);
    }

    #[test]
    fn test_suggestion_category_from_str() {
        assert_eq!(SuggestionCategory::from_str("follow_up"), SuggestionCategory::FollowUp);
        assert_eq!(SuggestionCategory::from_str("explore"), SuggestionCategory::Explore);
        assert_eq!(SuggestionCategory::from_str("action"), SuggestionCategory::Action);
        assert_eq!(SuggestionCategory::from_str("refine"), SuggestionCategory::Refine);
        assert_eq!(SuggestionCategory::from_str("correction"), SuggestionCategory::Refine);
        assert_eq!(SuggestionCategory::from_str("unknown"), SuggestionCategory::FollowUp);
    }
}
