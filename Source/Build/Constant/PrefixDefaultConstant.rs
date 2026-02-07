//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/PrefixDefaultConstant.rs
//=============================================================================//
// Module: PrefixDefaultConstant
//
// Brief Description: Defines the default bundle identifier prefix constant.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the default prefix for bundle identifiers
// - Ensure consistent reverse domain naming convention
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
// - None
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
// Example 1: Using the default bundle prefix
// ```rust
// use crate::Maintain::Source::Build::Constant::PrefixDefaultConstant;
// let prefix = PrefixDefaultConstant;
// // prefix = "land.editor.binary"
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Default bundle identifier prefix.
///
/// This constant specifies the default reverse domain name prefix for
/// application bundle identifiers. It follows the conventional reverse
/// domain naming scheme used for application identifiers on various
/// platforms (macOS, Windows, Linux).
///
/// The prefix is combined with additional suffixes to create unique bundle
/// identifiers for different build flavors and configurations.
///
/// # Value
///
/// * `"land.editor.binary"` - The default reverse domain prefix
///
/// # Bundle Identifier Format
///
/// Full bundle identifiers follow the format: `{prefix}.{suffix}`
/// e.g., `land.editor.binary.production.mountain`
pub const PrefixDefault: &str = "land.editor.binary";
