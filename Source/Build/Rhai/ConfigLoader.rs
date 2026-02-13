//=============================================================================//
// File Path: Element/Maintain/Source/Build/Rhai/ConfigLoader.rs
//=============================================================================//
// Module: ConfigLoader
//
// Brief Description: Loads and parses the land-config.json configuration file.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Load the JSON5 configuration file
// - Parse profiles, templates, and build commands
// - Validate configuration structure
// - Provide fast access to configuration data
//
// Secondary:
// - Cache parsed configuration for performance
// - Handle configuration errors gracefully
//
//=============================================================================//

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

//=============================================================================
// Configuration Types
//=============================================================================

/// The main configuration structure loaded from land-config.json
#[derive(Debug, Deserialize, Clone)]
pub struct LandConfig {
	/// Configuration version
	pub version: String,
	/// Build profiles (debug, production, release, etc.)
	pub profiles: HashMap<String, Profile>,
	/// Default template values
	pub templates: Option<Templates>,
	/// Environment variable prefixes per crate
	#[serde(rename = "env_prefixes")]
	pub env_prefixes: Option<HashMap<String, String>>,
	/// Build command templates
	#[serde(rename = "build_commands")]
	pub build_commands: Option<HashMap<String, String>>,
}

/// A build profile configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Profile {
	/// Human-readable description
	pub description: Option<String>,
	/// Static environment variables for this profile
	pub env: Option<HashMap<String, String>>,
	/// Path to Rhai script for this profile
	#[serde(rename = "rhai_script")]
	pub rhai_script: Option<String>,
}

/// Default template values used across profiles
#[derive(Debug, Deserialize, Clone)]
pub struct Templates {
	/// Default environment variables
	pub env: HashMap<String, String>,
}

//=============================================================================
// Public API
//=============================================================================

/// Loads the land-config.json file from the .vscode directory.
///
/// # Arguments
///
/// * `workspace_root` - Path to the workspace root directory
///
/// # Returns
///
/// Result containing the parsed LandConfig or an error
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::Rhai::ConfigLoader;
/// let config = ConfigLoader::load(".")?;
/// let debug_profile = config.profiles.get("debug");
/// ```
pub fn load(workspace_root: &str) -> Result<LandConfig, String> {
	let config_path = Path::new(workspace_root)
		.join(".vscode")
		.join("land-config.json");

	if !config_path.exists() {
		return Err(format!(
			"Configuration file not found: {}",
			config_path.display()
		));
	}

	let content = std::fs::read_to_string(&config_path)
		.map_err(|e| format!("Failed to read config file: {}", e))?;

	// Parse JSON5 (using json5 crate for comment support)
	let config: LandConfig = json5::from_str(&content)
		.map_err(|e| format!("Failed to parse config JSON: {}", e))?;

	Ok(config)
}

/// Gets a specific profile by name.
///
/// # Arguments
///
/// * `config` - The loaded configuration
/// * `profile_name` - Name of the profile to retrieve
///
/// # Returns
///
/// Option containing the profile if found
pub fn get_profile<'a>(config: &'a LandConfig, profile_name: &str) -> Option<&'a Profile> {
	config.profiles.get(profile_name)
}

/// Resolves all environment variables for a profile.
///
/// This merges template variables with profile-specific variables,
/// with profile variables taking precedence.
///
/// # Arguments
///
/// * `config` - The loaded configuration
/// * `profile_name` - Name of the profile to resolve
///
/// # Returns
///
/// HashMap of all environment variables for the profile
pub fn resolve_profile_env(config: &LandConfig, profile_name: &str) -> HashMap<String, String> {
	let mut env_vars = HashMap::new();

	// Start with template values
	if let Some(templates) = &config.templates {
		for (key, value) in &templates.env {
			env_vars.insert(key.clone(), value.clone());
		}
	}

	// Apply profile-specific values (overriding templates)
	if let Some(profile) = config.profiles.get(profile_name) {
		if let Some(profile_env) = &profile.env {
			for (key, value) in profile_env {
				env_vars.insert(key.clone(), value.clone());
			}
		}
	}

	env_vars
}
