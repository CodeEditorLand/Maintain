//=============================================================================//
// File Path: Element/Maintain/Source/Build/Error/mod.rs
//=============================================================================//
// Module: Error
//
// Brief Description: Build system error types and handling.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Define comprehensive error types for build operations
// - Provide automatic error conversions from underlying libraries
// - Support clear error messages for debugging
//
// Secondary:
// - Enable proper error propagation through the build system
// - Provide context for build failure diagnosis
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
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - File manipulation functions
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
// - Hot path optimizations: None
//
// Thread Safety:
// - Thread-safe: Yes (immutable error enum)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: This module defines all build errors
// - Recovery strategies: Propagate errors up; Guard restores files
//
// EXAMPLES:
// =========
//
// Example 1: Returning an error
/// ```rust
/// use crate::Maintain::Source::Build::Error::BuildError;
/// fn read_file(path: &Path) -> Result<String, BuildError> {
///     fs::read_to_string(path)?
/// }
/// ```
//
// Example 2: Handling errors
/// ```rust
/// match result {
///     Ok(_) => println!("Success"),
///     Err(BuildError::Io(e)) => println!("IO error: {}", e),
///     Err(BuildError::Missing(path)) => println!("Missing: {}", path.display()),
///     Err(e) => println!("Error: {}", e),
/// }
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

pub mod BuildError;

// Re-export the Error type and variants for convenience
pub use self::BuildError::Error;
pub use self::BuildError::Error::*;
