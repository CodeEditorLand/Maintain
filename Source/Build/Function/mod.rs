//=============================================================================//
// File Path: Element/Maintain/Source/Build/Function/mod.rs
//=============================================================================//
// Module: Function
//
// Brief Description: Build system functions and orchestration logic.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide file editing functions (TOML and JavaScriptObjectNotation)
// - Provide string transformation utilities
// - Provide build orchestration functions
// - Provide logging initialization
// - Provide platform detection utilities
//
// Secondary:
// - Export public API for the build system
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Core/Functionality layer
// - Build logic implementation
//
// Dependencies (What this module requires):
// - External crates: env_logger, json5, log, serde, serde_json, std (env, fs,
//   path, process, os), toml, toml_edit
// - Internal modules: Constant, Definition, Error
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Entry point functions
// - Build orchestration
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Function pattern
// - Orchestration pattern
//
// Performance Considerations:
// - Complexity: O(n) - file I/O operations dominate
// - Memory usage patterns: Local variables in functions
// - Hot path optimizations: None
//
// Thread Safety:
// - Thread-safe: Functions are (except Process which requires exclusive file access)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: BuildError
// - Recovery strategies: Guard restores files on error
//
// EXAMPLES:
// =========
//
// Example 1: Editing TOML files
/// ```rust
/// use crate::Maintain::Source::Build::Function;
/// Function::TomlEdit(&cargo_path, "OldName", "NewName")?;
/// ```
//
// Example 2: Editing JavaScriptObjectNotation files
/// ```rust
/// use crate::Maintain::Source::Build::Function;
/// Function::JavaScriptObjectNotationEdit(&config_path, "Product", "Id", "Version", None)?;
/// ```
//
// Example 3: String transformation
/// ```rust
/// use crate::Maintain::Source::Build::Function;
/// let pascal = Function::Pascalize("kebab_case");
/// ```
//
// Example 4: Build orchestration
/// ```rust
/// use crate::Maintain::Source::Build::Function;
/// Function::Process(&argument)?;
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

// Public API functions
pub use FnFunction::{Fn, Fn as Main};
pub use GetTauriTargetTripleFunction::GetTauriTargetTriple;
pub use JavaScriptObjectNotationEditingFunction::*;
pub use LoggerFunction::*;
pub use PascalizeFunction::*;
pub use ProcessFunction::*;
pub use TomlEditingFunction::*;
pub use WordsFromPascalFunction::*;

// Make Pascalize and WordsFromPascal public for other modules to use
pub use PascalizeFunction::Pascalize;
pub use WordsFromPascalFunction::WordsFromPascal;

// Function modules
pub mod FnFunction;

pub mod GetTauriTargetTripleFunction;

pub mod JavaScriptObjectNotationEditingFunction;

pub mod LoggerFunction;

pub mod PascalizeFunction;

pub mod ProcessFunction;

pub mod TomlEditingFunction;

pub mod WordsFromPascalFunction;

// Re-export for internal module use
// Internal use statements
mod internal_functions {
	pub(in crate::Build) use super::GetTauriTargetTripleFunction;
	pub(in crate::Build) use super::WordsFromPascal;
}
