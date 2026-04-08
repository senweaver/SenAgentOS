// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Fork Subagent — mirrors claude-code-typescript-src `tools/AgentTool/forkSubagent.ts`.
// Enables implicit forking where a child agent inherits parent's conversation context.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Fork subagent feature flag name.
pub const FORK_FEATURE_FLAG: &str = "FORK_SUBAGENT";

/// Synthetic agent type for fork subagents.
pub const FORK_SUBAGENT_TYPE: &str = "fork";

/// Fork boilerplate XML tag for conversation tracking.
pub const FORK_BOILERPLATE_TAG: &str = "fork-boilerplate";

/// Fork directive prefix for identifying fork requests.
pub const FORK_DIRECTIVE_PREFIX: &str = "fork:";

/// Placeholder text for tool_result blocks in fork prefix.
/// Must be identical across all fork children for prompt cache sharing.
pub const FORK_PLACEHOLDER_RESULT: &str = "Fork started — processing in background";

/// Fork subagent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkAgentConfig {
    /// Maximum turns for the fork child.
    pub max_turns: usize,
    /// Permission mode for the fork child.
    pub permission_mode: PermissionMode,
    /// Model to use (inherit = use parent's model).
    pub model: String,
}

impl Default for ForkAgentConfig {
    fn default() -> Self {
        Self {
            max_turns: 200,
            permission_mode: PermissionMode::Bubble,
            model: "inherit".to_string(),
        }
    }
}

/// Permission mode for subagents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PermissionMode {
    /// Bubble permission prompts to parent terminal.
    Bubble,
    /// Use default permission handling.
    Default,
}

/// Check if fork subagent feature is enabled.
pub fn is_fork_subagent_enabled(is_coordinator_mode: bool, is_non_interactive: bool) -> bool {
    if is_coordinator_mode {
        return false;
    }
    if is_non_interactive {
        return false;
    }
    true
}

/// Guard against recursive forking by detecting the fork boilerplate tag.
pub fn is_in_fork_child(content: &str) -> bool {
    content.contains(&format!("<{}>", FORK_BOILERPLATE_TAG))
}

/// Build the forked conversation messages for the child agent.
/// For prompt cache sharing, all fork children must produce byte-identical
/// API request prefixes.
pub fn build_forked_messages(directive: &str) -> Vec<ForkMessage> {
    let mut messages = Vec::new();

    // Build user message with fork directive
    let user_content = build_child_message(directive);
    messages.push(ForkMessage {
        role: "user".to_string(),
        content: user_content,
    });

    messages
}

/// Build the user message content for a fork child.
pub fn build_child_message(directive: &str) -> String {
    format!(
        r#"<{}>
<directive>
{}
</directive>
"#,
        FORK_BOILERPLATE_TAG, directive
    )
}

/// A simplified message for fork context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkMessage {
    pub role: String,
    pub content: String,
}

/// Fork agent definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkAgent {
    /// Agent type name.
    pub agent_type: String,
    /// Description of when to use this agent.
    pub when_to_use: String,
    /// Tools that are allowed.
    pub tools: Vec<String>,
    /// Maximum turns.
    pub max_turns: usize,
    /// Model to use.
    pub model: String,
    /// Permission mode.
    pub permission_mode: String,
    /// Source of the agent.
    pub source: String,
}

impl ForkAgent {
    /// Create a new fork agent.
    pub fn new() -> Self {
        Self {
            agent_type: FORK_SUBAGENT_TYPE.to_string(),
            when_to_use: "Implicit fork — inherits full conversation context. Not selectable via subagent_type; triggered by omitting subagent_type when the fork experiment is active.".to_string(),
            tools: vec!["*".to_string()],
            max_turns: 200,
            model: "inherit".to_string(),
            permission_mode: "bubble".to_string(),
            source: "built-in".to_string(),
        }
    }
}

impl Default for ForkAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// Fork agent registry for managing fork sessions.
pub struct ForkAgentRegistry {
    sessions: Arc<RwLock<std::collections::HashMap<String, ForkSession>>>,
}

impl ForkAgentRegistry {
    /// Create a new fork registry.
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Register a new fork session.
    pub async fn register(&self, session: ForkSession) {
        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session);
    }

    /// Get a fork session by ID.
    pub async fn get(&self, id: &str) -> Option<ForkSession> {
        let sessions = self.sessions.read().await;
        sessions.get(id).cloned()
    }

    /// Remove a fork session.
    pub async fn remove(&self, id: &str) {
        let mut sessions = self.sessions.write().await;
        sessions.remove(id);
    }

    /// List all active fork sessions.
    pub async fn list(&self) -> Vec<ForkSession> {
        let sessions = self.sessions.read().await;
        sessions.values().cloned().collect()
    }

    /// Get the count of active fork sessions.
    pub async fn count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }
}

impl Default for ForkAgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A fork session representing an active forked subagent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkSession {
    /// Unique session ID.
    pub id: String,
    /// Parent session ID.
    pub parent_id: String,
    /// Directive for this fork.
    pub directive: String,
    /// Status of the fork session.
    pub status: ForkStatus,
    /// When the session was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp.
    pub last_activity: chrono::DateTime<chrono::Utc>,
}

impl ForkSession {
    /// Create a new fork session.
    pub fn new(parent_id: String, directive: String) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id,
            directive,
            status: ForkStatus::Running,
            created_at: now,
            last_activity: now,
        }
    }

    /// Mark the session as completed.
    pub fn complete(&mut self) {
        self.status = ForkStatus::Completed;
        self.last_activity = chrono::Utc::now();
    }

    /// Mark the session as failed.
    pub fn fail(&mut self) {
        self.status = ForkStatus::Failed;
        self.last_activity = chrono::Utc::now();
    }
}

/// Status of a fork session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForkStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fork_feature_enabled() {
        assert!(is_fork_subagent_enabled(false, false));
        assert!(!is_fork_subagent_enabled(true, false));
        assert!(!is_fork_subagent_enabled(false, true));
    }

    #[test]
    fn test_is_in_fork_child() {
        assert!(is_in_fork_child(
            "<fork-boilerplate>content</fork-boilerplate>"
        ));
        assert!(!is_in_fork_child("normal message"));
    }

    #[test]
    fn test_build_child_message() {
        let msg = build_child_message("analyze this code");
        assert!(msg.contains(FORK_BOILERPLATE_TAG));
        assert!(msg.contains("analyze this code"));
    }

    #[test]
    fn test_fork_session_lifecycle() {
        let mut session = ForkSession::new("parent-1".to_string(), "do something".to_string());
        assert_eq!(session.status, ForkStatus::Running);

        session.complete();
        assert_eq!(session.status, ForkStatus::Completed);

        let mut session = ForkSession::new("parent-2".to_string(), "fail this".to_string());
        session.fail();
        assert_eq!(session.status, ForkStatus::Failed);
    }

    #[tokio::test]
    async fn test_fork_registry() {
        let registry = ForkAgentRegistry::new();

        let session = ForkSession::new("parent-1".to_string(), "test".to_string());
        let id = session.id.clone();
        registry.register(session).await;

        assert_eq!(registry.count().await, 1);

        let retrieved = registry.get(&id).await;
        assert!(retrieved.is_some());

        registry.remove(&id).await;
        assert_eq!(registry.count().await, 0);
    }
}
