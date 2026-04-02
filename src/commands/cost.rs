// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /cost command — mirrors claude-code-typescript-src`commands/cost/`.
// Shows session cost and token usage summary.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(_ctx: CommandContext) -> CommandResult {
    CommandResult::ok("Cost summary (delegated to bootstrap state / cost tracker)")
}
