// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI Handlers module — mirrors claude-code-typescript-src `cli/handlers/`.
// Provides handlers for all CLI subcommands.

pub mod agents;
pub mod auth;
pub mod auto_mode;
pub mod plugins;

// Re-export handlers
pub use agents::{AgentDisplayInfo, AgentGroup, AgentListOutput, AgentSource, AgentsHandler};
pub use auth::{AuthHandler, AuthStatus, LoginOptions, LogoutOptions};
pub use auto_mode::{AutoModeHandler, CRITIQUE_SYSTEM_PROMPT};
pub use plugins::{
    PluginHandler, PluginInfo, PluginManifest, PluginRegistry, VALID_INSTALLABLE_SCOPES,
    VALID_UPDATE_SCOPES,
};

// Re-export CLI utilities
pub use crate::config::schema::AutoModeRules;
