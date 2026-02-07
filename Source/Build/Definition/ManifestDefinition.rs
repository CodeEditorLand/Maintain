//=============================================================================//
// File Path: Element/Maintain/Source/Build/Definition/ManifestDefinition.rs
//=============================================================================//
// Module: ManifestDefinition
//
// Brief Description: Defines the Cargo.toml manifest parsing structures.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Parse the package section from Cargo.toml
// - Extract version information from Cargo manifest
// - Support deserialization of TOML into Rust structs
//
// Secondary:
// - None
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Configuration layer
// - TomsL manifest parsing
//
// Dependencies (What this module requires):
// - External crates: serde (Deserialize)
// - Internal modules: None
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions (for version extraction)
// - Version discovery logic
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Data transfer object pattern
// - Serde deserialization
//
// Performance Considerations:
// - Complexity: O(n) - deserialization based on file size
// - Memory usage patterns: Struct with String fields
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: Yes (immutable struct; derives Debug)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: toml::de::Error (propagated)
// - Recovery strategies: Propagate error up the call stack
//
// EXAMPLES:
// =========
//
// Example 1: Parsing a Cargo.toml file
/// ```rust
/// use std::fs;
/// use crate::Maintain::Source::Build::Definition::ManifestDefinition;
/// let content = fs::read_to_string("Cargo.toml")?;
/// let manifest: Manifest = toml::from_str(&content)?;
/// let version = manifest.package.version;
/// println!("Version: {}", version);
/// ```
//
// Example 2: Extracting version from manifest
/// ```rust
/// use crate::Maintain::Source::Build::Definition::ManifestDefinition;
/// fn get_version(cargo_path: &Path) -> Result<String, Error> {
///     let content = fs::read_to_string(cargo_path)?;
///     let manifest: Manifest = toml::from_str(&content)?;
///     Ok(manifest.package.version)
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use serde::Deserialize;

/// Represents the `package` section of a `Cargo.toml` manifest.
///
/// This struct contains metadata extracted from the `[package]` section
/// of a Cargo.toml file. It is used for deserializing TOML content using
/// the `toml` crate's deserialization functionality.
///
/// Currently, this struct only extracts the version information, which
/// is needed for updating the Tauri configuration file with the correct
/// version during the build process.
///
/// # TOML Format
///
/// The expected TOML structure:
/// ```toml
/// [package]
/// name = "Mountain"
/// version = "1.0.0"
/// # ... other fields
/// ```
///
/// # Fields
///
/// - **package**: Contains the metadata extracted from the package section
#[derive(Deserialize, Debug)]
pub struct Manifest {
	/// Represents metadata within the `package` section of `Cargo.toml`.
	///
	/// This nested struct contains the individual metadata fields from the
	/// package section, including the version string.
	package: Meta,
}

impl Manifest {
	/// Retrieves the version string from the manifest.
	///
	/// This method provides access to the version field, which is stored
	/// privately. The version is used when updating the Tauri configuration
	/// file during the build process.
	///
	/// # Returns
	///
	/// A string slice containing the version number.
	///
	/// # Example
	///
	/// ```no_run
	/// use crate::Maintain::Source::Build::Definition::ManifestDefinition;
	/// let version = manifest.get_version();
	/// println!("Application version: {}", version);
	/// ```
	pub fn get_version(&self) -> &str {
		&self.package.version
	}
}

/// Represents metadata within the `package` section of `Cargo.toml`.
///
/// This struct contains individual metadata fields from the package section.
/// Currently, only the version field is stored, but additional fields can
/// be added as needed (e.g., name, description, authors, etc.).
///
/// All fields are private and should be accessed through methods on the
/// parent `Manifest` struct.
#[derive(Deserialize, Debug)]
struct Meta {
	/// The version string from the package metadata.
	///
	/// This field contains the version identifier as specified in the
	/// Cargo.toml file (e.g., "1.0.0", "2.3.4-beta.1").
	version: String,
}
