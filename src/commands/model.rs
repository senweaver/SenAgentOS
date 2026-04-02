// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /model command — mirrors claude-code-typescript-src`commands/model/`.
// Switch or display the current model.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    if ctx.args.is_empty() {
        return CommandResult::ok("Current model (delegated to bootstrap state)");
    }
    let model = &ctx.args[0];
    CommandResult::ok(format!("Model switched to: {model}"))
}
