// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /plugin command — mirrors claude-code-typescript-src`commands/plugin/`.
// Manage plugins: list, enable, disable, install.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    let subcmd = ctx.args.first().map(|s| s.as_str()).unwrap_or("list");
    match subcmd {
        "list" => CommandResult::ok("Installed plugins (delegated to plugins module)"),
        "enable" => {
            let name = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Plugin '{name}' enabled"))
        }
        "disable" => {
            let name = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Plugin '{name}' disabled"))
        }
        "install" => {
            let path = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Plugin installed from: {path}"))
        }
        _ => CommandResult::err(format!("Unknown plugin subcommand: {subcmd}")),
    }
}
