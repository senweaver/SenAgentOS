// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Remote IO module - mirrors claude-code-typescript-src remote/ and cli/transports/.
// Provides WebSocket and SSE transport implementations for SDK/remote mode.

pub mod manager;
pub mod permission_bridge;
pub mod websocket;

// Re-export transport modules
pub mod transports {
    pub mod sse;
    pub mod transport_utils;
    pub mod websocket;
}

// Re-export common types
pub use transports::sse::SSETransport;
pub use transports::transport_utils::{
    TransportConfig, TransportEvent, TransportResult, TransportType, build_transport_headers,
    get_transport_for_url, parse_url_for_transport,
};
pub use transports::websocket::WebSocketTransport;
