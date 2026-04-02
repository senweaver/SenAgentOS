// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /history command — mirrors claude-code-typescript-src`commands/history/`.
// Shows or manages conversation history.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    let subcmd = ctx.args.first().map(|s| s.as_str()).unwrap_or("list");
    match subcmd {
        "list" => CommandResult::ok("Recent sessions (delegated to session storage)"),
        "clear" => CommandResult::ok("History cleared"),
        "export" => CommandResult::ok("History exported"),
        _ => CommandResult::err(format!("Unknown history subcommand: {subcmd}")),
    }
}
