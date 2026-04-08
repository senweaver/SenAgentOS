// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Explore Agent — mirrors claude-code-typescript-src `tools/AgentTool/built-in/exploreAgent.ts`.
// A fast, read-only file search specialist agent.

use serde::{Deserialize, Serialize};

/// Minimum number of queries for explore agent to be useful.
pub const EXPLORE_AGENT_MIN_QUERIES: usize = 3;

/// Explore agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreAgentConfig {
    /// Thoroughness level for the search.
    pub thoroughness: ExploreThoroughness,
    /// Whether to use embedded search tools (find/grep vs glob/grep).
    pub use_embedded_tools: bool,
}

impl Default for ExploreAgentConfig {
    fn default() -> Self {
        Self {
            thoroughness: ExploreThoroughness::Medium,
            use_embedded_tools: false,
        }
    }
}

/// Thoroughness level for explore agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExploreThoroughness {
    /// Quick basic search.
    Quick,
    /// Moderate exploration.
    Medium,
    /// Comprehensive analysis.
    Thorough,
}

impl ExploreThoroughness {
    /// Get the description for this thoroughness level.
    pub fn description(&self) -> &'static str {
        match self {
            ExploreThoroughness::Quick => "quick",
            ExploreThoroughness::Medium => "medium",
            ExploreThoroughness::Thorough => "very thorough",
        }
    }
}

/// Build the system prompt for explore agent.
pub fn build_explore_agent_system_prompt(use_embedded_tools: bool) -> String {
    let glob_guidance = if use_embedded_tools {
        "- Use `find` via BashTool for broad file pattern matching"
    } else {
        "- Use GlobTool for broad file pattern matching"
    };

    let grep_guidance = if use_embedded_tools {
        "- Use `grep` via BashTool for searching file contents with regex"
    } else {
        "- Use GrepTool for searching file contents with regex"
    };

    format!(
        r#"You are a file search specialist for SenAgentOS, an autonomous agent runtime. You excel at thoroughly navigating and exploring codebases.

=== CRITICAL: READ-ONLY MODE - NO FILE MODIFICATIONS ===
This is a READ-ONLY exploration task. You are STRICTLY PROHIBITED from:
- Creating new files (no Write, touch, or file creation of any kind)
- Modifying existing files (no Edit operations)
- Deleting files (no rm or deletion)
- Moving or copying files (no mv or cp)
- Creating temporary files anywhere, including /tmp
- Using redirect operators (>, >>, |) or heredocs to write to files
- Running ANY commands that change system state

Your role is EXCLUSIVELY to search and analyze existing code. You do NOT have access to file editing tools - attempting to edit files will fail.

Your strengths:
- Rapidly finding files using glob patterns
- Searching code and text with powerful regex patterns
- Reading and analyzing file contents

Guidelines:
{}
{}
- Use FileReadTool when you know the specific file path you need to read
- Use BashTool ONLY for read-only operations (ls, git status, git log, git diff, find, cat, head, tail)
- NEVER use BashTool for: mkdir, touch, rm, cp, mv, git add, git commit, npm install, pip install, or any file creation/modification
- Adapt your search approach based on the thoroughness level specified by the caller
- Communicate your final report directly as a regular message - do NOT attempt to create files

NOTE: You are meant to be a fast agent that returns output as quickly as possible. In order to achieve this you must:
- Make efficient use of the tools that you have at your disposal: be smart about how you search for files and implementations
- Wherever possible you should try to spawn multiple parallel tool calls for grepping and reading files

Complete the user's search request efficiently and report your findings clearly."#,
        glob_guidance, grep_guidance
    )
}

/// Explore agent definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploreAgent {
    /// Agent type name.
    pub agent_type: String,
    /// Description of when to use this agent.
    pub when_to_use: String,
    /// Tools that are disallowed for this agent.
    pub disallowed_tools: Vec<String>,
    /// Model to use (inherit from parent or specific model).
    pub model: String,
    /// Whether to omit CLAUDE.md rules.
    pub omit_claude_md: bool,
    /// Feature flag for this agent.
    pub feature_flag: Option<String>,
}

impl ExploreAgent {
    /// Create a new explore agent.
    pub fn new() -> Self {
        Self {
            agent_type: "Explore".to_string(),
            when_to_use: EXPLORE_WHEN_TO_USE.to_string(),
            disallowed_tools: vec![
                "AgentTool".to_string(),
                "ExitModeTool".to_string(),
                "FileEditTool".to_string(),
                "FileWriteTool".to_string(),
                "NotebookEditTool".to_string(),
            ],
            model: "haiku".to_string(),
            omit_claude_md: true,
            feature_flag: None,
        }
    }

    /// Get the system prompt for this agent.
    pub fn get_system_prompt(&self, use_embedded_tools: bool) -> String {
        build_explore_agent_system_prompt(use_embedded_tools)
    }

    /// Check if this agent should use the parent's model.
    pub fn inherits_model(&self) -> bool {
        self.model == "inherit"
    }
}

impl Default for ExploreAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// When to use guidance for explore agent.
pub const EXPLORE_WHEN_TO_USE: &str = "Fast agent specialized for exploring codebases. Use this when you need to quickly find files by patterns (eg. 'src/components/**/*.tsx'), search code for keywords (eg. 'API endpoints'), or answer questions about the codebase (eg. 'how do API endpoints work?'). When calling this agent, specify the desired thoroughness level: 'quick' for basic searches, 'medium' for moderate exploration, or 'very thorough' for comprehensive analysis across multiple locations and naming conventions.";

/// Explore agent tool for spawning explore subagents.
pub struct ExploreAgentTool {
    /// Default configuration.
    config: ExploreAgentConfig,
}

impl ExploreAgentTool {
    /// Create a new explore agent tool.
    pub fn new() -> Self {
        Self {
            config: ExploreAgentConfig::default(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: ExploreAgentConfig) -> Self {
        Self { config }
    }

    /// Get the explore agent definition.
    pub fn get_agent(&self) -> ExploreAgent {
        ExploreAgent::new()
    }

    /// Get the system prompt based on current config.
    pub fn get_system_prompt(&self) -> String {
        self.get_agent()
            .get_system_prompt(self.config.use_embedded_tools)
    }
}

impl Default for ExploreAgentTool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explore_system_prompt_without_embedded() {
        let prompt = build_explore_agent_system_prompt(false);
        assert!(prompt.contains("GlobTool"));
        assert!(prompt.contains("GrepTool"));
        assert!(prompt.contains("READ-ONLY"));
    }

    #[test]
    fn test_explore_system_prompt_with_embedded() {
        let prompt = build_explore_agent_system_prompt(true);
        assert!(prompt.contains("find"));
        assert!(prompt.contains("grep"));
        assert!(prompt.contains("READ-ONLY"));
    }

    #[test]
    fn test_thoroughness_description() {
        assert_eq!(ExploreThoroughness::Quick.description(), "quick");
        assert_eq!(ExploreThoroughness::Medium.description(), "medium");
        assert_eq!(ExploreThoroughness::Thorough.description(), "very thorough");
    }

    #[test]
    fn test_explore_agent_creation() {
        let agent = ExploreAgent::new();
        assert_eq!(agent.agent_type, "Explore");
        assert!(agent.disallowed_tools.contains(&"FileEditTool".to_string()));
        assert_eq!(agent.model, "haiku");
        assert!(agent.omit_claude_md);
    }

    #[test]
    fn test_explore_agent_inherits_model() {
        let agent = ExploreAgent::new();
        assert!(!agent.inherits_model());
    }
}
