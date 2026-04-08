// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// AutoMode CLI Handler — mirrors claude-code-typescript-src `cli/handlers/autoMode.ts`.
// Provides auto mode rules management commands.

use crate::config::schema::{AutoModeConfig, AutoModeRules};
use serde::{Deserialize, Serialize};

/// AutoMode handler for CLI commands.
pub struct AutoModeHandler {
    config: AutoModeConfig,
    default_rules: AutoModeRules,
}

impl AutoModeHandler {
    /// Create a new AutoMode handler.
    pub fn new(config: AutoModeConfig, default_rules: AutoModeRules) -> Self {
        Self {
            config,
            default_rules,
        }
    }

    /// Get the default rules.
    pub fn get_defaults(&self) -> AutoModeRules {
        self.default_rules.clone()
    }

    /// Get the effective config (user config merged with defaults).
    pub fn get_effective(&self) -> AutoModeRules {
        let user_rules = &self.config.rules;
        AutoModeRules {
            allow: if !user_rules.allow.is_empty() {
                user_rules.allow.clone()
            } else {
                self.default_rules.allow.clone()
            },
            soft_deny: if !user_rules.soft_deny.is_empty() {
                user_rules.soft_deny.clone()
            } else {
                self.default_rules.soft_deny.clone()
            },
            environment: if !user_rules.environment.is_empty() {
                user_rules.environment.clone()
            } else {
                self.default_rules.environment.clone()
            },
        }
    }

    /// Check if custom rules are configured.
    pub fn has_custom_rules(&self) -> bool {
        let rules = &self.config.rules;
        !rules.allow.is_empty() || !rules.soft_deny.is_empty() || !rules.environment.is_empty()
    }

    /// Format rules as JSON.
    pub fn format_rules(&self, rules: &AutoModeRules) -> String {
        serde_json::to_string_pretty(rules).unwrap_or_default()
    }

    /// Format rules for critique prompt.
    pub fn format_rules_for_critique(&self, rules: &AutoModeRules) -> String {
        let mut parts = Vec::new();

        parts.push("Allow rules:".to_string());
        for rule in &rules.allow {
            parts.push(format!("  - {}", rule));
        }

        parts.push("\nSoft-deny rules:".to_string());
        for rule in &rules.soft_deny {
            parts.push(format!("  - {}", rule));
        }

        parts.push("\nEnvironment rules:".to_string());
        for rule in &rules.environment {
            parts.push(format!("  - {}", rule));
        }

        parts.join("\n")
    }
}

/// Build the default system prompt for auto mode classifier.
pub fn build_default_external_system_prompt() -> String {
    r#"You are an expert classifier for Claude Code's auto mode.

Auto mode automatically approves or denies tool calls based on configurable rules.

Your job is to classify each tool call as:
- **allow**: Auto-approve the tool call
- **soft_deny**: Require user confirmation

Consider the user's environment and preferences when making decisions.
"#
    .to_string()
}

/// Critique prompt for analyzing custom rules.
pub const CRITIQUE_SYSTEM_PROMPT: &str = r#"You are an expert reviewer of auto mode classifier rules for Claude Code.

Claude Code has an "auto mode" that uses an AI classifier to decide whether tool calls should be auto-approved or require user confirmation. Users can write custom rules in three categories:

- **allow**: Actions the classifier should auto-approve
- **soft_deny**: Actions the classifier should block (require user confirmation)
- **environment**: Context about the user's setup that helps the classifier make decisions

Your job is to critique the user's custom rules for clarity, completeness, and potential issues. The classifier is an LLM that reads these rules as part of its system prompt.

For each rule, evaluate:
1. **Clarity**: Is the rule unambiguous? Could the classifier misinterpret it?
2. **Completeness**: Are there gaps or edge cases the rule doesn't cover?
3. **Conflicts**: Do any of the rules conflict with each other?
4. **Actionability**: Is the rule specific enough for the classifier to act on?

Be concise and constructive. Only comment on rules that could be improved. If all rules look good, say so.
"#;

/// AutoMode CLI output formats.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModeOutput {
    /// Whether this is a default rules dump.
    pub is_default: bool,
    /// Rules data.
    pub rules: AutoModeRules,
    /// Formatted text output.
    pub text: String,
}

impl AutoModeOutput {
    /// Create output for default rules.
    pub fn defaults(rules: AutoModeRules) -> Self {
        Self {
            is_default: true,
            rules: rules.clone(),
            text: serde_json::to_string_pretty(&rules).unwrap_or_default(),
        }
    }

    /// Create output for effective rules.
    pub fn effective(rules: AutoModeRules) -> Self {
        Self {
            is_default: false,
            rules: rules.clone(),
            text: serde_json::to_string_pretty(&rules).unwrap_or_default(),
        }
    }
}

/// AutoMode CLI commands.
#[derive(Debug, Clone)]
pub enum AutoModeCommand {
    /// Dump default rules.
    Defaults,
    /// Dump effective config.
    Config,
    /// Critique custom rules.
    Critique { model: Option<String> },
}

impl AutoModeHandler {
    /// Handle an auto-mode CLI command.
    pub fn handle(&self, command: AutoModeCommand) -> AutoModeOutput {
        match command {
            AutoModeCommand::Defaults => AutoModeOutput::defaults(self.get_defaults()),
            AutoModeCommand::Config => AutoModeOutput::effective(self.get_effective()),
            AutoModeCommand::Critique { .. } => AutoModeOutput::effective(self.get_effective()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> AutoModeConfig {
        AutoModeConfig {
            enabled: true,
            rules: AutoModeRules {
                allow: vec!["Read files in the project".to_string()],
                soft_deny: vec!["Delete files".to_string()],
                environment: vec!["This is a Rust project".to_string()],
            },
        }
    }

    fn test_defaults() -> AutoModeRules {
        AutoModeRules {
            allow: vec!["Read files".to_string()],
            soft_deny: vec!["Write files".to_string()],
            environment: vec!["Development environment".to_string()],
        }
    }

    #[test]
    fn test_get_effective_with_custom() {
        let handler = AutoModeHandler::new(test_config(), test_defaults());
        let effective = handler.get_effective();

        // Should use custom config
        assert_eq!(effective.allow[0], "Read files in the project");
        assert_eq!(effective.soft_deny[0], "Delete files");
    }

    #[test]
    fn test_get_effective_fallback_to_defaults() {
        let empty_config = AutoModeConfig::default();
        let handler = AutoModeHandler::new(empty_config, test_defaults());
        let effective = handler.get_effective();

        // Should fallback to defaults
        assert_eq!(effective.allow[0], "Read files");
        assert_eq!(effective.soft_deny[0], "Write files");
    }

    #[test]
    fn test_has_custom_rules() {
        let handler = AutoModeHandler::new(test_config(), test_defaults());
        assert!(handler.has_custom_rules());

        let empty_handler = AutoModeHandler::new(AutoModeConfig::default(), test_defaults());
        assert!(!empty_handler.has_custom_rules());
    }

    #[test]
    fn test_format_rules() {
        let handler = AutoModeHandler::new(AutoModeConfig::default(), test_defaults());
        let rules = handler.get_defaults();
        let formatted = handler.format_rules(&rules);

        assert!(formatted.contains("Read files"));
    }
}
