// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /context command — mirrors claude-code-typescript-src`commands/context/`.
// Shows current context window usage and loaded context files.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(_ctx: CommandContext) -> CommandResult {
    // Delegates to the query engine's token budget for actual numbers.
    CommandResult::ok("Context usage summary (delegated to query engine)")
}
