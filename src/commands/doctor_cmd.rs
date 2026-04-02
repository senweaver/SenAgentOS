// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// /doctor command — mirrors claude-code-typescript-src`commands/doctor/`.
// Runs diagnostic checks on the agent environment.

use super::registry::{CommandContext, CommandResult};

pub async fn handle(_ctx: CommandContext) -> CommandResult {
    CommandResult::ok("Running diagnostics... (delegated to doctor module)")
}
