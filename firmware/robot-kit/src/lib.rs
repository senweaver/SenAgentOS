//! # senagent Robot Kit
//!
//! A standalone robotics toolkit that integrates with senagent for AI-powered robots.
//!
//! ## Features
//!
//! - **Drive**: Omni-directional motor control (ROS2, serial, GPIO, mock)
//! - **Look**: Camera capture + vision model description (Ollama)
//! - **Listen**: Speech-to-text via Whisper.cpp
//! - **Speak**: Text-to-speech via Piper TTS
//! - **Sense**: LIDAR, motion sensors, ultrasonic distance
//! - **Emote**: LED matrix expressions and sound effects
//! - **Safety**: Independent safety monitor (collision avoidance, E-stop, watchdog)
//!
//! ## Architecture
//!
//! ```text
//! ???????????????????????????????????????????????????????????//! ? senagent AI Brain (or any controller)                  ?//! ? "Move forward, find the ball, tell me what you see"    ?//! ???????????????????????????????????????????????????????????//!                       ?Tool calls
//!                       ?//! ???????????????????????????????????????????????????????????//! ? SenAgentOS-robot-kit                                     ?//! ? ???????????????????????????????????????????????  ?//! ? ?drive   ??look ??listen ??speak ??sense ?  ?//! ? ???????????????????????????????????????????????  ?//! ?      ?        ?        ?         ?        ?       ?//! ? ???????????????????????????????????????????????????? ?//! ? ?             SafetyMonitor (parallel)             ? ?//! ? ? ?Pre-move obstacle check                        ? ?//! ? ? ?Proximity-based speed limiting                 ? ?//! ? ? ?Bump sensor response                           ? ?//! ? ? ?Watchdog auto-stop                             ? ?//! ? ? ?Hardware E-stop override                       ? ?//! ? ???????????????????????????????????????????????????? ?//! ???????????????????????????????????????????????????????????//!                       ?//!                       ?//! ???????????????????????????????????????????????????????????//! ? Hardware: Motors, Camera, Mic, Speaker, LIDAR, LEDs    ?//! ???????????????????????????????????????????????????????????//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use SenAgentOS_robot_kit::{RobotConfig, DriveTool, SafetyMonitor, SafeDrive};
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Load configuration
//!     let config = RobotConfig::default();
//!
//!     // Create safety monitor
//!     let (safety, _rx) = SafetyMonitor::new(config.safety.clone());
//!     let safety = Arc::new(safety);
//!
//!     // Wrap drive with safety
//!     let drive = Arc::new(DriveTool::new(config.clone()));
//!     let safe_drive = SafeDrive::new(drive, safety.clone());
//!
//!     // Use tools...
//!     let result = safe_drive.execute(serde_json::json!({
//!         "action": "forward",
//!         "distance": 1.0
//!     })).await;
//! }
//! ```
//!
//! ## Standalone Usage
//!
//! This crate can be used independently of SenAgentOS. It defines its own
//! `Tool` trait that is compatible with SenAgentOS's but doesn't require it.
//!
//! ## Integration with SenAgentOS
//!
//! When used as a dependency of the main `senagent-os` crate, enable the
//! `senagent-os-integration` feature to access the [`integration`] module,
//! which provides a zero-cost adapter that converts all robot-kit tools to
//! SenAgentOS `Box<dyn Tool>` for registration in the agent loop:
//!
//! ```toml
//! # In senagent-os/Cargo.toml dependencies section:
//! senagent-robot-kit = { path = "firmware/robot-kit", features = ["senagent-os-integration"] }
//! ```
//!
//! Then in your code:
//!
//! ```rust,ignore
//! use senagent_robot_kit::{RobotConfig, create_tools};
//! use senagent_robot_kit::integration::IntoSenAgentTools;
//!
//! // Create robot tools and convert to SenAgentOS format in one step:
//! let robot_tools: Vec<Box<dyn senagentos::tools::Tool>> =
//!     create_tools(&config).into_senagent_tools();
//! ```
//!
//! ## Safety
//!
//! **The AI can REQUEST movement, but SafetyMonitor ALLOWS it.**
//!
//! The safety system runs as an independent task and can override any
//! AI decision. This prevents collisions even if the LLM hallucinates.

// Documentation coverage in progress  missing_docs warning will be re-enabled
// once all public API items have complete doc comments.
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod paths;
pub mod traits;

pub mod drive;
pub mod emote;
pub mod listen;
pub mod look;
pub mod sense;
pub mod speak;

#[cfg(feature = "senagent-os-integration")]
pub mod integration;

#[cfg(feature = "safety")]
pub mod safety;

#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use config::RobotConfig;
pub use paths::{
    captures_dir, ensure_dirs, recordings_dir, robot_dir, senagent_root, sounds_dir,
    tts_cache_dir, voice_models_dir, whisper_models_dir,
};
pub use traits::{Tool, ToolResult, ToolSpec};

pub use drive::DriveTool;
pub use emote::EmoteTool;
pub use listen::ListenTool;
pub use look::LookTool;
pub use sense::SenseTool;
pub use speak::SpeakTool;

#[cfg(feature = "safety")]
pub use safety::{preflight_check, SafeDrive, SafetyEvent, SafetyMonitor, SensorReading};

/// Re-exports from the integration module (only when the senagent-os-integration
/// feature is enabled). Provides `RobotToolAdapter` and `IntoSenAgentTools` for
/// bridging robot-kit tools into the SenAgentOS agent loop.
#[cfg(feature = "senagent-os-integration")]
pub use integration::{create_senagent_tools, IntoSenAgentTools, RobotToolAdapter};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create all robot tools with default configuration
///
/// Returns a Vec of boxed tools ready for use with an agent.
pub fn create_tools(config: &RobotConfig) -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(DriveTool::new(config.clone())),
        Box::new(LookTool::new(config.clone())),
        Box::new(ListenTool::new(config.clone())),
        Box::new(SpeakTool::new(config.clone())),
        Box::new(SenseTool::new(config.clone())),
        Box::new(EmoteTool::new(config.clone())),
    ]
}

/// Create all robot tools with safety wrapper on drive
#[cfg(feature = "safety")]
pub fn create_safe_tools(
    config: &RobotConfig,
    safety: std::sync::Arc<SafetyMonitor>,
) -> Vec<Box<dyn Tool>> {
    let drive = std::sync::Arc::new(DriveTool::new(config.clone()));
    let safe_drive = SafeDrive::new(drive, safety);

    vec![
        Box::new(safe_drive),
        Box::new(LookTool::new(config.clone())),
        Box::new(ListenTool::new(config.clone())),
        Box::new(SpeakTool::new(config.clone())),
        Box::new(SenseTool::new(config.clone())),
        Box::new(EmoteTool::new(config.clone())),
    ]
}
