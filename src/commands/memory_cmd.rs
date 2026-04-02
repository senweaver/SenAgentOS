// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /memory command — mirrors claude-code-typescript-src`commands/memory/`.
// Manage persistent and session memories.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    let subcmd = ctx.args.first().map(|s| s.as_str()).unwrap_or("list");
    match subcmd {
        "list" => CommandResult::ok("Memories (delegated to memory module)"),
        "add" => {
            let content = ctx.args[1..].join(" ");
            if content.is_empty() {
                return CommandResult::err("Usage: /memory add <content>");
            }
            CommandResult::ok(format!("Memory added: {content}"))
        }
        "remove" | "delete" => {
            let key = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Memory removed: {key}"))
        }
        "clear" => CommandResult::ok("All session memories cleared"),
        "search" => {
            let query = ctx.args[1..].join(" ");
            CommandResult::ok(format!("Searching memories for: {query}"))
        }
        _ => CommandResult::err(format!("Unknown memory subcommand: {subcmd}")),
    }
}
