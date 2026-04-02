// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// QueryEngine — the core orchestrator for model queries.
// Mirrors claude-code-typescript-src`QueryEngine.ts`.

use serde::{Deserialize, Serialize};

use super::stop_hooks::{StopHook, StopHookContext, StopHookResult, evaluate_stop_hooks};
use super::token_budget::TokenBudget;

/// Outcome of a single query execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// The model's response text.
    pub response_text: String,
    /// Tool calls requested by the model (if any).
    pub tool_calls: Vec<ToolCallRequest>,
    /// Token usage for this query.
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    /// Cost in USD.
    pub cost_usd: f64,
    /// API duration in milliseconds.
    pub api_duration_ms: u64,
    /// Stop reason from the model.
    pub stop_reason: Option<String>,
    /// Whether the query was aborted.
    pub aborted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRequest {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

/// The query engine manages the full lifecycle of a model query:
/// system prompt assembly, message construction, API call, tool execution
/// loop, stop-hook evaluation, and result collection.
pub struct QueryEngine {
    stop_hooks: Vec<Box<dyn StopHook>>,
    budget: TokenBudget,
}

impl QueryEngine {
    pub fn new(context_window: u32, max_output_tokens: u32) -> Self {
        Self {
            stop_hooks: Vec::new(),
            budget: TokenBudget::new(context_window, max_output_tokens),
        }
    }

    /// Register a stop hook.
    pub fn add_stop_hook(&mut self, hook: Box<dyn StopHook>) {
        self.stop_hooks.push(hook);
    }

    /// Access the token budget (for callers that need to inspect/update).
    pub fn budget(&self) -> &TokenBudget {
        &self.budget
    }

    pub fn budget_mut(&mut self) -> &mut TokenBudget {
        &mut self.budget
    }

    /// Evaluate all registered stop hooks.
    pub async fn check_stop_hooks(
        &self,
        tool_turn_count: u32,
        total_tokens_used: u64,
        current_cost_usd: f64,
        budget_limit_usd: Option<f64>,
        max_tool_turns: Option<u32>,
        model_stop_reason: Option<String>,
    ) -> StopHookResult {
        let ctx = StopHookContext {
            tool_turn_count,
            total_tokens_used,
            context_window: self.budget.context_window,
            model_stop_reason,
            current_cost_usd,
            budget_limit_usd,
            max_tool_turns,
        };
        evaluate_stop_hooks(&self.stop_hooks, &ctx).await
    }

    /// Check whether the context is getting full and compaction is needed.
    pub fn needs_compaction(&self, threshold: f64) -> bool {
        self.budget.should_compact(threshold)
    }
}
