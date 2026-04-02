// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Commands module — mirrors claude-code's `commands/` and `commands.ts`.
//
// Provides slash-command infrastructure: registration, discovery,
// filtering by availability/context, and execution. Each submodule
// implements one or more slash commands.

pub mod self_test;
pub mod update;

// -- New commands ported from claude-code-typescript-src--
pub mod add_dir;
pub mod clear;
pub mod compact;
pub mod config_cmd;
pub mod context;
pub mod cost;
pub mod doctor_cmd;
pub mod help;
pub mod history;
pub mod memory_cmd;
pub mod model;
pub mod plan;
pub mod plugin_cmd;
pub mod resume;
pub mod skills_cmd;
pub mod status;
pub mod tasks_cmd;
pub mod theme;
pub mod voice_cmd;
pub mod registry;

#[allow(unused_imports)]
pub use registry::{CommandRegistry, SlashCommand, CommandContext, CommandResult};
