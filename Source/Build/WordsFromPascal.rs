//=============================================================================//
// File Path: Element/Maintain/Source/Build/WordsFromPascal.rs
//=============================================================================//
// Module: WordsFromPascal
//
// Brief Description: Converts PascalCase strings into lowercase word vectors.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Convert PascalCase strings to lowercase word vectors
// - Split PascalCase into constituent words
// - Handle multi-letter uppercase sequences
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
// - Memory usage patterns: Creates new String for each word
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
// Example 1: Simple PascalCase
/// ```rust
/// use crate::Maintain::Source::Build::WordsFromPascal;
/// let words = WordsFromPascal("Development");
/// assert_eq!(words, vec!["development"]);
/// ```
// Example 2: Multi-word PascalCase
/// ```rust
/// use crate::Maintain::Source::Build::WordsFromPascal;
/// let words = WordsFromPascal("NodeEnvironment");
/// assert_eq!(words, vec!["node", "environment"]);
/// ```
// Example 3: Acronyms
/// ```rust
/// use crate::Maintain::Source::Build::WordsFromPascal;
/// let words = WordsFromPascal("TauriAppsTauri");
/// assert_eq!(words, vec!["tauri", "apps", "tauri"]);
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

/// Converts a `PascalCase` string into a vector of its lowercase constituent
/// words.
///
/// This function splits PascalCase strings into their individual words by
/// detecting word boundaries where lowercase letters appear after uppercase
/// letters. It handles common cases including:
///
/// - Simple words: `"Hello"` → `["hello"]`
/// - Standard PascalCase: `"HelloWorld"` → `["hello", "world"]`
/// - Multi-letter acronyms: `"TauriApps"` → `["tauri", "apps"]`
///
/// The algorithm tracks when transitions occur from lowercase to uppercase
/// to determine word boundaries.
///
/// # Parameters
///
/// * `Text` - The PascalCase string to split
///
/// # Returns
///
/// A vector of lowercase strings representing the constituent words.
///
/// # Behavior
///
/// - Splits at boundaries where lowercase letters are followed by uppercase
/// - Handles consecutive uppercase letters as part of the same word
/// - Converts all words to lowercase
/// - Returns an empty vector for empty strings
///
/// # Examples
///
/// ```
/// use crate::Maintain::Source::Build::WordsFromPascal;
/// assert_eq!(WordsFromPascal("Hello"), vec!["hello"]);
/// assert_eq!(WordsFromPascal("HelloWorld"), vec!["hello", "world"]);
/// assert_eq!(WordsFromPascal("NodeEnvironment"), vec!["node", "environment"]);
/// assert_eq!(WordsFromPascal("TauriAppsTauri"), vec!["tauri", "apps", "tauri"]);
/// assert_eq!(WordsFromPascal(""), Vec::<String>::new());
/// ```
///
/// # Edge Cases
///
/// - Empty string returns an empty vector
/// - Single character strings work correctly
/// - Strings with all lowercase are treated as a single word
/// - Strings with all uppercase are treated as a single word
///
/// # Algorithm
///
/// The function iterates through each character:
/// 1. If the character is uppercase:
///    - If we have accumulated lowercase characters, end the current word
///    - Add the uppercase character to the current word
///    - Track that we're processing uppercase characters
/// 2. If the character is lowercase:
///    - Add to the current word
///    - Track that we're processing lowercase characters
/// 3. After iteration, add the final word
///
/// This ensures that multi-letter sequences like "Apps" stay together while
/// properly splitting "HelloWorld" into "hello" and "world".
pub fn WordsFromPascal(Text:&str) -> Vec<String> {

	if Text.is_empty() {

		return Vec::new();
	}

	let mut Words = Vec::new();

	let mut CurrentWord = String::new();

	let mut LastCharWasUppercase = false;

	for Char in Text.chars() {

		if Char.is_uppercase() {

			if !CurrentWord.is_empty() && !LastCharWasUppercase {

				Words.push(CurrentWord.to_ascii_lowercase());

				CurrentWord.clear();
			}

			CurrentWord.push(Char);

			LastCharWasUppercase = true;
		} else {

			CurrentWord.push(Char);

			LastCharWasUppercase = false;
		}
	}

	if !CurrentWord.is_empty() {

		Words.push(CurrentWord.to_ascii_lowercase());
	}

	Words
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_single_word() {

		assert_eq!(WordsFromPascal("Hello"), vec!["hello"]);

		assert_eq!(WordsFromPascal("World"), vec!["world"]);
	}

	#[test]
	fn test_two_words() {

		assert_eq!(WordsFromPascal("HelloWorld"), vec!["hello", "world"]);

		assert_eq!(WordsFromPascal("NodeEnvironment"), vec!["node", "environment"]);
	}

	#[test]
	fn test_multiple_words() {

		assert_eq!(WordsFromPascal("TauriAppsTauri"), vec!["tauri", "apps", "tauri"]);

		assert_eq!(WordsFromPascal("MyAwesomeAppName"), vec!["my", "awesome", "app", "name"]);
	}

	#[test]
	fn test_empty_string() {

		assert_eq!(WordsFromPascal(""), Vec::<String>::new());
	}

	#[test]
	fn test_all_lowercase() {

		assert_eq!(WordsFromPascal("hello"), vec!["hello"]);
	}

	#[test]
	fn test_all_uppercase() {

		assert_eq!(WordsFromPascal("HELLO"), vec!["hello"]);
	}

	#[test]
	fn test_single_character() {

		assert_eq!(WordsFromPascal("A"), vec!["a"]);

		assert_eq!(WordsFromPascal("a"), vec!["a"]);
	}
}
