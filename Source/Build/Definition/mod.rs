//=============================================================================//
// File Path: Element/Maintain/Source/Build/Definition/mod.rs
//=============================================================================//
// Module: Definition
//
// Brief Description: Build system type definitions and data structures.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Define argument parsing structures
// - Define configuration file parsing structures
// - Define file guard structures
//
// Secondary:
// - Provide type-safe access to build configuration
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Data structures layer
// - Type definitions
//
// Dependencies (What this module requires):
// - External crates: clap, serde, std (fs, log, path)
// - Internal modules: Constant, Error
// - Traits implemented: Drop (for Guard), Parser (for Argument)
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - Entry point functions
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Builder pattern (via clap derive)
// - Data transfer object pattern
// - RAII pattern (Guard)
//
// Performance Considerations:
// - Complexity: O(n) - depends on operation
// - Memory usage patterns: Struct-based memory layout
// - Hot path optimizations: None
//
// Thread Safety:
// - Thread-safe: Yes (Argument derives Clone, Guard not thread-safe by design)
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
// Example 1: Parsing arguments
/// ```rust
/// use crate::Maintain::Source::Build::Definition;
/// let argument = Definition::Argument::parse();
/// ```
//
// Example 2: Creating a guard
/// ```rust
/// use crate::Maintain::Source::Build::Definition;
/// let guard = Definition::Guard::New(file_path, description.to_string())?;
/// ```
//
// Example 3: Parsing cargo.toml
/// ```rust
/// use crate::Maintain::Source::Build::Definition;
/// let manifest: Manifest = toml::from_str(&content)?;
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

pub use ArgumentDefinition::*;
pub use GuardDefinition::*;
pub use ManifestDefinition::*;

pub mod ArgumentDefinition;

pub mod GuardDefinition;

pub mod ManifestDefinition;
