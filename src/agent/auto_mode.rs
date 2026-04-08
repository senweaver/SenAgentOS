// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Auto Mode — mirrors claude-code-typescript-src `cli/handlers/autoMode.ts`.
// AI-powered classifier for auto-approving tool execution based on user-defined rules.

use crate::config::schema::AutoModeRules;
use serde::{Deserialize, Serialize};

/// Classifier result for auto mode decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoModeDecision {
    /// Whether to auto-approve or require user confirmation.
    pub approved: bool,
    /// Reason for the decision.
    pub reason: String,
    /// The matched rule category (if any).
    pub category: Option<String>,
}

/// Build the default external auto mode rules for external users.
/// Mirrors `getDefaultExternalAutoModeRules()` from cc-typescript-src.
pub fn get_default_external_rules() -> AutoModeRules {
    AutoModeRules {
        allow: vec![
            // Safe read operations
            "Read files in the current project".to_string(),
            "Search for files and content".to_string(),
            "List directory contents".to_string(),
            "View git status and history".to_string(),
            // Safe code exploration
            "Run read-only commands (git status, git log, git diff)".to_string(),
            "Search for function and variable definitions".to_string(),
            "View file contents".to_string(),
            // Safe analysis
            "Run linters and type checkers".to_string(),
            "View test results".to_string(),
            "Check build output".to_string(),
        ],
        soft_deny: vec![
            // Potentially risky operations
            "Write or modify files".to_string(),
            "Run shell commands".to_string(),
            "Git push or commit".to_string(),
            "Delete files or directories".to_string(),
            "Network requests to external services".to_string(),
        ],
        environment: vec![
            "The user is working in a development environment".to_string(),
            "Files in the project are code under active development".to_string(),
            "The working directory is a git repository".to_string(),
        ],
    }
}

/// Auto mode classifier that decides whether to auto-approve tool calls.
/// Mirrors the LLM-based classifier from cc-typescript-src.
pub struct AutoModeClassifier {
    rules: AutoModeRules,
}

impl AutoModeClassifier {
    /// Create a new classifier with the given rules.
    pub fn new(rules: AutoModeRules) -> Self {
        Self { rules }
    }

    /// Classify a tool call against the rules.
    /// Returns a decision with the reason for the classification.
    pub fn classify(&self, tool_name: &str, action_description: &str) -> AutoModeDecision {
        let lower_action = action_description.to_lowercase();
        let _lower_tool = tool_name.to_lowercase();

        // Check soft_deny rules first (they take precedence)
        for rule in &self.rules.soft_deny {
            if self.rule_matches(&lower_action, rule) {
                return AutoModeDecision {
                    approved: false,
                    reason: format!("Matched soft_deny rule: {}", rule),
                    category: Some("soft_deny".to_string()),
                };
            }
        }

        // Check allow rules
        for rule in &self.rules.allow {
            if self.rule_matches(&lower_action, rule) {
                return AutoModeDecision {
                    approved: true,
                    reason: format!("Matched allow rule: {}", rule),
                    category: Some("allow".to_string()),
                };
            }
        }

        // Default to requiring confirmation for unknown operations
        AutoModeDecision {
            approved: false,
            reason: "No matching rule found — user confirmation required".to_string(),
            category: None,
        }
    }

    /// Check if a rule matches the action description.
    fn rule_matches(&self, action: &str, rule: &str) -> bool {
        let rule_lower = rule.to_lowercase();

        // Check for keyword overlaps
        let rule_words: Vec<&str> = rule_lower.split_whitespace().collect();
        let action_words: Vec<&str> = action.split_whitespace().collect();

        // Count matching words (at least 2 or 50% of rule words must match)
        let matches: usize = rule_words
            .iter()
            .filter(|w| action_words.contains(w))
            .count();

        if rule_words.len() >= 2 {
            matches >= 2 || (matches as f64 / rule_words.len() as f64) >= 0.5
        } else {
            matches >= 1
        }
    }

    /// Merge user rules with defaults, with user rules taking precedence.
    /// Mirrors the merge behavior from cc-typescript-src.
    pub fn merge_with_defaults(user_rules: Option<&AutoModeRules>) -> Self {
        let defaults = get_default_external_rules();
        let rules = match user_rules {
            Some(user) => AutoModeRules {
                allow: if user.allow.is_empty() {
                    defaults.allow
                } else {
                    user.allow.clone()
                },
                soft_deny: if user.soft_deny.is_empty() {
                    defaults.soft_deny
                } else {
                    user.soft_deny.clone()
                },
                environment: if user.environment.is_empty() {
                    defaults.environment
                } else {
                    user.environment.clone()
                },
            },
            None => defaults,
        };
        Self { rules }
    }
}

/// Build the classifier system prompt from rules.
/// Mirrors `buildDefaultExternalSystemPrompt()` from cc-typescript-src.
pub fn build_classifier_system_prompt(rules: &AutoModeRules) -> String {
    let mut prompt = String::from(
        r#"You are an expert classifier for an AI coding assistant's auto-approve system.

Your job is to classify tool execution requests as either:
- **APPROVE**: Safe operations that can be auto-approved
- **CONFIRM**: Operations that require user confirmation before executing

## Classification Guidelines

### APPROVE when:
- The action is read-only (viewing files, searching, inspecting)
- The action is non-destructive and reversible
- The action follows project conventions and best practices
- The action is in a development/safe context

### CONFIRM when:
- The action is destructive (deleting, overwriting)
- The action affects shared systems or remote services
- The action could have irreversible consequences
- The action involves committing/pushing code
- The action involves running untrusted commands

## User Environment Context:"#,
    );

    for env in &rules.environment {
        prompt.push_str(&format!("\n- {}", env));
    }

    prompt.push_str("\n\n## Specific Allow Rules (APPROVE these operations):\n");
    for rule in &rules.allow {
        prompt.push_str(&format!("- {}\n", rule));
    }

    prompt.push_str("\n## Specific Soft-Deny Rules (CONFIRM these operations):\n");
    for rule in &rules.soft_deny {
        prompt.push_str(&format!("- {}\n", rule));
    }

    prompt.push_str(
        r#"

## Your Task

Given a tool name and action description, classify it as APPROVE or CONFIRM.
Respond with exactly one word: APPROVE or CONFIRM.
"#,
    );

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_rules(allow: Vec<&str>, soft_deny: Vec<&str>, environment: Vec<&str>) -> AutoModeRules {
        AutoModeRules {
            allow: allow.into_iter().map(String::from).collect(),
            soft_deny: soft_deny.into_iter().map(String::from).collect(),
            environment: environment.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_allow_rule_matches() {
        let rules = make_rules(vec!["Read files in the current project"], vec![], vec![]);
        let classifier = AutoModeClassifier::new(rules);
        let decision = classifier.classify("Read", "Read files in the current project");
        assert!(decision.approved);
        assert_eq!(decision.category, Some("allow".to_string()));
    }

    #[test]
    fn test_soft_deny_precedence() {
        let rules = make_rules(
            vec!["Read files"],
            vec!["Delete files or directories"],
            vec![],
        );
        let classifier = AutoModeClassifier::new(rules);
        let decision = classifier.classify("Delete", "Delete files or directories");
        assert!(!decision.approved);
        assert_eq!(decision.category, Some("soft_deny".to_string()));
    }

    #[test]
    fn test_no_match_requires_confirmation() {
        let rules = make_rules(vec!["Read files"], vec![], vec![]);
        let classifier = AutoModeClassifier::new(rules);
        let decision = classifier.classify("Unknown", "Some unknown operation");
        assert!(!decision.approved);
        assert!(decision.category.is_none());
    }

    #[test]
    fn test_merge_with_defaults_empty_user_rules() {
        let user_rules: Option<&AutoModeRules> = None;
        let classifier = AutoModeClassifier::merge_with_defaults(user_rules);
        let defaults = get_default_external_rules();
        assert_eq!(classifier.rules.allow, defaults.allow);
        assert_eq!(classifier.rules.soft_deny, defaults.soft_deny);
    }
}
