//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/DependencyEnvironmentConstant.rs
//=============================================================================//
// Module: DependencyEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the dependency flavour.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for dependency configuration
// - Enable dependency-specific build behavior activation
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
// Example 1: Setting the environment variable with a specific dependency
// ```sh
// export Dependency=tauri-apps/tauri
// ```
//
// Example 2: Passing as command-line argument
// ```sh
/// --dependency myorg/myrepo
// ```
//
// Example 3: Reading the environment variable in Rust
/// ```rust
/// use std::env;
/// use crate::Maintain::Source::Build::Constant::DependencyEnvironmentConstant;
/// let dependency = env::var(DependencyEnvironmentConstant).ok();
/// if let Some(dep) = dependency {
///     // Use dependency information for the build
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for specifying a dependency flavour.
///
/// This constant specifies the environment variable name used to configure
// dependency information for the build. The value can be:
/// - A boolean string ("true" for generic dependencies)
/// - An organization/repository pair in "org/repo" format
/// - A custom dependency identifier
///
/// The dependency information is incorporated into the generated product name
/// and bundle identifier to create unique identifiers for builds with
/// different dependency configurations.
///
/// # Value
///
/// * `"Dependency"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export Dependency=tauri-apps/tauri
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --dependency myorg/myrepo
/// ```
///
/// Example values:
/// - `true` - Generic dependency (becomes "GenericDependency" in product name)
/// - `tauri-apps/tauri` - Specific dependency (becomes "TauriAppsTauriDependency")
/// - `my-custom-dep` - Custom dependency name
///
/// When set:
/// - Product name includes dependency information
/// - Bundle identifier includes dependency components (e.g., "land.editor.binary.tauri.apps.tauri.dependency.mountain")
pub const DependencyEnv: &str = "Dependency";
