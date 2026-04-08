// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// CLI exit helpers — mirrors claude-code-typescript-src `cli/exit.ts`.
// Consolidates CLI exit patterns for error/success handling.

use std::process;

/// Write an error message to stderr and exit with code 1.
pub fn cli_error(msg: Option<&str>) -> ! {
    if let Some(m) = msg {
        eprintln!("{}", m);
    }
    process::exit(1)
}

/// Write a message to stdout and exit with code 0.
pub fn cli_ok(msg: Option<&str>) -> ! {
    if let Some(m) = msg {
        println!("{}", m);
    }
    process::exit(0)
}

/// Result type for CLI operations that may fail with user-facing errors.
pub type CliResult<T> = Result<T, CliError>;

/// CLI error with optional message.
#[derive(Debug, Clone)]
pub struct CliError {
    pub message: Option<String>,
    pub exit_code: i32,
}

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: Some(message.into()),
            exit_code: 1,
        }
    }

    pub fn with_code(mut self, code: i32) -> Self {
        self.exit_code = code;
        self
    }

    pub fn exit(&self) -> ! {
        if let Some(ref msg) = self.message {
            eprintln!("{}", msg);
        }
        process::exit(self.exit_code)
    }
}

impl From<anyhow::Error> for CliError {
    fn from(e: anyhow::Error) -> Self {
        Self::new(e.to_string())
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        Self::new(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_error_creation() {
        let err = CliError::new("Something went wrong");
        assert_eq!(err.message, Some("Something went wrong".to_string()));
        assert_eq!(err.exit_code, 1);
    }

    #[test]
    fn test_cli_error_with_code() {
        let err = CliError::new("Custom error").with_code(2);
        assert_eq!(err.exit_code, 2);
    }
}
