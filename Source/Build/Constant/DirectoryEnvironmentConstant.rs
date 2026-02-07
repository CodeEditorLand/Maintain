//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/DirectoryEnvironmentConstant.rs
//=============================================================================//
// Module: DirectoryEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the project directory.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for project directory configuration
// - Enable flexible project directory specification
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
// - Command-line interface setup
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Configuration through environment variables pattern
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
// Example 1: Setting the environment variable
// ```sh
// export MOUNTAIN_DIR="Element/CustomDirectory"
// ```
//
// Example 2: Reading the environment variable in Rust
// ```rust
// use std::env;
// use crate::Maintain::Source::Build::Constant::DirectoryEnvironmentConstant;
// let dir = env::var(DirectoryEnvironmentConstant).unwrap_or_else(|_| "Element/Mountain".to_string());
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for the project directory.
///
/// This constant specifies the environment variable name used to configure
/// the project directory for the build system. When set, it overrides the
/// default directory path ("Element/Mountain").
///
/// This allows build scripts to be configured without modifying code,
/// enabling flexible deployment across different directory structures.
///
/// # Value
///
/// * `"MOUNTAIN_DIR"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export MOUNTAIN_DIR="Custom/Path"
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --directory Custom/Path
/// ```
pub const DirEnv: &str = "MOUNTAIN_DIR";
