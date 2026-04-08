// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Team System — mirrors claude-code-typescript-src team-related functionality.
// Provides team messaging, task management, and inter-agent communication.
// This module enables the multi-agent team paradigm where a coordinator
// can spawn workers and communicate with them via structured messages.

use crate::event_bus::{EventBus, EventBusHandle};
use crate::tasks::TaskId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unique team identifier.
pub type TeamId = String;

/// Unique agent identifier within a team.
pub type AgentId = String;

/// Unique message identifier.
pub type MessageId = String;

/// A team of agents working together under a coordinator.
/// Mirrors the team concept from cc-typescript-src.
pub struct Team {
    id: TeamId,
    name: String,
    coordinator_id: AgentId,
    members: Arc<RwLock<HashMap<AgentId, TeamMember>>>,
    message_queue: Arc<RwLock<HashMap<AgentId, Vec<TeamMessage>>>>,
    task_board: Arc<RwLock<TaskBoard>>,
    event_bus: EventBusHandle,
}

/// A member of a team.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMember {
    pub id: AgentId,
    pub name: String,
    pub role: TeamRole,
    pub status: MemberStatus,
    pub joined_at: chrono::DateTime<chrono::Utc>,
    pub last_active_at: chrono::DateTime<chrono::Utc>,
    /// Tasks completed by this member.
    pub tasks_completed: u64,
}

/// Team member roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TeamRole {
    /// Coordinates the team and delegates work.
    Coordinator,
    /// Executes assigned tasks.
    Worker,
    /// Reviews and verifies work.
    Reviewer,
    /// Specialized expert agent.
    Specialist,
}

/// Member operational status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemberStatus {
    Online,
    Busy,
    Idle,
    Offline,
}

/// A message between team members.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamMessage {
    pub id: MessageId,
    pub from: AgentId,
    pub to: AgentId,
    pub content: String,
    pub message_type: MessageType,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub task_id: Option<TaskId>,
    pub metadata: HashMap<String, String>,
}

/// Types of team messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Direct message to another agent.
    Direct,
    /// Broadcast to all team members.
    Broadcast,
    /// Task assignment.
    TaskAssignment,
    /// Task completion notification.
    TaskResult,
    /// Status update.
    StatusUpdate,
    /// Coordination message.
    Coordination,
}

/// Task board for tracking team tasks.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskBoard {
    /// All tasks in the team.
    tasks: HashMap<TaskId, BoardTask>,
    /// Task ID counter.
    next_id: u64,
}

/// Task status for the task board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoardTaskStatus {
    Todo,
    InProgress,
    Done,
    Blocked,
    Cancelled,
}

impl Default for BoardTaskStatus {
    fn default() -> Self {
        Self::Todo
    }
}

/// A task on the task board.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardTask {
    pub id: TaskId,
    pub title: String,
    pub description: String,
    pub assigned_to: Option<AgentId>,
    pub status: BoardTaskStatus,
    pub priority: TaskPriority,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub result: Option<String>,
    pub tags: Vec<String>,
}

/// Task priority levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPriority {
    Critical,
    High,
    Normal,
    Low,
}

impl Default for TaskPriority {
    fn default() -> Self {
        Self::Normal
    }
}

impl Team {
    /// Create a new team.
    pub fn new(id: TeamId, name: String, coordinator_id: AgentId) -> Self {
        let event_bus = EventBus::new().into();
        Self {
            id,
            name,
            coordinator_id,
            members: Arc::new(RwLock::new(HashMap::new())),
            message_queue: Arc::new(RwLock::new(HashMap::new())),
            task_board: Arc::new(RwLock::new(TaskBoard::default())),
            event_bus,
        }
    }

    /// Get the team ID.
    pub fn id(&self) -> &TeamId {
        &self.id
    }

    /// Get the team name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the coordinator ID.
    pub fn coordinator_id(&self) -> &AgentId {
        &self.coordinator_id
    }

    /// Add a member to the team.
    pub async fn add_member(
        &self,
        id: AgentId,
        name: String,
        role: TeamRole,
    ) -> anyhow::Result<()> {
        let member = TeamMember {
            id: id.clone(),
            name,
            role,
            status: MemberStatus::Online,
            joined_at: chrono::Utc::now(),
            last_active_at: chrono::Utc::now(),
            tasks_completed: 0,
        };

        let mut members = self.members.write().await;
        if members.contains_key(&id) {
            anyhow::bail!("Member {} already exists in team", id);
        }
        members.insert(id.clone(), member);

        // Initialize message queue for the member
        let mut queue = self.message_queue.write().await;
        queue.entry(id.clone()).or_insert_with(Vec::new);

        tracing::info!(team = %self.name, member = %id, role = ?role, "Team member joined");

        Ok(())
    }

    /// Remove a member from the team.
    pub async fn remove_member(&self, id: &AgentId) -> anyhow::Result<()> {
        let mut members = self.members.write().await;
        let removed = members.remove(id);
        if removed.is_none() {
            anyhow::bail!("Member {} not found in team", id);
        }

        // Remove message queue
        let mut queue = self.message_queue.write().await;
        queue.remove(id);

        tracing::info!(team = %self.name, member = %id, "Team member removed");

        Ok(())
    }

    /// Get all team members.
    pub async fn members(&self) -> Vec<TeamMember> {
        let members = self.members.read().await;
        members.values().cloned().collect()
    }

    /// Get a specific member.
    pub async fn get_member(&self, id: &AgentId) -> Option<TeamMember> {
        let members = self.members.read().await;
        members.get(id).cloned()
    }

    /// Update member status.
    pub async fn update_member_status(&self, id: &AgentId, status: MemberStatus) {
        let mut members = self.members.write().await;
        if let Some(member) = members.get_mut(id) {
            member.status = status;
            member.last_active_at = chrono::Utc::now();
        }
    }

    /// Send a message between team members.
    pub async fn send_message(
        &self,
        from: &AgentId,
        to: &AgentId,
        content: String,
        message_type: MessageType,
        task_id: Option<TaskId>,
    ) -> anyhow::Result<MessageId> {
        // Verify both members exist
        {
            let members = self.members.read().await;
            if !members.contains_key(from) {
                anyhow::bail!("Sender {} not found in team", from);
            }
            if !members.contains_key(to) {
                anyhow::bail!("Recipient {} not found in team", to);
            }
        }

        let message = TeamMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: from.clone(),
            to: to.clone(),
            content,
            message_type,
            created_at: chrono::Utc::now(),
            task_id,
            metadata: HashMap::new(),
        };

        let message_id = message.id.clone();

        // Add to recipient's queue
        {
            let mut queue = self.message_queue.write().await;
            queue
                .entry(to.clone())
                .or_insert_with(Vec::new)
                .push(message);
        }

        tracing::debug!(
            team = %self.name,
            from = %from,
            to = %to,
            msg_id = %message_id,
            msg_type = ?message_type,
            "Team message sent"
        );

        Ok(message_id)
    }

    /// Send a broadcast message to all team members.
    pub async fn broadcast_message(
        &self,
        from: &AgentId,
        content: String,
    ) -> anyhow::Result<Vec<MessageId>> {
        let members = self.members.read().await;
        let mut message_ids = Vec::new();

        for member_id in members.keys() {
            if member_id != from {
                let id = self
                    .send_message(
                        from,
                        member_id,
                        content.clone(),
                        MessageType::Broadcast,
                        None,
                    )
                    .await?;
                message_ids.push(id);
            }
        }

        Ok(message_ids)
    }

    /// Get pending messages for a member.
    pub async fn get_messages(&self, agent_id: &AgentId) -> Vec<TeamMessage> {
        let mut queue = self.message_queue.write().await;
        queue
            .entry(agent_id.clone())
            .or_insert_with(Vec::new)
            .drain(..)
            .collect()
    }

    /// Check if a member has pending messages.
    pub async fn has_messages(&self, agent_id: &AgentId) -> bool {
        let queue = self.message_queue.read().await;
        queue.get(agent_id).map(|q| !q.is_empty()).unwrap_or(false)
    }

    /// Create a task on the task board.
    pub async fn create_task(
        &self,
        title: String,
        description: String,
        priority: TaskPriority,
        assigned_to: Option<AgentId>,
    ) -> anyhow::Result<TaskId> {
        let mut board = self.task_board.write().await;
        let id = format!("{}-{}", self.id, board.next_id);
        board.next_id += 1;

        let task = BoardTask {
            id: TaskId(id.clone()),
            title,
            description,
            assigned_to,
            status: BoardTaskStatus::Todo,
            priority,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            result: None,
            tags: Vec::new(),
        };

        board.tasks.insert(TaskId(id.clone()), task);

        tracing::info!(team = %self.name, task_id = %id, "Task created on team board");

        Ok(TaskId(id))
    }

    /// Assign a task to a team member.
    pub async fn assign_task(&self, task_id: &TaskId, agent_id: &AgentId) -> anyhow::Result<()> {
        // Verify member exists
        {
            let members = self.members.read().await;
            if !members.contains_key(agent_id) {
                anyhow::bail!("Agent {} not found in team", agent_id);
            }
        }

        let mut board = self.task_board.write().await;
        let task = board
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", task_id))?;

        task.assigned_to = Some(agent_id.clone());
        task.status = BoardTaskStatus::InProgress;
        task.updated_at = chrono::Utc::now();

        tracing::info!(
            team = %self.name,
            task_id = %task_id,
            assignee = %agent_id,
            "Task assigned"
        );

        Ok(())
    }

    /// Complete a task.
    pub async fn complete_task(&self, task_id: &TaskId, result: String) -> anyhow::Result<()> {
        let mut board = self.task_board.write().await;
        let task = board
            .tasks
            .get_mut(task_id)
            .ok_or_else(|| anyhow::anyhow!("Task {} not found", task_id))?;

        task.status = BoardTaskStatus::Done;
        task.result = Some(result);
        task.updated_at = chrono::Utc::now();

        // Update member's task count
        let assigned_agent_id = task.assigned_to.clone();
        drop(board); // Release write lock on board
        if let Some(ref agent_id) = assigned_agent_id {
            let mut members = self.members.write().await;
            if let Some(member) = members.get_mut(agent_id) {
                member.tasks_completed += 1;
                member.last_active_at = chrono::Utc::now();
            }
        }

        tracing::info!(team = %self.name, task_id = %task_id, "Task completed");

        Ok(())
    }

    /// Get all tasks.
    pub async fn get_tasks(&self) -> Vec<BoardTask> {
        let board = self.task_board.read().await;
        board.tasks.values().cloned().collect()
    }

    /// Get tasks assigned to a specific member.
    pub async fn get_member_tasks(&self, agent_id: &AgentId) -> Vec<BoardTask> {
        let board = self.task_board.read().await;
        board
            .tasks
            .values()
            .filter(|t| t.assigned_to.as_deref() == Some(agent_id))
            .cloned()
            .collect()
    }

    /// Get team statistics.
    pub async fn stats(&self) -> TeamStats {
        let members = self.members.read().await;
        let board = self.task_board.read().await;

        let total_tasks = board.tasks.len();
        let completed_tasks = board
            .tasks
            .values()
            .filter(|t| t.status == BoardTaskStatus::Done)
            .count();
        let in_progress_tasks = board
            .tasks
            .values()
            .filter(|t| t.status == BoardTaskStatus::InProgress)
            .count();

        TeamStats {
            team_id: self.id.clone(),
            team_name: self.name.clone(),
            member_count: members.len(),
            online_count: members
                .values()
                .filter(|m| m.status == MemberStatus::Online)
                .count(),
            busy_count: members
                .values()
                .filter(|m| m.status == MemberStatus::Busy)
                .count(),
            total_tasks,
            completed_tasks,
            in_progress_tasks,
            pending_tasks: total_tasks - completed_tasks - in_progress_tasks,
        }
    }

    /// Get the event bus handle.
    pub fn event_bus(&self) -> &EventBusHandle {
        &self.event_bus
    }
}

/// Team statistics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamStats {
    pub team_id: TeamId,
    pub team_name: String,
    pub member_count: usize,
    pub online_count: usize,
    pub busy_count: usize,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub in_progress_tasks: usize,
    pub pending_tasks: usize,
}

/// Team registry for managing multiple teams.
pub struct TeamRegistry {
    teams: Arc<RwLock<HashMap<TeamId, Arc<Team>>>>,
}

impl TeamRegistry {
    /// Create a new team registry.
    pub fn new() -> Self {
        Self {
            teams: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new team.
    pub async fn create_team(
        &self,
        name: String,
        coordinator_id: AgentId,
    ) -> anyhow::Result<Arc<Team>> {
        let team_id = uuid::Uuid::new_v4().to_string();
        let team = Arc::new(Team::new(
            team_id.clone(),
            name.clone(),
            coordinator_id.clone(),
        ));

        // Add coordinator as first member
        team.add_member(
            coordinator_id.clone(),
            "coordinator".to_string(),
            TeamRole::Coordinator,
        )
        .await?;

        let mut teams = self.teams.write().await;
        teams.insert(team_id.clone(), team.clone());

        tracing::info!(team_id = %team_id, name = %name, coordinator = %coordinator_id, "Team created");

        Ok(team)
    }

    /// Get a team by ID.
    pub async fn get_team(&self, team_id: &TeamId) -> Option<Arc<Team>> {
        let teams = self.teams.read().await;
        teams.get(team_id).cloned()
    }

    /// List all teams.
    pub async fn list_teams(&self) -> Vec<(TeamId, String)> {
        let teams = self.teams.read().await;
        teams
            .iter()
            .map(|(id, team)| (id.clone(), team.name.clone()))
            .collect()
    }

    /// Delete a team.
    pub async fn delete_team(&self, team_id: &TeamId) -> anyhow::Result<()> {
        let mut teams = self.teams.write().await;
        let removed = teams.remove(team_id);
        if removed.is_none() {
            anyhow::bail!("Team {} not found", team_id);
        }
        tracing::info!(team_id = %team_id, "Team deleted");
        Ok(())
    }
}

impl Default for TeamRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_team_creation() {
        let team = Team::new(
            "test-team".to_string(),
            "Coordinator Bot".to_string(),
            "coordinator-1".to_string(),
        );

        assert_eq!(team.name(), "Coordinator Bot");
        assert_eq!(team.coordinator_id(), "coordinator-1");
    }

    #[tokio::test]
    async fn test_add_member() {
        let team = Team::new(
            "test-team".to_string(),
            "Test Team".to_string(),
            "coordinator-1".to_string(),
        );

        team.add_member(
            "worker-1".to_string(),
            "Worker One".to_string(),
            TeamRole::Worker,
        )
        .await
        .unwrap();

        let members = team.members().await;
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "Worker One");
    }

    #[tokio::test]
    async fn test_send_message() {
        let team = Team::new(
            "test-team".to_string(),
            "Test Team".to_string(),
            "coordinator-1".to_string(),
        );

        team.add_member(
            "coordinator-1".to_string(),
            "Coordinator".to_string(),
            TeamRole::Coordinator,
        )
        .await
        .unwrap();
        team.add_member(
            "worker-1".to_string(),
            "Worker".to_string(),
            TeamRole::Worker,
        )
        .await
        .unwrap();

        let msg_id = team
            .send_message(
                &"coordinator-1".to_string(),
                &"worker-1".to_string(),
                "Hello worker!".to_string(),
                MessageType::Direct,
                None,
            )
            .await
            .unwrap();

        assert!(!msg_id.is_empty());
        assert!(team.has_messages(&"worker-1".to_string()).await);

        let messages = team.get_messages(&"worker-1".to_string()).await;
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello worker!");
    }

    #[tokio::test]
    async fn test_task_board() {
        let team = Team::new(
            "test-team".to_string(),
            "Test Team".to_string(),
            "coordinator-1".to_string(),
        );

        team.add_member(
            "worker-1".to_string(),
            "Worker".to_string(),
            TeamRole::Worker,
        )
        .await
        .unwrap();

        let task_id = team
            .create_task(
                "Fix bug".to_string(),
                "Fix the authentication bug".to_string(),
                TaskPriority::High,
                None,
            )
            .await
            .unwrap();

        team.assign_task(&task_id, &"worker-1".to_string())
            .await
            .unwrap();

        let tasks = team.get_member_tasks(&"worker-1".to_string()).await;
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].title, "Fix bug");

        team.complete_task(&task_id, "Bug fixed successfully".to_string())
            .await
            .unwrap();

        let stats = team.stats().await;
        assert_eq!(stats.completed_tasks, 1);
    }

    #[tokio::test]
    async fn test_team_registry() {
        let registry = TeamRegistry::new();

        let team = registry
            .create_team("Alpha Team".to_string(), "coordinator-1".to_string())
            .await
            .unwrap();

        assert_eq!(team.name(), "Alpha Team");

        let teams = registry.list_teams().await;
        assert_eq!(teams.len(), 1);

        registry.delete_team(team.id()).await.unwrap();
        assert!(registry.get_team(team.id()).await.is_none());
    }
}
