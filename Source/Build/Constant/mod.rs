//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/mod.rs
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
/// let cargo_path = project_dir.join(Constant::CargoFile);
/// ```
//
// Example 2: Using delimiter constants
/// ```rust
/// use crate::Maintain::Source::Build::Constant;
/// let product_name = parts.join(Constant::NameDelimiter);
/// ```
//
// Example 3: Using environment variable constants
/// ```rust
/// use crate::Maintain::Source::Build::Constant;
/// let node_env = std::env::var(Constant::NodeEnvironmentConstant);
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

// File path constants
pub use CargoFileConstant::*;
pub use DirectoryDefaultConstant::*;
pub use JsonFileConstant::*;
pub use NameDefaultConstant::*;
pub use PrefixDefaultConstant::*;

// Delimiter constants
pub use BackupSuffixConstant::*;
pub use IdDelimiterConstant::*;
pub use NameDelimiterConstant::*;

// Environment variable constants
pub use BrowserEnvironmentConstant::*;
pub use BundleEnvironmentConstant::*;
pub use CleanEnvironmentConstant::*;
pub use CompileEnvironmentConstant::*;
pub use DebugEnvironmentConstant::*;
pub use DependencyEnvironmentConstant::*;
pub use DirectoryEnvironmentConstant::*;
pub use LogEnvironmentConstant::*;
pub use NameEnvironmentConstant::*;
pub use NodeEnvironmentConstant::*;
pub use NodeVersionEnvironmentConstant::*;
pub use PrefixEnvironmentConstant::*;

// Re-export individual constant modules
pub mod BackupSuffixConstant;

pub mod BrowserEnvironmentConstant;

pub mod BundleEnvironmentConstant;

pub mod CargoFileConstant;

pub mod CleanEnvironmentConstant;

pub mod CompileEnvironmentConstant;

pub mod DebugEnvironmentConstant;

pub mod DependencyEnvironmentConstant;

pub mod DirectoryDefaultConstant;

pub mod DirectoryEnvironmentConstant;

pub mod IdDelimiterConstant;

pub mod JsonFileConstant;

pub mod LogEnvironmentConstant;

pub mod NameDefaultConstant;

pub mod NameDelimiterConstant;

pub mod NameEnvironmentConstant;

pub mod NodeEnvironmentConstant;

pub mod NodeVersionEnvironmentConstant;

pub mod PrefixDefaultConstant;

pub mod PrefixEnvironmentConstant;
