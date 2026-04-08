// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI Agents Handler — mirrors claude-code-typescript-src `cli/handlers/agents.ts`.
// Provides agent listing and management commands.

use crate::agent::registry::AgentInfo;
use crate::agent::registry::AgentRegistry;
use crate::agent::registry::AgentState;
use serde::{Deserialize, Serialize};

/// Agent display information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDisplayInfo {
    /// Agent ID.
    pub id: String,
    /// Agent name.
    pub name: String,
    /// Agent type.
    pub agent_type: String,
    /// Model being used.
    pub model: Option<String>,
    /// Memory configuration.
    pub memory: Option<String>,
    /// Source of the agent definition.
    pub source: AgentSource,
    /// Whether this agent is overridden.
    pub overridden_by: Option<String>,
    /// Current state.
    pub state: AgentState,
}

impl From<AgentInfo> for AgentDisplayInfo {
    fn from(info: AgentInfo) -> Self {
        Self {
            id: info.id.clone(),
            name: info.name.clone(),
            agent_type: info.role.clone(),
            model: None,
            memory: None,
            source: AgentSource::Builtin,
            overridden_by: None,
            state: AgentState::Idle,
        }
    }
}

/// Source of agent definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSource {
    /// Built-in agent.
    Builtin,
    /// Agent from project config.
    Project,
    /// Agent from global config.
    Global,
    /// Agent from agents directory.
    Directory,
    /// Agent from CLI override.
    Cli,
}

/// Format an agent for display.
pub fn format_agent(agent: &AgentDisplayInfo) -> String {
    let mut parts = vec![agent.agent_type.clone()];

    if let Some(ref model) = agent.model {
        parts.push(model.clone());
    }

    if let Some(ref memory) = agent.memory {
        parts.push(format!("{} memory", memory));
    }

    parts.join(" · ")
}

/// Get a label for the agent source.
pub fn get_source_label(source: AgentSource) -> &'static str {
    match source {
        AgentSource::Builtin => "Built-in",
        AgentSource::Project => "Project",
        AgentSource::Global => "Global",
        AgentSource::Directory => "Directory",
        AgentSource::Cli => "CLI Override",
    }
}

/// Compare agents by name for sorting.
pub fn compare_agents_by_name(a: &AgentDisplayInfo, b: &AgentDisplayInfo) -> std::cmp::Ordering {
    a.name.to_lowercase().cmp(&b.name.to_lowercase())
}

/// Agent list output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentListOutput {
    /// Total active agents count.
    pub total_active: usize,
    /// Groups of agents by source.
    pub groups: Vec<AgentGroup>,
}

/// A group of agents from the same source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGroup {
    /// Source label.
    pub label: String,
    /// Source of the agents.
    pub source: AgentSource,
    /// Agents in this group.
    pub agents: Vec<AgentDisplayInfo>,
}

/// Build agent list output from registry.
pub fn build_agent_list_output(registry: &AgentRegistry) -> AgentListOutput {
    let all_agents: Vec<AgentDisplayInfo> = registry
        .all()
        .into_iter()
        .map(AgentDisplayInfo::from)
        .collect();

    // Group agents by source
    let mut builtin = Vec::new();
    let mut project = Vec::new();
    let mut global = Vec::new();
    let mut directory = Vec::new();
    let mut cli = Vec::new();

    for agent in &all_agents {
        match agent.source {
            AgentSource::Builtin => builtin.push(agent.clone()),
            AgentSource::Project => project.push(agent.clone()),
            AgentSource::Global => global.push(agent.clone()),
            AgentSource::Directory => directory.push(agent.clone()),
            AgentSource::Cli => cli.push(agent.clone()),
        }
    }

    let mut groups = Vec::new();

    if !builtin.is_empty() {
        builtin.sort_by(compare_agents_by_name);
        groups.push(AgentGroup {
            label: "Built-in".to_string(),
            source: AgentSource::Builtin,
            agents: builtin,
        });
    }

    if !project.is_empty() {
        project.sort_by(compare_agents_by_name);
        groups.push(AgentGroup {
            label: "Project".to_string(),
            source: AgentSource::Project,
            agents: project,
        });
    }

    if !global.is_empty() {
        global.sort_by(compare_agents_by_name);
        groups.push(AgentGroup {
            label: "Global".to_string(),
            source: AgentSource::Global,
            agents: global,
        });
    }

    if !directory.is_empty() {
        directory.sort_by(compare_agents_by_name);
        groups.push(AgentGroup {
            label: "Directory".to_string(),
            source: AgentSource::Directory,
            agents: directory,
        });
    }

    if !cli.is_empty() {
        cli.sort_by(compare_agents_by_name);
        groups.push(AgentGroup {
            label: "CLI".to_string(),
            source: AgentSource::Cli,
            agents: cli,
        });
    }

    let total_active = all_agents.len();

    AgentListOutput {
        total_active,
        groups,
    }
}

/// Format agent list as text output.
pub fn format_agent_list_text(output: &AgentListOutput) -> String {
    if output.groups.is_empty() {
        return "No agents found.".to_string();
    }

    let mut lines = vec![format!("{} active agent(s)\n", output.total_active)];

    for group in &output.groups {
        lines.push(format!("{}:", group.label));

        for agent in &group.agents {
            let formatted = format_agent(agent);
            let shadow = if agent.overridden_by.is_some() {
                format!("  (shadowed by CLI) {}", formatted)
            } else {
                format!("  {}", formatted)
            };
            lines.push(shadow);
        }

        lines.push(String::new());
    }

    lines.join("\n")
}

/// CLI agents handler.
/// Mirrors the TypeScript agentsHandler function.
pub struct AgentsHandler {
    registry: AgentRegistry,
}

impl AgentsHandler {
    /// Create a new agents handler.
    pub fn new(registry: AgentRegistry) -> Self {
        Self { registry }
    }

    /// Handle the agents list command.
    pub fn list(&self) -> AgentListOutput {
        build_agent_list_output(&self.registry)
    }

    /// Format and print the agent list.
    pub fn print_list(&self) -> String {
        let output = self.list();
        format_agent_list_text(&output)
    }

    /// Get a specific agent by ID.
    pub fn get(&self, id: &str) -> Option<AgentDisplayInfo> {
        self.registry.get(id).map(AgentDisplayInfo::from)
    }

    /// Get agents by state.
    pub fn get_by_state(&self, state: AgentState) -> Vec<AgentDisplayInfo> {
        self.registry
            .all()
            .into_iter()
            .filter(|a| a.state == state)
            .map(AgentDisplayInfo::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_agent() {
        let agent = AgentDisplayInfo {
            id: "test-1".to_string(),
            name: "Test Agent".to_string(),
            agent_type: "worker".to_string(),
            model: Some("claude-3-opus".to_string()),
            memory: Some("32k".to_string()),
            source: AgentSource::Builtin,
            overridden_by: None,
            state: AgentState::Idle,
        };

        let formatted = format_agent(&agent);
        assert!(formatted.contains("worker"));
        assert!(formatted.contains("claude-3-opus"));
        assert!(formatted.contains("32k memory"));
    }

    #[test]
    fn test_compare_agents_by_name() {
        let agent1 = AgentDisplayInfo {
            id: "1".to_string(),
            name: "Alpha".to_string(),
            agent_type: "worker".to_string(),
            model: None,
            memory: None,
            source: AgentSource::Builtin,
            overridden_by: None,
            state: AgentState::Idle,
        };

        let agent2 = AgentDisplayInfo {
            id: "2".to_string(),
            name: "Beta".to_string(),
            agent_type: "worker".to_string(),
            model: None,
            memory: None,
            source: AgentSource::Builtin,
            overridden_by: None,
            state: AgentState::Idle,
        };

        assert_eq!(
            compare_agents_by_name(&agent1, &agent2),
            std::cmp::Ordering::Less
        );
    }

    #[test]
    fn test_get_source_label() {
        assert_eq!(get_source_label(AgentSource::Builtin), "Built-in");
        assert_eq!(get_source_label(AgentSource::Project), "Project");
        assert_eq!(get_source_label(AgentSource::Global), "Global");
        assert_eq!(get_source_label(AgentSource::Directory), "Directory");
        assert_eq!(get_source_label(AgentSource::Cli), "CLI Override");
    }
}
