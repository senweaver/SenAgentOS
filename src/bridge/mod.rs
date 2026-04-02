// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Bridge module — remote control bridge for WebSocket-based session management.
// Mirrors claude-code's `bridge/` directory.
//
// Provides device pairing, JWT-based authentication, remote session creation,
// message relay, and capacity/wake management for controlling agent sessions
// from mobile/web clients.

pub mod types;
pub mod config;
pub mod messaging;
pub mod session;
pub mod jwt;
pub mod transport;
pub mod device;
pub mod api;
pub mod status;

pub use types::{BridgeMessage, BridgeStatus, BridgeEvent};
pub use config::BridgeConfig;
pub use messaging::BridgeMessaging;
pub use session::BridgeSession;
pub use transport::BridgeTransport;
pub use device::TrustedDevice;
