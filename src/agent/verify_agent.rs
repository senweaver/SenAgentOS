// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Verification Agent — mirrors claude-code-typescript-src `skills/bundled/verify.ts`.
// Verifies code changes work correctly by running tests and checking functionality.

use serde::{Deserialize, Serialize};

/// Verification skill configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyConfig {
    /// Whether verification is enabled.
    pub enabled: bool,
    /// Command to run for verification.
    pub command: Option<String>,
    /// Additional verification steps.
    pub steps: Vec<VerifyStep>,
}

impl Default for VerifyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            command: None,
            steps: Vec::new(),
        }
    }
}

/// A verification step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyStep {
    /// Step name.
    pub name: String,
    /// Command to run.
    pub command: String,
    /// Expected output pattern (optional).
    pub expected_pattern: Option<String>,
    /// Whether this step is critical.
    pub critical: bool,
    /// Timeout in seconds.
    pub timeout_secs: u64,
}

impl VerifyStep {
    /// Create a new verification step.
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            expected_pattern: None,
            critical: false,
            timeout_secs: 60,
        }
    }

    /// Set expected pattern.
    pub fn with_expected(mut self, pattern: &str) -> Self {
        self.expected_pattern = Some(pattern.to_string());
        self
    }

    /// Set as critical step.
    pub fn with_critical(mut self) -> Self {
        self.critical = true;
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }
}

/// Result of a verification step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyStepResult {
    /// Step name.
    pub step: String,
    /// Whether the step passed.
    pub passed: bool,
    /// Output from the step.
    pub output: String,
    /// Error message if failed.
    pub error: Option<String>,
}

/// Result of a verification run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResult {
    /// Whether all steps passed.
    pub success: bool,
    /// Results of individual steps.
    pub step_results: Vec<VerifyStepResult>,
    /// Total time taken.
    pub duration_ms: u64,
    /// Summary message.
    pub summary: String,
}

impl VerifyResult {
    /// Create a successful result.
    pub fn success(step_results: Vec<VerifyStepResult>, duration_ms: u64) -> Self {
        let all_passed = step_results.iter().all(|r| r.passed);
        let passed_count = step_results.iter().filter(|r| r.passed).count();
        let total_count = step_results.len();

        Self {
            success: all_passed,
            step_results,
            duration_ms,
            summary: format!(
                "Verification {}: {}/{} steps passed",
                if all_passed { "succeeded" } else { "failed" },
                passed_count,
                total_count
            ),
        }
    }

    /// Check if a critical step failed.
    pub fn has_critical_failure(&self) -> bool {
        self.step_results.iter().any(|r| !r.passed)
    }
}

/// Verification agent system prompt.
pub fn build_verify_agent_system_prompt(user_request: Option<&str>) -> String {
    let user_section = user_request
        .map(|req| format!("\n\n## User Request\n\n{}\n", req))
        .unwrap_or_default();

    format!(
        r#"You are a verification specialist for SenAgentOS. Your job is to verify that code changes work correctly.

=== Verification Approach ===
1. Understand what the code is supposed to do
2. Run appropriate verification steps (tests, build, lint, etc.)
3. Check that outputs match expectations
4. Report results clearly

=== Verification Steps ===
- Run tests to verify functionality
- Check for compilation/build errors
- Verify linting passes
- Run any custom verification commands specified

=== Output Format ===
Provide a clear report with:
- Which steps passed/failed
- Any errors encountered
- Whether the code change is verified

{:#?}
Guidelines:
- Be thorough but efficient
- If a step fails, note it clearly
- Provide actionable feedback
- Don't modify files - only verify"#,
        user_section
    )
}

/// Verify skill for running verification tasks.
pub struct VerifySkill {
    /// Configuration.
    config: VerifyConfig,
    /// Registered verification steps.
    steps: Vec<VerifyStep>,
}

impl VerifySkill {
    /// Create a new verify skill.
    pub fn new() -> Self {
        Self {
            config: VerifyConfig::default(),
            steps: Vec::new(),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(config: VerifyConfig) -> Self {
        let mut skill = Self {
            config,
            steps: Vec::new(),
        };
        // Add default test step
        skill.steps.push(VerifyStep::new("Run Tests", "cargo test"));
        skill
    }

    /// Add a verification step.
    pub fn add_step(&mut self, step: VerifyStep) {
        self.steps.push(step);
    }

    /// Get all steps.
    pub fn get_steps(&self) -> &[VerifyStep] {
        &self.steps
    }

    /// Check if verification is enabled.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Build the prompt for a verification request.
    pub fn build_prompt(&self, user_args: Option<&str>) -> String {
        let mut prompt = build_verify_agent_system_prompt(user_args);

        if !self.steps.is_empty() {
            prompt.push_str("\n\n## Configured Steps:\n");
            for step in &self.steps {
                prompt.push_str(&format!("- {}: `{}`\n", step.name, step.command));
            }
        }

        prompt
    }
}

impl Default for VerifySkill {
    fn default() -> Self {
        Self::new()
    }
}

/// Built-in verification patterns.
pub mod patterns {
    use regex::Regex;

    /// Check if output contains test failures.
    pub fn has_test_failures(output: &str) -> bool {
        output.contains("FAILED") || output.contains("test result: FAILED")
    }

    /// Check if output contains compilation errors.
    pub fn has_compilation_errors(output: &str) -> bool {
        output.contains("error:")
    }

    /// Check if output contains linting issues.
    pub fn has_lint_issues(output: &str) -> bool {
        output.contains("warning:") && output.contains("lint")
    }

    /// Extract error count from test output.
    pub fn extract_error_count(output: &str) -> Option<usize> {
        // Look for patterns like "3 failed" or "failures: 5"
        let re = Regex::new(r"(\d+)\s+(?:failed|failures?)").ok()?;
        let caps = re.captures(output)?;
        caps.get(1)?.as_str().parse().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_step_builder() {
        let step = VerifyStep::new("Test", "cargo test")
            .with_expected("test result: ok")
            .with_critical()
            .with_timeout(120);

        assert_eq!(step.name, "Test");
        assert_eq!(step.command, "cargo test");
        assert!(step.expected_pattern.is_some());
        assert!(step.critical);
        assert_eq!(step.timeout_secs, 120);
    }

    #[test]
    fn test_verify_result_success() {
        let step_results = vec![
            VerifyStepResult {
                step: "Build".to_string(),
                passed: true,
                output: "Build successful".to_string(),
                error: None,
            },
            VerifyStepResult {
                step: "Test".to_string(),
                passed: true,
                output: "All tests passed".to_string(),
                error: None,
            },
        ];

        let result = VerifyResult::success(step_results, 5000);

        assert!(result.success);
        assert_eq!(result.summary, "Verification succeeded: 2/2 steps passed");
    }

    #[test]
    fn test_verify_result_failure() {
        let step_results = vec![
            VerifyStepResult {
                step: "Build".to_string(),
                passed: true,
                output: "Build successful".to_string(),
                error: None,
            },
            VerifyStepResult {
                step: "Test".to_string(),
                passed: false,
                output: "".to_string(),
                error: Some("Test failed".to_string()),
            },
        ];

        let result = VerifyResult::success(step_results, 5000);

        assert!(!result.success);
        assert!(result.has_critical_failure());
    }

    #[test]
    fn test_verify_skill_prompt() {
        let skill = VerifySkill::with_config(VerifyConfig::default());
        let prompt = skill.build_prompt(Some("Verify my changes"));

        assert!(prompt.contains("verification"));
        assert!(prompt.contains("Verify my changes"));
    }

    #[test]
    fn test_patterns() {
        assert!(patterns::has_test_failures("test result: FAILED"));
        assert!(patterns::has_compilation_errors("error: expected"));
        assert!(patterns::has_lint_issues("warning: unused variable lint"));
    }
}
