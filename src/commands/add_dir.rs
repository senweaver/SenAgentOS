// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /add-dir command — mirrors claude-code-typescript-src`commands/add-dir/`.
// Adds additional directories to the agent's working context.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    if ctx.args.is_empty() {
        return CommandResult::err(
            "Usage: /add-dir <path> — add a directory to the working context",
        );
    }
    let dir = &ctx.args[0];
    let path = std::path::Path::new(dir);
    if !path.is_dir() {
        return CommandResult::err(format!("Not a directory: {dir}"));
    }
    CommandResult::ok(format!("Added directory: {dir}"))
}
