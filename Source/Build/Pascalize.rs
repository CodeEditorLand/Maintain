//=============================================================================//
// File Path: Element/Maintain/Source/Build/Pascalize.rs
//=============================================================================//
// Module: Pascalize
//
// Brief Description: Converts kebab-case and snake_case strings to PascalCase.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Convert kebab-case strings to PascalCase
// - Convert snake_case strings to PascalCase
// - Handle delimiter-separated strings of any length
//
// Secondary:
// - None
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Utility layer
// - String transformation utilities
//
// Dependencies (What this module requires):
// - External crates: None
// - Internal modules: None
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - Product name generation logic
// - Bundle identifier generation logic
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - String transformation pattern
// - Functional pattern
//
// Performance Considerations:
// - Complexity: O(n) - where n is the length of the input string
// - Memory usage patterns: Creates new String with allocated capacity
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: Yes (pure function with immutable input and output)
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
// Example 1: Kebab-case conversion
/// ```rust
/// use crate::Maintain::Source::Build::Pascalize;
/// let result = Pascalize("development");
/// assert_eq!(result, "Development");
/// ```
// Example 2: Snake_case conversion
/// ```rust
/// use crate::Maintain::Source::Build::Pascalize;
/// let result = Pascalize("node_environment");
/// assert_eq!(result, "NodeEnvironment");
/// ```
// Example 3: Mixed delimiters
/// ```rust
/// use crate::Maintain::Source::Build::Pascalize;
/// let result = Pascalize("tauri-apps_tauri");
/// assert_eq!(result, "TauriAppsTauri");
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Converts a kebab-case or snake_case string to `PascalCase`.
///
/// This function processes strings separated by hyphens (`-`) or underscores
/// (`_`) and converts them to PascalCase by capitalizing the first letter of
/// each word. The function handles strings with single or multiple delimiters
/// and filters out empty segments.
///
/// # Parameters
///
/// * `Text` - The input string to convert (kebab-case or snake_case)
///
/// # Returns
///
/// A new String in PascalCase format.
///
/// # Behavior
///
/// - Splits the input on both hyphen (`-`) and underscore (`_`) characters
/// - Filters out empty segments (from consecutive delimiters)
/// - Capitalizes the first character of each segment
/// - Preserves the case of remaining characters
/// - Joins all segments together without delimiters
///
/// # Examples
///
/// ```
/// use crate::Maintain::Source::Build::Pascalize;
/// assert_eq!(Pascalize("development"), "Development");
/// assert_eq!(Pascalize("node_environment"), "NodeEnvironment");
/// assert_eq!(Pascalize("tauri-apps"), "TauriApps");
/// assert_eq!(Pascalize("my-app_name"), "MyAppName");
/// ```
///
/// # Edge Cases
///
/// - Empty string returns an empty string
/// - Strings with only delimiters return an empty string
/// - Single word strings are capitalized
/// - Words already in PascalCase are not modified (no delimiter detection)
pub fn Pascalize(Text:&str) -> String {
	Text.split(|c:char| c == '-' || c == '_')
		.filter(|s| !s.is_empty())
		.map(|s| {
			let mut c = s.chars();

			c.next()
				.map_or(String::new(), |f| f.to_uppercase().collect::<String>() + c.as_str())
		})
		.collect()
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_kebab_case() {
		assert_eq!(Pascalize("development"), "Development");

		assert_eq!(Pascalize("node-version"), "NodeVersion");

		assert_eq!(Pascalize("tauri-apps"), "TauriApps");
	}

	#[test]
	fn test_snake_case() {
		assert_eq!(Pascalize("node_environment"), "NodeEnvironment");

		assert_eq!(Pascalize("my_variable_name"), "MyVariableName");
	}

	#[test]
	fn test_mixed_delimiters() {
		assert_eq!(Pascalize("tauri-apps_tauri"), "TauriAppsTauri");

		assert_eq!(Pascalize("my-app_name"), "MyAppName");
	}

	#[test]
	fn test_empty_string() {
		assert_eq!(Pascalize(""), "");
	}

	#[test]
	fn test_only_delimiters() {
		assert_eq!(Pascalize("---"), "");

		assert_eq!(Pascalize("___"), "");

		assert_eq!(Pascalize("-_-"), "");
	}

	#[test]
	fn test_single_word() {
		assert_eq!(Pascalize("hello"), "Hello");

		assert_eq!(Pascalize("WORLD"), "WORLD");
	}
}
