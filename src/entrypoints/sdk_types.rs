// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// SDK types — mirrors claude-code-typescript-src`entrypoints/agentSdkTypes.ts`.
// Public types for the programmatic SDK embedding API.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// SDK session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkStatus {
    Idle,
    Running,
    Waiting,
    Stopped,
    Error,
}

/// SDK configuration for creating an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkConfig {
    pub model: Option<String>,
    pub cwd: Option<PathBuf>,
    pub system_prompt: Option<String>,
    pub max_turns: Option<u32>,
    pub allowed_tools: Vec<String>,
    pub denied_tools: Vec<String>,
    pub mcp_servers: Vec<SdkMcpServer>,
    pub permission_mode: PermissionMode,
    pub structured_output_schema: Option<serde_json::Value>,
    pub metadata: HashMap<String, String>,
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            model: None,
            cwd: None,
            system_prompt: None,
            max_turns: None,
            allowed_tools: Vec::new(),
            denied_tools: Vec::new(),
            mcp_servers: Vec::new(),
            permission_mode: PermissionMode::Default,
            structured_output_schema: None,
            metadata: HashMap::new(),
        }
    }
}

/// Permission mode for tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionMode {
    /// Default: ask user for approval on destructive operations.
    Default,
    /// Auto-approve everything (use with caution).
    AutoApprove,
    /// Deny all tool executions.
    DenyAll,
    /// Plan-only mode (no tool execution).
    PlanOnly,
}

/// An MCP server configuration for SDK usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkMcpServer {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
}

/// A message in the SDK API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkMessage {
    pub role: String,
    pub content: String,
    pub tool_calls: Vec<SdkToolCall>,
    pub metadata: Option<SdkMessageMetadata>,
}

/// A tool call in the SDK API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    pub output: Option<String>,
    pub is_error: bool,
}

/// Metadata attached to SDK messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdkMessageMetadata {
    pub model: Option<String>,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
    pub duration_ms: Option<u64>,
}

/// Model usage statistics for SDK consumers.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SdkModelUsage {
    pub model: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
    pub total_cost_usd: f64,
    pub request_count: u64,
}

/// Hook events that SDK consumers can register callbacks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    Notification,
    Stop,
    SubagentStop,
}
