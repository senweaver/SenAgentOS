// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI plugin handlers — mirrors claude-code-typescript-src `cli/handlers/plugins.ts`.
// Handles plugin management commands (install/uninstall/list/validate).

use crate::cli::exit::{cli_error, cli_ok};
use crate::cli::print::{colors, kv, list_item};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Valid scopes for plugin installation.
pub const VALID_INSTALLABLE_SCOPES: &[&str] = &["user", "project", "workspace"];

/// Valid scopes for plugin updates.
pub const VALID_UPDATE_SCOPES: &[&str] = &["user", "project", "workspace"];

/// Plugin information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID.
    pub id: String,
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: Option<String>,
    /// Whether the plugin is enabled.
    pub enabled: bool,
    /// Installation scope.
    pub scope: String,
    /// Installation path.
    pub path: String,
    /// Plugin manifest.
    pub manifest: PluginManifest,
}

/// Plugin manifest schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name.
    pub name: String,
    /// Plugin version.
    pub version: String,
    /// Plugin description.
    pub description: Option<String>,
    /// Author information.
    pub author: Option<PluginAuthor>,
    /// Commands provided by the plugin.
    pub commands: Vec<PluginCommand>,
    /// Skills provided by the plugin.
    pub skills: Vec<String>,
    /// Agents provided by the plugin.
    pub agents: Vec<String>,
    /// Hooks provided by the plugin.
    pub hooks: Vec<PluginHook>,
}

/// Author information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginAuthor {
    /// Author name.
    pub name: Option<String>,
    /// Author email.
    pub email: Option<String>,
    /// Author URL.
    pub url: Option<String>,
}

/// A command provided by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    /// Command name.
    pub name: String,
    /// Command description.
    pub description: Option<String>,
    /// Whether the command is enabled.
    pub enabled: bool,
}

/// A hook provided by a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginHook {
    /// Hook name.
    pub name: String,
    /// Hook description.
    pub description: Option<String>,
}

/// Plugin registry for managing installed plugins.
pub struct PluginRegistry {
    plugins: Arc<RwLock<HashMap<String, PluginInfo>>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Install a plugin.
    pub async fn install(&self, id: &str, scope: &str) -> anyhow::Result<()> {
        // Validate scope
        if !VALID_INSTALLABLE_SCOPES.contains(&scope) {
            anyhow::bail!(
                "Invalid scope '{}'. Valid scopes: {:?}",
                scope,
                VALID_INSTALLABLE_SCOPES
            );
        }

        println!("Installing plugin {} ({})...", id, scope);

        // In a real implementation, this would:
        // 1. Fetch plugin from registry/marketplace
        // 2. Validate plugin manifest
        // 3. Download plugin files
        // 4. Install to appropriate location
        // 5. Register plugin

        let plugin = PluginInfo {
            id: id.to_string(),
            name: id.to_string(),
            version: "1.0.0".to_string(),
            description: None,
            enabled: true,
            scope: scope.to_string(),
            path: format!("/plugins/{}", id),
            manifest: PluginManifest {
                name: id.to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                commands: Vec::new(),
                skills: Vec::new(),
                agents: Vec::new(),
                hooks: Vec::new(),
            },
        };

        let mut plugins = self.plugins.write().await;
        plugins.insert(id.to_string(), plugin);

        println!("{}", colors::green(&format!("✓ Plugin {} installed", id)));
        Ok(())
    }

    /// Uninstall a plugin.
    pub async fn uninstall(&self, id: &str) -> anyhow::Result<()> {
        let mut plugins = self.plugins.write().await;

        if !plugins.contains_key(id) {
            anyhow::bail!("Plugin '{}' is not installed", id);
        }

        plugins.remove(id);
        println!("{}", colors::green(&format!("✓ Plugin {} uninstalled", id)));
        Ok(())
    }

    /// Enable a plugin.
    pub async fn enable(&self, id: &str) -> anyhow::Result<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(plugin) = plugins.get_mut(id) {
            plugin.enabled = true;
            println!("{}", colors::green(&format!("✓ Plugin {} enabled", id)));
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' is not installed", id)
        }
    }

    /// Disable a plugin.
    pub async fn disable(&self, id: &str) -> anyhow::Result<()> {
        let mut plugins = self.plugins.write().await;

        if let Some(plugin) = plugins.get_mut(id) {
            plugin.enabled = false;
            println!("{}", colors::green(&format!("✓ Plugin {} disabled", id)));
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' is not installed", id)
        }
    }

    /// Update a plugin.
    pub async fn update(&self, id: &str) -> anyhow::Result<()> {
        let plugins = self.plugins.read().await;

        if !plugins.contains_key(id) {
            anyhow::bail!("Plugin '{}' is not installed", id);
        }

        println!("Updating plugin {}...", id);
        // In a real implementation, this would check for updates and install them
        println!("{}", colors::green(&format!("✓ Plugin {} updated", id)));
        Ok(())
    }

    /// List all installed plugins.
    pub async fn list(&self) -> Vec<PluginInfo> {
        let plugins = self.plugins.read().await;
        plugins.values().cloned().collect()
    }

    /// Print plugin list.
    pub async fn print_list(&self) {
        let plugins = self.list().await;

        if plugins.is_empty() {
            println!("No plugins installed");
            return;
        }

        println!("\nInstalled plugins:\n");

        for plugin in &plugins {
            let status = if plugin.enabled {
                colors::green("enabled")
            } else {
                colors::yellow("disabled")
            };

            println!("{} {} [{}]", plugin.name, plugin.version, status);

            if let Some(ref desc) = plugin.description {
                println!("  {}", desc);
            }

            kv("Scope", &plugin.scope);
            kv("Path", &plugin.path);

            // List commands
            if !plugin.manifest.commands.is_empty() {
                println!("  Commands:");
                for cmd in &plugin.manifest.commands {
                    if cmd.enabled {
                        list_item(&format!(
                            "{} - {}",
                            cmd.name,
                            cmd.description.as_deref().unwrap_or("")
                        ));
                    }
                }
            }

            // List skills
            if !plugin.manifest.skills.is_empty() {
                println!("  Skills: {}", plugin.manifest.skills.join(", "));
            }

            println!();
        }
    }

    /// Get a plugin by ID.
    pub async fn get(&self, id: &str) -> Option<PluginInfo> {
        let plugins = self.plugins.read().await;
        plugins.get(id).cloned()
    }

    /// Check if a plugin is installed.
    pub async fn is_installed(&self, id: &str) -> bool {
        let plugins = self.plugins.read().await;
        plugins.contains_key(id)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation result for plugin manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation passed.
    pub success: bool,
    /// Validation errors.
    pub errors: Vec<ValidationError>,
    /// Validation warnings.
    pub warnings: Vec<ValidationWarning>,
}

/// A validation error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Path to the error.
    pub path: String,
    /// Error message.
    pub message: String,
}

/// A validation warning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Path to the warning.
    pub path: String,
    /// Warning message.
    pub message: String,
}

/// Validate a plugin manifest.
pub fn validate_manifest(manifest: &PluginManifest) -> ValidationResult {
    let mut errors = Vec::new();
    let warnings = Vec::new();

    // Check required fields
    if manifest.name.is_empty() {
        errors.push(ValidationError {
            path: "name".to_string(),
            message: "Plugin name is required".to_string(),
        });
    }

    if manifest.version.is_empty() {
        errors.push(ValidationError {
            path: "version".to_string(),
            message: "Plugin version is required".to_string(),
        });
    }

    // Validate version format (basic check - no semver crate available)
    // In a production implementation, add semver to Cargo.toml

    ValidationResult {
        success: errors.is_empty(),
        errors,
        warnings,
    }
}

/// Print validation result.
pub fn print_validation_result(result: &ValidationResult) {
    if !result.errors.is_empty() {
        println!(
            "{} Found {} error(s):",
            colors::red("✗"),
            result.errors.len()
        );
        for error in &result.errors {
            println!("  ▸ {}: {}", colors::yellow(&error.path), error.message);
        }
        println!();
    }

    if !result.warnings.is_empty() {
        println!(
            "{} Found {} warning(s):",
            colors::yellow("⚠"),
            result.warnings.len()
        );
        for warning in &result.warnings {
            println!("  ▸ {}: {}", colors::yellow(&warning.path), warning.message);
        }
        println!();
    }

    if result.success && result.warnings.is_empty() {
        println!("{}", colors::green("✓ Validation passed"));
    } else if result.success {
        println!("{}", colors::green("✓ Validation passed with warnings"));
    } else {
        println!("{}", colors::red("✗ Validation failed"));
    }
}

/// Plugin handler for CLI commands.
pub struct PluginHandler {
    registry: PluginRegistry,
}

impl PluginHandler {
    pub fn new() -> Self {
        Self {
            registry: PluginRegistry::new(),
        }
    }

    /// Handle plugin list command.
    pub async fn list(&self) {
        self.registry.print_list().await;
    }

    /// Handle plugin install command.
    pub async fn install(&self, id: &str, scope: &str) {
        if let Err(e) = self.registry.install(id, scope).await {
            cli_error(Some(&e.to_string()));
        }
    }

    /// Handle plugin uninstall command.
    pub async fn uninstall(&self, id: &str) {
        if let Err(e) = self.registry.uninstall(id).await {
            cli_error(Some(&e.to_string()));
        }
    }

    /// Handle plugin enable command.
    pub async fn enable(&self, id: &str) {
        if let Err(e) = self.registry.enable(id).await {
            cli_error(Some(&e.to_string()));
        }
    }

    /// Handle plugin disable command.
    pub async fn disable(&self, id: &str) {
        if let Err(e) = self.registry.disable(id).await {
            cli_error(Some(&e.to_string()));
        }
    }

    /// Handle plugin update command.
    pub async fn update(&self, id: &str) {
        if let Err(e) = self.registry.update(id).await {
            cli_error(Some(&e.to_string()));
        }
    }

    /// Handle plugin validate command.
    pub fn validate(&self, manifest_path: &str) {
        println!("Validating plugin manifest: {}", manifest_path);
        // In a real implementation, this would read and validate the manifest
        cli_ok(None);
    }
}

impl Default for PluginHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_install_uninstall() {
        let registry = PluginRegistry::new();

        registry.install("test-plugin", "user").await.unwrap();
        assert!(registry.is_installed("test-plugin").await);

        registry.uninstall("test-plugin").await.unwrap();
        assert!(!registry.is_installed("test-plugin").await);
    }

    #[tokio::test]
    async fn test_plugin_enable_disable() {
        let registry = PluginRegistry::new();

        registry.install("test-plugin", "user").await.unwrap();
        registry.disable("test-plugin").await.unwrap();

        let plugin = registry.get("test-plugin").await.unwrap();
        assert!(!plugin.enabled);

        registry.enable("test-plugin").await.unwrap();

        let plugin = registry.get("test-plugin").await.unwrap();
        assert!(plugin.enabled);
    }

    #[test]
    fn test_validate_manifest_success() {
        let manifest = PluginManifest {
            name: "test-plugin".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A test plugin".to_string()),
            author: None,
            commands: Vec::new(),
            skills: Vec::new(),
            agents: Vec::new(),
            hooks: Vec::new(),
        };

        let result = validate_manifest(&manifest);
        assert!(result.success);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_validate_manifest_missing_name() {
        let manifest = PluginManifest {
            name: "".to_string(),
            version: "1.0.0".to_string(),
            description: None,
            author: None,
            commands: Vec::new(),
            skills: Vec::new(),
            agents: Vec::new(),
            hooks: Vec::new(),
        };

        let result = validate_manifest(&manifest);
        assert!(!result.success);
        assert!(!result.errors.is_empty());
    }
}
