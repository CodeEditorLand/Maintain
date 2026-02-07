//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/BrowserEnvironmentConstant.rs
//=============================================================================//
// Module: BrowserEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the browser build flag.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for browser build flag configuration
// - Enable browser-specific build behavior activation
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
// export Browser=true
// ```
//
// Example 2: Passing as command-line flag
// ```sh
/// --browser true
// ```
//
// Example 3: Reading the environment variable in Rust
/// ```rust
/// use std::env;
/// use crate::Maintain::Source::Build::Constant::BrowserEnvironmentConstant;
/// let browser = env::var(BrowserEnvironmentConstant).ok();
/// if browser.as_deref() == Some("true") {
///     // Enable browser-specific behavior
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for the "Browser" build flag.
///
/// This constant specifies the environment variable name used to control
/// browser-specific build behavior. When set to "true", it enables browser
/// configuration and modifies the generated product name and bundle
/// identifier accordingly.
///
/// This flag is typically used when building web browser versions of the
/// application or when the build process needs to include browser-specific
/// resources and configurations.
///
/// # Value
///
/// * `"Browser"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export Browser=true
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --browser true
/// ```
///
/// When enabled:
/// - Product name includes "Browser" suffix (e.g., "Debug_Browser_Mountain")
/// - Bundle identifier includes "browser" component (e.g., "land.editor.binary.debug.browser.mountain")
pub const BrowserEnv: &str = "Browser";
