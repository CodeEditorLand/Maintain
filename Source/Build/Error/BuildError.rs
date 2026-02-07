//=============================================================================//
// File Path: Element/Maintain/Source/Build/Error/BuildError.rs
//=============================================================================//
// Module: BuildError
//
// Brief Description: Defines the error enum for build operations.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Represent all possible errors that can occur during build orchestration
// - Provide clear error messages for different failure scenarios
// - Support automatic conversion from underlying library errors
//
// Secondary:
// - Enable proper error propagation through the build system
// - Provide context for debugging build failures
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Error handling layer
// - Build system error types
//
// Dependencies (What this module requires):
// - External crates: thiserror, std
// - Internal modules: None
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - JavaScriptObjectNotation editing functions
// - TomsL editing functions
// - Guard implementation
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Error handling pattern with thiserror derive macro
// - From trait implementations for automatic error conversion
//
// Performance Considerations:
// - Complexity: O(1) - enum variant selection
// - Memory usage patterns: Enum discrimination + variant data
// - Hot path optimizations: None needed for error types
//
// Thread Safety:
// - Thread-safe: Yes (immutable error enum)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: This enum represents all build errors
// - Recovery strategies: Propagate errors up the call stack; let the Guard restore files
//
// EXAMPLES:
// =========
//
// Example 1: Returning an IO error
/// ```rust
/// use std::fs;
/// use crate::Maintain::Source::Build::Error::BuildError;
/// fn read_file(path: &Path) -> Result<String, BuildError> {
///     fs::read_to_string(path)?; // Automatically converted to BuildError::Io
///     Ok(contents)
/// }
/// ```
//
// Example 2: Returning a parsing error
/// ```rust
/// use crate::Maintain::Source::Build::Error::BuildError;
/// fn parse_json(data: &str) -> Result<JsonValue, BuildError> {
///     let value = serde_json::from_str(data)?;
///     Ok(value)
/// }
/// ```
//
// Example 3: Returning a custom error
/// ```rust
/// use crate::Maintain::Source::Build::Error::BuildError;
/// fn check_directory(path: &Path) -> Result<(), BuildError> {
///     if !path.is_dir() {
///         return Err(BuildError::Missing(path.to_path_buf()));
///     }
///     Ok(())
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use std::{io, path::PathBuf};
use thiserror::Error;

/// Represents all possible errors that can occur during the build script's
/// execution.
///
/// This enum provides a comprehensive error type for the build orchestration
/// system, covering all failure modes from IO operations to parsing errors
/// to command execution failures. Each variant includes relevant context
/// to help diagnose the issue.
///
/// The enum derives `Error` from the `thiserror` crate, which provides
/// automatic implementations of the `Error` trait and error display formatting.
///
/// # Automatic Conversions
///
/// This type implements `From` for several error types, allowing the `?`
/// operator to automatically convert library-specific errors into `BuildError`:
/// - `io::Error` → `BuildError::Io`
/// - `toml_edit::TomlError` → `BuildError::Edit`
/// - `toml::de::Error` → `BuildError::Parse`
/// - `serde_json::Error` → `BuildError::Json`
/// - `json5::Error` → `BuildError::Jsonfive`
/// - `std::string::FromUtf8Error` → `BuildError::Utf`
#[derive(Error, Debug)]
pub enum Error {
	/// IO operation error.
	///
	/// This variant wraps standard Rust IO errors that can occur during
	/// file operations like reading, writing, copying, or deleting files.
	#[error("IO: {0}")]
	Io(#[from] io::Error),

	/// Toml editing error.
	///
	/// This variant wraps errors that occur when manipulating TOML documents
	/// using the `toml_edit` library, such as invalid mutations or parsing
	/// issues when reading TOML content.
	#[error("Toml Editing: {0}")]
	Edit(#[from] toml_edit::TomlError),

	/// Toml deserialization error.
	///
	/// This variant wraps errors that occur when parsing TOML files into
	/// Rust structs using the `toml` library's deserialization functionality.
	#[error("Toml Parsing: {0}")]
	Parse(#[from] toml::de::Error),

	/// JSON parsing/error.
	///
	/// This variant wraps errors that occur when parsing or validating
	/// standard JSON files using the `serde_json` library.
	#[error("Json: {0}")]
	Json(#[from] serde_json::Error),

	/// JSON5 parsing/error.
	///
	/// This variant wraps errors that occur when parsing or validating
	/// JSON5 files (a more flexible JSON format) using the `json5` library.
	#[error("Json5: {0}")]
	Jsonfive(#[from] json5::Error),

	/// Missing directory error.
	///
	/// This variant is used when a required directory does not exist at
	/// the specified path. The path is included for debugging purposes,
	#[error("Missing Directory: {0}")]
	Missing(PathBuf),

	/// Shell command execution failure.
	///
	/// This variant is used when the final build command fails to execute
	/// successfully. The exit status is included to provide information
	/// about the failure.
	#[error("Command Failed: {0}")]
	Shell(std::process::ExitStatus),

	/// No command provided error.
	///
	/// This variant is used when the build script is invoked without
	/// specifying a build command to execute.
	#[error("No Command Provided")]
	NoCommand,

	/// Tauri configuration file not found error.
	///
	/// This variant is used when neither `tauri.conf.json` nor
	/// `tauri.conf.json5` can be found in the project directory.
	#[error("Tauri Configuration File Not Found")]
	Config,

	/// Backup file already exists error.
	///
	/// This variant is used when the Guard attempts to create a backup but
	/// a backup file already exists at the target location. This prevents
	/// overwriting existing backups and ensures data safety.
	#[error("Backup File Exists: {0}")]
	Exists(PathBuf),

	/// UTF-8 conversion error.
	///
	/// This variant wraps errors that occur when converting byte vectors
	/// to UTF-8 strings, particularly when dealing with file contents or
	/// command output.
	#[error("UTF-8 Conversion: {0}")]
	Utf(#[from] std::string::FromUtf8Error),

	/// Missing environment variable error.
	///
	/// This variant is used when a required environment variable is not
	/// set. The variable name is included in the error message for easy
	/// identification and resolution.
	#[error("Environment Variable Missing: {0}")]
	Environment(String),
}
