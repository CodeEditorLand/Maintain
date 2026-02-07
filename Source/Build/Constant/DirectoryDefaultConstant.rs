//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/DirectoryDefaultConstant.rs
//=============================================================================//
// Module: DirectoryDefaultConstant
//
// Brief Description: Defines the default project directory path constant.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the default directory path for the Mountain project
// - Serve as fallback when no project directory is specified
//
// Secondary:
// - None
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Configuration layer
// - Build system configuration constants
//
// Dependencies (What this module requires):
// - External crates: None
// - Internal modules: None
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - Argument parsing module
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - None
//
// Performance Considerations:
// - Complexity: O(1) - constant value
// - Memory usage patterns: Static string slice
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: Yes (immutable)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: None
// - Recovery strategies: Not applicable
//
// EXAMPLES:
// =========
//
// Example 1: Using the default directory
// ```rust
// use crate::Maintain::Source::Build::Constant::DirectoryDefaultConstant;
// let dir = DirectoryDefaultConstant;
// // dir = "Element/Mountain"
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Default project directory relative to the workspace root.
///
/// This constant specifies the default location of the Mountain project
/// within the workspace. It serves as a fallback value when no project
/// directory is explicitly specified via command-line arguments or
/// environment variables.
///
/// # Value
///
/// * `"Element/Mountain"` - The relative path from the workspace root
pub const DirectoryDefault: &str = "Element/Mountain";
