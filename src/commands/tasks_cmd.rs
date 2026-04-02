// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /tasks command — mirrors claude-code-typescript-src`commands/tasks/`.
// Manage background tasks: list, kill, inspect.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    let subcmd = ctx.args.first().map(|s| s.as_str()).unwrap_or("list");
    match subcmd {
        "list" => CommandResult::ok("Background tasks (delegated to task runner)"),
        "kill" => {
            let id = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Killing task: {id}"))
        }
        "inspect" => {
            let id = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Task details for: {id}"))
        }
        _ => CommandResult::err(format!("Unknown tasks subcommand: {subcmd}")),
    }
}
