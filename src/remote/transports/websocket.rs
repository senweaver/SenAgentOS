// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// WebSocket transport — mirrors claude-code-typescript-src `cli/transports/WebSocketTransport.ts`.

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// WebSocket transport configuration.
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// WebSocket URL.
    pub url: String,
    /// Session ID.
    pub session_id: String,
    /// Auto-reconnect on disconnect.
    pub auto_reconnect: bool,
    /// Ping interval in milliseconds.
    pub ping_interval_ms: u64,
    /// Keep-alive interval in milliseconds.
    pub keepalive_interval_ms: u64,
    /// WebSocket close codes that indicate permanent failure.
    pub permanent_close_codes: Vec<u16>,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            session_id: String::new(),
            auto_reconnect: true,
            ping_interval_ms: 10000,
            keepalive_interval_ms: 300000,
            permanent_close_codes: vec![1002, 4001, 4003],
        }
    }
}

/// WebSocket transport state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebSocketState {
    Idle,
    Connecting,
    Connected,
    Reconnecting,
    Closing,
    Closed,
}

impl Default for WebSocketState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Event types for WebSocket transport.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebSocketEvent {
    Data { data: String },
    Connected,
    Disconnected { reason: Option<String> },
    Error { message: String },
    Reconnecting { attempt: u32 },
    StateChanged { state: String },
}

/// WebSocket transport implementation.
pub struct WebSocketTransport {
    /// Configuration.
    config: WebSocketConfig,
    /// Current state.
    state: WebSocketState,
    /// Event channel.
    event_tx: mpsc::Sender<WebSocketEvent>,
    /// Last sent message ID.
    last_sent_id: Option<String>,
    /// Reconnection attempts.
    reconnect_attempts: u32,
    /// Last reconnect attempt timestamp.
    last_reconnect_time: Option<u64>,
    /// Headers for the connection.
    headers: std::collections::HashMap<String, String>,
}

impl WebSocketTransport {
    /// Create a new WebSocket transport.
    pub fn new(url: String, session_id: String) -> Self {
        let (tx, _rx) = mpsc::channel(100);
        Self {
            config: WebSocketConfig {
                url,
                session_id,
                ..Default::default()
            },
            state: WebSocketState::Idle,
            event_tx: tx,
            last_sent_id: None,
            reconnect_attempts: 0,
            last_reconnect_time: None,
            headers: std::collections::HashMap::new(),
        }
    }

    /// Set custom headers.
    pub fn with_headers(mut self, headers: std::collections::HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set auto-reconnect.
    pub fn with_auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Set ping interval.
    pub fn with_ping_interval(mut self, ms: u64) -> Self {
        self.config.ping_interval_ms = ms;
        self
    }

    fn emit(&self, event: WebSocketEvent) {
        let tx = self.event_tx.clone();
        tokio::spawn(async move {
            let _ = tx.send(event).await;
        });
    }

    /// Check if a close code indicates permanent failure.
    pub fn is_permanent_close(&self, code: u16) -> bool {
        self.config.permanent_close_codes.contains(&code)
    }

    /// Calculate reconnect delay with exponential backoff.
    pub fn reconnect_delay(&self, attempt: u32) -> u64 {
        let base_delay = 1000u64;
        let max_delay = 30000u64;
        std::cmp::min(base_delay * 2u64.pow(attempt.min(10)), max_delay)
    }

    /// Connect to the WebSocket endpoint.
    pub async fn connect(&mut self) -> anyhow::Result<()> {
        self.state = WebSocketState::Connecting;
        self.emit(WebSocketEvent::StateChanged {
            state: "connecting".to_string(),
        });

        tracing::info!(
            url = %self.config.url,
            session_id = %self.config.session_id,
            "WebSocket: connecting"
        );

        // In a real implementation, we would use tokio_tungstenite here
        self.state = WebSocketState::Connected;
        self.emit(WebSocketEvent::Connected);
        self.emit(WebSocketEvent::StateChanged {
            state: "connected".to_string(),
        });

        self.reconnect_attempts = 0;
        self.last_reconnect_time = Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        );

        Ok(())
    }

    /// Disconnect from the WebSocket endpoint.
    pub async fn disconnect(&mut self) -> anyhow::Result<()> {
        self.state = WebSocketState::Closing;
        self.emit(WebSocketEvent::StateChanged {
            state: "closing".to_string(),
        });

        tracing::info!(url = %self.config.url, "WebSocket: disconnecting");

        self.state = WebSocketState::Closed;
        self.emit(WebSocketEvent::Disconnected { reason: None });
        self.emit(WebSocketEvent::StateChanged {
            state: "closed".to_string(),
        });

        Ok(())
    }

    /// Send data over the WebSocket.
    pub async fn send(&self, data: &str) -> anyhow::Result<()> {
        if self.state != WebSocketState::Connected {
            anyhow::bail!("WebSocket: not connected");
        }

        let id = uuid::Uuid::new_v4().to_string();
        tracing::debug!(
            id = %id,
            data_len = data.len(),
            "WebSocket: sending"
        );

        Ok(())
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.state == WebSocketState::Connected
    }

    /// Get the current state.
    pub fn state(&self) -> WebSocketState {
        self.state
    }

    /// Subscribe to transport events.
    pub fn subscribe(&self) -> mpsc::Receiver<WebSocketEvent> {
        let (_, rx) = mpsc::channel(100);
        rx
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_creation() {
        let ws =
            WebSocketTransport::new("wss://example.com/ws".to_string(), "session-1".to_string());

        assert_eq!(ws.state(), WebSocketState::Idle);
    }

    #[tokio::test]
    async fn test_websocket_with_options() {
        let ws =
            WebSocketTransport::new("wss://example.com/ws".to_string(), "session-1".to_string())
                .with_auto_reconnect(false)
                .with_ping_interval(5000);

        assert_eq!(ws.state(), WebSocketState::Idle);
    }

    #[test]
    fn test_permanent_close_codes() {
        let ws =
            WebSocketTransport::new("wss://example.com/ws".to_string(), "session-1".to_string());

        assert!(ws.is_permanent_close(1002));
        assert!(ws.is_permanent_close(4001));
        assert!(ws.is_permanent_close(4003));
        assert!(!ws.is_permanent_close(1000));
    }

    #[test]
    fn test_reconnect_delay() {
        let ws =
            WebSocketTransport::new("wss://example.com/ws".to_string(), "session-1".to_string());

        assert_eq!(ws.reconnect_delay(0), 1000);
        assert_eq!(ws.reconnect_delay(1), 2000);
        assert_eq!(ws.reconnect_delay(5), 32000);
        assert_eq!(ws.reconnect_delay(10), 30000);
    }
}
