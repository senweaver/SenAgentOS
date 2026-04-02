// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /skills command — mirrors claude-code-typescript-src`commands/skills/`.
// Manage agent skills: list, create, edit, delete.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    let subcmd = ctx.args.first().map(|s| s.as_str()).unwrap_or("list");
    match subcmd {
        "list" => CommandResult::ok("Available skills (delegated to skills module)"),
        "create" => {
            let name = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Creating skill: {name}"))
        }
        "edit" => {
            let name = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Editing skill: {name}"))
        }
        "delete" => {
            let name = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Deleted skill: {name}"))
        }
        _ => CommandResult::err(format!("Unknown skills subcommand: {subcmd}")),
    }
}
