// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /clear command — mirrors claude-code-typescript-src`commands/clear/`.
// Clears the terminal screen.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(_ctx: CommandContext) -> CommandResult {
    CommandResult::ok("Screen cleared")
}
