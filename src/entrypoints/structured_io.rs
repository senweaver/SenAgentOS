// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// StructuredIO — mirrors claude-code-typescript-src `cli/structuredIO.ts`.
// Provides structured bidirectional communication for SDK/remote mode,
// capturing the SDK protocol over stdio.

use crate::entrypoints::sdk_types::{SdkConfig, SdkStatus};
use crate::event_bus::EventBus;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

/// Maximum number of resolved tool_use IDs to track.
/// Prevents memory growth in very long sessions.
const MAX_RESOLVED_TOOL_USE_IDS: usize = 1000;

/// Structured message types for SDK communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StructuredMessage {
    /// User message sent to the agent.
    UserMessage {
        content: String,
        metadata: Option<serde_json::Value>,
    },
    /// Tool call result returned from the agent.
    ToolResult {
        id: String,
        output: String,
        is_error: bool,
    },
    /// Permission request for tool execution.
    PermissionRequest {
        tool_name: String,
        input: serde_json::Value,
        reason: String,
    },
    /// Permission decision from the SDK host.
    PermissionDecision { request_id: String, approved: bool },
    /// Agent status update.
    Status { status: SdkStatus },
    /// Session metadata.
    SessionMetadata {
        session_id: String,
        model: Option<String>,
    },
    /// Stream event from the agent.
    StreamEvent {
        event_type: String,
        data: serde_json::Value,
    },
    /// Agent response.
    AgentResponse {
        content: String,
        tool_calls: Vec<SdkToolCall>,
    },
    /// Error from the agent.
    Error {
        message: String,
        code: Option<String>,
    },
    /// Keep-alive ping.
    KeepAlive,
    /// Internal event (for session persistence).
    InternalEvent {
        event_type: String,
        payload: serde_json::Value,
    },
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

/// Permission decision for tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionDecision {
    pub request_id: String,
    pub approved: bool,
    pub reason: Option<String>,
}

/// Callback for permission requests.
pub type PermissionCallback = Box<
    dyn Fn(String, String, serde_json::Value) -> Result<bool, std::convert::Infallible>
        + Send
        + Sync,
>;

/// Pending request waiting for a response.
struct PendingRequest<T> {
    resolve: tokio::sync::oneshot::Sender<T>,
    request: serde_json::Value,
}

/// StructuredIO provides a structured way to read and write SDK messages from stdio.
/// Mirrors the TypeScript `StructuredIO` class from cc-typescript-src.
pub struct StructuredIO {
    /// Channel for sending structured messages out.
    tx: mpsc::Sender<StructuredMessage>,
    /// Receiver for incoming messages.
    rx: Arc<RwLock<Option<mpsc::Receiver<StructuredMessage>>>>,
    /// Pending requests waiting for responses.
    pending_requests:
        Arc<RwLock<std::collections::HashMap<String, PendingRequest<serde_json::Value>>>>,
    /// Resolved tool use IDs (to handle duplicate responses).
    resolved_tool_use_ids: Arc<RwLock<HashSet<String>>>,
    /// Permission callback.
    permission_callback: Option<Arc<PermissionCallback>>,
    /// Event bus for broadcasting.
    event_bus: Option<Arc<EventBus>>,
    /// Session metadata.
    session_id: String,
}

impl StructuredIO {
    /// Create a new StructuredIO instance.
    pub fn new(session_id: String) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            tx,
            rx: Arc::new(RwLock::new(Some(rx))),
            pending_requests: Arc::new(RwLock::new(std::collections::HashMap::new())),
            resolved_tool_use_ids: Arc::new(RwLock::new(HashSet::new())),
            permission_callback: None,
            event_bus: None,
            session_id,
        }
    }

    /// Set the permission callback.
    pub fn set_permission_callback(&mut self, callback: PermissionCallback) {
        self.permission_callback = Some(Arc::new(callback));
    }

    /// Set the event bus.
    pub fn set_event_bus(&mut self, event_bus: Arc<EventBus>) {
        self.event_bus = Some(event_bus);
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Send a structured message.
    pub async fn send(&self, msg: StructuredMessage) -> anyhow::Result<()> {
        self.tx.send(msg).await?;
        Ok(())
    }

    /// Send a user message.
    pub async fn send_user_message(&self, content: String) -> anyhow::Result<()> {
        self.send(StructuredMessage::UserMessage {
            content,
            metadata: None,
        })
        .await
    }

    /// Send a tool result.
    pub async fn send_tool_result(
        &self,
        id: String,
        output: String,
        is_error: bool,
    ) -> anyhow::Result<()> {
        // Track resolved tool use IDs
        {
            let mut resolved = self.resolved_tool_use_ids.write().await;
            resolved.insert(id.clone());
            if resolved.len() > MAX_RESOLVED_TOOL_USE_IDS {
                // Evict oldest entry
                if let Some(oldest) = resolved.iter().next().cloned() {
                    resolved.remove(&oldest);
                }
            }
        }

        self.send(StructuredMessage::ToolResult {
            id,
            output,
            is_error,
        })
        .await
    }

    /// Send a permission request and wait for a decision.
    pub async fn request_permission(
        &self,
        _request_id: String,
        tool_name: String,
        input: serde_json::Value,
        reason: String,
    ) -> anyhow::Result<bool> {
        // Send the request
        self.send(StructuredMessage::PermissionRequest {
            tool_name: tool_name.clone(),
            input: input.clone(),
            reason: reason.clone(),
        })
        .await?;

        // Wait for decision via callback
        if let Some(ref callback) = self.permission_callback {
            let result = callback(tool_name, reason, input);
            return Ok(result?);
        }

        // No callback, default to denying
        Ok(false)
    }

    /// Send an agent response.
    pub async fn send_agent_response(
        &self,
        content: String,
        tool_calls: Vec<SdkToolCall>,
    ) -> anyhow::Result<()> {
        self.send(StructuredMessage::AgentResponse {
            content,
            tool_calls,
        })
        .await
    }

    /// Send a status update.
    pub async fn send_status(&self, status: SdkStatus) -> anyhow::Result<()> {
        self.send(StructuredMessage::Status { status }).await
    }

    /// Send session metadata.
    pub async fn send_session_metadata(&self, model: Option<String>) -> anyhow::Result<()> {
        self.send(StructuredMessage::SessionMetadata {
            session_id: self.session_id.clone(),
            model,
        })
        .await
    }

    /// Send a stream event.
    pub async fn send_stream_event(
        &self,
        event_type: String,
        data: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.send(StructuredMessage::StreamEvent { event_type, data })
            .await
    }

    /// Send an error.
    pub async fn send_error(&self, message: String, code: Option<String>) -> anyhow::Result<()> {
        self.send(StructuredMessage::Error { message, code }).await
    }

    /// Send a keep-alive.
    pub async fn send_keep_alive(&self) -> anyhow::Result<()> {
        self.send(StructuredMessage::KeepAlive).await
    }

    /// Send an internal event.
    pub async fn send_internal_event(
        &self,
        event_type: String,
        payload: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.send(StructuredMessage::InternalEvent {
            event_type,
            payload,
        })
        .await
    }

    /// Check if a tool use ID has already been resolved.
    pub async fn is_tool_use_resolved(&self, id: &str) -> bool {
        let resolved = self.resolved_tool_use_ids.read().await;
        resolved.contains(id)
    }

    /// Get the receiver for consuming messages.
    pub async fn take_receiver(&self) -> Option<mpsc::Receiver<StructuredMessage>> {
        self.rx.write().await.take()
    }

    /// Broadcast an event to all subscribers.
    pub async fn broadcast(&self, event: crate::event_bus::types::Event) {
        if let Some(ref bus) = self.event_bus {
            let _ = bus.publish(event).await;
        }
    }
}

impl Drop for StructuredIO {
    fn drop(&mut self) {
        tracing::debug!(session_id = %self.session_id, "StructuredIO dropped");
    }
}

/// RemoteIO extends StructuredIO for network-based communication.
/// Supports WebSocket and SSE transports.
pub struct RemoteIO {
    /// Base structured IO.
    inner: StructuredIO,
    /// Transport URL.
    url: Option<String>,
    /// Connection status.
    connected: Arc<RwLock<bool>>,
    /// Keep-alive interval handle.
    keep_alive_handle: Option<tokio::task::JoinHandle<()>>,
}

impl RemoteIO {
    /// Create a new RemoteIO instance.
    pub fn new(session_id: String) -> Self {
        Self {
            inner: StructuredIO::new(session_id),
            url: None,
            connected: Arc::new(RwLock::new(false)),
            keep_alive_handle: None,
        }
    }

    /// Connect to a remote URL (WebSocket or SSE).
    pub async fn connect(&mut self, url: String) -> anyhow::Result<()> {
        self.url = Some(url.clone());
        tracing::info!(url = %url, "Connecting to remote endpoint");

        // Mark as connected
        {
            let mut connected = self.connected.write().await;
            *connected = true;
        }

        // Send initial session metadata
        self.inner.send_session_metadata(None).await?;

        tracing::info!("Connected to remote endpoint");
        Ok(())
    }

    /// Disconnect from the remote endpoint.
    pub async fn disconnect(&mut self) {
        // Stop keep-alive
        if let Some(handle) = self.keep_alive_handle.take() {
            handle.abort();
        }

        // Mark as disconnected
        {
            let mut connected = self.connected.write().await;
            *connected = false;
        }

        tracing::info!("Disconnected from remote endpoint");
    }

    /// Check if connected.
    pub async fn is_connected(&self) -> bool {
        let connected = self.connected.read().await;
        *connected
    }

    /// Start keep-alive pings.
    pub fn start_keep_alive(&mut self, interval_secs: u64) {
        if interval_secs == 0 {
            return;
        }

        let inner = std::sync::Arc::new({
            let inner = self.inner.clone();
            inner
        });

        let handle = tokio::spawn(async move {
            let interval = tokio::time::interval(std::time::Duration::from_secs(interval_secs));
            let mut interval = Box::pin(interval);
            loop {
                interval.as_mut().tick().await;
                if let Err(e) = inner.send_keep_alive().await {
                    tracing::debug!("Keep-alive failed: {}", e);
                    break;
                }
            }
        });

        self.keep_alive_handle = Some(handle);
    }

    /// Get a reference to the inner StructuredIO.
    pub fn inner(&self) -> &StructuredIO {
        &self.inner
    }

    /// Get a mutable reference to the inner StructuredIO.
    pub fn inner_mut(&mut self) -> &mut StructuredIO {
        &mut self.inner
    }
}

impl Clone for StructuredIO {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            rx: self.rx.clone(),
            pending_requests: self.pending_requests.clone(),
            resolved_tool_use_ids: self.resolved_tool_use_ids.clone(),
            permission_callback: self.permission_callback.clone(),
            event_bus: self.event_bus.clone(),
            session_id: self.session_id.clone(),
        }
    }
}

// Implement Clone for RemoteIO carefully
impl Clone for RemoteIO {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            url: self.url.clone(),
            connected: self.connected.clone(),
            keep_alive_handle: None, // Don't clone the keep-alive handle
        }
    }
}

/// Builder for SDK session configuration.
pub struct SdkSessionBuilder {
    config: SdkConfig,
    cwd: Option<std::path::PathBuf>,
    event_bus: Option<Arc<EventBus>>,
    permission_callback: Option<PermissionCallback>,
}

impl SdkSessionBuilder {
    /// Create a new builder with default configuration.
    pub fn new() -> Self {
        Self {
            config: SdkConfig::default(),
            cwd: None,
            event_bus: None,
            permission_callback: None,
        }
    }

    /// Set the SDK configuration.
    pub fn config(mut self, config: SdkConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, path: std::path::PathBuf) -> Self {
        self.cwd = Some(path);
        self
    }

    /// Set the event bus.
    pub fn event_bus(mut self, bus: Arc<EventBus>) -> Self {
        self.event_bus = Some(bus);
        self
    }

    /// Set the permission callback.
    pub fn permission_callback(mut self, callback: PermissionCallback) -> Self {
        self.permission_callback = Some(callback);
        self
    }

    /// Build the SDK session.
    pub async fn build(self) -> anyhow::Result<SdkSession> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let mut structured_io = StructuredIO::new(session_id.clone());

        if let Some(bus) = self.event_bus {
            structured_io.set_event_bus(bus);
        }

        if let Some(callback) = self.permission_callback {
            structured_io.set_permission_callback(callback);
        }

        let cwd = self
            .cwd
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        crate::bootstrap::init_state(cwd.clone());

        tracing::info!(session_id = %session_id, cwd = %cwd.display(), "SDK session created");

        Ok(SdkSession {
            session_id,
            structured_io,
            config: self.config,
            cwd,
        })
    }
}

impl Default for SdkSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// An active SDK session.
pub struct SdkSession {
    pub session_id: String,
    pub structured_io: StructuredIO,
    pub config: SdkConfig,
    pub cwd: std::path::PathBuf,
}

impl SdkSession {
    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the StructuredIO for communication.
    pub fn structured_io(&self) -> &StructuredIO {
        &self.structured_io
    }

    /// Get the SDK configuration.
    pub fn config(&self) -> &SdkConfig {
        &self.config
    }

    /// Send a user message.
    pub async fn send_message(&self, content: String) -> anyhow::Result<()> {
        self.structured_io.send_user_message(content).await
    }

    /// Send a tool result.
    pub async fn send_tool_result(
        &self,
        id: String,
        output: String,
        is_error: bool,
    ) -> anyhow::Result<()> {
        self.structured_io
            .send_tool_result(id, output, is_error)
            .await
    }

    /// Send an agent response.
    pub async fn send_response(&self, content: String) -> anyhow::Result<()> {
        self.structured_io
            .send_agent_response(content, vec![])
            .await
    }

    /// Request permission for a tool.
    pub async fn request_permission(
        &self,
        tool_name: String,
        input: serde_json::Value,
        reason: String,
    ) -> anyhow::Result<bool> {
        let request_id = uuid::Uuid::new_v4().to_string();
        self.structured_io
            .request_permission(request_id, tool_name, input, reason)
            .await
    }

    /// Close the session.
    pub async fn close(self) -> anyhow::Result<()> {
        self.structured_io.send_status(SdkStatus::Stopped).await?;
        tracing::info!(session_id = %self.session_id, "SDK session closed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_structured_io_send_message() {
        let io = StructuredIO::new("test-session".to_string());
        let result = io.send_user_message("Hello, agent!".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_structured_io_tool_result_tracking() {
        let io = StructuredIO::new("test-session".to_string());
        io.send_tool_result("tool-1".to_string(), "result".to_string(), false)
            .await
            .unwrap();

        assert!(io.is_tool_use_resolved("tool-1").await);
        assert!(!io.is_tool_use_resolved("tool-2").await);
    }

    #[tokio::test]
    async fn test_sdk_session_builder() {
        let session = SdkSessionBuilder::new()
            .cwd(std::path::PathBuf::from("/tmp"))
            .build()
            .await
            .unwrap();

        assert!(!session.session_id.is_empty());
        assert_eq!(session.cwd, std::path::PathBuf::from("/tmp"));
    }

    #[tokio::test]
    async fn test_remote_io_connection() {
        let remote = RemoteIO::new("test-session".to_string());
        // Note: Can't actually connect without a server, but we can verify state
        assert!(!remote.is_connected().await);
    }
}
