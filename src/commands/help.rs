// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /help command — mirrors claude-code-typescript-src`commands/help/`.
// Shows available commands and usage information.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    if ctx.args.is_empty() {
        CommandResult::ok(
            "Available commands:\n\
             /help [command]  — Show help\n\
             /compact         — Compact conversation\n\
             /clear           — Clear screen\n\
             /config          — View/set configuration\n\
             /context         — Show context usage\n\
             /cost            — Show session cost\n\
             /doctor          — Run diagnostics\n\
             /history         — Show conversation history\n\
             /memory          — Manage memories\n\
             /model           — Switch model\n\
             /plan            — Toggle plan mode\n\
             /plugin          — Manage plugins\n\
             /resume          — Resume a session\n\
             /skills          — Manage skills\n\
             /status          — Show agent status\n\
             /tasks           — Manage background tasks\n\
             /theme           — Change output style\n\
             /voice           — Toggle voice mode\n\
             /add-dir         — Add directory to context\n\
             \nType /help <command> for details on a specific command.",
        )
    } else {
        CommandResult::ok(format!("Help for /{} (delegated to registry)", ctx.args[0]))
    }
}
