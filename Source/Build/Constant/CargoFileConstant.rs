//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/CargoFileConstant.rs
//=============================================================================//
// Module: CargoFileConstant
//
// Brief Description: Defines the Cargo.toml configuration filename constant.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the standard Cargo.toml filename
// - Ensure consistent access to Rust project manifest
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
// - Toml editing functions
// - Version extraction logic
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
// Example 1: Using the Cargo.toml filename constant
// ```rust
// use crate::Maintain::Source::Build::Constant::CargoFileConstant;
// let cargo_file = CargoFileConstant;
// // cargo_file = "Cargo.toml"
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Cargo configuration filename.
///
/// This constant specifies the standard filename for Rust Cargo project
/// manifest files. The Cargo.toml file contains project metadata,
/// dependencies, and build configuration used by the Cargo package manager.
///
/// During the build process, this file is modified to update the
/// `package.name`, `package.default-run`, and bin names for different
/// build flavors.
///
/// # Value
///
/// * `"Cargo.toml"` - The standard Rust project manifest filename
pub const CargoFile: &str = "Cargo.toml";
