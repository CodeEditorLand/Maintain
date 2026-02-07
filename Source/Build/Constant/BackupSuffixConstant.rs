//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/BackupSuffixConstant.rs
//=============================================================================//
// Module: BackupSuffixConstant
//
// Brief Description: Defines the suffix for backup files created by the Guard.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the suffix for backup file naming
// - Ensure consistent backup file identification
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
// - Guard implementation
// - Build orchestration functions
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
// Example 1: Using the backup suffix constant
// ```rust
// use crate::Maintain::Source::Build::Constant::BackupSuffixConstant;
// let suffix = BackupSuffixConstant;
// // suffix = ".Backup"
// ```
//
// Example 2: Creating a backup filename
// ```rust
// let original = "Cargo.toml";
// let backup = format!("{}{}", original, BackupSuffixConstant);
// // backup = "Cargo.toml.Backup"
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Suffix used for backup files created by the `Guard`.
///
/// This constant specifies the suffix appended to original filenames to
/// create backup files. The Guard pattern uses this to create temporary
/// backups of configuration files (Cargo.toml, tauri.conf.json) before
/// modifying them for different build flavors.
///
/// The backup is automatically restored when the Guard goes out of scope,
/// ensuring the original file state is preserved regardless of build success
/// or failure.
///
/// # Value
///
/// * `".Backup"` - The suffix appended to backup filenames
///
/// # Backup Filename Format
///
/// Backup files follow the format: `{original_filename}.{extension}{suffix}`
/// e.g., `Cargo.toml.Backup` or `tauri.conf.json.Backup`
pub const BackupSuffix: &str = ".Backup";
