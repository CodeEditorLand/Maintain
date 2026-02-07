//=============================================================================//
// File Path: Element/Maintain/Source/Build/Constant/NameDelimiterConstant.rs
//=============================================================================//
// Module: NameDelimiterConstant
//
// Brief Description: Defines the delimiter for productName parts.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide the delimiter for joining product name components
// - Ensure consistent product name formatting
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
// - Product name generation logic
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
// Example 1: Using the name delimiter constant
// ```rust
// use crate::Maintain::Source::Build::Constant::NameDelimiterConstant;
// let delimiter = NameDelimiterConstant;
// // delimiter = "_"
// ```
//
// Example 2: Creating a product name
/// ```rust
/// let parts = vec!["Debug", "NodeEnvironment", "Mountain"];
/// let product_name = parts.join(NameDelimiterConstant);
/// // product_name = "Debug_NodeEnvironment_Mountain"
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Delimiter for parts of the generated `productName`.
///
/// This constant specifies the character used to join individual components
/// when building a product name for different build flavors. This allows
/// for descriptive product names that indicate the specific build configuration
/// (e.g., "Debug_NodeEnvironment_Mountain").
///
/// The delimiter is used to combine environment, dependency, version, and
/// build flag information into a single product name string.
///
/// # Value
///
/// * `"_"` - Underscore character used as product name delimiter
///
/// # Product Name Format
///
/// Product names follow the format: `{flavor1}_{flavor2}_{base_name}`
/// e.g., `Debug_NodeEnvironment_Mountain`
pub const NameDelimiter: &str = "_";
