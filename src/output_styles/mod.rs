// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Output styles module — mirrors claude-code's `outputStyles/` directory.
// Loads and manages custom output styles that modify agent response behaviour.

pub mod loader;
pub mod types;

pub use loader::load_output_styles;
pub use types::{OutputStyle, OutputStyleSource};
