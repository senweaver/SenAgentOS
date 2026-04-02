// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Remote module — mirrors claude-code's `remote/` directory.
// Manages remote session connections, WebSocket communication,
// permission bridging, and SDK message adaptation.

pub mod manager;
pub mod permission_bridge;
pub mod websocket;

pub use manager::RemoteSessionManager;
pub use permission_bridge::RemotePermissionBridge;
pub use websocket::SessionWebSocket;
