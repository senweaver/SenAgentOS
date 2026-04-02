// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// ServiceContainer — centralized service initialization and access.
// Wires all services ported from claude-code-typescript-srcinto a single dependency-injectable
// container that the agent core, commands, hooks, and TUI can consume.

use std::path::PathBuf;
use std::sync::{Arc, OnceLock};

use super::agent_summary::AgentSummaryService;
use super::analytics::AnalyticsService;
use super::auto_dream::AutoDreamService;
use super::compact::CompactService;
use super::extract_memories::ExtractionConfig;
use super::lsp::LspService;
use super::mcp_manager::McpManager;
use super::notifier::Notifier;
use super::oauth::OAuthService;
use super::plugin_service::PluginService;
use super::policy_limits::{PolicyLimitsService, PolicyRule};
use super::prompt_suggestion::PromptSuggestionService;
use super::rate_limit::RateLimiter;
use super::session_memory::SessionMemoryService;
use super::settings_sync::{ConflictStrategy, SettingsSyncService};
use super::team_memory_sync::TeamMemorySyncService;
use super::token_estimation::TokenEstimator;
use super::tool_use_summary::ToolUseSummaryService;

use crate::commands::registry::CommandRegistry;
use crate::tasks::runner::TaskRunner;

/// All services wired together for the agent runtime.
pub struct ServiceContainer {
    // -- Core services --
    pub analytics: AnalyticsService,
    pub compact: CompactService,
    pub lsp: LspService,
    pub mcp: McpManager,
    pub notifier: Notifier,
    pub oauth: OAuthService,
    pub rate_limiter: RateLimiter,
    pub session_memory: SessionMemoryService,
    pub token_estimator: TokenEstimator,

    // -- Services added from claude-code-typescript-srcaudit --
    pub agent_summary: AgentSummaryService,
    pub auto_dream: AutoDreamService,
    pub extraction_config: ExtractionConfig,
    pub plugin_service: PluginService,
    pub policy_limits: PolicyLimitsService,
    pub prompt_suggestion: PromptSuggestionService,
    pub settings_sync: SettingsSyncService,
    pub team_memory_sync: TeamMemorySyncService,
    pub tool_use_summary: Arc<std::sync::Mutex<ToolUseSummaryService>>,

    // -- Command & task systems --
    pub command_registry: CommandRegistry,
    pub task_runner: TaskRunner,
}

/// Configuration for building a ServiceContainer.
pub struct ServiceContainerConfig {
    pub data_dir: PathBuf,
    pub auto_dream_enabled: bool,
    pub team_sync_enabled: bool,
    pub policy_rules: Vec<PolicyRule>,
    pub conflict_strategy: ConflictStrategy,
}

impl Default for ServiceContainerConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from(".senagent"),
            auto_dream_enabled: false,
            team_sync_enabled: false,
            policy_rules: Vec::new(),
            conflict_strategy: ConflictStrategy::LastWriterWins,
        }
    }
}

impl ServiceContainer {
    /// Build and initialize all services.
    pub fn new(cfg: ServiceContainerConfig) -> Self {
        let sync_file = cfg.data_dir.join("settings_sync.json");

        // Build the command registry with all slash commands registered
        let command_registry = register_all_commands();

        Self {
            analytics: AnalyticsService::new(true),
            compact: CompactService,
            lsp: LspService::new(),
            mcp: McpManager::new(),
            notifier: Notifier::new(),
            oauth: OAuthService::new(),
            rate_limiter: RateLimiter::new(),
            session_memory: SessionMemoryService::new(),
            token_estimator: TokenEstimator::new(4.0),

            agent_summary: AgentSummaryService,
            auto_dream: AutoDreamService::new(cfg.auto_dream_enabled),
            extraction_config: ExtractionConfig::default(),
            plugin_service: PluginService::new(),
            policy_limits: PolicyLimitsService::new(cfg.policy_rules),
            prompt_suggestion: PromptSuggestionService,
            settings_sync: SettingsSyncService::new(sync_file, cfg.conflict_strategy),
            team_memory_sync: TeamMemorySyncService::new(cfg.team_sync_enabled),
            tool_use_summary: Arc::new(std::sync::Mutex::new(ToolUseSummaryService::new())),

            command_registry,
            task_runner: TaskRunner::new(),
        }
    }

    /// Check if a slash command exists.
    pub fn has_command(&self, name: &str) -> bool {
        self.command_registry.find(name).is_some()
    }

    /// Check if a tool is allowed by policy.
    pub fn check_tool_policy(&self, tool_name: &str) -> bool {
        self.policy_limits.check_tool(tool_name).allowed
    }

    /// Check if a model is allowed by policy.
    pub fn check_model_policy(&self, model_id: &str) -> bool {
        self.policy_limits.check_model(model_id).allowed
    }

    /// Check if spending is within limits (in USD cents).
    pub fn check_spending_policy(&self, current_cents: u64) -> bool {
        self.policy_limits.check_spending(current_cents).allowed
    }
}

// ---------------------------------------------------------------------------
// Global singleton (optional — for code that cannot take &ServiceContainer)
// ---------------------------------------------------------------------------

static GLOBAL_SERVICES: OnceLock<ServiceContainer> = OnceLock::new();

/// Initialize the global service container. Call once from main.
pub fn init_services(cfg: ServiceContainerConfig) -> &'static ServiceContainer {
    GLOBAL_SERVICES.get_or_init(|| ServiceContainer::new(cfg))
}

/// Access the global service container (panics if not initialized).
pub fn get_services() -> &'static ServiceContainer {
    GLOBAL_SERVICES
        .get()
        .expect("ServiceContainer not initialized — call init_services() first")
}

// ---------------------------------------------------------------------------
// Command registration — wires all command handlers
// ---------------------------------------------------------------------------

fn register_all_commands() -> CommandRegistry {
    use crate::commands::registry::{CommandCategory, SlashCommand};
    use std::sync::Arc;

    let mut registry = CommandRegistry::new();

    macro_rules! register_cmd {
        ($name:expr, $desc:expr, $usage:expr, $cat:expr, $handler:path) => {
            registry.register(SlashCommand {
                name: $name.to_string(),
                aliases: Vec::new(),
                description: $desc.to_string(),
                usage: $usage.to_string(),
                category: $cat,
                hidden: false,
                requires_interactive: false,
                remote_safe: true,
                handler: Arc::new(|ctx| Box::pin($handler(ctx))),
            });
        };
        ($name:expr, $desc:expr, $usage:expr, $cat:expr, $handler:path, interactive) => {
            registry.register(SlashCommand {
                name: $name.to_string(),
                aliases: Vec::new(),
                description: $desc.to_string(),
                usage: $usage.to_string(),
                category: $cat,
                hidden: false,
                requires_interactive: true,
                remote_safe: false,
                handler: Arc::new(|ctx| Box::pin($handler(ctx))),
            });
        };
    }

    use CommandCategory::*;

    register_cmd!("add-dir", "Add a directory to context", "/add-dir <path>", General, crate::commands::add_dir::handle);
    register_cmd!("clear", "Clear the terminal", "/clear", Session, crate::commands::clear::handle, interactive);
    register_cmd!("compact", "Compact conversation", "/compact [prompt]", Session, crate::commands::compact::handle);
    register_cmd!("config", "View or modify config", "/config <subcommand>", Configuration, crate::commands::config_cmd::handle);
    register_cmd!("context", "Show context usage", "/context", General, crate::commands::context::handle);
    register_cmd!("cost", "Show session cost", "/cost", General, crate::commands::cost::handle);
    register_cmd!("doctor", "Run diagnostics", "/doctor", Debug, crate::commands::doctor_cmd::handle);
    register_cmd!("help", "Show help", "/help [command]", General, crate::commands::help::handle);
    register_cmd!("history", "Manage conversation history", "/history <subcommand>", Session, crate::commands::history::handle);
    register_cmd!("memory", "Manage memories", "/memory <subcommand>", Memory, crate::commands::memory_cmd::handle);
    register_cmd!("model", "Switch or show model", "/model [name]", Configuration, crate::commands::model::handle);
    register_cmd!("plan", "Toggle plan mode", "/plan", Session, crate::commands::plan::handle);
    register_cmd!("plugin", "Manage plugins", "/plugin <subcommand>", Tools, crate::commands::plugin_cmd::handle);
    register_cmd!("resume", "Resume a session", "/resume [session_id]", Session, crate::commands::resume::handle);
    register_cmd!("skills", "Manage skills", "/skills <subcommand>", Skills, crate::commands::skills_cmd::handle);
    register_cmd!("status", "Show agent status", "/status", General, crate::commands::status::handle);
    register_cmd!("tasks", "Manage background tasks", "/tasks <subcommand>", Tasks, crate::commands::tasks_cmd::handle);
    register_cmd!("theme", "Change output theme", "/theme [name]", Configuration, crate::commands::theme::handle);
    register_cmd!("voice", "Toggle voice mode", "/voice", Session, crate::commands::voice_cmd::handle, interactive);

    registry
}
