//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant.rs
//=============================================================================//
// Module: Constant
//
// Brief Description: Build system constants and configuration values.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide file path constants
// - Provide delimiter constants
// - Provide environment variable name constants
// - Serve as single source of truth for constant values
//
// Secondary:
// - Ensure consistent naming across the build system
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Configuration layer
// - Constant definitions
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
// - Constant module pattern
//
// Performance Considerations:
// - Complexity: O(1) - all are static constants
// - Memory usage patterns: Static string slices
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: Yes (all are immutable)
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
// Example 1: Using file path constants
/// ```rust
/// use crate::Maintain::Source::Build::Constant;
/// let cargo_path = project_dir.join(CargoFile);
/// ```
//
// Example 2: Using delimiter constants
/// ```rust
/// use crate::Maintain::Source::Build::Constant;
/// let product_name = parts.join(NameDelimiter);
/// ```
//
// Example 3: Using environment variable constants
/// ```rust
/// use crate::Maintain::Source::Build::Constant;
/// let node_env = std::env::var(NodeEnv);
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

// File path constants
pub const CargoFile: &str = "Cargo.toml";
pub const JsonFile: &str = "tauri.conf.json";
pub const JsonfiveFile: &str = "tauri.conf.json5";

// Default values
pub const DirectoryDefault: &str = "Element/Mountain";
pub const NameDefault: &str = "Mountain";
pub const PrefixDefault: &str = "land.editor.binary";

// Delimiter constants
pub const BackupSuffix: &str = ".Backup";
pub const IdDelimiter: &str = ".";
pub const NameDelimiter: &str = "_";

// Environment variable constants
pub const BrowserEnv: &str = "Browser";
pub const BundleEnv: &str = "Bundle";
pub const CleanEnv: &str = "Clean";
pub const CompileEnv: &str = "Compile";
pub const DebugEnv: &str = "Debug";
pub const DependencyEnv: &str = "Dependency";
pub const DirEnv: &str = "MOUNTAIN_DIR";
pub const LogEnv: &str = "RUST_LOG";
pub const NameEnv: &str = "MOUNTAIN_ORIGINAL_BASE_NAME";
pub const NodeEnv: &str = "NODE_ENV";
pub const NodeVersionEnv: &str = "NODE_VERSION";
pub const PrefixEnv: &str = "MOUNTAIN_BUNDLE_ID_PREFIX";
