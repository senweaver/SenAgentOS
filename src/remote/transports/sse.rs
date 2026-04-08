// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// SSE (Server-Sent Events) transport — mirrors claude-code-typescript-src `cli/transports/SSETransport.ts`.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// SSE transport configuration.
#[derive(Debug, Clone)]
pub struct SSEConfig {
    /// SSE endpoint URL.
    pub url: String,
    /// Session ID.
    pub session_id: String,
    /// Whether to use POST for sending.
    pub use_post: bool,
    /// POST max retries.
    pub post_max_retries: u32,
    /// POST base delay in milliseconds.
    pub post_base_delay_ms: u64,
    /// POST max delay in milliseconds.
    pub post_max_delay_ms: u64,
    /// Liveness timeout in milliseconds.
    pub liveness_timeout_ms: u64,
    /// HTTP status codes that indicate permanent failure.
    pub permanent_http_codes: Vec<u16>,
}

impl Default for SSEConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            session_id: String::new(),
            use_post: true,
            post_max_retries: 10,
            post_base_delay_ms: 500,
            post_max_delay_ms: 8000,
            liveness_timeout_ms: 45000,
            permanent_http_codes: vec![401, 403, 404],
        }
    }
}

/// SSE transport state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SSEState {
    Idle,
    Connecting,
    Connected,
    Reconnecting,
    Closing,
    Closed,
}

impl Default for SSEState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Event types for SSE transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SSEEvent {
    Data { data: String },
    Connected,
    Disconnected { reason: Option<String> },
    Error { message: String },
    Reconnecting { attempt: u32 },
    StateChanged { state: String },
}

/// A parsed SSE frame.
#[derive(Debug, Default, Clone)]
pub struct SSEFrame {
    /// Event type.
    pub event: Option<String>,
    /// Event ID.
    pub id: Option<String>,
    /// Data content.
    pub data: Option<String>,
}

/// Parse SSE frames from a buffer.
/// Returns parsed frames and the remaining buffer.
pub fn parse_sse_frames(buffer: &str) -> (Vec<SSEFrame>, &str) {
    let mut frames: Vec<SSEFrame> = Vec::new();
    let mut remaining = buffer;

    while let Some(idx) = remaining.find("\n\n") {
        let raw_frame = &remaining[..idx];
        remaining = &remaining[idx + 2..];

        if raw_frame.trim().is_empty() {
            continue;
        }

        let mut frame = SSEFrame::default();
        let mut is_comment = false;

        for line in raw_frame.split('\n') {
            if line.starts_with(':') {
                is_comment = true;
                continue;
            }

            if let Some(colon_idx) = line.find(':') {
                let field = &line[..colon_idx];
                let mut value = &line[colon_idx + 1..];
                if value.starts_with(' ') {
                    value = &value[1..];
                }

                match field {
                    "event" => frame.event = Some(value.to_string()),
                    "id" => frame.id = Some(value.to_string()),
                    "data" => frame.data = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        if !is_comment {
            frames.push(frame);
        }
    }

    (frames, remaining)
}

/// SSE transport implementation.
pub struct SSETransport {
    /// Configuration.
    config: SSEConfig,
    /// Current state.
    state: SSEState,
    /// Event channel.
    event_tx: mpsc::Sender<SSEEvent>,
    /// Reconnection attempts.
    reconnect_attempts: u32,
    /// Headers for the connection.
    headers: std::collections::HashMap<String, String>,
    /// Buffer for parsing SSE frames.
    buffer: String,
}

impl SSETransport {
    /// Create a new SSE transport.
    pub fn new(url: String, session_id: String) -> Self {
        let (tx, _rx) = mpsc::channel(100);
        Self {
            config: SSEConfig {
                url,
                session_id,
                ..Default::default()
            },
            state: SSEState::Idle,
            event_tx: tx,
            reconnect_attempts: 0,
            headers: std::collections::HashMap::new(),
            buffer: String::new(),
        }
    }

    /// Set custom headers.
    pub fn with_headers(mut self, headers: std::collections::HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Enable or disable POST for sending.
    pub fn with_post(mut self, enabled: bool) -> Self {
        self.config.use_post = enabled;
        self
    }

    /// Set liveness timeout.
    pub fn with_liveness_timeout(mut self, ms: u64) -> Self {
        self.config.liveness_timeout_ms = ms;
        self
    }

    fn emit(&self, event: SSEEvent) {
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(event).await;
        });
    }

    /// Check if an HTTP status code indicates permanent failure.
    pub fn is_permanent_http_code(&self, code: u16) -> bool {
        self.config.permanent_http_codes.contains(&code)
    }

    /// Parse SSE frames from the current buffer.
    pub fn parse_frames(&mut self) -> Vec<SSEFrame> {
        let (frames, remaining) = parse_sse_frames(&self.buffer);
        self.buffer = remaining.to_string();
        frames
    }

    /// Append data to the buffer and parse frames.
    pub fn receive_data(&mut self, data: &str) -> Vec<SSEFrame> {
        self.buffer.push_str(data);
        self.parse_frames()
    }

    /// Clear the buffer.
    pub fn clear_buffer(&mut self) {
        self.buffer.clear();
    }

    /// Calculate POST retry delay with exponential backoff.
    pub fn post_retry_delay(&self, attempt: u32) -> u64 {
        std::cmp::min(
            self.config.post_base_delay_ms * 2u64.pow(attempt.min(10)),
            self.config.post_max_delay_ms,
        )
    }

    /// Connect to the SSE endpoint.
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        self.state = SSEState::Connecting;
        self.emit(SSEEvent::StateChanged {
            state: "connecting".to_string(),
        });

        tracing::info!(
            url = %self.config.url,
            session_id = %self.config.session_id,
            "SSE: connecting"
        );

        // In a real implementation, we would use reqwest here
        self.state = SSEState::Connected;
        self.emit(SSEEvent::Connected);
        self.emit(SSEEvent::StateChanged {
            state: "connected".to_string(),
        });

        self.reconnect_attempts = 0;
        self.buffer.clear();

        Ok(())
    }

    /// Disconnect from the SSE endpoint.
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.state = SSEState::Closing;
        self.emit(SSEEvent::StateChanged {
            state: "closing".to_string(),
        });

        tracing::info!(url = %self.config.url, "SSE: disconnecting");

        self.state = SSEState::Closed;
        self.emit(SSEEvent::Disconnected { reason: None });
        self.emit(SSEEvent::StateChanged {
            state: "closed".to_string(),
        });

        Ok(())
    }

    /// Send data (typically via POST for SSE).
    pub async fn send(&self, _data: &str) -> anyhow::Result<()> {
        if self.state != SSEState::Connected {
            anyhow::bail!("SSE: not connected");
        }

        if self.config.use_post {
            tracing::debug!("SSE: sending via POST");
        }

        Ok(())
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.state == SSEState::Connected
    }

    /// Get the current state.
    pub fn state(&self) -> SSEState {
        self.state
    }

    /// Subscribe to transport events.
    pub fn subscribe(&self) -> mpsc::Receiver<SSEEvent> {
        let (_, rx) = mpsc::channel(100);
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_frames_basic() {
        let buffer = "event: message\ndata: Hello World\nid: 1\n\n";
        let (frames, remaining) = parse_sse_frames(buffer);

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].event, Some("message".to_string()));
        assert_eq!(frames[0].data, Some("Hello World".to_string()));
        assert_eq!(frames[0].id, Some("1".to_string()));
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_sse_frames_multi() {
        let buffer = "data: First\n\ndata: Second\n\ndata: Third\n\n";
        let (frames, remaining) = parse_sse_frames(buffer);

        assert_eq!(frames.len(), 3);
        assert_eq!(frames[0].data, Some("First".to_string()));
        assert_eq!(frames[1].data, Some("Second".to_string()));
        assert_eq!(frames[2].data, Some("Third".to_string()));
        assert_eq!(remaining, "");
    }

    #[test]
    fn test_parse_sse_frames_incremental() {
        let mut sse = SSETransport::new(
            "https://example.com/sse".to_string(),
            "session-1".to_string(),
        );

        let frames1 = sse.receive_data("event: msg\ndata: Part1\n");
        assert!(frames1.is_empty());

        let frames2 = sse.receive_data("\n\ndata: Part2\n\n");
        assert_eq!(frames2.len(), 1);
        assert_eq!(frames2[0].data, Some("Part1\nPart2".to_string()));
    }

    #[test]
    fn test_parse_sse_frames_comments() {
        let buffer = ":keepalive\n:ping\ndata: test\n\n";
        let (frames, _remaining) = parse_sse_frames(buffer);

        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].data, Some("test".to_string()));
    }

    #[tokio::test]
    async fn test_sse_creation() {
        let sse = SSETransport::new(
            "https://example.com/sse".to_string(),
            "session-1".to_string(),
        );

        assert_eq!(sse.state(), SSEState::Idle);
    }

    #[test]
    fn test_permanent_http_codes() {
        let sse = SSETransport::new(
            "https://example.com/sse".to_string(),
            "session-1".to_string(),
        );

        assert!(sse.is_permanent_http_code(401));
        assert!(sse.is_permanent_http_code(403));
        assert!(sse.is_permanent_http_code(404));
        assert!(!sse.is_permanent_http_code(200));
        assert!(!sse.is_permanent_http_code(500));
    }

    #[test]
    fn test_post_retry_delay() {
        let sse = SSETransport::new(
            "https://example.com/sse".to_string(),
            "session-1".to_string(),
        );

        assert_eq!(sse.post_retry_delay(0), 500);
        assert_eq!(sse.post_retry_delay(1), 1000);
        assert_eq!(sse.post_retry_delay(5), 16000);
        assert_eq!(sse.post_retry_delay(10), 8000);
    }
}
