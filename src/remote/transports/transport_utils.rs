// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Transport utilities — mirrors claude-code-typescript-src `cli/transports/transportUtils.ts`.

use crate::remote::transports::sse::SSETransport;
use crate::remote::transports::websocket::WebSocketTransport;

/// Transport configuration.
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// URL for the transport.
    pub url: String,
    /// Session ID.
    pub session_id: String,
    /// Authorization token.
    pub auth_token: Option<String>,
    /// Additional headers.
    pub headers: std::collections::HashMap<String, String>,
    /// Transport type.
    pub transport_type: TransportType,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            session_id: String::new(),
            auth_token: None,
            headers: std::collections::HashMap::new(),
            transport_type: TransportType::WebSocket,
        }
    }
}

/// Type of transport to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportType {
    WebSocket,
    SSE,
    Hybrid,
}

impl Default for TransportType {
    fn default() -> Self {
        Self::WebSocket
    }
}

/// Transport event types.
#[derive(Debug, Clone)]
pub enum TransportEvent {
    Data(String),
    Connected,
    Disconnected(Option<String>),
    Error(String),
    Reconnecting(u32),
    StateChanged(String),
}

/// Result of getting a transport.
pub enum TransportResult {
    WebSocket(WebSocketTransport),
    SSE(SSETransport),
    None,
}

/// Get the appropriate transport for a given URL.
pub fn get_transport_for_url(
    url: &str,
    session_id: &str,
    auth_token: Option<String>,
) -> TransportResult {
    if url.starts_with("wss://") || url.starts_with("ws://") {
        let mut ws = WebSocketTransport::new(url.to_string(), session_id.to_string());
        if let Some(token) = auth_token {
            let mut headers = std::collections::HashMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
            ws = ws.with_headers(headers);
        }
        TransportResult::WebSocket(ws)
    } else if url.starts_with("https://") || url.starts_with("http://") {
        let mut sse = SSETransport::new(url.to_string(), session_id.to_string());
        if let Some(token) = auth_token {
            let mut headers = std::collections::HashMap::new();
            headers.insert("Authorization".to_string(), format!("Bearer {}", token));
            sse = sse.with_headers(headers);
        }
        TransportResult::SSE(sse)
    } else {
        TransportResult::None
    }
}

/// Parse URL to determine transport type.
pub fn parse_url_for_transport(url: &str) -> TransportType {
    if url.starts_with("wss://") || url.starts_with("ws://") {
        TransportType::WebSocket
    } else if url.starts_with("https://") || url.starts_with("http://") {
        TransportType::SSE
    } else {
        TransportType::Hybrid
    }
}

/// Build headers for transport connection.
pub fn build_transport_headers(
    auth_token: Option<&str>,
    extra_headers: Option<&std::collections::HashMap<String, String>>,
) -> std::collections::HashMap<String, String> {
    let mut headers = std::collections::HashMap::new();

    if let Some(token) = auth_token {
        headers.insert("Authorization".to_string(), format!("Bearer {}", token));
    }

    if let Some(extra) = extra_headers {
        for (key, value) in extra.iter() {
            headers.insert(key.clone(), value.clone());
        }
    }

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_websocket_url() {
        assert_eq!(
            parse_url_for_transport("wss://example.com/ws"),
            TransportType::WebSocket
        );
        assert_eq!(
            parse_url_for_transport("ws://localhost:8080/socket"),
            TransportType::WebSocket
        );
    }

    #[test]
    fn test_parse_sse_url() {
        assert_eq!(
            parse_url_for_transport("https://example.com/sse"),
            TransportType::SSE
        );
        assert_eq!(
            parse_url_for_transport("http://localhost:3000/events"),
            TransportType::SSE
        );
    }

    #[test]
    fn test_build_headers() {
        let headers = build_transport_headers(Some("token123"), None);
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_get_transport_for_websocket_url() {
        let result = get_transport_for_url(
            "wss://example.com/ws",
            "session-1",
            Some("token".to_string()),
        );
        match result {
            TransportResult::WebSocket(_) => {}
            _ => panic!("Expected WebSocket transport"),
        }
    }

    #[test]
    fn test_get_transport_for_sse_url() {
        let result = get_transport_for_url(
            "https://example.com/sse",
            "session-1",
            Some("token".to_string()),
        );
        match result {
            TransportResult::SSE(_) => {}
            _ => panic!("Expected SSE transport"),
        }
    }
}
