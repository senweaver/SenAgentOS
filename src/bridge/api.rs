// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Bridge API — HTTP endpoints for bridge management.
// Mirrors claude-code-typescript-src`bridge/bridgeApi.ts` and `bridge/codeSessionApi.ts`.

use serde::{Deserialize, Serialize};

/// Request to create a new bridge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub device_id: String,
    pub device_name: Option<String>,
    pub paircode: Option<String>,
    pub token: Option<String>,
}

/// Response after creating a bridge session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResponse {
    pub session_id: String,
    pub token: String,
    pub expires_at_epoch_ms: u64,
}

/// Request to send a message via the bridge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub session_id: String,
    pub content: String,
    pub attachments: Vec<AttachmentPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentPayload {
    pub filename: String,
    pub media_type: String,
    pub data_base64: String,
}

/// Response for session status queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStatusResponse {
    pub session_id: String,
    pub status: String,
    pub agent_status: String,
    pub created_at_epoch_ms: u64,
    pub last_activity_epoch_ms: u64,
    pub message_count: u64,
}

/// Bridge health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeHealthResponse {
    pub status: String,
    pub version: String,
    pub active_sessions: u32,
    pub uptime_secs: u64,
}

/// API for managing bridge sessions and messaging.
pub struct BridgeApi;

impl BridgeApi {
    /// Create session endpoint handler.
    pub async fn handle_create_session(
        _req: CreateSessionRequest,
    ) -> anyhow::Result<CreateSessionResponse> {
        // Integration point: this would be wired into the gateway's
        // HTTP router (e.g., axum/actix handler). The actual logic
        // delegates to BridgeSessionManager and DeviceManager.
        anyhow::bail!("Bridge API not yet wired to gateway")
    }

    /// Session status endpoint handler.
    pub async fn handle_session_status(
        _session_id: &str,
    ) -> anyhow::Result<SessionStatusResponse> {
        anyhow::bail!("Bridge API not yet wired to gateway")
    }

    /// Health check endpoint handler.
    pub async fn handle_health() -> BridgeHealthResponse {
        BridgeHealthResponse {
            status: "ok".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            active_sessions: 0,
            uptime_secs: 0,
        }
    }
}
