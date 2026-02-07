//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/NodeVersionEnvironmentConstant.rs
//=============================================================================//
// Module: NodeVersionEnvironmentConstant
//
// Brief Description: Defines the environment variable name for the Node.js version selection.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the environment variable name for Node.js version configuration
// - Enable Node.js version-specific sidecar bundling
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
// - Sidecar bundling logic
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
// Example 1: Setting the environment variable for Node.js version 22
// ```sh
// export NODE_VERSION=22
// ```
//
// Example 2: Passing as command-line argument
// ```sh
/// --node-version 18
// ```
//
// Example 3: Reading the environment variable in Rust
/// ```rust
/// use std::env;
/// use crate::Maintain::Source::Build::Constant::NodeVersionEnvironmentConstant;
/// let node_version = env::var(NodeVersionEnvironmentConstant).ok();
/// if let Some(version) = node_version {
///     // Bundle the specified Node.js version with the application
///     println!("Bundling Node.js version: {}", version);
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Environment variable for selecting the Node.js sidecar version.
///
/// This constant specifies the environment variable name used to configure
/// which Node.js version should be bundled as a sidecar binary with the
/// application. The version should be specified as a numeric string (e.g., "22").
///
/// When a version is specified, the build system locates the pre-downloaded
/// Node.js executable from the Element/SideCar directory and stages it for
/// bundling with the Tauri application.
///
/// # Value
///
/// * `"NODE_VERSION"` - The environment variable name
///
/// # Usage
///
/// Set via environment:
/// ```sh
/// export NODE_VERSION=22
/// ```
///
/// Or pass as command-line argument:
/// ```sh
/// --node-version 18
/// ```
///
/// Example values:
/// - `22` - Node.js version 22
/// - `20` - Node.js version 20
/// - `18` - Node.js version 18
///
/// When set:
/// - Product name includes version information (e.g., "22NodeVersion_Mountain")
/// - Bundle identifier includes version components (e.g., "land.editor.binary.node.22.mountain")
/// - The specified Node.js version is bundled as a sidecar binary
///
/// # Sidecar Binary Location
///
/// The Node.js executable is sourced from:
/// - Windows: `./Element/SideCar/{triple}/NODE/{version}/node.exe`
/// - Unix-like: `./Element/SideCar/{triple}/NODE/{version}/bin/node`
///
/// Where `{triple}` is the target triple (e.g., `x86_64-apple-darwin`, `aarch64-apple-darwin`)
pub const NodeVersionEnv: &str = "NODE_VERSION";
