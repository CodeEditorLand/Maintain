//=============================================================================//
// File Path: Element/Maintain/Source/Build/Function/JavaScriptObjectNotationEditingFunction.rs
//=============================================================================//
// Module: JavaScriptObjectNotationEditingFunction
//
// Brief Description: Implements JSON/JSON5 file editing for Tauri configuration.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Dynamically modify fields in tauri.conf.json/tauri.conf.json5 files
// - Update product name, bundle identifier, version, and sidecar path
// - Support both JSON and JSON5 formats
//
// Secondary:
// - Provide detailed logging of changes made
// - Return whether any modifications occurred
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/File manipulation layer
// - JavaScriptObjectNotation editing functionality
//
// Dependencies (What this module requires):
// - External crates: serde_json, json5, serde (Serialize), std (fs, log, io)
// - Internal modules: Error::BuildError
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - Process function
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Builder pattern (via serde)
// - Functional pattern
//
// Performance Considerations:
// - Complexity: O(n) - parsing and modifying based on file size
// - Memory usage patterns: In-memory document manipulation
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: No (not designed for concurrent access to files)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: BuildError (Json, Jsonfive, Io types)
// - Recovery strategies: Propagate error up; Guard restores original file
//
// EXAMPLES:
// =========
//
// Example 1: Full configuration update
/// ```rust
/// use crate::Maintain::Source::Build::Function::JavaScriptObjectNotationEditingFunction;
/// let config_path = PathBuf::from("tauri.conf.json");
/// let modified = JsonEdit(
///     &config_path,
///     "Debug_Mountain",
///     "land.editor.binary.debug.mountain",
///     "1.0.0",
///     Some("Binary/node")
/// )?;
/// ```
//
// Example 2: Version and identifier update only
/// ```rust
/// use crate::Maintain::Source::Build::Function::JavaScriptObjectNotationEditingFunction;
/// let modified = JsonEdit(
///     &config_path,
///     "Mountain",
///     "land.editor.binary.mountain",
///     "1.0.0",
///     None
/// )?;
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use crate::Build::BuildError;

use log::{debug, info};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{fs, io::Write, path::Path};

/// Dynamically modifies fields in a `tauri.conf.json` or `tauri.conf.json5`
/// file, including the sidecar path.
///
/// This function updates the following fields in the Tauri configuration:
/// - `version` - The application version
/// - `productName` - The product name displayed to users
/// - `identifier` - The bundle identifier (reverse domain format)
/// - `bundle.externalBin` - Adds the sidecar binary path if provided
///
/// The function automatically detects and handles both JSON and JSON5 formats,
/// ensuring compatibility with different Tauri configuration styles.
///
/// # Parameters
///
/// * `File` - Path to the Tauri configuration file
/// * `Product` - The product name to set (displayed to users)
/// * `Id` - The bundle identifier to set (reverse domain format)
/// * `Version` - The version string to set
/// * `SidecarPath` - Optional path to the sidecar binary to bundle
///
/// # Returns
///
/// Returns a `Result<bool>` indicating:
/// - `Ok(true)` - The file was modified and saved
/// - `Ok(false)` - No changes were needed (all values already match)
/// - `Err(BuildError)` - An error occurred during modification
///
/// # Errors
///
/// * `BuildError::Io` - If the file cannot be read or written
/// * `BuildError::Json` - If JSON parsing or serialization fails
/// * `BuildError::Jsonfive` - If JSON5 parsing fails
/// * `BuildError::Utf` - If UTF-8 conversion fails
///
/// # Behavior
///
/// - Only modifies fields that don't match the specified values
/// - Creates nested structures (`bundle`, `externalBin`) as needed
/// - Writes output with tab indentation for human-readable formatting
/// - Logs configuration changes at INFO level
///
/// # JSON5 Support
///
/// JSON5 is a superset of JSON that allows:
/// - Trailing commas
/// - Unquoted property names
/// - Comments
/// - Multi-line strings
///
/// The function automatically detects JSON5 files by their `.json5` extension
/// and uses the appropriate parser.
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::Function::JavaScriptObjectNotationEditingFunction;
/// let path = PathBuf::from("tauri.conf.json");
/// let modified = JsonEdit(
///     &path,
///     "Debug_Mountain",
///     "land.editor.binary.debug.mountain",
///     "1.0.0",
///     Some("Binary/node")
/// )?;
/// ```
pub fn JsonEdit(
	File: &Path,
	Product: &str,
	Id: &str,
	Version: &str,
	SidecarPath: Option<&str>,
) -> Result<bool, BuildError> {
	debug!(target: "Build::Json", "Attempting to modify JSON file: {}", File.display());

	let Data = fs::read_to_string(File)?;

	let mut Parsed: JsonValue = if File.extension().and_then(|s| s.to_str()) == Some("json5") {
		json5::from_str(&Data)?
	} else {
		serde_json::from_str(&Data)?
	};

	let mut Modified = false;

	let Root = Parsed
		.as_object_mut()
		.ok_or_else(|| {
			BuildError::Io(std::io::Error::new(
				std::io::ErrorKind::InvalidData,
				"JSON root is not an object",
			))
		})?;

	// Update version
	if Root.get("version").and_then(JsonValue::as_str) != Some(Version) {
		Root.insert("version".to_string(), JsonValue::String(Version.to_string()));

		Modified = true;
	}

	// Update productName
	if Root.get("productName").and_then(JsonValue::as_str) != Some(Product) {
		Root.insert("productName".to_string(), JsonValue::String(Product.to_string()));

		Modified = true;
	}

	// Update identifier
	if Root.get("identifier").and_then(JsonValue::as_str) != Some(Id) {
		Root.insert("identifier".to_string(), JsonValue::String(Id.to_string()));

		Modified = true;
	}

	// Add sidecar path if provided
	if let Some(Path) = SidecarPath {
		let Bundle = Root
			.entry("bundle")
			.or_insert_with(|| JsonValue::Object(Default::default()))
			.as_object_mut()
			.unwrap();

		let Bins = Bundle
			.entry("externalBin")
			.or_insert_with(|| JsonValue::Array(Default::default()))
			.as_array_mut()
			.unwrap();

		Bins.push(JsonValue::String(Path.to_string()));

		Modified = true;
	}

	// Write the file if any changes were made
	if Modified {
		let mut Buffer = Vec::new();

		let Formatter = serde_json::ser::PrettyFormatter::with_indent(b"\t");

		let mut Serializer = serde_json::Serializer::with_formatter(&mut Buffer, Formatter);

		Parsed.serialize(&mut Serializer)?;

		fs::write(File, String::from_utf8(Buffer)?)?;

		info!(target: "Build::Json", "Dynamically configured {}", File.display());
	}

	Ok(Modified)
}
