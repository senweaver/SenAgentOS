// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// SkillManager — mirrors claude-code-typescript-src skill-related functionality.
// Provides skill discovery, registration, and execution management.

use super::traits::{Tool, ToolResult};
use crate::event_bus::EventBusHandle;
use crate::event_bus::types::{Event, EventPayload};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// A skill definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    /// Unique skill identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of what the skill does.
    pub description: String,
    /// Category for grouping.
    pub category: SkillCategory,
    /// Version string.
    pub version: String,
    /// Author information.
    pub author: Option<String>,
    /// Commands provided by this skill.
    pub commands: Vec<SkillCommand>,
    /// Tags for search.
    pub tags: Vec<String>,
    /// Whether the skill is enabled.
    pub enabled: bool,
}

/// Skill categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SkillCategory {
    Development,
    Infrastructure,
    Security,
    Data,
    Communication,
    Automation,
    Custom,
}

/// A command provided by a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCommand {
    /// Command name (e.g., "/commit", "/verify").
    pub name: String,
    /// Description of the command.
    pub description: String,
    /// Whether this command requires confirmation.
    pub requires_confirmation: bool,
    /// Example usage.
    pub example: Option<String>,
}

/// Skill discovery result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDiscoveryResult {
    pub skill: Skill,
    pub match_score: f32,
    pub matched_on: Vec<String>,
}

/// SkillManager handles skill discovery, registration, and execution.
/// Mirrors the TypeScript skill management functionality.
pub struct SkillManager {
    /// Registered skills by ID.
    skills: Arc<RwLock<HashMap<String, Skill>>>,
    /// Event bus for notifications.
    event_bus: Option<EventBusHandle>,
    /// Skills directory path.
    skills_dir: PathBuf,
}

impl SkillManager {
    /// Create a new SkillManager.
    pub fn new(skills_dir: PathBuf) -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
            event_bus: None,
            skills_dir,
        }
    }

    /// Set the event bus for notifications.
    pub fn with_event_bus(mut self, event_bus: EventBusHandle) -> Self {
        self.event_bus = Some(event_bus);
        self
    }

    /// Register a skill.
    pub async fn register(&self, skill: Skill) -> anyhow::Result<()> {
        let mut skills = self.skills.write().await;
        let skill_id = skill.id.clone();

        if skills.contains_key(&skill_id) {
            anyhow::bail!("Skill {} is already registered", skill_id);
        }

        skills.insert(skill_id.clone(), skill.clone());

        // Publish skill registered event
        if let Some(ref bus) = self.event_bus {
            let event = Event::new(
                "skill_manager",
                crate::event_bus::types::EventTarget::Broadcast,
                EventPayload::System {
                    category: crate::event_bus::types::SystemCategory::ConfigReload,
                    message: format!("Skill '{}' registered", skill.name),
                },
            );
            let _ = bus.publish(event).await;
        }

        tracing::info!(skill_id = %skill_id, name = %skill.name, "Skill registered");
        Ok(())
    }

    /// Unregister a skill.
    pub async fn unregister(&self, skill_id: &str) -> anyhow::Result<()> {
        let mut skills = self.skills.write().await;
        let removed = skills.remove(skill_id);

        if removed.is_none() {
            anyhow::bail!("Skill {} not found", skill_id);
        }

        tracing::info!(skill_id = %skill_id, "Skill unregistered");
        Ok(())
    }

    /// Get a skill by ID.
    pub async fn get(&self, skill_id: &str) -> Option<Skill> {
        let skills = self.skills.read().await;
        skills.get(skill_id).cloned()
    }

    /// List all registered skills.
    pub async fn list(&self) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.values().cloned().collect()
    }

    /// Get skills by category.
    pub async fn by_category(&self, category: SkillCategory) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills
            .values()
            .filter(|s| s.category == category)
            .cloned()
            .collect()
    }

    /// Search skills by query.
    pub async fn search(&self, query: &str) -> Vec<SkillDiscoveryResult> {
        let skills = self.skills.read().await;
        let query_lower = query.to_lowercase();
        let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

        let mut results: Vec<SkillDiscoveryResult> = Vec::new();

        for skill in skills.values() {
            let mut matched_on = Vec::new();
            let mut score = 0.0f32;

            // Check name
            if skill.name.to_lowercase().contains(&query_lower) {
                score += 3.0;
                matched_on.push("name".to_string());
            }

            // Check description
            if skill.description.to_lowercase().contains(&query_lower) {
                score += 2.0;
                matched_on.push("description".to_string());
            }

            // Check tags
            for tag in &skill.tags {
                if tag.to_lowercase().contains(&query_lower) {
                    score += 2.5;
                    matched_on.push(format!("tag:{}", tag));
                }
            }

            // Check category
            if format!("{:?}", skill.category)
                .to_lowercase()
                .contains(&query_lower)
            {
                score += 1.5;
                matched_on.push("category".to_string());
            }

            // Check commands
            for cmd in &skill.commands {
                if cmd.name.to_lowercase().contains(&query_lower) {
                    score += 2.0;
                    matched_on.push(format!("command:{}", cmd.name));
                }
            }

            // Check individual terms
            for term in &query_terms {
                if skill.description.to_lowercase().contains(term) {
                    score += 0.5;
                }
            }

            if score > 0.0 {
                results.push(SkillDiscoveryResult {
                    skill: skill.clone(),
                    match_score: score,
                    matched_on,
                });
            }
        }

        // Sort by score descending
        results.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());
        results
    }

    /// Enable a skill.
    pub async fn enable(&self, skill_id: &str) -> anyhow::Result<()> {
        let mut skills = self.skills.write().await;
        let skill = skills
            .get_mut(skill_id)
            .ok_or_else(|| anyhow::anyhow!("Skill {} not found", skill_id))?;

        skill.enabled = true;
        tracing::info!(skill_id = %skill_id, "Skill enabled");
        Ok(())
    }

    /// Disable a skill.
    pub async fn disable(&self, skill_id: &str) -> anyhow::Result<()> {
        let mut skills = self.skills.write().await;
        let skill = skills
            .get_mut(skill_id)
            .ok_or_else(|| anyhow::anyhow!("Skill {} not found", skill_id))?;

        skill.enabled = false;
        tracing::info!(skill_id = %skill_id, "Skill disabled");
        Ok(())
    }

    /// Discover skills from the skills directory.
    pub async fn discover_skills(&self) -> anyhow::Result<Vec<Skill>> {
        let mut discovered = Vec::new();

        if !self.skills_dir.exists() {
            tracing::debug!(dir = %self.skills_dir.display(), "Skills directory does not exist");
            return Ok(discovered);
        }

        let mut entries = fs::read_dir(&self.skills_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                // Check for skill.toml or skill.json
                let manifest_path = path.join("skill.toml");
                let json_manifest = path.join("skill.json");

                let skill = if manifest_path.exists() {
                    let content = fs::read_to_string(&manifest_path).await?;
                    toml::from_str::<Skill>(&content)?
                } else if json_manifest.exists() {
                    let content = fs::read_to_string(&json_manifest).await?;
                    serde_json::from_str::<Skill>(&content)?
                } else {
                    continue;
                };

                discovered.push(skill);
            }
        }

        tracing::info!(
            count = discovered.len(),
            dir = %self.skills_dir.display(),
            "Discovered skills"
        );

        Ok(discovered)
    }

    /// Get all enabled skills.
    pub async fn enabled_skills(&self) -> Vec<Skill> {
        let skills = self.skills.read().await;
        skills.values().filter(|s| s.enabled).cloned().collect()
    }

    /// Get commands from all enabled skills.
    pub async fn available_commands(&self) -> Vec<SkillCommand> {
        let skills = self.enabled_skills().await;
        skills.into_iter().flat_map(|s| s.commands).collect()
    }
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new(PathBuf::from("skills"))
    }
}

/// SkillTool wraps SkillManager as a Tool for LLM use.
pub struct SkillDiscoveryTool {
    manager: Arc<SkillManager>,
}

impl SkillDiscoveryTool {
    /// Create a new SkillDiscoveryTool.
    pub fn new(manager: Arc<SkillManager>) -> Self {
        Self { manager }
    }
}

#[async_trait]
impl Tool for SkillDiscoveryTool {
    fn name(&self) -> &str {
        "SkillTool"
    }

    fn description(&self) -> &str {
        "Search and discover available skills for various tasks. Use this to find relevant skills for development, infrastructure, security, and automation tasks."
    }

    fn parameters_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query to find relevant skills"
                },
                "category": {
                    "type": "string",
                    "description": "Filter by category (development, infrastructure, security, data, communication, automation)"
                }
            },
            "required": ["query"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");

        let category_filter = args.get("category").and_then(|v| v.as_str());

        let results = self.manager.search(query).await;

        // Filter by category if specified
        let results = if let Some(cat) = category_filter {
            let cat_lower = cat.to_lowercase();
            results
                .into_iter()
                .filter(|r| {
                    format!("{:?}", r.skill.category)
                        .to_lowercase()
                        .contains(&cat_lower)
                })
                .collect()
        } else {
            results
        };

        let output = if results.is_empty() {
            format!("No skills found matching '{}'", query)
        } else {
            let formatted: Vec<String> = results
                .iter()
                .take(10)
                .map(|r| {
                    format!(
                        "- {} ({}): {}",
                        r.skill.name,
                        format!("{:?}", r.skill.category).to_lowercase(),
                        r.skill.description
                    )
                })
                .collect();
            format!(
                "Found {} skill(s):\n{}",
                results.len(),
                formatted.join("\n")
            )
        };

        Ok(ToolResult {
            success: true,
            output,
            error: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_skill() -> Skill {
        Skill {
            id: "test-skill".to_string(),
            name: "Test Skill".to_string(),
            description: "A test skill for unit testing".to_string(),
            category: SkillCategory::Development,
            version: "1.0.0".to_string(),
            author: Some("Test Author".to_string()),
            commands: vec![SkillCommand {
                name: "/test".to_string(),
                description: "Run tests".to_string(),
                requires_confirmation: false,
                example: Some("/test run".to_string()),
            }],
            tags: vec!["test".to_string(), "unit".to_string()],
            enabled: true,
        }
    }

    #[tokio::test]
    async fn test_skill_registration() {
        let manager = SkillManager::default();
        let skill = test_skill();

        manager.register(skill.clone()).await.unwrap();

        let retrieved = manager.get(&skill.id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Skill");
    }

    #[tokio::test]
    async fn test_skill_search() {
        let manager = SkillManager::default();
        let skill = test_skill();
        manager.register(skill).await.unwrap();

        let results = manager.search("test").await;
        assert!(!results.is_empty());
        assert_eq!(results[0].skill.name, "Test Skill");
    }

    #[tokio::test]
    async fn test_skill_enable_disable() {
        let manager = SkillManager::default();
        let skill = test_skill();
        manager.register(skill).await.unwrap();

        manager.disable("test-skill").await.unwrap();
        assert!(!manager.get("test-skill").await.unwrap().enabled);

        manager.enable("test-skill").await.unwrap();
        assert!(manager.get("test-skill").await.unwrap().enabled);
    }
}
