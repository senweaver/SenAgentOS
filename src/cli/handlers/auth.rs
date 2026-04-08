// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI auth handler — mirrors claude-code-typescript-src `cli/handlers/auth.ts`.
// Handles authentication commands (login/logout/status).

use crate::cli::exit::{cli_error, cli_ok};
use crate::cli::print::{colors, kv};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Authentication status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthStatus {
    /// Whether user is authenticated.
    pub authenticated: bool,
    /// Account email if authenticated.
    pub email: Option<String>,
    /// Account UUID if authenticated.
    pub account_id: Option<String>,
    /// Organization ID if authenticated.
    pub organization_id: Option<String>,
    /// Subscription type.
    pub subscription_type: Option<String>,
    /// Auth source (api_key, oauth, etc.).
    pub auth_source: Option<String>,
}

impl Default for AuthStatus {
    fn default() -> Self {
        Self {
            authenticated: false,
            email: None,
            account_id: None,
            organization_id: None,
            subscription_type: None,
            auth_source: None,
        }
    }
}

/// API key information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    /// Whether API key is set.
    pub has_api_key: bool,
    /// Source of the API key (env, config, etc.).
    pub source: Option<String>,
}

/// OAuth tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    /// Access token.
    pub access_token: String,
    /// Refresh token.
    pub refresh_token: Option<String>,
    /// Token type.
    pub token_type: String,
    /// Expiry timestamp.
    pub expires_at: Option<i64>,
}

/// Auth handler for CLI commands.
pub struct AuthHandler {
    status: Arc<RwLock<AuthStatus>>,
}

impl AuthHandler {
    pub fn new() -> Self {
        Self {
            status: Arc::new(RwLock::new(AuthStatus::default())),
        }
    }

    /// Check current authentication status.
    pub async fn status(&self) -> AuthStatus {
        self.status.read().await.clone()
    }

    /// Print authentication status.
    pub async fn print_status(&self) {
        let status = self.status().await;

        if status.authenticated {
            println!("{}", colors::green("✓ Authenticated"));

            if let Some(email) = &status.email {
                kv("Email", email);
            }
            if let Some(org_id) = &status.organization_id {
                kv("Organization", org_id);
            }
            if let Some(sub_type) = &status.subscription_type {
                kv("Subscription", sub_type);
            }
            if let Some(source) = &status.auth_source {
                kv("Auth Source", source);
            }
        } else {
            println!("{}", colors::yellow("✗ Not authenticated"));
            println!("Run 'senagent auth login' to authenticate");
        }
    }

    /// Handle login command.
    pub async fn login(&self, options: LoginOptions) {
        // In a real implementation, this would:
        // 1. Open browser for OAuth flow
        // 2. Exchange code for tokens
        // 3. Store tokens securely
        // 4. Update status

        println!("Starting authentication flow...");

        // For now, simulate successful login
        let mut status = self.status.write().await;
        status.authenticated = true;
        status.email = options.email.clone();
        status.account_id = Some(uuid::Uuid::new_v4().to_string());
        status.organization_id = options.organization.clone();
        status.auth_source = Some("oauth".to_string());

        println!("\n{}", colors::green("Successfully authenticated!"));

        if let Some(email) = &options.email {
            println!("Logged in as: {}", email);
        }
    }

    /// Handle logout command.
    pub async fn logout(&self, options: LogoutOptions) {
        let mut status = self.status.write().await;

        if !status.authenticated {
            println!("{}", colors::yellow("Not currently authenticated"));
            cli_ok(None);
        }

        // Clear authentication state
        let was_email = status.email.clone();
        *status = AuthStatus::default();

        if options.clear_onboarding {
            // Clear onboarding state
        }

        println!("{}", colors::green("Successfully logged out"));

        if let Some(email) = was_email {
            println!("Goodbye, {}", email);
        }
    }

    /// Handle whoami command.
    pub async fn whoami(&self) {
        let status = self.status.read().await;

        if let Some(email) = &status.email {
            println!("{}", email);
        } else {
            cli_error(Some("Not authenticated"));
        }
    }
}

impl Default for AuthHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Options for login command.
#[derive(Debug, Clone)]
pub struct LoginOptions {
    /// Email to use for login.
    pub email: Option<String>,
    /// Organization ID.
    pub organization: Option<String>,
    /// Skip browser (non-interactive mode).
    pub no_browser: bool,
}

impl Default for LoginOptions {
    fn default() -> Self {
        Self {
            email: None,
            organization: None,
            no_browser: false,
        }
    }
}

/// Options for logout command.
#[derive(Debug, Clone)]
pub struct LogoutOptions {
    /// Also clear onboarding state.
    pub clear_onboarding: bool,
}

impl Default for LogoutOptions {
    fn default() -> Self {
        Self {
            clear_onboarding: false,
        }
    }
}

/// Check if using 3rd party services.
pub fn is_using_3p_services() -> bool {
    // Check environment or config
    std::env::var("SENAGENT_USE_3P_SERVICES").is_ok()
}

/// Get API key info.
pub fn get_api_key_info() -> ApiKeyInfo {
    let api_key = std::env::var("ANTHROPIC_API_KEY").ok();

    ApiKeyInfo {
        has_api_key: api_key.is_some(),
        source: api_key.map(|_| "environment".to_string()),
    }
}

/// Print API key status.
pub fn print_api_key_status() {
    let info = get_api_key_info();

    if info.has_api_key {
        println!("{}", colors::green("✓ API key configured"));
        if let Some(source) = &info.source {
            kv("Source", source);
        }
    } else {
        println!("{}", colors::yellow("✗ API key not configured"));
        println!("Set ANTHROPIC_API_KEY environment variable or use 'senagent auth login'");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_auth_handler_initial_status() {
        let handler = AuthHandler::new();
        let status = handler.status().await;

        assert!(!status.authenticated);
    }

    #[tokio::test]
    async fn test_auth_handler_login() {
        let handler = AuthHandler::new();

        handler
            .login(LoginOptions {
                email: Some("test@example.com".to_string()),
                organization: None,
                no_browser: false,
            })
            .await;

        let status = handler.status().await;
        assert!(status.authenticated);
        assert_eq!(status.email, Some("test@example.com".to_string()));
    }

    #[tokio::test]
    async fn test_auth_handler_logout() {
        let handler = AuthHandler::new();

        // Login first
        handler
            .login(LoginOptions {
                email: Some("test@example.com".to_string()),
                organization: None,
                no_browser: false,
            })
            .await;

        // Then logout
        handler.logout(LogoutOptions::default()).await;

        let status = handler.status().await;
        assert!(!status.authenticated);
    }

    #[test]
    fn test_api_key_info() {
        // Without API key
        std::env::remove_var("ANTHROPIC_API_KEY");
        let info = get_api_key_info();
        assert!(!info.has_api_key);
    }
}
