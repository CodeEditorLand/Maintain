//=============================================================================//
// File Path: Element/Maintain/Source/Build/Function/TomlEditingFunction.rs
//=============================================================================//
// Module: TomlEditingFunction
//
// Brief Description: Implements TOML file editing for Cargo.toml.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Dynamically modify name fields in Cargo.toml files
// - Update package.name, package.default-run, lib.name, and bin.name
// - Preserve the original structure and formatting
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
// - TomsL editing functionality
//
// Dependencies (What this module requires):
// - External crates: toml_edit, std (fs, log)
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
// - Builder pattern (via toml_edit)
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
// - Error types returned: BuildError (Edit type)
// - Recovery strategies: Propagate error up; Guard restores original file
//
// EXAMPLES:
// =========
//
// Example 1: Product name change
/// ```rust
/// use crate::Maintain::Source::Build::Function::TomlEditingFunction;
/// let cargo_path = PathBuf::from("Cargo.toml");
/// let old_name = "Mountain";
/// let new_name = "Debug_NodeEnvironment_Mountain";
/// let modified = TomlEdit(&cargo_path, old_name, new_name)?;
/// if modified {
///     println!("Cargo.toml was updated");
/// }
/// ```
//
// Example 2: No change needed
/// ```rust
/// use crate::Maintain::Source::Build::Function::TomlEditingFunction;
/// let modified = TomlEdit(&cargo_path, "Mountain", "Mountain")?;
/// // modified = false, no changes made
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use crate::Build::BuildError;

use log::{debug, info, warn};
use std::{fs, path::Path};
use toml_edit::{DocumentMut as TomlDocument, Item as TomlItem, Value as TomlValue};

/// Dynamically modifies specific name fields within a `Cargo.toml` file.
///
/// This function searches for and updates the following fields if they match
/// the old name:
/// - `package.name` - The package name identifier
/// - `package.default-run` - The default binary to run
/// - `lib.name` - The library name
/// - `bin.name` - The binary name (first occurrence)
///
/// The function preserves the existing file structure and ensures that only
/// matching names are updated, preventing unintended modifications.
///
/// # Parameters
///
/// * `File` - Path to the Cargo.toml file to modify
/// * `Old` - The current name to search for and replace
/// * `Current` - The new name to set
///
/// # Returns
///
/// Returns a `Result<bool>` indicating:
/// - `Ok(true)` - The file was modified and saved
/// - `Ok(false)` - No changes were made (either no matches or old == current)
/// - `Err(BuildError)` - An error occurred during modification
///
/// # Errors
///
/// * `BuildError::Io` - If the file cannot be read or written
/// * `BuildError::Edit` - If the TOML document cannot be parsed or modified
///
/// # Behavior
///
/// - If `Old` equals `Current`, returns `Ok(false)` without modifying the file
/// - Only updates fields that exactly match the old name
/// - Updates are atomic: the file is written only if at least one change occurs
/// - Logs all changes made at INFO level
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::Function::TomlEditingFunction;
/// let path = PathBuf::from("Cargo.toml");
/// let modified = TomlEdit(&path, "Mountain", "Debug_Mountain")?;
/// if modified {
///     println!("Successfully updated Cargo.toml");
/// }
/// ```
pub fn TomlEdit(File: &Path, Old: &str, Current: &str) -> Result<bool, BuildError> {
	debug!(target: "Build::Toml", "Attempting to modify TOML file: {}", File.display());

	let Data = fs::read_to_string(File)?;

	if Old == Current {
		info!(
			target: "Build::Toml",
			"Old name '{}' is the same as current name '{}'. No changes needed for {}.",
			Old,
			Current,
			File.display()
		);

		return Ok(false);
	}

	debug!(target: "Build::Toml", "Old name: '{}', Current name: '{}'", Old, Current);

	let mut Parsed: TomlDocument = Data.parse()?;

	let mut PackageChange = false;

	let mut LibraryChange = false;

	let mut BinaryChange = false;

	let mut DefaultChange = false;

	// Update package.name
	if let Some(PackageTable) = Parsed.get_mut("package").and_then(|Item| Item.as_table_mut()) {
		if let Some(NameItem) = PackageTable.get_mut("name") {
			if NameItem.as_str() == Some(Old) {
				*NameItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

				PackageChange = true;

				debug!(target: "Build::Toml", "Changed package.name");
			}
		}

		// Update package.default-run
		if let Some(RunItem) = PackageTable.get_mut("default-run") {
			if RunItem.as_str() == Some(Old) {
				*RunItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

				DefaultChange = true;

				debug!(target: "Build::Toml", "Changed package.default-run");
			}
		}
	}

	// Update lib.name
	if let Some(LibraryTable) = Parsed.get_mut("lib").and_then(|Item| Item.as_table_mut()) {
		if let Some(NameItem) = LibraryTable.get_mut("name") {
			if NameItem.as_str() == Some(Old) {
				*NameItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

				LibraryChange = true;

				debug!(target: "Build::Toml", "Changed lib.name");
			}
		}
	}

	// Update bin.name (first occurrence)
	if let Some(BinArray) = Parsed.get_mut("bin").and_then(|Item| Item.as_array_of_tables_mut()) {
		for Table in BinArray.iter_mut() {
			if let Some(NameItem) = Table.get_mut("name") {
				if NameItem.as_str() == Some(Old) {
					*NameItem = TomlItem::Value(TomlValue::String(toml_edit::Formatted::new(Current.to_string())));

					BinaryChange = true;

					debug!(target: "Build::Toml", "Changed a bin.name entry to '{}'", Current);

					break;
				}
			}
		}
	}

	// Write the file if any changes were made
	if PackageChange || LibraryChange || BinaryChange || DefaultChange {
		let Output = Parsed.to_string();

		fs::write(File, Output)?;

		let mut ModifiedItems = Vec::new();

		if PackageChange {
			ModifiedItems.push("package.name");
		}

		if DefaultChange {
			ModifiedItems.push("package.default-run");
		}

		if LibraryChange {
			ModifiedItems.push("lib.name");
		}

		if BinaryChange {
			ModifiedItems.push("bin.name");
		}

		info!(
			target: "Build::Toml",
			"Temporarily changed {} in {} to: {}",
			ModifiedItems.join(", "),
			File.display(),
			Current
		);

		Ok(true)
	} else {
		warn!(
			target: "Build::Toml",
			"Name '{}' not found in relevant sections of {}. No changes made to file.",
			Old,
			File.display()
		);

		Ok(false)
	}
}
