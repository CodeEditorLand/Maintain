//=============================================================================//
// File Path: Element/Maintain/Source/Run/Environment.rs
//=============================================================================//
// Module: Environment
//
// Brief Description: Environment variable management for run operations.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Resolve environment variables from profiles
// - Merge environment variables from multiple sources
// - Validate environment configuration
//
// Secondary:
// - Provide environment variable templates
// - Support environment inheritance
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Environment management layer
// - Environment resolution
//
// Dependencies (What this module requires):
// - External crates: std
// - Internal modules: Constant, Definition, Error
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Run CLI module
// - Run Process module
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use std::collections::HashMap;

use crate::Run::{Constant::*, Definition::Profile, Error::Result};

/// Resolves environment variables for a run profile.
///
/// This function combines environment variables from multiple sources:
/// 1. Template defaults
/// 2. Shell environment (if enabled)
/// 3. Profile-specific variables
/// 4. CLI overrides
///
/// # Arguments
///
/// * `profile` - The profile to resolve environment for
/// * `merge_shell` - Whether to merge with shell environment
/// * `overrides` - CLI-provided environment overrides
///
/// # Returns
///
/// A HashMap of resolved environment variables
pub fn Resolve(Profile:&Profile, MergeShell:bool, Overrides:&[(String, String)]) -> Result<HashMap<String, String>> {

	let mut env = HashMap::new();

	// Layer 1: Template defaults
	ApplyTemplateDefaults(&mut env);

	// Layer 2: Shell environment (if enabled)
	if MergeShell {

		MergeShellEnv(&mut env);
	}

	// Layer 3: Profile environment
	if let Some(ProfileEnvironment) = &Profile.env {

		for (Key, Value) in ProfileEnvironment {

			env.insert(Key.clone(), Value.clone());
		}
	}

	// Layer 4: CLI overrides (highest priority)
	for (Key, Value) in Overrides {

		env.insert(Key.clone(), Value.clone());
	}

	Ok(env)
}

/// Applies template default environment variables.
///
/// # Arguments
///
/// * `env` - The environment HashMap to populate
fn ApplyTemplateDefaults(env:&mut HashMap<String, String>) {

	// Default Node.js configuration
	env.entry("NODE_VERSION".to_string()).or_insert("22".to_string());

	env.entry("NODE_OPTIONS".to_string())
		.or_insert("--max-old-space-size=16384".to_string());

	// Default run configuration
	env.entry("HOT_RELOAD".to_string()).or_insert("true".to_string());

	env.entry("WATCH".to_string()).or_insert("true".to_string());

	env.entry("LIVE_RELOAD_PORT".to_string())
		.or_insert(DefaultLiveReloadPort.to_string());

	// Default logging
	env.entry("Level".to_string()).or_insert("silent".to_string());

	env.entry("RUST_LOG".to_string()).or_insert("info".to_string());
}

/// Merges shell environment variables into the environment.
///
/// Only merges build system-related environment variables.
///
/// # Arguments
///
/// * `env` - The environment HashMap to merge into
fn MergeShellEnv(env:&mut HashMap<String, String>) {

	let RelevantVars = [
		"Browser",

		"Bundle",

		"Clean",

		"Compile",

		"Debug",

		"Dependency",

		"Mountain",

		"Wind",

		"Electron",

		"BrowserProxy",

		"NODE_ENV",

		"NODE_VERSION",

		"NODE_OPTIONS",

		"RUST_LOG",

		"AIR_LOG_JSON",

		"AIR_LOG_FILE",

		"Level",

		"HOT_RELOAD",

		"WATCH",

		"LIVE_RELOAD_PORT",
	];

	for Var in RelevantVars {

		if let Ok(Value) = std::env::var(Var) {

			env.insert(Var.to_string(), Value);
		}
	}
}

/// Validates environment variables for a run.
///
/// # Arguments
///
/// * `env` - The environment variables to validate
///
/// # Returns
///
/// A list of validation errors (empty if valid)
pub fn Validate(Env:&HashMap<String, String>) -> Vec<String> {

	let mut Errors = Vec::new();

	// Check NODE_VERSION is set
	if !Env.contains_key("NODE_VERSION") {

		Errors.push("NODE_VERSION environment variable is required".to_string());
	}

	// Check NODE_ENV is set
	if !Env.contains_key("NODE_ENV") {

		Errors.push("NODE_ENV environment variable is required".to_string());
	}

	// Check at least one workbench is enabled
	let Workbenches = ["Browser", "Wind", "Mountain", "Electron"];

	let HasWorkbench = Workbenches.iter().any(|W| Env.get(*W).map(|V| V == "true").unwrap_or(false));

	if !HasWorkbench {

		Errors.push("At least one workbench must be enabled (Browser, Wind, Mountain, or Electron)".to_string());
	}

	// Check LIVE_RELOAD_PORT is valid
	if let Some(PortStr) = Env.get("LIVE_RELOAD_PORT") {

		if let Ok(Port) = PortStr.parse::<u16>() {

			if Port == 0 {

				Errors.push("LIVE_RELOAD_PORT cannot be 0".to_string());
			}
		} else {

			Errors.push("LIVE_RELOAD_PORT must be a valid port number".to_string());
		}
	}

	Errors
}

/// Gets the workbench type from environment variables.
///
/// # Arguments
///
/// * `env` - The environment variables
///
/// # Returns
///
/// The name of the enabled workbench, or None if none enabled
pub fn get_workbench(env:&HashMap<String, String>) -> Option<String> {

	let workbenches = [
		("Browser", "Browser"),

		("Wind", "Wind"),

		("Mountain", "Mountain"),

		("Electron", "Electron"),
	];

	for (var, name) in workbenches {

		if env.get(var).map(|v| v == "true").unwrap_or(false) {

			return Some(name.to_string());
		}
	}

	None
}

/// Checks if debug mode is enabled.
///
/// # Arguments
///
/// * `env` - The environment variables
///
/// # Returns
///
/// True if debug mode is enabled
pub fn is_debug(env:&HashMap<String, String>) -> bool { env.get(DebugEnv).map(|v| v == "true").unwrap_or(false) }

/// Checks if hot-reload is enabled.
///
/// # Arguments
///
/// * `env` - The environment variables
///
/// # Returns
///
/// True if hot-reload is enabled
pub fn is_hot_reload_enabled(env:&HashMap<String, String>) -> bool {

	env.get(HotReloadEnv).map(|v| v == "true").unwrap_or(true)
}

/// Checks if watch mode is enabled.
///
/// # Arguments
///
/// * `env` - The environment variables
///
/// # Returns
///
/// True if watch mode is enabled
pub fn is_watch_enabled(env:&HashMap<String, String>) -> bool { env.get(WatchEnv).map(|v| v == "true").unwrap_or(true) }

/// Formats environment variables for display.
///
/// # Arguments
///
/// * `env` - The environment variables to format
///
/// # Returns
///
/// A formatted string representation
pub fn format_for_display(env:&HashMap<String, String>) -> String {

	let mut lines:Vec<String> = env
		.iter()
		.map(|(k, v)| {
			let display_value = if v.is_empty() { "(empty)".to_string() } else { v.clone() };
			format!("  {} = {}", k, display_value)
		})
		.collect();

	lines.sort();

	lines.join("\n")
}

/// Filters environment variables to only include run-related ones.
///
/// # Arguments
///
/// * `env` - The full environment
///
/// # Returns
///
/// A filtered HashMap with only run-related variables
pub fn filter_run_vars(env:&HashMap<String, String>) -> HashMap<String, String> {

	let run_prefixes = [
		"NODE_",

		"HOT_",

		"WATCH",

		"LIVE_",

		"AIR_",

		"RUST_",

		"Browser",

		"Bundle",

		"Clean",

		"Compile",

		"Debug",

		"Dependency",

		"Mountain",

		"Wind",

		"Electron",

		"Level",
	];

	env.iter()
		.filter(|(k, _)| run_prefixes.iter().any(|prefix| k.starts_with(prefix)))
		.map(|(k, v)| (k.clone(), v.clone()))
		.collect()
}
