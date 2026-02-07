//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/DebugEnvironmentConstant.rs
//=============================================================================//
// Module: DebugEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the debug build flag.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for debug build flag configuration
// - Enable debug-specific build behavior activation
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
// - Build flags pattern
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
// export Debug=true
// ```
//
// Example 2: Passing as command-line flag
// ```sh
/// --debug true
// ```
//
// Example 3: Reading the environment variable in Rust
/// ```rust
/// use std::env;
/// use crate::Maintain::Source::Build::Constant::DebugEnvironmentConstant;
/// let debug = env::var(DebugEnvironmentConstant).ok();
/// if debug.as_deref() == Some("true") {
///     // Enable debug-specific behavior
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for the "Debug" build flag.
///
/// This constant specifies the environment variable name used to control
/// debug-specific build behavior. When set to "true", it enables debug
/// configuration and modifies the generated product name and bundle
/// identifier accordingly.
///
/// This flag is also automatically detected if the build command contains
/// the "--debug" flag, allowing the build system to infer debug builds without
/// explicit configuration.
///
/// # Value
///
/// * `"Debug"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export Debug=true
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --debug true
/// ```
///
/// When enabled:
/// - Product name includes "Debug" suffix (e.g., "Debug_NodeEnvironment_Mountain")
/// - Bundle identifier includes "debug" component (e.g., "land.editor.binary.debug.node.environment.mountain")
pub const DebugEnv: &str = "Debug";
