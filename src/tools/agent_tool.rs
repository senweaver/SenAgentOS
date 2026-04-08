// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// AgentTool — mirrors claude-code-typescript-src `tools/AgentTool.ts`.
// Spawns worker agents for multi-agent coordination.

use super::traits::{Tool, ToolResult};
use crate::agent::registry::AgentRegistryHandle;
use crate::agent::task_queue::{Task, TaskPriority, TaskQueueHandle};
use crate::event_bus::EventBusHandle;
use crate::event_bus::types::{Event, EventPayload};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Agent spawn options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpawnOptions {
    /// Human-readable description of the agent's role.
    pub description: String,
    /// Initial prompt/instructions for the agent.
    pub prompt: String,
    /// Available tools for the agent.
    pub tools: Vec<String>,
    /// Subagent type (worker, reviewer, etc.).
    #[serde(default)]
    pub subagent_type: String,
    /// Priority for the agent's tasks.
    #[serde(default)]
    pub priority: String,
}

impl Default for AgentSpawnOptions {
    fn default() -> Self {
        Self {
            description: "Worker agent".to_string(),
            prompt: String::new(),
            tools: Vec::new(),
            subagent_type: "worker".to_string(),
            priority: "normal".to_string(),
        }
    }
}

/// Result of spawning an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpawnResult {
    /// Unique agent ID.
    pub agent_id: String,
    /// Description of the agent.
    pub description: String,
    /// When the agent was created.
    pub created_at: String,
}

/// AgentTool — spawns and manages worker agents.
/// Mirrors the TypeScript AgentTool implementation.
pub struct AgentTool {
    /// Agent registry for tracking agents.
    agent_registry: AgentRegistryHandle,
    /// Task queue for distributing work.
    task_queue: TaskQueueHandle,
    /// Event bus for notifications.
    event_bus: Option<EventBusHandle>,
}

impl AgentTool {
    /// Create a new AgentTool.
    pub fn new(agent_registry: AgentRegistryHandle, task_queue: TaskQueueHandle) -> Self {
        Self {
            agent_registry,
            task_queue,
            event_bus: None,
        }
    }

    /// Set the event bus for notifications.
    pub fn with_event_bus(mut self, event_bus: EventBusHandle) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Spawn a new agent with the given options.
    pub async fn spawn(&self, options: AgentSpawnOptions) -> anyhow::Result<AgentSpawnResult> {
        let agent_id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now();

        // Create a task for the agent
        let task = Task::new(
            options.description.clone(),
            options.prompt.clone(),
            options.subagent_type.clone(),
            "coordinator".to_string(),
        )
        .with_priority(self.priority_from_str(&options.priority));

        // Submit the task to the queue
        self.task_queue.submit(task);

        // Publish spawn event
        if let Some(ref bus) = self.event_bus {
            let event = Event::new(
                "agent_tool",
                crate::event_bus::types::EventTarget::Broadcast,
                EventPayload::Lifecycle {
                    phase: crate::event_bus::types::LifecyclePhase::Spawned,
                    error: None,
                },
            );
            let _ = bus.publish(event).await;
        }

        tracing::info!(
            agent_id = %agent_id,
            description = %options.description,
            "Agent spawned via AgentTool"
        );

        Ok(AgentSpawnResult {
            agent_id,
            description: options.description,
            created_at: now.to_rfc3339(),
        })
    }

    /// Send a message to an existing agent.
    pub async fn send_message(&self, agent_id: &str, message: String) -> anyhow::Result<()> {
        // Find the agent
        let agent = self.agent_registry.get(agent_id);
        if agent.is_none() {
            anyhow::bail!("Agent {} not found", agent_id);
        }

        // Publish message event
        if let Some(ref bus) = self.event_bus {
            let event = Event::new(
                "agent_tool",
                crate::event_bus::types::EventTarget::Agent(agent_id.to_string()),
                EventPayload::MessageReceived {
                    channel: "direct".to_string(),
                    preview: message.chars().take(50).collect(),
                },
            );
            let _ = bus.publish(event).await;
        }

        tracing::info!(
            agent_id = %agent_id,
            message_len = message.len(),
            "Message sent to agent"
        );

        Ok(())
    }

    /// Stop a running agent.
    pub async fn stop(&self, agent_id: &str) -> anyhow::Result<()> {
        let removed = self.agent_registry.deregister(agent_id);
        if removed.is_none() {
            anyhow::bail!("Agent {} not found", agent_id);
        }

        // Publish stop event
        if let Some(ref bus) = self.event_bus {
            let event = Event::new(
                "agent_tool",
                crate::event_bus::types::EventTarget::Agent(agent_id.to_string()),
                EventPayload::Lifecycle {
                    phase: crate::event_bus::types::LifecyclePhase::Terminated,
                    error: None,
                },
            );
            let _ = bus.publish(event).await;
        }

        tracing::info!(agent_id = %agent_id, "Agent stopped via AgentTool");
        Ok(())
    }

    fn priority_from_str(&self, s: &str) -> TaskPriority {
        match s.to_lowercase().as_str() {
            "critical" => TaskPriority::Critical,
            "high" => TaskPriority::High,
            "low" => TaskPriority::Low,
            "background" => TaskPriority::Background,
            _ => TaskPriority::Normal,
        }
    }
}

#[async_trait]
impl Tool for AgentTool {
    fn name(&self) -> &str {
        "AgentTool"
    }

    fn description(&self) -> &str {
        "Spawn a new worker agent to execute tasks in parallel. Use this to delegate complex work to specialized agents."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "description": {
                    "type": "string",
                    "description": "Human-readable description of the agent's role"
                },
                "prompt": {
                    "type": "string",
                    "description": "Initial instructions for the agent"
                },
                "tools": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Available tools for the agent"
                },
                "subagent_type": {
                    "type": "string",
                    "description": "Type of subagent (worker, reviewer)",
                    "default": "worker"
                },
                "priority": {
                    "type": "string",
                    "description": "Task priority (critical, high, normal, low)",
                    "default": "normal"
                }
            },
            "required": ["description", "prompt"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let options: AgentSpawnOptions = serde_json::from_value(args)
            .map_err(|e| anyhow::anyhow!("Failed to parse AgentTool arguments: {}", e))?;

        match self.spawn(options).await {
            Ok(result) => Ok(ToolResult {
                success: true,
                output: serde_json::to_string_pretty(&result).unwrap_or_default(),
                error: None,
            }),
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(e.to_string()),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_tool_creation() {
        let registry = crate::agent::registry::AgentRegistry::new();
        let queue = crate::agent::task_queue::TaskQueue::new();
        let tool = AgentTool::new(
            crate::agent::registry::AgentRegistryHandle::new(registry),
            crate::agent::task_queue::TaskQueueHandle::new(queue),
        );

        assert_eq!(tool.name(), "AgentTool");
    }

    #[tokio::test]
    async fn test_agent_spawn() {
        let registry = crate::agent::registry::AgentRegistry::new();
        let queue = crate::agent::task_queue::TaskQueue::new();
        let tool = AgentTool::new(
            crate::agent::registry::AgentRegistryHandle::new(registry),
            crate::agent::task_queue::TaskQueueHandle::new(queue),
        );

        let options = AgentSpawnOptions {
            description: "Code reviewer".to_string(),
            prompt: "Review the PR for bugs".to_string(),
            tools: vec!["Read".to_string(), "Grep".to_string()],
            subagent_type: "worker".to_string(),
            priority: "high".to_string(),
        };

        let result = tool.spawn(options).await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.description, "Code reviewer");
        assert!(!result.agent_id.is_empty());
    }
}
