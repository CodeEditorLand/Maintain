//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/JsonFileConstant.rs
//=============================================================================//
// Module: JsonFileConstant
//
// Brief Description: Defines the Tauri JSON configuration filename constants.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the standard Tauri JSON configuration filenames
// - Support both JSON and JSON5 format files
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
// - JavaScriptObjectNotation editing functions
// - Configuration detection logic
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - None
//
// Performance Considerations:
// - Complexity: O(1) - constant values
// - Memory usage patterns: Static string slices
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
// Example 1: Using the JSON5 filename constant
// ```rust
// use crate::Maintain::Source::Build::Constant::JsonFileConstant;
// let json5_file = JsonfiveFile;
// // json5_file = "tauri.conf.json5"
// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Tauri JSON5 configuration filename.
///
/// This constant specifies the filename for Tauri configuration files in
/// JSON5 format. JSON5 is an extended JSON format that allows for more
/// human-friendly syntax including comments, trailing commas, and
/// unquoted property names.
///
/// The build system checks for this file first before falling back to
/// the standard JSON format.
///
/// # Value
///
/// * `"tauri.conf.json5"` - The JSON5 Tauri configuration filename
pub const JsonfiveFile: &str = "tauri.conf.json5";

/// Tauri JSON configuration filename.
///
/// This constant specifies the filename for Tauri configuration files in
/// standard JSON format. This is used as a fallback when the JSON5 version
/// is not present in the project directory.
///
/// # Value
///
/// * `"tauri.conf.json"` - The standard Tauri configuration filename
pub const JsonFile: &str = "tauri.conf.json";
