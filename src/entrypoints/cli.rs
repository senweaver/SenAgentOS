// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI entrypoint — mirrors claude-code-typescript-src`entrypoints/cli.tsx`.
// Bootstraps the interactive terminal REPL session.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// CLI launch options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOptions {
    /// Initial prompt (non-interactive single-shot mode).
    pub prompt: Option<String>,
    /// Resume a previous session.
    pub resume: Option<String>,
    /// Enable plan mode from the start.
    pub plan_mode: bool,
    /// Model override.
    pub model: Option<String>,
    /// Working directory override.
    pub cwd: Option<PathBuf>,
    /// Output format (text, json, stream-json).
    pub output_format: OutputFormat,
    /// Maximum turns for non-interactive mode.
    pub max_turns: Option<u32>,
    /// System prompt override/append.
    pub system_prompt_append: Option<String>,
    /// MCP server configs to load.
    pub mcp_servers: Vec<String>,
    /// Tool allow-list (empty = all).
    pub allowed_tools: Vec<String>,
    /// Tool deny-list.
    pub denied_tools: Vec<String>,
    /// Enable verbose logging.
    pub verbose: bool,
    /// Additional directories to load CLAUDE.md / AGENTS.md from.
    pub add_dirs: Vec<PathBuf>,
    /// Enable remote bridge mode.
    pub remote: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Text,
    Json,
    StreamJson,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            prompt: None,
            resume: None,
            plan_mode: false,
            model: None,
            cwd: None,
            output_format: OutputFormat::Text,
            max_turns: None,
            system_prompt_append: None,
            mcp_servers: Vec::new(),
            allowed_tools: Vec::new(),
            denied_tools: Vec::new(),
            verbose: false,
            add_dirs: Vec::new(),
            remote: false,
        }
    }
}

/// CLI entrypoint — bootstraps and runs the interactive agent session.
pub struct CliEntrypoint;

impl CliEntrypoint {
    /// Run the CLI entrypoint with the given options.
    /// This is the main integration point called from `main.rs`.
    pub async fn run(options: CliOptions) -> anyhow::Result<()> {
        // 1. Initialise bootstrap state
        let cwd = options
            .cwd
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        crate::bootstrap::init_state(cwd.clone());

        // 2. Load configuration
        tracing::info!(cwd = %cwd.display(), "CLI entrypoint starting");

        // 3. Delegate to the agent loop (existing infrastructure in agent::loop_)
        // The actual REPL loop integration happens in main.rs; this struct
        // provides the structured options and setup logic.
        Ok(())
    }
}
