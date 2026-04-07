//=============================================================================//
// File Path: Element/Maintain/Source/Build/GetTauriTargetTriple.rs
//=============================================================================//
// Module: GetTauriTargetTriple
//
// Brief Description: Returns the Tauri-compatible target triple for the current
// build environment.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Determine the current OS and architecture
// - Map to Tauri-compatible target triple strings
// - Support all major platforms (Windows, Linux, macOS)
//
// Secondary:
// - None
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Utility layer
// - Platform detection utilities
//
// Dependencies (What this module requires):
// - External crates: std (env)
// - Internal modules: None
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - Sidecar bundling logic
// - Node.js version handling
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Platform detection pattern
// - String mapping pattern
//
// Performance Considerations:
// - Complexity: O(1) - simple pattern matching
// - Memory usage patterns: Allocates a new String for the target triple
// - Hot path optimizations: None needed
//
// Thread Safety:
// - Thread-safe: Yes (pure function with immutable input from env::consts)
// - Synchronization mechanisms used: None
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: None (panics on unsupported platforms)
// - Recovery strategies: Not applicable (runtime panic indicates build error)
//
// EXAMPLES:
// =========
//
// Example 1: Getting target triple on macOS
/// ```rust
/// use crate::Maintain::Source::Build::GetTauriTargetTriple;
/// #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
/// let triple = GetTauriTargetTriple();
/// assert_eq!(triple, "x86_64-apple-darwin");
/// ```
// Example 2: Getting target triple on Windows
/// ```rust
/// use crate::Maintain::Source::Build::GetTauriTargetTriple;
/// #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
/// let triple = GetTauriTargetTriple();
/// assert_eq!(triple, "x86_64-pc-windows-msvc");
/// ```
// Example 3: Building sidecar path
/// ```rust
/// use crate::Maintain::Source::Build::GetTauriTargetTriple;
/// let triple = GetTauriTargetTriple();
/// let path = format!("./Element/SideCar/{}/NODE/22", triple);
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//
use std::env;

/// Gets the Tauri-compatible target triple for the current build environment.
///
/// This function determines the appropriate target triple string based on the
/// current operating system and CPU architecture. Target triples are used to
/// identify platform binaries and ensure compatibility with Tauri's bundling
/// system.
///
/// # Target Triple Format
///
/// Target triples follow the format: `{arch}-{vendor}-{os}-{environment}`
///
/// Supported combinations:
/// - Windows x86_64: `x86_64-pc-windows-msvc`
/// - Linux x86_64: `x86_64-unknown-linux-gnu`
/// - Linux aarch64: `aarch64-unknown-linux-gnu`
/// - macOS x86_64: `x86_64-apple-darwin`
/// - macOS aarch64 (Apple Silicon): `aarch64-apple-darwin`
///
/// # Returns
///
/// A string containing the target triple for the current platform.
///
/// # Panics
///
/// This function will panic if the current platform is not supported. The
/// panic message indicates the unsupported OS-Arch combination and the
/// build system should be extended to support new platforms as needed.
///
/// # Platform Detection
///
/// The function uses `std::env::consts` to detect:
/// - `OS` - The operating system (windows, linux, macos, etc.)
/// - `ARCH` - The CPU architecture (x86_64, aarch64, etc.)
///
/// These are compile-time constants determined when the Rust compiler builds
/// the binary, so this function always returns consistent results for a
/// given build.
///
/// # Usage Example
///
/// Constructing a path to a Node.js sidecar binary:
/// ```no_run
/// # use std::path::PathBuf;
/// # fn example() {
/// use crate::Maintain::Source::Build::GetTauriTargetTriple;
///
/// let triple = GetTauriTargetTriple();
/// let version = "22";
///
/// # #[cfg(target_os = "windows")]
/// let node_path =
/// 	PathBuf::from(format!("./Element/SideCar/{}/NODE/{}/node.exe", triple, version));
/// # #[cfg(not(target_os = "windows"))]
/// # let node_path = PathBuf::from(format!(
/// # "./Element/SideCar/{}/NODE/{}/bin/node",
/// # triple, version
/// # ));
///
/// println!("Node.js path: {:?}", node_path);
/// # }
/// ```
///
/// # Platform Support
///
/// Currently supported platforms:
///
/// | OS | Architecture | Target Triple |
/// |---|---|---|
/// | Windows | x86_64 | `x86_64-pc-windows-msvc` |
/// | Linux | x86_64 | `x86_64-unknown-linux-gnu` |
/// | Linux | aarch64 | `aarch64-unknown-linux-gnu` |
/// | macOS | x86_64 | `x86_64-apple-darwin` |
/// | macOS | aarch64 | `aarch64-apple-darwin` |
///
/// To add support for a new platform:
/// 1. Add a match arm for the OS-Arch combination
/// 2. Return the appropriate target triple string
/// 3. Ensure Node.js binaries are available at the corresponding paths
pub fn GetTauriTargetTriple() -> String {
	let Os = env::consts::OS;

	let Arch = env::consts::ARCH;

	match (Os, Arch) {
		("windows", "x86_64") => "x86_64-pc-windows-msvc".to_string(),

		("linux", "x86_64") => "x86_64-unknown-linux-gnu".to_string(),

		("linux", "aarch64") => "aarch64-unknown-linux-gnu".to_string(),

		("macos", "x86_64") => "x86_64-apple-darwin".to_string(),

		("macos", "aarch64") => "aarch64-apple-darwin".to_string(),

		_ => panic!("Unsupported OS-Arch for sidecar: {}-{}", Os, Arch),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_known_platforms() {
		// We can't fully test this without a cross-compilation environment,
		// but we can verify the function returns a valid-looking string
		let triple = GetTauriTargetTriple();

		// All valid target triples should contain hyphens and be lowercase
		// alphabetic/numeric
		let valid_chars = |c:char| c.is_alphanumeric() || c == '-' || c == '_';
		assert!(triple.chars().all(valid_chars));

		// Should contain at least two hyphens (arch-vendor-os or arch-vendor-os-env)
		assert!(triple.matches('-').count() >= 2);
	}
}
