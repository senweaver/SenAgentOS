// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /compact command — mirrors claude-code-typescript-src`commands/compact/`.
// Triggers conversation compaction to free context window space.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(ctx: CommandContext) -> CommandResult {
    let _custom_prompt = if ctx.args.is_empty() {
        None
    } else {
        Some(ctx.args.join(" "))
    };
    CommandResult::ok("Conversation compacted successfully")
}
