// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Coordinator Mode — mirrors claude-code-typescript-src `coordinator/coordinatorMode.ts`.
// Provides multi-agent orchestration with worker spawning, messaging, and coordination.

use crate::agent::registry::AgentRegistryHandle;
use crate::agent::task_queue::{Task, TaskQueueHandle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};

/// Coordinator mode state for managing multi-agent workflows.
pub struct CoordinatorMode {
    /// Whether coordinator mode is active.
    is_active: bool,
    /// Active worker agents keyed by agent ID.
    workers: Arc<RwLock<HashMap<String, WorkerInfo>>>,
    /// Task queue handle for task management.
    task_queue: TaskQueueHandle,
    /// Agent registry for spawning and managing agents.
    agent_registry: AgentRegistryHandle,
    /// Channel for receiving task notifications.
    notification_rx: Option<mpsc::Receiver<TaskNotification>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    pub id: String,
    pub description: String,
    pub status: WorkerStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Context snapshot from when the worker was spawned.
    pub context_snapshot: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkerStatus {
    Running,
    Waiting,
    Completed,
    Failed,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNotification {
    pub task_id: String,
    pub status: WorkerStatus,
    pub summary: String,
    pub result: Option<String>,
    pub usage: Option<TaskUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUsage {
    pub total_tokens: u64,
    pub tool_uses: u64,
    pub duration_ms: u64,
}

/// Coordination messages between coordinator and workers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationMessage {
    /// Spawn a new worker agent.
    Spawn {
        description: String,
        prompt: String,
        tools: Vec<String>,
    },
    /// Send a message to an existing worker.
    SendMessage { to: String, message: String },
    /// Stop a running worker.
    Stop { task_id: String },
    /// Get status of all workers.
    Status,
}

impl CoordinatorMode {
    /// Create a new coordinator mode instance.
    pub fn new(task_queue: TaskQueueHandle, agent_registry: AgentRegistryHandle) -> Self {
        Self {
            is_active: false,
            workers: Arc::new(RwLock::new(HashMap::new())),
            task_queue,
            agent_registry,
            notification_rx: None,
        }
    }

    /// Check if coordinator mode is currently active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Activate coordinator mode.
    pub fn activate(&mut self) {
        self.is_active = true;
        tracing::info!("Coordinator mode activated");
    }

    /// Deactivate coordinator mode and clean up workers.
    pub async fn deactivate(&mut self) {
        self.is_active = false;
        // Stop all active workers
        let mut workers = self.workers.write().await;
        for (id, worker) in workers.iter_mut() {
            if worker.status == WorkerStatus::Running || worker.status == WorkerStatus::Waiting {
                tracing::info!(worker_id = %id, "Stopping worker during deactivation");
                worker.status = WorkerStatus::Stopped;
            }
        }
        workers.clear();
        tracing::info!("Coordinator mode deactivated");
    }

    /// Spawn a new worker agent.
    pub async fn spawn_worker(
        &self,
        description: String,
        prompt: String,
        tools: Vec<String>,
    ) -> anyhow::Result<String> {
        if !self.is_active {
            anyhow::bail!("Coordinator mode is not active");
        }

        let agent_id = uuid::Uuid::new_v4().to_string();
        let worker_info = WorkerInfo {
            id: agent_id.clone(),
            description: description.clone(),
            status: WorkerStatus::Running,
            created_at: chrono::Utc::now(),
            last_message_at: Some(chrono::Utc::now()),
            context_snapshot: Some(prompt.clone()),
        };

        // Register the worker
        {
            let mut workers = self.workers.write().await;
            workers.insert(agent_id.clone(), worker_info);
        }

        // Submit the worker task to the queue
        let task = Task::new(
            description.clone(),
            prompt,
            "worker".to_string(),
            "coordinator".to_string(),
        );
        self.task_queue.submit(task);

        tracing::info!(
            worker_id = %agent_id,
            description = %description,
            tool_count = tools.len(),
            "Worker spawned"
        );

        Ok(agent_id)
    }

    /// Send a message to an existing worker.
    pub async fn send_to_worker(&self, worker_id: &str, _message: String) -> anyhow::Result<()> {
        if !self.is_active {
            anyhow::bail!("Coordinator mode is not active");
        }

        let mut workers = self.workers.write().await;
        let worker = workers
            .get_mut(worker_id)
            .ok_or_else(|| anyhow::anyhow!("Worker {} not found", worker_id))?;

        worker.last_message_at = Some(chrono::Utc::now());
        if worker.status == WorkerStatus::Waiting {
            worker.status = WorkerStatus::Running;
        }

        tracing::debug!(worker_id = %worker_id, "Message sent to worker");
        Ok(())
    }

    /// Stop a running worker by task ID.
    pub async fn stop_worker(&self, task_id: &str) -> anyhow::Result<()> {
        if !self.is_active {
            anyhow::bail!("Coordinator mode is not active");
        }

        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(task_id) {
            if worker.status == WorkerStatus::Running || worker.status == WorkerStatus::Waiting {
                worker.status = WorkerStatus::Stopped;
                tracing::info!(worker_id = %task_id, "Worker stopped");
            }
        }

        Ok(())
    }

    /// Get the status of all workers.
    pub async fn get_status(&self) -> Vec<WorkerStatusInfo> {
        let workers = self.workers.read().await;
        workers
            .values()
            .map(|w| WorkerStatusInfo {
                id: w.id.clone(),
                description: w.description.clone(),
                status: w.status,
                created_at: w.created_at,
                last_message_at: w.last_message_at,
            })
            .collect()
    }

    /// Update worker status from a task notification.
    pub async fn update_from_notification(&self, notification: TaskNotification) {
        let mut workers = self.workers.write().await;
        if let Some(worker) = workers.get_mut(&notification.task_id) {
            worker.status = notification.status;
            if let Some(result) = notification.result {
                tracing::info!(
                    worker_id = %notification.task_id,
                    result_len = result.len(),
                    "Worker completed"
                );
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatusInfo {
    pub id: String,
    pub description: String,
    pub status: WorkerStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_message_at: Option<chrono::DateTime<chrono::Utc>>,
}

// ── Coordinator System Prompt Sections ────────────────────────────────────────
// Mirrors the coordinator system prompt sections from cc-typescript-src.

/// Build the coordinator-specific system prompt sections.
/// Mirrors `getCoordinatorSystemPrompt()` and `getCoordinatorUserContext()` from cc-typescript-src.
pub fn build_coordinator_system_prompt() -> String {
    r#"You are SenAgentOS, an AI assistant that orchestrates software engineering tasks across multiple workers.

## 1. Your Role

You are a **coordinator**. Your job is to:
- Help the user achieve their goal
- Direct workers to research, implement and verify code changes
- Synthesize results and communicate with the user
- Answer questions directly when possible — don't delegate work that you can handle without tools

Every message you send is to the user. Worker results and system notifications are internal signals, not conversation partners — never thank or acknowledge them. Summarize new information for the user as it arrives.

## 2. Your Tools

- **AgentTool** - Spawn a new worker
- **SendMessageTool** - Continue an existing worker (send a follow-up to its agent ID)
- **TaskStopTool** - Stop a running worker

When calling AgentTool:
- Do not use one worker to check on another. Workers will notify you when they are done.
- Do not use workers to trivially report file contents or run commands. Give them higher-level tasks.
- Do not set the model parameter. Workers need the default model for the substantive tasks you delegate.
- Continue workers whose work is complete via SendMessageTool to take advantage of their loaded context
- After launching agents, briefly tell the user what you launched and end your response. Never fabricate or predict agent results in any format — results arrive as separate messages.

## 3. Workers

When calling AgentTool, use subagent_type `worker`. Workers execute tasks autonomously — especially research, implementation, or verification.

Workers have access to standard tools, MCP tools from configured MCP servers, and project skills via the Skill tool. Delegate skill invocations (e.g. /commit, /verify) to workers.

## 4. Task Workflow

Most tasks can be broken down into the following phases:

| Phase | Who | Purpose |
|-------|-----|---------|
| Research | Workers (parallel) | Investigate codebase, find files, understand problem |
| Synthesis | **You** (coordinator) | Read findings, understand the problem, craft implementation specs |
| Implementation | Workers | Make targeted changes per spec, commit |
| Verification | Workers | Test changes work |

### Concurrency

**Parallelism is your superpower. Workers are async. Launch independent workers concurrently whenever possible — don't serialize work that can run simultaneously and look for opportunities to fan out.**

### What Real Verification Looks Like

Verification means **proving the code works**, not confirming it exists.

- Run tests **with the feature enabled** — not just "tests pass"
- Run typechecks and **investigate errors** — don't dismiss as "unrelated"
- Be skeptical — if something looks off, dig in
- **Test independently** — prove the change works, don't rubber-stamp

## 5. Writing Worker Prompts

**Workers can't see your conversation.** Every prompt must be self-contained with everything the worker needs.

### Always synthesize — your most important job

When workers report research findings, **you must understand them before directing follow-up work**. Include specific file paths, line numbers, and exactly what to change.

Never write "based on your findings" or "based on the research." These phrases delegate understanding to the worker instead of doing it yourself.

### Always include:
- Specific file paths and line numbers
- What "done" looks like
- For implementation: "Run relevant tests and typecheck, then commit your changes and report the hash"

### Choose continue vs. spawn by context overlap

| Situation | Mechanism | Why |
|-----------|-----------|-----|
| Research explored exactly the files that need editing | **Continue** (SendMessage) | Worker already has the files in context |
| Research was broad but implementation is narrow | **Spawn fresh** (AgentTool) | Avoid dragging along exploration noise |
| Verifying code a different worker just wrote | **Spawn fresh** | Verifier should see the code with fresh eyes |

## 6. Handling Worker Failures

When a worker reports failure (tests failed, build errors, file not found):
- Continue the same worker with SendMessageTool — it has the full error context
- If a correction attempt fails, try a different approach or report to the user"#
        .to_string()
}

/// Build the worker tools context for subagents.
/// Mirrors `getCoordinatorUserContext()` from cc-typescript-src.
pub fn build_worker_tools_context(available_tools: &[String]) -> String {
    format!(
        "Workers spawned via the AgentTool have access to these tools: {}",
        available_tools.join(", ")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::registry::AgentRegistry;
    use crate::agent::task_queue::TaskQueue;

    #[tokio::test]
    async fn test_coordinator_activation() {
        let task_queue = TaskQueueHandle::new(TaskQueue::new());
        let agent_registry = AgentRegistryHandle::new(AgentRegistry::new());
        let mut coordinator = CoordinatorMode::new(task_queue, agent_registry);

        assert!(!coordinator.is_active());
        coordinator.activate();
        assert!(coordinator.is_active());
    }

    #[tokio::test]
    async fn test_worker_spawn() {
        let task_queue = TaskQueueHandle::new(TaskQueue::new());
        let agent_registry = AgentRegistryHandle::new(AgentRegistry::new());
        let mut coordinator = CoordinatorMode::new(task_queue, agent_registry);
        coordinator.activate();

        let worker_id = coordinator
            .spawn_worker(
                "test worker".to_string(),
                "do something".to_string(),
                vec![],
            )
            .await
            .unwrap();

        assert!(!worker_id.is_empty());
        let status = coordinator.get_status().await;
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].description, "test worker");
    }

    #[tokio::test]
    async fn test_worker_stop() {
        let task_queue = TaskQueueHandle::new(TaskQueue::new());
        let agent_registry = AgentRegistryHandle::new(AgentRegistry::new());
        let mut coordinator = CoordinatorMode::new(task_queue, agent_registry);
        coordinator.activate();

        let worker_id = coordinator
            .spawn_worker("test".to_string(), "do".to_string(), vec![])
            .await
            .unwrap();

        coordinator.stop_worker(&worker_id).await.unwrap();
        let status = coordinator.get_status().await;
        assert_eq!(status[0].status, WorkerStatus::Stopped);
    }
}
