//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/NameEnvironmentConstant.rs
//=============================================================================//
// Module: NameEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the original project base name.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for project base name configuration
// - Enable flexible project naming without code modification
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
// export MOUNTAIN_ORIGINAL_BASE_NAME="MyProject"
// ```
//
// Example 2: Reading the environment variable in Rust
// ```rust
// use std::env;
// use crate::Maintain::Source::Build::Constant::NameEnvironmentConstant;
// let name = env::var(NameEnvironmentConstant).unwrap_or_else(|_| "Mountain".to_string());
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for the original base name of the project.
///
/// This constant specifies the environment variable name used to configure
/// the original base name of the project. When set, it overrides the default
/// project name ("Mountain") and serves as the suffix for generated product
/// names.
///
/// This allows the build system to be configured for different project names
/// without modifying the source code, enabling flexible deployment and
/// customization.
///
/// # Value
///
/// * `"MOUNTAIN_ORIGINAL_BASE_NAME"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export MOUNTAIN_ORIGINAL_BASE_NAME="MyProject"
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --name MyProject
/// ```
///
/// The base name is used as the final component of generated product names,
/// e.g., "Debug_NodeEnvironment_**MyProject**"
pub const NameEnv: &str = "MOUNTAIN_ORIGINAL_BASE_NAME";
