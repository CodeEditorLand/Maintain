//! # Build - Dynamic Build Orchestrator Module
//!
//! Orchestrates the build process from start to finish, including:
//! - Generating unique product names and bundle identifiers
//! - Dynamically modifying project configuration files (JSON5, TOML, plist)
//! - Staging and bundling Node.js sidecar binaries
//! - Executing final build commands
//! - Restoring original configuration files via RAII guard pattern
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`crate::Build::CLI`] | Command-line interface for configuration-based builds |
//! | [`crate::Build::Constant`] | Build system constants and configuration values |
//! | [`crate::Build::Definition`] | Type definitions and data structures |
//! | [`crate::Build::Error`] | Comprehensive error types |
//! | [`crate::Build::Fn`] | Main entry point |
//! | [`crate::Build::GetTauriTargetTriple`] | Tauri target triple detection |
//! | [`crate::Build::JsonEdit`] | JSON/JSON5 file editing |
//! | [`crate::Build::Logger`] | Colored logging initialization |
//! | [`crate::Build::Pascalize`] | PascalCase string conversion |
//! | [`crate::Build::PlistEdit`] | Apple Info.plist editing |
//! | [`crate::Build::Process`] | Build orchestration logic |
//! | [`crate::Build::Rhai`] | Rhai scripting engine integration |
//! | [`crate::Build::TomlEdit`] | TOML file editing |
//! | [`crate::Build::WordsFromPascal`] | PascalCase word splitting |
//!
//! ## Usage
//!
//! ```rust
//! use crate::Build::Fn;
//! Fn();
//! ```
//!
//! For more details, see the [deep-dive
//! documentation](../../Documentation/GitHub/DeepDive.md).

pub mod CLI;

pub mod Constant;

pub mod Definition;

pub mod Error;

pub mod Fn;

pub mod GetTauriTargetTriple;

pub mod JsonEdit;

pub mod Logger;

pub mod Pascalize;

pub mod PlistEdit;

pub mod Process;

pub mod Rhai;

pub mod TomlEdit;

pub mod WordsFromPascal;
