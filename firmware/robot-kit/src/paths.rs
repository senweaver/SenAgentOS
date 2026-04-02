// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//! Centralized paths and constants for robot-kit.
//!
//! All paths follow the same conventions as the SenAgentOS project:
//! - User data root: `~/.senagent/` (NOT `~/.SenAgentOS/`)
//! - Robot-specific data lives in `~/.senagent/robot/`
//!
//! This ensures configuration, caches, and logs are co-located with
//! the rest of SenAgentOS when the robot is part of an agent system.

use std::path::PathBuf;

/// SenAgentOS user data root (matches the main senagent-os crate).
pub fn senagent_root() -> PathBuf {
    directories::BaseDirs::new()
        .map(|d| d.home_dir().join(".senagent"))
        .unwrap_or_else(|| PathBuf::from("/tmp/senagent"))
}

/// Robot-specific subdirectory under `~/.senagent/`.
pub fn robot_dir() -> PathBuf {
    senagent_root().join("robot")
}

/// Camera capture directory.
pub fn captures_dir() -> PathBuf {
    robot_dir().join("captures")
}

/// TTS audio cache directory.
pub fn tts_cache_dir() -> PathBuf {
    robot_dir().join("tts_cache")
}

/// Audio recordings directory.
pub fn recordings_dir() -> PathBuf {
    robot_dir().join("recordings")
}

/// Sound effects directory.
pub fn sounds_dir() -> PathBuf {
    robot_dir().join("sounds")
}

/// Voice model directory.
pub fn voice_models_dir() -> PathBuf {
    robot_dir().join("models/voice")
}

/// Whisper model directory.
pub fn whisper_models_dir() -> PathBuf {
    robot_dir().join("models/whisper")
}

/// Create all robot directories (idempotent).
pub fn ensure_dirs() {
    for dir in [
        captures_dir(),
        tts_cache_dir(),
        recordings_dir(),
        sounds_dir(),
        voice_models_dir(),
        whisper_models_dir(),
    ] {
        let _ = std::fs::create_dir_all(&dir);
    }
}
