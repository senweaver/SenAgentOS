// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Structured IO for CLI — mirrors claude-code-typescript-src `cli/structuredIO.ts`.
// Provides structured communication protocol for CLI commands.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Structured message types for CLI communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StructuredMessage {
    /// Tool call request.
    ToolCall {
        id: String,
        name: String,
        arguments: HashMap<String, serde_json::Value>,
    },
    /// Tool result response.
    ToolResult {
        id: String,
        success: bool,
        output: String,
        error: Option<String>,
    },
    /// Permission request.
    PermissionRequest {
        id: String,
        tool: String,
        arguments: HashMap<String, serde_json::Value>,
        reason: String,
    },
    /// Permission response.
    PermissionResponse { id: String, approved: bool },
    /// Status update.
    StatusUpdate {
        state: String,
        message: Option<String>,
    },
    /// Error message.
    Error { code: String, message: String },
    /// Heartbeat for keep-alive.
    Heartbeat,
}

/// Structured IO handler for processing CLI messages.
pub struct StructuredIO {
    /// Session ID for this IO session.
    session_id: String,
    /// Current state.
    state: StructuredIOState,
    /// Pending requests.
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    /// Message history.
    message_history: Arc<RwLock<Vec<StructuredMessage>>>,
}

impl StructuredIO {
    /// Create a new StructuredIO instance.
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            state: StructuredIOState::Idle,
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the current state.
    pub fn state(&self) -> StructuredIOState {
        self.state.clone()
    }

    /// Process an incoming message.
    pub async fn process_message(
        &self,
        message: &str,
    ) -> anyhow::Result<Option<StructuredMessage>> {
        let parsed: StructuredMessage = serde_json::from_str(message)?;

        // Store in history
        {
            let mut history = self.message_history.write().await;
            history.push(parsed.clone());
        }

        // Handle based on type
        match &parsed {
            StructuredMessage::ToolCall { .. } => {
                self.update_state(StructuredIOState::Processing);
            }
            StructuredMessage::ToolResult { .. } => {
                self.update_state(StructuredIOState::Idle);
            }
            StructuredMessage::PermissionRequest { .. } => {
                self.update_state(StructuredIOState::AwaitingPermission);
            }
            StructuredMessage::StatusUpdate { state, .. } => {
                self.update_state(StructuredIOState::from_str(state));
            }
            StructuredMessage::Heartbeat => {
                return Ok(Some(StructuredMessage::Heartbeat));
            }
            _ => {}
        }

        Ok(Some(parsed))
    }

    /// Send a message.
    pub async fn send(&self, message: &StructuredMessage) -> anyhow::Result<String> {
        let json = serde_json::to_string(message)?;
        Ok(json)
    }

    /// Send a tool call.
    pub async fn send_tool_call(
        &self,
        name: &str,
        arguments: HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();

        let request = PendingRequest {
            request_type: "tool_call".to_string(),
            id: id.clone(),
            tool_name: name.to_string(),
            created_at: chrono::Utc::now(),
        };

        {
            let mut pending = self.pending_requests.write().await;
            pending.insert(id.clone(), request);
        }

        let message = StructuredMessage::ToolCall {
            id: id.clone(),
            name: name.to_string(),
            arguments,
        };

        self.send(&message).await
    }

    /// Send a tool result.
    pub async fn send_tool_result(
        &self,
        id: &str,
        success: bool,
        output: &str,
        error: Option<&str>,
    ) -> anyhow::Result<String> {
        {
            let mut pending = self.pending_requests.write().await;
            pending.remove(id);
        }

        let message = StructuredMessage::ToolResult {
            id: id.to_string(),
            success,
            output: output.to_string(),
            error: error.map(String::from),
        };

        self.send(&message).await
    }

    /// Send a permission response.
    pub async fn send_permission_response(
        &self,
        id: &str,
        approved: bool,
    ) -> anyhow::Result<String> {
        let message = StructuredMessage::PermissionResponse {
            id: id.to_string(),
            approved,
        };

        self.send(&message).await
    }

    /// Get pending requests.
    pub async fn pending_requests(&self) -> Vec<PendingRequest> {
        let pending = self.pending_requests.read().await;
        pending.values().cloned().collect()
    }

    /// Get message history.
    pub async fn message_history(&self) -> Vec<StructuredMessage> {
        let history = self.message_history.read().await;
        history.clone()
    }

    /// Clear message history.
    pub async fn clear_history(&self) {
        let mut history = self.message_history.write().await;
        history.clear();
    }

    fn update_state(&self, _new_state: StructuredIOState) {
        // In a real implementation, this would update state with proper synchronization
    }
}

/// A pending request tracked by StructuredIO.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingRequest {
    /// Type of request.
    pub request_type: String,
    /// Request ID.
    pub id: String,
    /// Tool name (for tool calls).
    pub tool_name: String,
    /// When the request was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Structured IO states.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StructuredIOState {
    Idle,
    Processing,
    AwaitingPermission,
    WaitingForInput,
    Error,
}

impl StructuredIOState {
    pub fn as_str(&self) -> &'static str {
        match self {
            StructuredIOState::Idle => "idle",
            StructuredIOState::Processing => "processing",
            StructuredIOState::AwaitingPermission => "awaiting_permission",
            StructuredIOState::WaitingForInput => "waiting_for_input",
            StructuredIOState::Error => "error",
        }
    }

    pub fn from_str(s: &str) -> StructuredIOState {
        match s {
            "idle" => StructuredIOState::Idle,
            "processing" => StructuredIOState::Processing,
            "awaiting_permission" => StructuredIOState::AwaitingPermission,
            "waiting_for_input" => StructuredIOState::WaitingForInput,
            "error" => StructuredIOState::Error,
            _ => StructuredIOState::Idle,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_structured_io_creation() {
        let io = StructuredIO::new("test-session".to_string());
        assert_eq!(io.session_id(), "test-session");
    }

    #[tokio::test]
    async fn test_send_tool_call() {
        let io = StructuredIO::new("test-session".to_string());
        let mut args = HashMap::new();
        args.insert("path".to_string(), serde_json::json!("/tmp/test"));

        let msg = io.send_tool_call("read_file", args).await.unwrap();
        assert!(msg.contains("read_file"));
    }

    #[tokio::test]
    async fn test_send_tool_result() {
        let io = StructuredIO::new("test-session".to_string());

        let msg = io
            .send_tool_result("req-1", true, "File content", None)
            .await
            .unwrap();
        assert!(msg.contains("tool_result"));
    }

    #[tokio::test]
    async fn test_process_message() {
        let io = StructuredIO::new("test-session".to_string());

        let json = r#"{"type":"heartbeat"}"#;
        let result = io.process_message(json).await.unwrap();
        assert!(matches!(result, Some(StructuredMessage::Heartbeat)));
    }

    #[test]
    fn test_state_conversion() {
        assert_eq!(StructuredIOState::Idle.as_str(), "idle");
        assert_eq!(
            StructuredIOState::from_str("processing"),
            StructuredIOState::Processing
        );
    }
}
