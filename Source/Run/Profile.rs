//=============================================================================//
// File Path: Element/Maintain/Source/Run/Profile.rs
//=============================================================================//
// Module: Profile
//
// Brief Description: Profile resolution and management for run operations.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Load and parse run profiles from configuration
// - Resolve profile names and aliases
// - Validate profile configurations
//
// Secondary:
// - Provide profile metadata
// - Support profile inheritance
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Profile management layer
// - Profile resolution
//
// Dependencies (What this module requires):
// - External crates: serde, std
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

use std::{collections::HashMap, path::Path};

use crate::Run::{
	Constant::ProfileDefault,
	Definition::{Profile, RunProfileConfig},
	Error::{Error, Result},
};

/// Loads profiles from a configuration file.
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
///
/// # Returns
///
/// A HashMap of profile names to Profile instances
pub fn load_profiles(config_path:&Path) -> Result<HashMap<String, Profile>> {

	let content =
		std::fs::read_to_string(config_path).map_err(|_e| Error::ConfigNotFound(config_path.to_path_buf()))?;

	let config:serde_json::Value = serde_json::from_str(&content).map_err(|e| Error::ConfigParse(e.to_string()))?;

	let mut profiles = HashMap::new();

	if let Some(profiles_obj) = config.get("profiles").and_then(|v| v.as_object()) {

		for (name, value) in profiles_obj {

			if let Ok(profile) = serde_json::from_value::<Profile>(value.clone()) {

				profiles.insert(name.clone(), profile);
			}
		}
	}

	Ok(profiles)
}

/// Resolves a profile name, handling aliases.
///
/// # Arguments
///
/// * `name` - The profile name or alias to resolve
/// * `aliases` - Map of alias -> target profile
///
/// # Returns
///
/// The resolved profile name
pub fn resolve_name(name:&str, aliases:&HashMap<String, String>) -> String {

	aliases.get(name).cloned().unwrap_or_else(|| name.to_string())
}

/// Gets the default profile name.
///
/// # Returns
///
/// The default profile name
pub fn default_name() -> String { ProfileDefault.to_string() }

/// Validates a profile configuration.
///
/// # Arguments
///
/// * `profile` - The profile to validate
///
/// # Returns
///
/// A list of validation warnings and issues
pub fn validate(profile:&Profile) -> (Vec<String>, Vec<String>) {

	let mut warnings = Vec::new();

	let mut issues = Vec::new();

	// Check description
	if profile.description.is_none() {

		warnings.push("Profile has no description".to_string());
	}

	// Check workbench
	if profile.workbench.is_none() {

		issues.push("Profile has no workbench type specified".to_string());
	}

	// Check environment variables
	if profile.env.is_none() || profile.env.as_ref().unwrap().is_empty() {

		warnings.push("Profile has no environment variables defined".to_string());
	}

	// Check run_config if present
	if let Some(run_config) = &profile.run_config {

		if run_config.live_reload_port == 0 {

			issues.push("Live-reload port cannot be 0".to_string());
		}
	}

	(warnings, issues)
}

/// Merges two profiles, with the second overriding the first.
///
/// # Arguments
///
/// * `base` - The base profile
/// * `override_profile` - The overriding profile
///
/// # Returns
///
/// A merged profile
pub fn merge(base:&Profile, override_profile:&Profile) -> Profile {

	Profile {

		name:base.name.clone(),

		description:override_profile.description.clone().or_else(|| base.description.clone()),

		workbench:override_profile.workbench.clone().or_else(|| base.workbench.clone()),

		env:{

			let mut env = base.env.clone().unwrap_or_default();

			if let Some(override_env) = &override_profile.env {

				for (key, value) in override_env {

					env.insert(key.clone(), value.clone());
				}
			}

			if env.is_empty() { None } else { Some(env) }
		},

		run_config:override_profile.run_config.clone().or_else(|| base.run_config.clone()),
	}
}

/// Creates a profile from environment variables.
///
/// # Arguments
///
/// * `name` - Profile name
/// * `env_vars` - Environment variables to use
///
/// # Returns
///
/// A new Profile instance
pub fn from_env(name:&str, env_vars:HashMap<String, String>) -> Profile {

	Profile {

		name:name.to_string(),

		description:Some("Environment-based profile".to_string()),

		workbench:env_vars.get("Workbench").cloned(),

		env:Some(env_vars),

		run_config:None,
	}
}

/// Gets the run configuration for a profile.
///
/// # Arguments
///
/// * `profile` - The profile to get config from
///
/// # Returns
///
/// The run configuration, or default if not specified
pub fn get_run_config(profile:&Profile) -> RunProfileConfig {

	profile
		.run_config
		.clone()
		.unwrap_or_else(|| RunProfileConfig { hot_reload:true, watch:true, live_reload_port:3001, features:None })
}
