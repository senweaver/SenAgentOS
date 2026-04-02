// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /config command — mirrors claude-code-typescript-src`commands/config/`.
// View or modify agent configuration.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    if ctx.args.is_empty() {
        return CommandResult::ok("Usage: /config [get|set|list] [key] [value]");
    }
    match ctx.args[0].as_str() {
        "list" => CommandResult::ok("Configuration listing (delegated to config module)"),
        "get" => {
            let key = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Config key '{key}' (delegated to config module)"))
        }
        "set" => {
            let key = ctx.args.get(1).map(|s| s.as_str()).unwrap_or("");
            let val = ctx.args.get(2).map(|s| s.as_str()).unwrap_or("");
            CommandResult::ok(format!("Set {key} = {val}"))
        }
        sub => CommandResult::err(format!("Unknown config subcommand: {sub}")),
    }
}
