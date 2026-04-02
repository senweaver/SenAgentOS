// SPDX-License-Identifier: MIT
// Copyright (c) 2025-2026 SenAgentOS
// Licensed under the MIT License.
//
// Keybindings module — mirrors claude-code's `keybindings/` directory.
// Provides keyboard shortcut management: default bindings, user overrides,
// key parsing, and action resolution.

pub mod defaults;
pub mod parser;
pub mod resolver;
pub mod schema;

pub use defaults::default_bindings;
pub use parser::parse_key_sequence;
pub use resolver::KeybindingResolver;
pub use schema::{KeyBinding, KeyAction, KeyModifier};
