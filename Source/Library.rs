#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	unused_imports,
	unused_variables,
	unused_assignments
)]

//! # Maintain: CI/CD and Build Orchestrator for Code Editor Land
//!
//! Maintain is the build system that compiles, bundles, and packages Land for
//! all three platforms (macOS, Windows, Linux) from a single command. It
//! replaces ad-hoc shell scripts with a structured Rust binary that handles
//! profile management, Tauri builds, and development server orchestration.
//!
//! ## Two Modes
//!
//! **Build mode** (default): Compile Land for release or debug with named
//! profiles that configure Cargo flags, Tauri targets, and environment vars.
//!
//! ```bash
//! cargo run --bin Maintain -- --profile debug-mountain
//! cargo run --bin Maintain -- --list-profiles
//! ```
//!
//! **Run mode** (`--run`): Start the development server with hot reload.
//!
//! ```bash
//! cargo run --bin Maintain -- --run --profile debug-mountain
//! ```
//!
//! ## Modules
//!
//! - [`Build`]: Build orchestration, profile resolution, Tauri invocation
//! - [`Run`]: Development server, watch mode, profile-aware dev builds
//! - [`Architecture`]: Target triple detection and platform support
//!
//! ## Binary target
//!
//! CLI dispatch (run / build / eliminate / legacy) lives in `Source/main.rs`,
//! the only entry point. Keeping dispatch in the binary means this library
//! exposes only orchestration modules and cannot drift from argument parsing.

// =============================================================================
// MODULE DECLARATIONS
// =============================================================================

/// Build Orchestrator Module.
///
/// This module contains all the build orchestration logic, including:
///
/// - **CLI**: Command-line interface for configuration-based builds
/// - **Constant**: File paths, delimiters, and environment variable names
/// - **Definition**: Data structures for arguments, manifests, and file guards
/// - **Error**: Comprehensive error types for build operations
/// - **Fn**: Main build orchestration function
/// - **GetTauriTargetTriple**: Target triple detection
/// - **JsonEdit**: JSON configuration editing
/// - **Logger**: Logging utilities
/// - **Pascalize**: PascalCase conversion utilities
/// - **Process**: Process management
/// - **Rhai**: Rhai scripting support
/// - **TomlEdit**: TOML configuration editing
/// - **WordsFromPascal**: Extract words from PascalCase strings
///
/// See the Build module documentation for detailed information about the
/// build system's capabilities and usage.
pub mod Build;

/// Development Run Module.
///
/// This module contains all the development run orchestration logic, including:
///
/// - **CLI**: Command-line interface for configuration-based runs
/// - **Constant**: File paths, delimiters, and environment variable names
/// - **Definition**: Data structures for arguments and run configuration
/// - **Environment**: Environment variable resolution and management
/// - **Error**: Comprehensive error types for run operations
/// - **Fn**: Main run orchestration function
/// - **Logger**: Logging utilities
/// - **Process**: Process management for development servers
/// - **Profile**: Profile resolution and management
///
/// See the Run module documentation for detailed information about the
/// development run system's capabilities and usage.
pub mod Run;

/// Eliminate Module - inline single-use Rust `let` bindings.
///
/// Uses `syn` to parse Rust source files and `prettyplease` to re-format the
/// result after iteratively removing intermediate variables that are used
/// exactly once and are safe to substitute at their use site.
///
/// Invoked as:
/// ```bash
/// cargo run --bin Maintain -- eliminate --path ./Source --glob "**/*.rs"
/// cargo run --bin Maintain -- eliminate --path ./Source/Foo.rs --dry-run
/// ```
pub mod Eliminate;

pub mod Architecture;
