//=============================================================================//
// File Path: Element/Maintain/Source/Build/CLI/mod.rs
//=============================================================================//
// Module: CLI - Command Line Interface for Configuration-Based Builds
//
// This module provides the cargo-first CLI interface that enables triggering
// builds directly with the Cargo utility instead of shell scripts.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Parse command-line arguments for profile-based builds
// - Load and validate configuration from land-config.json
// - Resolve environment variables from configuration
// - Execute builds with resolved configuration
//
// Secondary:
// - Provide utility commands (--list-profiles, --show-profile)
// - Support dry-run mode for configuration preview
// - Enable profile aliases for quick access
//
// USAGE:
// ======
//
// Basic usage:
// ```bash
// cargo run --bin Maintain -- --profile debug-mountain
// ```
//
// List profiles:
// ```bash
// cargo run --bin Maintain -- --list-profiles
// ```
//
// Dry run:
// ```bash
// cargo run --bin Maintain -- --profile debug --dry-run
// ```
//
//===================================================================================

pub mod Cli;

pub mod Commands;

pub mod GetAllProfiles;

pub mod OutputFormat;

//=============================================================================
// Parser Helpers (shared by struct/enum attribute macros)
//=============================================================================

/// Parse and validate profile name
pub(super) fn parse_profile_name(s:&str) -> Result<String, String> {
	let name = s.trim().to_lowercase();

	if name.is_empty() {
		return Err("Profile name cannot be empty".to_string());
	}

	if name.contains(' ') {
		return Err("Profile name cannot contain spaces".to_string());
	}

	Ok(name)
}

/// Parse a key=value pair from command line.
pub(super) fn parse_key_val<K, V>(s:&str) -> Result<(K, V), String>
where
	K: std::str::FromStr,
	V: std::str::FromStr,
	K::Err: std::fmt::Display,
	V::Err: std::fmt::Display, {
	let pos = s.find('=').ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;

	Ok((
		s[..pos].parse().map_err(|e| format!("key parse error: {e}"))?,
		s[pos + 1..].parse().map_err(|e| format!("value parse error: {e}"))?,
	))
}
