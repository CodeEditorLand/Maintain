//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/PrefixEnvironmentConstant.rs
//=============================================================================//
// Module: PrefixEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the bundle identifier prefix.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for bundle prefix configuration
// - Enable flexible bundle identifier customization
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
// - Bundle identifier generation logic
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
// export MOUNTAIN_BUNDLE_ID_PREFIX="com.mycompany.app"
// ```
//
// Example 2: Reading the environment variable in Rust
// ```rust
// use std::env;
// use crate::Maintain::Source::Build::Constant::PrefixEnvironmentConstant;
// let prefix = env::var(PrefixEnvironmentConstant).unwrap_or_else(|_| "land.editor.binary".to_string());
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for the bundle identifier prefix.
///
/// This constant specifies the environment variable name used to configure
/// the prefix for application bundle identifiers. When set, it overrides the
/// default prefix ("land.editor.binary").
///
/// The bundle identifier prefix follows the reverse domain naming convention
/// and is combined with additional details to create unique identifiers for
/// different build flavors and configurations.
///
/// # Value
///
/// * `"MOUNTAIN_BUNDLE_ID_PREFIX"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
// export MOUNTAIN_BUNDLE_ID_PREFIX="com.mycompany.app"
// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --prefix com.mycompany.app
/// ```
///
/// The prefix is used as the base of bundle identifiers, e.g.,
/// "com.mycompany.app.production.mountain"
pub const PrefixEnv: &str = "MOUNTAIN_BUNDLE_ID_PREFIX";
