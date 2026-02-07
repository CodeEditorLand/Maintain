//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/CleanEnvironmentConstant.rs
//=============================================================================//
// Module: CleanEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the clean build flag.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for clean build flag configuration
// - Enable clean build behavior activation
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
// export Clean=true
// ```
//
// Example 2: Passing as command-line flag
// ```sh
/// --clean true
// ```
//
// Example 3: Reading the environment variable in Rust
/// ```rust
/// use std::env;
/// use crate::Maintain::Source::Build::Constant::CleanEnvironmentConstant;
/// let clean = env::var(CleanEnvironmentConstant).ok();
/// if clean.as_deref() == Some("true") {
///     // Enable clean build behavior
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for the "Clean" build flag.
///
/// This constant specifies the environment variable name used to control
/// clean build behavior. When set to "true", it enables clean configuration
/// and modifies the generated product name and bundle identifier accordingly.
///
/// This flag is typically used when performing a clean build from scratch,
/// removing all build artifacts and cached data before rebuilding.
///
/// # Value
///
/// * `"Clean"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export Clean=true
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --clean true
/// ```
///
/// When enabled:
/// - Product name includes "Clean" suffix (e.g., "Debug_Clean_Mountain")
/// - Bundle identifier includes "clean" component (e.g., "land.editor.binary.debug.clean.mountain")
pub const CleanEnv: &str = "Clean";
