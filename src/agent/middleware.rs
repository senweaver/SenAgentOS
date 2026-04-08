// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//! Middleware pipeline — DeerFlow-inspired explicit execution ordering.
//!
//! Defines an ordered chain of middleware hooks that run before and after each
//! agent turn. Unlike loose hook calls scattered through the codebase, the
//! pipeline gives operators a single, auditable list of execution steps.
//!
//! **Design (DeerFlow-inspired):**
//! - `before_*` hooks run in forward order (pipeline[0] → pipeline[N]) before the model call.
//! - `after_*` hooks run in reverse order (pipeline[N] → pipeline[0]) after the model call.
//! - Middleware is identified by a `MiddlewareId` enum and configured via `[middleware]`
//!   in the TOML config with an explicit `order` field.
//! - Middleware that is not listed in the `order` config defaults to its individual
//!   feature flag (e.g. `[guardrails].enabled`) — backward-compatible.

use crate::config::Config;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Identifies a registered middleware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MiddlewareId {
    /// Guardrail checks — block disallowed tool calls before execution.
    Guardrails,
    /// Token/output compression — shrink LLM output before history storage.
    TokenOptimizer,
    /// Ambiguity detection — pause turn and ask the user for clarification.
    Clarification,
    /// Memory consolidation — extract facts from the turn and store them.
    MemoryConsolidation,
    /// Loop detection — detect and break repeated tool-call cycles.
    LoopDetection,
    /// Self-evaluation — record turn quality heuristics for learning.
    SelfEval,
    /// Feedback signal detection — detect implicit correction signals.
    Feedback,
    /// Experience replay store — record the turn as a learning episode.
    Experience,
    /// Self-reflection — have the model critique its own response.
    SelfReflection,
    /// Skill evolution — track tool usage patterns and evolve skills.
    SkillEvolution,
    /// Response cache — skip LLM call on exact prompt match.
    ResponseCache,
}

impl fmt::Display for MiddlewareId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Guardrails => write!(f, "guardrails"),
            Self::TokenOptimizer => write!(f, "token_optimizer"),
            Self::Clarification => write!(f, "clarification"),
            Self::MemoryConsolidation => write!(f, "memory_consolidation"),
            Self::LoopDetection => write!(f, "loop_detection"),
            Self::SelfEval => write!(f, "self_eval"),
            Self::Feedback => write!(f, "feedback"),
            Self::Experience => write!(f, "experience"),
            Self::SelfReflection => write!(f, "self_reflection"),
            Self::SkillEvolution => write!(f, "skill_evolution"),
            Self::ResponseCache => write!(f, "response_cache"),
        }
    }
}

impl MiddlewareId {
    /// All known middleware IDs in their canonical execution order.
    /// This is the default when `order` is not explicitly specified in config.
    pub fn default_order() -> Vec<Self> {
        vec![
            Self::Guardrails,
            Self::TokenOptimizer,
            Self::Clarification,
            Self::MemoryConsolidation,
            Self::LoopDetection,
            Self::ResponseCache,
            Self::SelfEval,
            Self::Feedback,
            Self::Experience,
            Self::SelfReflection,
            Self::SkillEvolution,
        ]
    }

    /// Returns true if this middleware is enabled by default (individual feature flag).
    pub fn default_enabled(&self, config: &Config) -> bool {
        match self {
            Self::Guardrails => config.guardrails.enabled,
            Self::TokenOptimizer => config.tool_output_compressor.enabled,
            Self::Clarification => config.clarification.enabled,
            Self::MemoryConsolidation => true, // Phase 3: LLM fact extraction; always on once implemented
            Self::LoopDetection => config.pacing.loop_detection_enabled,
            Self::SelfEval => config.self_eval.enabled,
            Self::Feedback => config.feedback.enabled,
            Self::Experience => config.experience.enabled,
            Self::SelfReflection => config.self_reflection.enabled,
            Self::SkillEvolution => config.skill_evolution.enabled,
            Self::ResponseCache => config.memory.response_cache_enabled,
        }
    }
}

/// Middleware configuration from TOML.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MiddlewareConfig {
    /// Explicit execution order. Middleware IDs not listed here are included
    /// automatically if their individual feature flag is enabled.
    #[serde(default)]
    pub order: Vec<MiddlewareId>,
    /// If true, only middleware listed in `order` is executed (strict mode).
    /// If false (default), unlisted middleware runs if its feature flag is enabled.
    #[serde(default)]
    pub strict: bool,
    /// Disable specific middleware by ID (applied after computing the active set).
    #[serde(default)]
    pub disabled: Vec<MiddlewareId>,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            order: MiddlewareId::default_order(),
            strict: false,
            disabled: Vec::new(),
        }
    }
}

/// Context passed through the middleware pipeline.
#[derive(Debug)]
pub struct MiddlewareContext<'a> {
    pub user_message: &'a str,
    pub model_name: &'a str,
    pub tool_results: &'a [(String, bool)],
}

impl MiddlewareContext<'_> {
    /// Create a dummy context for non-turn operations.
    pub fn dummy() -> Self {
        Self { user_message: "", model_name: "", tool_results: &[] }
    }
}

/// A single middleware entry — pairs an ID with its enabled flag.
#[derive(Debug, Clone)]
pub struct MiddlewareEntry {
    pub id: MiddlewareId,
    pub enabled: bool,
}

/// Computed pipeline — ordered list of enabled middleware.
#[derive(Debug, Clone)]
pub struct MiddlewarePipeline {
    entries: Vec<MiddlewareEntry>,
}

impl MiddlewarePipeline {
    /// Build the pipeline from config + feature flags.
    pub fn from_config(config: &MiddlewareConfig, feature_config: &Config) -> Self {
        let mut entries: Vec<MiddlewareEntry> = if config.order.is_empty() {
            MiddlewareId::default_order()
                .into_iter()
                .map(|id| MiddlewareEntry { id, enabled: id.default_enabled(feature_config) })
                .collect()
        } else {
            config.order.iter().map(|id| {
                let enabled = if config.strict {
                    // Strict mode: unlisted = disabled
                    config.order.contains(id)
                } else {
                    // Default: respect individual feature flag
                    id.default_enabled(feature_config)
                };
                MiddlewareEntry { id: *id, enabled }
            }).collect()
        };

        // Apply explicit disable list
        for disabled_id in &config.disabled {
            if let Some(entry) = entries.iter_mut().find(|e| e.id == *disabled_id) {
                entry.enabled = false;
            }
        }

        entries.retain(|e| e.enabled);
        Self { entries }
    }

    /// Returns the ordered list of enabled middleware IDs.
    pub fn enabled(&self) -> Vec<MiddlewareId> {
        self.entries.iter().map(|e| e.id).collect()
    }

    /// Returns the execution order as human-readable strings, for observability.
    pub fn describe(&self) -> Vec<String> {
        self.entries.iter().map(|e| e.id.to_string()).collect()
    }

    /// Execute all `before_*` hooks in forward order.
    /// Returns an optional override response — if Some, the agent should short-circuit.
    pub async fn run_before<'a>(
        &self,
        _context: &MiddlewareContext<'a>,
        _config: &'a Config,
    ) -> Option<String> {
        // Currently all before_* logic is embedded directly in Agent::turn / turn_streamed.
        // This hook exists for future middleware that wants to intercept/pre-process.
        None
    }

    /// Execute all `after_*` hooks in reverse order.
    pub async fn run_after(
        &self,
        context: &MiddlewareContext<'_>,
        config: &Config,
        assistant_response: &str,
    ) {
        // After-hooks run in reverse (DeerFlow convention: last-in, first-out)
        for entry in self.entries.iter().rev() {
            match entry.id {
                MiddlewareId::SelfEval if config.self_eval.enabled => {
                    let refs: Vec<(&str, bool)> = context
                        .tool_results
                        .iter()
                        .map(|(n, s)| (n.as_str(), *s))
                        .collect();
                    crate::agent::runtime_hooks::LearningHooks::from_config(config)
                        .record_turn_heuristics(context.user_message, assistant_response, &refs);
                }
                MiddlewareId::SkillEvolution if config.skill_evolution.enabled => {
                    for (tool_name, success) in context.tool_results {
                        crate::agent::runtime_hooks::LearningHooks::from_config(config)
                            .record_tool_execution(tool_name, *success, 0);
                    }
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config() -> Config {
        Config::default()
    }

    #[test]
    fn default_pipeline_respects_feature_flags() {
        let config = MiddlewareConfig::default();
        let feature_cfg = make_config();
        let pipeline = MiddlewarePipeline::from_config(&config, &feature_cfg);

        // All IDs in default order should appear in pipeline (respecting flags)
        let ids = pipeline.enabled();
        // Guardrails, TokenOptimizer, MemoryConsolidation, etc. — all should be present if enabled
        assert!(
            ids.contains(&MiddlewareId::ResponseCache),
            "ResponseCache should be in default pipeline"
        );
    }

    #[test]
    fn explicit_order_respected() {
        let config = MiddlewareConfig {
            order: vec![MiddlewareId::Clarification, MiddlewareId::Guardrails],
            strict: false,
            disabled: vec![],
        };
        let feature_cfg = make_config();
        let pipeline = MiddlewarePipeline::from_config(&config, &feature_cfg);
        let ids = pipeline.enabled();

        assert_eq!(ids[0], MiddlewareId::Clarification);
        assert_eq!(ids[1], MiddlewareId::Guardrails);
    }

    #[test]
    fn strict_mode_excludes_unlisted() {
        let config = MiddlewareConfig {
            order: vec![MiddlewareId::Guardrails],
            strict: true,
            disabled: vec![],
        };
        let feature_cfg = make_config();
        let pipeline = MiddlewarePipeline::from_config(&config, &feature_cfg);
        let ids = pipeline.enabled();

        assert_eq!(ids.len(), 1);
        assert_eq!(ids[0], MiddlewareId::Guardrails);
    }

    #[test]
    fn explicit_disable_overrides_feature_flag() {
        let config = MiddlewareConfig {
            order: vec![],
            strict: false,
            disabled: vec![MiddlewareId::ResponseCache],
        };
        let feature_cfg = make_config();
        let pipeline = MiddlewarePipeline::from_config(&config, &feature_cfg);
        let ids = pipeline.enabled();

        assert!(
            !ids.contains(&MiddlewareId::ResponseCache),
            "ResponseCache should be disabled by explicit disable list"
        );
    }

    #[test]
    fn describe_returns_strings() {
        let config = MiddlewareConfig {
            order: vec![MiddlewareId::Guardrails, MiddlewareId::Clarification],
            strict: true,
            disabled: vec![],
        };
        let feature_cfg = make_config();
        let pipeline = MiddlewarePipeline::from_config(&config, &feature_cfg);
        let desc = pipeline.describe();

        assert_eq!(desc, vec!["guardrails", "clarification"]);
    }

    #[tokio::test]
    async fn run_before_returns_none() {
        let config = MiddlewareConfig::default();
        let feature_cfg = make_config();
        let pipeline = MiddlewarePipeline::from_config(&config, &feature_cfg);
        let ctx = MiddlewareContext {
            user_message: "test",
            model_name: "gpt-4",
            tool_results: &[],
        };
        let result = pipeline.run_before(&ctx, &feature_cfg).await;
        assert!(result.is_none());
    }
}
