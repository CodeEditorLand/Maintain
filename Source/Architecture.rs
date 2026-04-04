//! # Architecture Detection
//!
//! Target triple detection and platform support utilities for the
//! Maintain build orchestrator.

/// Represents a supported target platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetArchitecture {
	/// The full Rust target triple (e.g., "aarch64-apple-darwin").
	pub Triple: String,
	/// Human-readable platform name.
	pub Name: String,
	/// Whether this architecture is supported for release builds.
	pub IsSupported: bool,
}

impl TargetArchitecture {
	/// Returns all supported target architectures.
	pub fn SupportedTargets() -> Vec<Self> {
		vec![
			Self {
				Triple: "aarch64-apple-darwin".to_string(),
				Name: "macOS (Apple Silicon)".to_string(),
				IsSupported: true,
			},
			Self {
				Triple: "x86_64-apple-darwin".to_string(),
				Name: "macOS (Intel)".to_string(),
				IsSupported: true,
			},
			Self {
				Triple: "x86_64-unknown-linux-gnu".to_string(),
				Name: "Linux (x86_64)".to_string(),
				IsSupported: true,
			},
			Self {
				Triple: "x86_64-pc-windows-msvc".to_string(),
				Name: "Windows (x86_64)".to_string(),
				IsSupported: true,
			},
		]
	}

	/// Returns the current host architecture.
	pub fn Current() -> Self {
		#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
		return Self {
			Triple: "aarch64-apple-darwin".to_string(),
			Name: "macOS (Apple Silicon)".to_string(),
			IsSupported: true,
		};

		#[cfg(all(target_arch = "x86_64", target_os = "macos"))]
		return Self {
			Triple: "x86_64-apple-darwin".to_string(),
			Name: "macOS (Intel)".to_string(),
			IsSupported: true,
		};

		#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
		return Self {
			Triple: "x86_64-unknown-linux-gnu".to_string(),
			Name: "Linux (x86_64)".to_string(),
			IsSupported: true,
		};

		#[cfg(all(target_arch = "x86_64", target_os = "windows"))]
		return Self {
			Triple: "x86_64-pc-windows-msvc".to_string(),
			Name: "Windows (x86_64)".to_string(),
			IsSupported: true,
		};

		#[cfg(not(any(
			all(target_arch = "aarch64", target_os = "macos"),
			all(target_arch = "x86_64", target_os = "macos"),
			all(target_arch = "x86_64", target_os = "linux"),
			all(target_arch = "x86_64", target_os = "windows"),
		)))]
		return Self {
			Triple: "unknown".to_string(),
			Name: "Unknown".to_string(),
			IsSupported: false,
		};
	}
}
