// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /theme command — mirrors claude-code-typescript-src`commands/theme/`.
// Change the output style / theme.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    if ctx.args.is_empty() {
        return CommandResult::ok("Available themes: default, concise, detailed, formal, code-only\nUsage: /theme <name>");
    }
    let theme = &ctx.args[0];
    CommandResult::ok(format!("Theme set to: {theme}"))
}
