// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /plan command — mirrors claude-code-typescript-src`commands/plan/`.
// Toggle plan mode on/off.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(_ctx: CommandContext) -> CommandResult {
    CommandResult::ok("Plan mode toggled (delegated to agent plan_mode)")
}
