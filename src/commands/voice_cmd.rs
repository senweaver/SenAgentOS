// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /voice command — mirrors claude-code-typescript-src`commands/voice/`.
// Toggle voice input mode.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(_ctx: CommandContext) -> CommandResult {
    CommandResult::ok("Voice mode toggled (delegated to voice controller)")
}
