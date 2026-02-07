//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/BundleEnvironmentConstant.rs
//=============================================================================//
// Module: BundleEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the bundle build flag.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for bundle build flag configuration
// - Enable bundle-specific build behavior activation
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
// export Bundle=true
// ```
//
// Example 2: Passing as command-line flag
// ```sh
// --bundle true
// ```
//
// Example 3: Reading the environment variable in Rust
/// ```rust
/// use std::env;
/// use crate::Maintain::Source::Build::Constant::BundleEnvironmentConstant;
/// let bundle = env::var(BundleEnvironmentConstant).ok();
/// if bundle.as_deref() == Some("true") {
///     // Enable bundle-specific behavior
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for the "Bundle" build flag.
///
/// This constant specifies the environment variable name used to control
/// bundle-specific build behavior. When set to "true", it enables bundle
/// configuration and modifies the generated product name and bundle
/// identifier accordingly.
///
/// This flag is typically used when building distribution packages (deb, rpm,
/// dmg, msi, etc.) and ensures the build process includes all necessary
// bundling steps.
///
/// # Value
///
/// * `"Bundle"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export Bundle=true
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --bundle true
/// ```
///
/// When enabled:
/// - Product name includes "Bundle" suffix (e.g., "Debug_Bundle_Mountain")
/// - Bundle identifier includes "bundle" component (e.g., "land.editor.binary.debug.bundle.mountain")
pub const BundleEnv: &str = "Bundle";
