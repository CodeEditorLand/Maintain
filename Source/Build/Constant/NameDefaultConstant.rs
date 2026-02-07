//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/NameDefaultConstant.rs
//=============================================================================//
// Module: NameDefaultConstant
//
// Brief Description: Defines the default project base name constant.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the default project base name
// - Serve as suffix for generated product names
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
// - Product name generation logic
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
// Example 1: Using the default project name
// ```rust
// use crate::Maintain::Source::Build::Constant::NameDefaultConstant;
// let name = NameDefaultConstant;
// // name = "Mountain"
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Default project base name, used as a suffix for generated names.
///
/// This constant specifies the default base name of the Mountain project.
/// It is used as a suffix when generating product names for different
/// build flavors (e.g., "Debug_NodeEnvironment_Mountain").
///
/// # Value
///
/// * `"Mountain"` - The default project base name
pub const NameDefault: &str = "Mountain";
