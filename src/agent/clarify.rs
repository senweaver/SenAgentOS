// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//! Clarification system — intercepts ambiguous user input before tool execution.
//!
//! Mirrors DeerFlow's `ClarificationMiddleware`. Before the agent takes action,
//! the engine evaluates whether the user message contains ambiguity, missing
//! information, or risky intent. If so, it returns a structured
//! [`ClarificationRequest`] that the gateway serializes as a WebSocket event,
//! pausing the agent turn until the user responds.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the clarification engine.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ClarificationConfig {
    /// Enable clarification interception. Default: true.
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Prompt the model to ask for clarification when confidence is below this threshold (0.0–1.0).
    /// Default: 0.6.
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,

    /// Timeout for the LLM clarification check (per turn). Default: 10 seconds.
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_enabled() -> bool {
    true
}
fn default_confidence_threshold() -> f64 {
    0.6
}
fn default_timeout_secs() -> u64 {
    10
}

impl Default for ClarificationConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            confidence_threshold: default_confidence_threshold(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

/// Categories of clarification that the engine can detect.
/// Matches DeerFlow's clarification taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ClarificationType {
    /// The user has not provided enough information to proceed.
    MissingInfo,
    /// The user's intent is ambiguous — multiple interpretations are plausible.
    AmbiguousRequirement,
    /// There are multiple valid approaches and the user should choose.
    ApproachChoice,
    /// The requested action carries significant risk and warrants explicit confirmation.
    RiskConfirmation,
    /// The agent has a suggestion that may better serve the user's intent.
    Suggestion,
}

impl std::fmt::Display for ClarificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingInfo => write!(f, "missing_info"),
            Self::AmbiguousRequirement => write!(f, "ambiguous_requirement"),
            Self::ApproachChoice => write!(f, "approach_choice"),
            Self::RiskConfirmation => write!(f, "risk_confirmation"),
            Self::Suggestion => write!(f, "suggestion"),
        }
    }
}

/// A structured clarification request sent to the frontend.
///
/// This is emitted as a `TurnEvent::Clarification` variant and serialized
/// by the gateway as `{"type":"clarification","category":"...","question":"...","choices":[...]}`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarificationRequest {
    /// The type of clarification detected.
    pub category: ClarificationType,
    /// Natural-language question to present to the user.
    pub question: String,
    /// Optional list of choices for structured responses (e.g. approach options).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    /// Context explaining why clarification is needed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Result of a clarification check.
#[derive(Debug, Clone)]
pub enum ClarificationResult {
    /// No clarification needed — proceed with normal execution.
    Proceed,
    /// Agent should pause and ask the user.
    Clarify(ClarificationRequest),
}

impl ClarificationResult {
    /// Returns true if this result requires user clarification.
    pub fn requires_clarification(&self) -> bool {
        matches!(self, Self::Clarify(_))
    }
}

/// The clarification engine.
///
/// Analyzes user input to detect ambiguity, missing info, or risky intent.
/// Uses the LLM via a lightweight structured prompt (not full tool execution).
pub struct ClarificationEngine {
    config: ClarificationConfig,
}

impl ClarificationEngine {
    pub fn new(config: ClarificationConfig) -> Self {
        Self { config }
    }

    /// Check whether a user message requires clarification.
    ///
    /// This is a non-blocking, synchronous check that runs a lightweight LLM call.
    /// Returns immediately with [`ClarificationResult::Proceed`] if clarification
    /// is disabled or the message appears well-formed.
    pub async fn check(
        &self,
        provider: &dyn crate::providers::Provider,
        model: &str,
        user_message: &str,
        conversation_history: &[(String, String)], // role, content
    ) -> ClarificationResult {
        if !self.config.enabled {
            return ClarificationResult::Proceed;
        }

        let prompt = self.build_check_prompt(user_message, conversation_history);

        let timeout = Duration::from_secs(self.config.timeout_secs);
        let result = tokio::time::timeout(
            timeout,
            provider.simple_chat(&prompt, model, 0.0),
        )
        .await;

        match result {
            Ok(Ok(response)) => self.parse_llm_response(&response),
            Ok(Err(e)) => {
                tracing::warn!(error = %e, "Clarification check LLM call failed — proceeding");
                ClarificationResult::Proceed
            }
            Err(_) => {
                tracing::warn!("Clarification check timed out after {}s — proceeding", self.config.timeout_secs);
                ClarificationResult::Proceed
            }
        }
    }

    fn build_check_prompt(
        &self,
        user_message: &str,
        history: &[(String, String)],
    ) -> String {
        let mut prompt = String::new();
        prompt.push_str(
            "You are a clarification classifier. Analyze the user's message and determine \
             whether clarification is needed before the assistant can safely act.\n\n",
        );
        prompt.push_str("## Conversation so far\n");
        for (role, content) in history.iter().rev().take(4) {
            prompt.push_str(&format!("{}: {}\n", role, content));
        }
        prompt.push_str(&format!("\n## User message\n{}\n\n", user_message));
        prompt.push_str(&format!(
            "## Your task\nDetermine if clarification is needed. Confidence threshold: {:.1}.\n\n",
            self.config.confidence_threshold
        ));
        prompt.push_str(
            "Respond with ONLY a JSON object (no markdown, no explanation):\n\
             {\"needs_clarification\": true/false, \"category\": \"...\", \"question\": \"...\", \
             \"choices\": [\"option1\", \"option2\"], \"context\": \"...\"}\n\n\
             Categories: missing_info, ambiguous_requirement, approach_choice, \
             risk_confirmation, suggestion\n\n\
             Rules:\n\
             - Set needs_clarification to false if the message is clear and actionable.\n\
             - Set needs_clarification to true if:\n\
               * Critical information is missing (file path, name, etc.)\n\
               * The intent could mean multiple things\n\
               * Multiple valid approaches exist and the user didn't specify\n\
               * The action is destructive or irreversible\n\
               * You have a better suggestion than what the user asked for\n\
             - question: a single, clear question to ask the user\n\
             - choices: only include if the clarification is an approach choice\n\
             - context: brief explanation of why clarification is needed\n",
        );
        prompt
    }

    fn parse_llm_response(&self, response: &str) -> ClarificationResult {
        // Strip markdown code fences if present
        let text = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let json: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                tracing::debug!(error = %e, raw = %response, "Failed to parse clarification JSON — proceeding");
                return ClarificationResult::Proceed;
            }
        };

        let needs_clarification = json
            .get("needs_clarification")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !needs_clarification {
            return ClarificationResult::Proceed;
        }

        let category = json
            .get("category")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "missing_info" => Some(ClarificationType::MissingInfo),
                "ambiguous_requirement" => Some(ClarificationType::AmbiguousRequirement),
                "approach_choice" => Some(ClarificationType::ApproachChoice),
                "risk_confirmation" => Some(ClarificationType::RiskConfirmation),
                "suggestion" => Some(ClarificationType::Suggestion),
                _ => None,
            })
            .unwrap_or(ClarificationType::MissingInfo);

        let question = json
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("Could you provide more details?")
            .to_string();

        let choices = json.get("choices").and_then(|v| {
            v.as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| item.as_str().map(String::from))
                        .collect()
                })
                .filter(|v: &Vec<String>| !v.is_empty())
        });

        let context = json
            .get("context")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        ClarificationResult::Clarify(ClarificationRequest {
            category,
            question,
            choices,
            context,
        })
    }
}

/// Format a clarification request as readable text for channel-based delivery.
///
/// Used as a fallback when the gateway cannot stream the structured event
/// (e.g. non-WebSocket channels like Telegram).
pub fn format_clarification_message(req: &ClarificationRequest) -> String {
    let mut msg = format!("**Clarification needed: {}**\n\n", req.category);
    msg.push_str(&req.question);
    msg.push('\n');

    if let Some(ref choices) = req.choices {
        msg.push_str("\n");
        for (i, choice) in choices.iter().enumerate() {
            msg.push_str(&format!("{}. {}\n", i + 1, choice));
        }
        msg.push_str("\n_Reply with a number or type your answer._\n");
    } else {
        msg.push_str("\n_Please provide more details and I will continue._\n");
    }

    if let Some(ref ctx) = req.context {
        msg.push_str(&format!("\n> {ctx}\n"));
    }

    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_proceed_response() {
        let engine = ClarificationEngine::new(ClarificationConfig::default());
        let json = r#"{"needs_clarification": false}"#;
        let result = engine.parse_llm_response(json);
        assert!(matches!(result, ClarificationResult::Proceed));
    }

    #[test]
    fn parse_clarify_missing_info() {
        let engine = ClarificationEngine::new(ClarificationConfig::default());
        let json = r#"{"needs_clarification": true, "category": "missing_info", "question": "Which file should I read?", "context": "No file path provided"}"#;
        let result = engine.parse_llm_response(json);
        match result {
            ClarificationResult::Clarify(req) => {
                assert!(matches!(req.category, ClarificationType::MissingInfo));
                assert_eq!(req.question, "Which file should I read?");
                assert!(req.choices.is_none());
                assert_eq!(req.context.as_deref(), Some("No file path provided"));
            }
            ClarificationResult::Proceed => panic!("Expected Clarify, got Proceed"),
        }
    }

    #[test]
    fn parse_clarify_with_choices() {
        let engine = ClarificationEngine::new(ClarificationConfig::default());
        let json = r#"{"needs_clarification": true, "category": "approach_choice", "question": "How should I search?", "choices": ["Web search", "Use internal knowledge"]}"#;
        let result = engine.parse_llm_response(json);
        match result {
            ClarificationResult::Clarify(req) => {
                assert!(matches!(req.category, ClarificationType::ApproachChoice));
                assert_eq!(req.choices.as_ref().map(|v| v.len()), Some(2));
            }
            ClarificationResult::Proceed => panic!("Expected Clarify, got Proceed"),
        }
    }

    #[test]
    fn parse_invalid_json_proceeds() {
        let engine = ClarificationEngine::new(ClarificationConfig::default());
        let result = engine.parse_llm_response("not json at all");
        assert!(matches!(result, ClarificationResult::Proceed));
    }

    #[test]
    fn parse_json_with_code_fences() {
        let engine = ClarificationEngine::new(ClarificationConfig::default());
        let json = "```json\n{\"needs_clarification\": true, \"category\": \"suggestion\", \"question\": \"Try this instead?\"}\n```";
        let result = engine.parse_llm_response(json);
        match result {
            ClarificationResult::Clarify(req) => {
                assert!(matches!(req.category, ClarificationType::Suggestion));
            }
            ClarificationResult::Proceed => panic!("Expected Clarify"),
        }
    }

    use async_trait::async_trait;

    struct FakeProvider;

    #[async_trait]
    impl crate::providers::Provider for FakeProvider {
        async fn chat_with_system(
            &self,
            _system: Option<&str>,
            _message: &str,
            _model: &str,
            _temperature: f64,
        ) -> anyhow::Result<String> {
            Ok(r#"{"needs_clarification": false}"#.to_string())
        }
    }

    #[test]
    fn disabled_engine_always_proceeds() {
        let config = ClarificationConfig {
            enabled: false,
            ..Default::default()
        };
        let engine = ClarificationEngine::new(config);
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let result = rt.block_on(engine.check(&FakeProvider, "gpt-4", "Fix the bug", &[]));
        // When disabled, check() returns Proceed without calling the provider.
        assert!(matches!(result, ClarificationResult::Proceed));
    }

    #[test]
    fn format_clarification_message_basic() {
        let req = ClarificationRequest {
            category: ClarificationType::MissingInfo,
            question: "Which directory?".to_string(),
            choices: None,
            context: Some("No directory specified".to_string()),
        };
        let msg = format_clarification_message(&req);
        assert!(msg.contains("missing_info"));
        assert!(msg.contains("Which directory?"));
        assert!(msg.contains("No directory specified"));
    }

    #[test]
    fn format_clarification_message_with_choices() {
        let req = ClarificationRequest {
            category: ClarificationType::ApproachChoice,
            question: "Pick an approach".to_string(),
            choices: Some(vec!["A".to_string(), "B".to_string()]),
            context: None,
        };
        let msg = format_clarification_message(&req);
        assert!(msg.contains("approach_choice"));
        assert!(msg.contains("1. A"));
        assert!(msg.contains("2. B"));
    }
}
