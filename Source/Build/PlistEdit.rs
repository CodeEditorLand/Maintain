//=============================================================================//
// File Path: Element/Maintain/Source/Build/PlistEdit.rs
//=============================================================================//
// Module: PlistEdit
//
// Brief Description: Implements Info.plist file editing for macOS bundle
// configuration.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Inject LSEnvironment dictionary into the Info.plist template used by Tauri
// - Ensure the bundled .app receives dev-control environment variables when
//   launched by LaunchServices (double-click / Finder / open)
//
// Secondary:
// - Provide detailed logging of changes made
// - Return whether any modifications occurred
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/File manipulation layer
// - Apple Property List editing functionality
//
// Dependencies (What this module requires):
// - External crates: std (fs, log), log::info
// - Internal modules: Error::BuildError
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions (Process)
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Functional pattern
//
// Performance Considerations:
// - Complexity: O(n) - parsing and writing based on file size
// - Memory usage patterns: In-memory string manipulation
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: No (not designed for concurrent access to files)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: BuildError (Io type)
// - Recovery strategies: Propagate error up; Guard restores original file
//
// WHY NOT A FULL PLIST PARSER:
// ==============================
//
// The Info.plist template in Element/Mountain is a small, hand-maintained
// document. Using serde_plist or libplist would add a workspace dependency
// and is unnecessary -- we only ever need to insert/replace a single
// <key>LSEnvironment</key><dict>...</dict> block. The implementation below
// uses a targeted string-replacement approach, identical in spirit to how
// JsonEdit works at the value level.
//
// The block is written as well-formed XML with tab indentation to match the
// document style. If the file already contains an LSEnvironment key the
// existing block is replaced in full; if it does not, the block is inserted
// before the closing </dict>.
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//
use std::{collections::BTreeMap, fs, path::Path};

use log::{debug, info};

use crate::Build::Error::Error as BuildError;

/// Injects or replaces the LSEnvironment dictionary in a macOS Info.plist.
///
/// Tauri uses the Info.plist in the project directory as a template during
/// bundling. When the bundled .app is launched by LaunchServices (double-click
/// in Finder, `open`, or Spotlight), any keys under LSEnvironment are injected
/// into the process environment before the executable starts. This is the
/// mechanism that makes the dev-control variables (Trace, Record, etc.)
/// available without requiring a wrapper script.
///
/// # Parameters
///
/// * `File` - Path to the Info.plist template file
/// * `EnvVars` - Map of environment variable names to their values. These
///   become key-value pairs in the LSEnvironment dictionary.
///
/// # Returns
///
/// Returns a `Result<bool>` indicating:
/// - `Ok(true)` - The file was modified and saved
/// - `Ok(false)` - No changes were needed (LSEnvironment already matches)
/// - `Err(BuildError)` - An error occurred during modification
///
/// # Errors
///
/// * `BuildError::Io` - If the file cannot be read or written
///
/// # Behavior
///
/// - If the file already contains an LSEnvironment key, the existing block
///   (from <key>LSEnvironment</key> through the matching </dict>) is replaced
///   with the new dictionary.
/// - If no LSEnvironment key exists, the dictionary is inserted before the
///   root-level </dict>.
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::PlistEdit;
/// let mut env = std::collections::BTreeMap::new();
/// env.insert("Trace".to_string(), "all".to_string());
/// env.insert("Record".to_string(), "1".to_string());
/// let modified = PlistEdit(&path, &env)?;
/// ```
pub fn PlistEdit(File:&Path, EnvVars:&BTreeMap<String, String>) -> Result<bool, BuildError> {
	debug!(target: "Build::Plist", "Attempting to modify plist: {}", File.display());

	let Data = fs::read_to_string(File)?;

	// Build the <dict> content for LSEnvironment.
	let mut DictContent = String::new();

	for (Key, Value) in EnvVars {
		let Escaped = EscapeXml(Value);

		// Tab-indented plist key/value pair to match hand-written style.
		DictContent.push_str(&format!("\t\t<key>{}</key>\n\t\t<string>{}</string>\n", Key, Escaped,));
	}

	let Block = format!("\t<key>LSEnvironment</key>\n\t<dict>\n{}</dict>\n", DictContent);

	// If LSEnvironment already exists, replace the existing block.
	if let Some(Start) = Data.find("<key>LSEnvironment</key>") {
		// Find the end: advance past the key line, then find the matching
		// </dict>. We need the first </dict> after the key that closes the
		// LSEnvironment dict (the root </dict> is the last one in the file).
		let AfterKey = Start + "<key>LSEnvironment</key>".len();

		// The content after <key>LSEnvironment</key> begins with
		// "<dict>...</dict>". Find the </dict> that closes this dict.
		let AfterDictOpen = match Data[AfterKey..].find("<dict>") {
			Some(off) => AfterKey + off + "<dict>".len(),
			None => {
				return Err(BuildError::Io(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"<dict> not found after LSEnvironment key",
				)));
			},
		};

		// Count nesting to find the correct closing </dict>.
		let AfterDictText = &Data[AfterDictOpen..];

		let DictClose = find_closing_dict(AfterDictText)
			.map(|pos| AfterDictOpen + pos + "</dict>".len())
			.ok_or_else(|| {
				BuildError::Io(std::io::Error::new(
					std::io::ErrorKind::InvalidData,
					"Could not find matching </dict> for LSEnvironment",
				))
			})?;

		// Reconstruct with new block, preserving surrounding content.
		let Before = &Data[..Start];

		let After = &Data[DictClose..];

		let Output = format!("{}{}{}", Before, Block, After);

		// Only write if content changed.
		if Output != Data {
			fs::write(File, &Output)?;

			info!(target: "Build::Plist", "Replaced LSEnvironment in {}", File.display());

			return Ok(true);
		}

		return Ok(false);
	}

	// No LSEnvironment key present -- insert before the root-level </dict>.
	// The root </dict> is the last occurrence of </dict> before </plist>.
	let ClosePlistPos = Data.rfind("</plist>").ok_or_else(|| {
		BuildError::Io(std::io::Error::new(
			std::io::ErrorKind::InvalidData,
			"</plist> not found in Info.plist",
		))
	})?;

	let RootDictClose = Data[..ClosePlistPos].rfind("</dict>").ok_or_else(|| {
		BuildError::Io(std::io::Error::new(
			std::io::ErrorKind::InvalidData,
			"Root </dict> not found in Info.plist",
		))
	})?;

	let Before = &Data[..RootDictClose];

	let After = &Data[RootDictClose..];

	let Output = format!("{}{}\n{}", Before, Block, After);

	fs::write(File, &Output)?;

	info!(target: "Build::Plist", "Inserted LSEnvironment into {}", File.display());

	Ok(true)
}

/// Finds the position of the closing </dict> tag accounting for nesting.
///
/// Returns the byte offset (relative to the start of `Text`) at which the
/// closing </dict> begins, or None if no matching close is found.
fn find_closing_dict(Text:&str) -> Option<usize> {
	let mut Depth = 1;

	let mut pos = 0;

	while pos < Text.len() {
		if Text[pos..].starts_with("<dict>") {
			pos += "<dict>".len();

			Depth += 1;
		} else if Text[pos..].starts_with("</dict>") {
			Depth -= 1;

			if Depth == 0 {
				return Some(pos);
			}

			pos += "</dict>".len();
		} else {
			pos += 1;
		}
	}

	None
}

/// Escapes the five XML-special characters in a string value.
fn EscapeXml(Value:&str) -> String {
	let mut Out = String::with_capacity(Value.len());

	for ch in Value.chars() {
		match ch {
			'&' => Out.push_str("&amp;"),
			'<' => Out.push_str("&lt;"),
			'>' => Out.push_str("&gt;"),
			'"' => Out.push_str("&quot;"),
			'\'' => Out.push_str("&apos;"),
			_ => Out.push(ch),
		}
	}

	Out
}
