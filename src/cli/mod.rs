// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI module — mirrors claude-code-typescript-src `cli/`.
// Provides CLI command handlers and entry points.

pub mod exit;
pub mod handlers;
pub mod ndjson;
pub mod print;
pub mod structured_io;
pub mod update;

// Re-export handlers
pub use handlers::{
    AgentDisplayInfo, AgentGroup, AgentListOutput, AgentSource, AgentsHandler, AuthHandler,
    AuthStatus, AutoModeHandler, CRITIQUE_SYSTEM_PROMPT, LoginOptions, LogoutOptions,
    PluginHandler, PluginInfo, PluginManifest, PluginRegistry, VALID_INSTALLABLE_SCOPES,
    VALID_UPDATE_SCOPES,
};

// Re-export CLI utilities
pub use exit::{CliError, CliResult, cli_error, cli_ok};
pub use ndjson::{contains_js_line_terminators, ndjson_safe_stringify, parse_ndjson_lines};
pub use print::{
    OutputFormat, Progress, Spinner, bullet, colors, error, figures, format_bytes, format_duration,
    info, kv, list_item, print_in_format, print_table, section, subsection, success, table_row,
    warning,
};
pub use update::{Diagnostic, UpdateResult, handle_update, run_diagnostic};
