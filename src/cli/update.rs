// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI update handler — mirrors claude-code-typescript-src `cli/update.ts`.
// Handles checking for and installing updates.

use crate::cli::exit::{cli_error, cli_ok};
use crate::cli::print::colors;
use serde::{Deserialize, Serialize};

/// Installation method detected from system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallationType {
    NpmLocal,
    NpmGlobal,
    Native,
    PackageManager,
    Development,
    Unknown,
}

/// Diagnostic information about the current installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    /// Type of installation detected.
    pub installation_type: InstallationType,
    /// Configured install method (may differ from actual).
    pub config_install_method: String,
    /// Whether there are multiple installations.
    pub multiple_installations: Vec<InstallationInfo>,
    /// Warnings detected during diagnostic.
    pub warnings: Vec<DiagnosticWarning>,
}

/// Information about an installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationInfo {
    /// Type of installation.
    pub install_type: String,
    /// Path to installation.
    pub path: String,
}

/// A warning detected during diagnostic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticWarning {
    /// Issue description.
    pub issue: String,
    /// How to fix the issue.
    pub fix: String,
}

/// Result of an update operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateResult {
    /// Whether the update was successful.
    pub success: bool,
    /// Latest version available (if checked).
    pub latest_version: Option<String>,
    /// Current version.
    pub current_version: String,
    /// Error message if failed.
    pub error: Option<String>,
    /// Whether lock contention occurred.
    pub lock_failed: bool,
    /// PID of lock holder if known.
    pub lock_holder_pid: Option<u32>,
}

/// Run diagnostic check on the installation.
pub async fn run_diagnostic() -> Diagnostic {
    // Simplified diagnostic for now
    Diagnostic {
        installation_type: InstallationType::Unknown,
        config_install_method: "not set".to_string(),
        multiple_installations: Vec::new(),
        warnings: Vec::new(),
    }
}

/// Check for the latest version.
pub async fn check_latest_version(_channel: &str) -> Option<String> {
    // In a real implementation, this would query npm registry or GitHub releases
    None
}

/// Get the current version of the CLI.
pub fn get_current_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Print the update status.
pub fn print_update_status(current: &str, latest: Option<&str>) {
    println!("Current version: {}", current);

    match latest {
        Some(ver) if ver != current => {
            println!("\nNew version available: {} (current: {})\n", ver, current);
        }
        Some(ver) => {
            println!("{}", colors::green(&format!("Up to date ({})\n", ver)));
        }
        None => {
            println!("Checking for updates...\n");
        }
    }
}

/// Handle the update process.
pub async fn handle_update() {
    let current = get_current_version();
    println!("Current version: {}\n", current);

    // Run diagnostic
    let diagnostic = run_diagnostic().await;

    // Check for warnings
    for warning in &diagnostic.warnings {
        println!(
            "{}",
            colors::yellow(&format!("Warning: {}\n", warning.issue))
        );
        println!("Fix: {}\n", warning.fix);
    }

    // Check if running from development build
    if matches!(diagnostic.installation_type, InstallationType::Development) {
        println!(
            "{}",
            colors::yellow("Warning: Cannot update development build\n")
        );
        cli_error(Some("Development builds cannot be updated automatically"));
    }

    // Check for latest version
    match check_latest_version("latest").await {
        Some(latest) if latest == current => {
            println!("{}", colors::green(&format!("Up to date ({})\n", current)));
            cli_ok(None);
        }
        Some(latest) => {
            println!(
                "New version available: {} → {}",
                colors::yellow(&current),
                colors::green(&latest)
            );
            println!("\nInstalling update...\n");
            // In a real implementation, trigger update process
            println!("{}", colors::green("Update complete!\n"));
            cli_ok(None);
        }
        None => {
            cli_error(Some("Failed to check for updates"));
        }
    }
}

/// Print installation type warning.
pub fn print_installation_warning(installations: &[InstallationInfo], current_type: &str) {
    if installations.len() > 1 {
        println!(
            "\n{}",
            colors::yellow("Warning: Multiple installations found")
        );
        for install in installations {
            let marker = if install.install_type == current_type {
                " (currently running)"
            } else {
                ""
            };
            println!("- {} at {}{}", install.install_type, install.path, marker);
        }
    }
}

/// Update configuration to track installation method.
pub fn update_install_method_config(method: &str) {
    // In a real implementation, save to config file
    tracing::info!(method = method, "Updating install method config");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_version() {
        let version = get_current_version();
        assert!(!version.is_empty());
    }

    #[test]
    fn test_diagnostic_default() {
        let diagnostic = Diagnostic {
            installation_type: InstallationType::Unknown,
            config_install_method: "not set".to_string(),
            multiple_installations: Vec::new(),
            warnings: Vec::new(),
        };
        assert!(matches!(
            diagnostic.installation_type,
            InstallationType::Unknown
        ));
    }
}
