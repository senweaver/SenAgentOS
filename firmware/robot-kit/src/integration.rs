// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//! Integration layer between robot-kit and SenAgentOS.
//!
//! This module provides adapters that bridge robot-kit's standalone `Tool` trait
//! to SenAgentOS's `Tool` trait, enabling all robot-kit tools to be registered
//! in the SenAgentOS agent loop.
//!
//! ## Usage in SenAgentOS
//!
//! ```rust,ignore
//! use senagent_robot_kit::integration::RobotToolAdapter;
//! use senagent_robot_kit::{create_safe_tools, RobotConfig};
//!
//! // Create robot tools
//! let config = RobotConfig::default();
//! let robot_tools = create_safe_tools(&config, safety_monitor);
//!
//! // Convert to SenAgentOS tool format
//! let os_tools: Vec<Box<dyn senagentos::tools::Tool>> = robot_tools
//!     .into_iter()
//!     .map(|t| RobotToolAdapter::new(t))
//!     .map(|a| Box::new(a) as Box<dyn senagentos::tools::Tool>)
//!     .collect();
//! ```

use async_trait::async_trait;
use crate::traits::{Tool as RobotTool, ToolResult as RobotToolResult};
use serde_json::Value as JsonValue;

/// Adapter that wraps a robot-kit `Tool` as a SenAgentOS `Tool`.
///
/// Since both traits have identical signatures (`name`, `description`,
/// `parameters_schema`, `execute`), this is a zero-overhead passthrough.
#[derive(Debug)]
pub struct RobotToolAdapter<T: RobotTool> {
    inner: T,
}

impl<T: RobotTool> RobotToolAdapter<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

impl<T: RobotTool + 'static> From<T> for RobotToolAdapter<T> {
    fn from(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<T: RobotTool> senagentos::tools::Tool for RobotToolAdapter<T> {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn description(&self) -> &str {
        self.inner.description()
    }

    fn parameters_schema(&self) -> JsonValue {
        self.inner.parameters_schema()
    }

    async fn execute(&self, args: JsonValue) -> anyhow::Result<senagentos::tools::ToolResult> {
        let result = self.inner.execute(args).await?;
        Ok(senagentos::tools::ToolResult {
            success: result.success,
            output: result.output,
            error: result.error,
        })
    }
}

/// Extension trait to convert a `Vec<Box<dyn RobotTool>>` into
/// `Vec<Box<dyn senagentos::tools::Tool>>` via [`RobotToolAdapter`].
pub trait IntoSenAgentTools {
    fn into_senagent_tools(self) -> Vec<Box<dyn senagentos::tools::Tool>>;
}

impl IntoSenAgentTools for Vec<Box<dyn RobotTool>> {
    fn into_senagent_tools(self) -> Vec<Box<dyn senagentos::tools::Tool>> {
        self.into_iter()
            .map(|t| {
                let adapter = RobotToolAdapter::new(t);
                Box::new(adapter) as Box<dyn senagentos::tools::Tool>
            })
            .collect()
    }
}

/// Create robot-kit tools and immediately convert them to SenAgentOS tools.
///
/// This is a convenience function for the most common integration pattern.
///
/// ```rust,ignore
/// use senagent_robot_kit::integration::create_senagent_tools;
/// use senagentos::tools::Tool;
///
/// let config = RobotConfig::default();
/// let tools: Vec<Box<dyn Tool>> = create_senagent_tools(&config);
/// ```
pub fn create_senagent_tools(
    config: &crate::RobotConfig,
) -> Vec<Box<dyn senagentos::tools::Tool>> {
    crate::create_tools(config).into_senagent_tools()
}
