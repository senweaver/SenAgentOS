// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Task management module — mirrors claude-code's `tasks/` and `Task.ts`.
//
// Provides typed task abstractions (local shell, local agent, remote agent,
// in-process teammate, workflow, monitor, dream) with lifecycle management,
// output capture, and status tracking.

pub mod types;
pub mod runner;
pub mod local_shell;
pub mod local_agent;
pub mod remote_agent;
pub mod teammate;
pub mod dream;

pub use types::{
    Task, TaskContext, TaskHandle, TaskId, TaskState, TaskStatus, TaskType,
    generate_task_id, is_terminal_status,
};
pub use runner::TaskRunner;
