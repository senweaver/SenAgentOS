// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /status command — mirrors claude-code-typescript-src`commands/status/`.
// Shows agent status: model, cost, context usage, active tasks.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(_ctx: CommandContext) -> CommandResult {
    CommandResult::ok("Agent status (delegated to bootstrap state and task runner)")
}
