// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// LocalAgentTask — spawns a local sub-agent process.
// Mirrors claude-code-typescript-src`tasks/LocalAgentTask/`.

use std::path::PathBuf;

use super::types::{Task, TaskHandle, TaskId, TaskState, TaskType, generate_task_id};
use tokio::sync::watch;

pub struct LocalAgentSpawnInput {
    pub prompt: String,
    pub description: String,
    pub agent_definition: Option<String>,
    pub tool_use_id: Option<String>,
    pub allowed_tools: Vec<String>,
    pub cwd: PathBuf,
}

pub struct LocalAgentTask;

impl LocalAgentTask {
    /// Spawn a local sub-agent task.
    pub async fn spawn(input: LocalAgentSpawnInput) -> anyhow::Result<(TaskState, TaskHandle)> {
        let task_id = generate_task_id(TaskType::LocalAgent);
        let mut state = TaskState::new(
            task_id.clone(),
            TaskType::LocalAgent,
            input.description.clone(),
            input.tool_use_id.clone(),
        );
        state.mark_running();

        let (cancel_tx, _cancel_rx) = watch::channel(false);

        // The actual agent execution is delegated to the agent runtime.
        // This is a placeholder for the spawn logic that would use the
        // agent module to create and run a sub-agent.

        let handle = TaskHandle {
            task_id,
            cancel_tx: Some(cancel_tx),
            cleanup: None,
        };

        Ok((state, handle))
    }
}

#[async_trait::async_trait]
impl Task for LocalAgentTask {
    fn name(&self) -> &str {
        "LocalAgentTask"
    }

    fn task_type(&self) -> TaskType {
        TaskType::LocalAgent
    }

    async fn kill(&self, _task_id: &TaskId) -> anyhow::Result<()> {
        Ok(())
    }
}
