// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /resume command — mirrors claude-code-typescript-src`commands/resume/`.
// Resume a previous conversation session.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    let session_id = ctx.args.first().map(|s| s.as_str());
    match session_id {
        Some(id) => CommandResult::ok(format!("Resuming session: {id}")),
        None => CommandResult::ok("Select a session to resume (delegated to session storage)"),
    }
}
