// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// SDK entrypoint — mirrors claude-code-typescript-src`entrypoints/sdk/`.
// Provides a programmatic API for embedding SenAgentOS in other applications.

use super::sdk_types::{SdkConfig, SdkMessage, SdkStatus};

/// SDK entrypoint for programmatic embedding.
pub struct SdkEntrypoint {
    config: SdkConfig,
    status: SdkStatus,
}

impl SdkEntrypoint {
    pub fn new(config: SdkConfig) -> Self {
        Self {
            config,
            status: SdkStatus::Idle,
        }
    }

    /// Start a new agent session via the SDK.
    pub async fn start_session(&mut self) -> anyhow::Result<String> {
        self.status = SdkStatus::Running;
        let session_id = uuid::Uuid::new_v4().to_string();
        let cwd = self
            .config
            .cwd
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
        crate::bootstrap::init_state(cwd);
        tracing::info!(session_id = %session_id, "SDK session started");
        Ok(session_id)
    }

    /// Send a user message and get the agent's response.
    pub async fn send_message(&self, message: SdkMessage) -> anyhow::Result<SdkMessage> {
        // This delegates to the agent loop for processing.
        // The SDK wraps the internal message types into the public SDK types.
        tracing::debug!(content = %message.content, "SDK message received");

        Ok(SdkMessage {
            role: "assistant".to_string(),
            content: String::new(), // Placeholder — actual response from agent loop
            tool_calls: Vec::new(),
            metadata: None,
        })
    }

    /// Get current session status.
    pub fn status(&self) -> SdkStatus {
        self.status
    }

    /// Stop the current session.
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        self.status = SdkStatus::Stopped;
        tracing::info!("SDK session stopped");
        Ok(())
    }
}
