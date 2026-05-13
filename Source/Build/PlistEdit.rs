//=============================================================================//
// File Path: Element/Maintain/Source/Build/PlistEdit.rs
//=============================================================================//
// Module: PlistEdit
//
// Brief Description: Implements Info.plist file editing for macOS bundle
// configuration.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Inject LSEnvironment dictionary into the Info.plist template used by Tauri
// - Ensure the bundled .app receives dev-control environment variables when
//   launched by LaunchServices (double-click / Finder / open)
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
// - Apple Property List editing functionality
//
// Dependencies (What this module requires):
// - External crates: std (fs, log), plist, log::info
// - Internal modules: Error::BuildError
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions (Process)
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Builder pattern (via plist serde deserialization)
// - Functional pattern
//
// Performance Considerations:
// - Complexity: O(n) - parsing and writing based on file size
// - Memory usage patterns: In-memory plist tree
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: No (not designed for concurrent access to files)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: BuildError (Plist, Io types)
// - Recovery strategies: Propagate error up; Guard restores original file
//
// WHY THE plist CRATE:
// ====================
//
// The plist crate provides a proper parse-modify-serialize pipeline, the same
// pattern JsonEdit uses via serde_json. The output is canonically formatted
// XML with consistent indentation and sorted keys, so the file written to
// disk is deterministic -- identical content produces identical bytes.
// This avoids the string-manipulation pitfalls of the prior implementation
// (nested-dict counting, whitespace drift, duplicate LSEnvironment blocks).
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//
use std::{collections::BTreeMap, fs, path::Path};

use log::{debug, info};
use plist::{Dictionary, Value, XmlWriteOptions};

use crate::Build::Error::Error as BuildError;

/// Injects or replaces the LSEnvironment dictionary in a macOS Info.plist.
///
/// Tauri uses the Info.plist in the project directory as a template during
/// bundling. When the bundled .app is launched by LaunchServices (double-click
/// in Finder, `open`, or Spotlight), any keys under LSEnvironment are injected
/// into the process environment before the executable starts. This is the
/// mechanism that makes the dev-control variables (Trace, Record, etc.)
/// available without requiring a wrapper script.
///
/// # Parameters
///
/// * `File` - Path to the Info.plist template file
/// * `EnvVars` - Map of environment variable names to their values. These
///   become key-value pairs in the LSEnvironment dictionary.
///
/// # Returns
///
/// Returns a `Result<bool>` indicating:
/// - `Ok(true)` - The file was modified and saved
/// - `Ok(false)` - No changes were needed (LSEnvironment already matches)
/// - `Err(BuildError)` - An error occurred during modification
///
/// # Errors
///
/// * `BuildError::Io` - If the file cannot be read or written
/// * `BuildError::Plist` - If the plist cannot be parsed or serialized
///
/// # Behavior
///
/// - Parses the plist into an in-memory tree.
/// - Inserts or replaces the LSEnvironment dictionary with the given env vars.
/// - Serialises back to XML with tab indentation.
/// - Only writes the file if the content changed.
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::PlistEdit;
/// let mut env = std::collections::BTreeMap::new();
/// env.insert("Trace".to_string(), "all".to_string());
/// env.insert("Record".to_string(), "1".to_string());
/// let modified = PlistEdit(&path, &env)?;
/// ```
pub fn PlistEdit(File:&Path, EnvVars:&BTreeMap<String, String>) -> Result<bool, BuildError> {
	debug!(target: "Build::Plist", "Attempting to modify plist: {}", File.display());

	let Data = fs::read(File)?;

	let mut Root:Value = plist::from_bytes(&Data)?;

	let Dict = Root.as_dictionary_mut().ok_or_else(|| {
		BuildError::Io(std::io::Error::new(
			std::io::ErrorKind::InvalidData,
			"Root plist is not a dictionary",
		))
	})?;

	// Build the new LSEnvironment dictionary as a plist Value.
	let EnvDict = Value::Dictionary(build_env_dict(EnvVars));

	// Check whether the existing value already matches exactly.
	if let Some(Existing) = Dict.get("LSEnvironment") {
		if *Existing == EnvDict {
			debug!(target: "Build::Plist", "LSEnvironment already up-to-date in {}", File.display());

			return Ok(false);
		}
	}

	// Insert or replace.
	Dict.insert("LSEnvironment".to_string(), EnvDict);

	let Written = write_plist(File, &Root)?;

	if Written {
		info!(target: "Build::Plist", "Updated LSEnvironment in {}", File.display());
	}

	Ok(Written)
}

/// Constructs a plist Dictionary from environment variable key-value pairs.
fn build_env_dict(EnvVars:&BTreeMap<String, String>) -> Dictionary {
	let mut Dict = Dictionary::new();

	for (Key, Value) in EnvVars {
		Dict.insert(Key.clone(), Value::String(Value.clone()));
	}

	Dict
}

/// Serialises an in-memory plist tree to XML and writes it to `File`.
///
/// Returns `true` if the file content changed, `false` if the serialisation
/// happens to match what is already on disk (e.g. no-op after a prior write).
fn write_plist(File:&Path, Root:&Value) -> Result<bool, BuildError> {
	// Use XmlWriteOptions with tab indentation to match the hand-written style.
	let Options = XmlWriteOptions::default().indent(b'\t', 1);

	let mut Buffer = Vec::new();

	// Serialise with the formatter.
	plist::to_writer_xml_with_options(&mut Buffer, &Root, &Options)?;

	// Ensure a trailing newline (plist::XmlWriteOptions does not add one).
	if !Buffer.ends_with(b"\n") {
		Buffer.push(b'\n');
	}

	// Read the existing file and compare bytes before writing.
	let Existing = fs::read(File).ok();

	if Existing.as_ref() == Some(&Buffer) {
		return Ok(false);
	}

	fs::write(File, &Buffer)?;

	Ok(true)
}
