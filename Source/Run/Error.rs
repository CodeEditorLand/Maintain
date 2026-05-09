//=============================================================================//
// File Path: Element/Maintain/Source/Run/Error.rs
//=============================================================================//
// Module: Error
//
// Brief Description: Error types for the Run module.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Define comprehensive error types for run operations
// - Provide error conversion and display implementations
// - Enable proper error propagation throughout the run module
//
// Secondary:
// - Support error context and chaining
// - Provide user-friendly error messages
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Error handling layer
// - Error type definitions
//
// Dependencies (What this module requires):
// - External crates: thiserror, std
// - Internal modules: None
// - Traits implemented: Display, Error, From
//
// Dependents (What depends on this module):
// - Run orchestration functions
// - Process management
// - Profile resolution
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use std::{io, path::PathBuf};

use thiserror::Error as ThisError;

/// Comprehensive error type for run operations.
///
/// This enum represents all possible errors that can occur during
/// the development run process. Each variant provides context-specific
/// information for debugging and error recovery.
#[derive(Debug, ThisError)]
pub enum Error {
	/// Error when a configuration file is not found or cannot be read.
	#[error("Configuration file not found or unreadable: {0}")]
	ConfigNotFound(PathBuf),

	/// Error when parsing configuration fails.
	#[error("Failed to parse configuration: {0}")]
	ConfigParse(String),

	/// Error when a profile is not found in the configuration.
	#[error("Profile '{0}' not found. Available profiles: {1}")]
	ProfileNotFound(String, String),

	/// Error when environment variable resolution fails.
	#[error("Failed to resolve environment variables: {0}")]
	EnvResolve(String),

	/// Error when a required environment variable is missing.
	#[error("Required environment variable '{0}' is not set")]
	EnvMissing(String),

	/// Error when file system operations fail.
	#[error("File system error: {0}")]
	Io(#[from] io::Error),

	/// Error when JSON parsing fails.
	#[error("JSON parsing error: {0}")]
	Json(#[from] serde_json::Error),

	/// Error when a process fails to start.
	#[error("Failed to start process: {0}")]
	ProcessStart(String),

	/// Error when a process exits with a non-zero status.
	#[error("Process exited with code: {0}")]
	ProcessExit(i32),

	/// Error when a process is terminated by a signal.
	#[error("Process terminated by signal: {0}")]
	ProcessSignal(String),

	/// Error when port binding fails.
	#[error("Failed to bind to port {0}: {1}")]
	PortBind(u16, String),

	/// Error when hot-reload setup fails.
	#[error("Hot-reload setup failed: {0}")]
	HotReload(String),

	/// Error when watch mode setup fails.
	#[error("Watch mode setup failed: {0}")]
	WatchMode(String),

	/// Error when a dependency is not found.
	#[error("Dependency not found: {0}")]
	DependencyNotFound(String),

	/// Error when an invalid argument is provided.
	#[error("Invalid argument: {0}")]
	InvalidArgument(String),

	/// Error when run configuration is invalid.
	#[error("Invalid run configuration: {0}")]
	InvalidConfig(String),

	/// Error when multiple conflicting options are provided.
	#[error("Conflicting options: {0}")]
	ConflictingOptions(String),
}

/// Result type alias for run operations.
///
/// This type alias simplifies function signatures by providing
/// a shorthand for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;
