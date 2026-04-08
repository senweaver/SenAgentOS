// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Query engine module — mirrors claude-code's `query/` and `query.ts`.
//
// Provides query configuration, token budget management, dependency
// injection for queries, and stop-hook evaluation.

pub mod config;
pub mod deps;
pub mod engine;
pub mod stop_hooks;
pub mod token_budget;

pub use config::QueryConfig;
pub use deps::QueryDeps;
pub use engine::QueryEngine;
pub use stop_hooks::{StopHook, StopHookResult};
pub use token_budget::TokenBudget;
