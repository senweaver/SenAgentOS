// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// LSP service — mirrors claude-code-typescript-src`services/lsp/`.
// Integrates with Language Server Protocol servers for code intelligence.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use serde::{Deserialize, Serialize};

/// A diagnostic from an LSP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspDiagnostic {
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// A symbol definition from LSP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub file: PathBuf,
    pub line: u32,
    pub column: u32,
    pub container: Option<String>,
}

/// Configuration for an LSP server connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    pub language_id: String,
    pub server_command: String,
    pub server_args: Vec<String>,
    pub root_path: PathBuf,
    pub initialization_options: Option<serde_json::Value>,
}

/// LSP service managing connections to language servers.
#[derive(Clone)]
pub struct LspService {
    inner: Arc<RwLock<LspServiceInner>>,
}

struct LspServiceInner {
    servers: HashMap<String, LspServerState>,
    diagnostics_cache: HashMap<PathBuf, Vec<LspDiagnostic>>,
}

#[derive(Debug, Clone)]
struct LspServerState {
    config: LspServerConfig,
    status: LspServerStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LspServerStatus {
    Stopped,
    Starting,
    Running,
    Error,
}

impl LspService {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(LspServiceInner {
                servers: HashMap::new(),
                diagnostics_cache: HashMap::new(),
            })),
        }
    }

    /// Register an LSP server configuration.
    pub async fn register_server(&self, config: LspServerConfig) {
        let mut inner = self.inner.write().await;
        let language_id = config.language_id.clone();
        inner.servers.insert(
            language_id,
            LspServerState {
                config,
                status: LspServerStatus::Stopped,
            },
        );
    }

    /// Get diagnostics for a file.
    pub async fn get_diagnostics(&self, file: &PathBuf) -> Vec<LspDiagnostic> {
        let inner = self.inner.read().await;
        inner
            .diagnostics_cache
            .get(file)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all diagnostics across all files.
    pub async fn get_all_diagnostics(&self) -> HashMap<PathBuf, Vec<LspDiagnostic>> {
        let inner = self.inner.read().await;
        inner.diagnostics_cache.clone()
    }

    /// Update diagnostics for a file (called by LSP notification handler).
    pub async fn update_diagnostics(&self, file: PathBuf, diagnostics: Vec<LspDiagnostic>) {
        let mut inner = self.inner.write().await;
        if diagnostics.is_empty() {
            inner.diagnostics_cache.remove(&file);
        } else {
            inner.diagnostics_cache.insert(file, diagnostics);
        }
    }

    /// List registered language servers.
    pub async fn list_servers(&self) -> Vec<(String, String)> {
        let inner = self.inner.read().await;
        inner
            .servers
            .iter()
            .map(|(id, state)| (id.clone(), format!("{:?}", state.status)))
            .collect()
    }
}

impl Default for LspService {
    fn default() -> Self {
        Self::new()
    }
}
