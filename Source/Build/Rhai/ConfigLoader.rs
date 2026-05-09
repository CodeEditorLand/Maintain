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
// - Parse profiles, templates, workbench, features, and build commands
// - Validate configuration structure
// - Provide fast access to configuration data
//
// Secondary:
// - Cache parsed configuration for performance
// - Handle configuration errors gracefully
// - Support workbench and feature flag resolution
//
//=============================================================================//

use std::{collections::HashMap, path::Path};

use serde::Deserialize;

//=============================================================================
// Configuration Types
//=============================================================================

/// The main configuration structure loaded from land-config.json
#[derive(Debug, Deserialize, Clone)]
pub struct LandConfig {

	/// Configuration version
	pub version:String,

	/// Workbench configuration
	pub workbench:Option<WorkbenchConfig>,

	/// Feature flags configuration
	pub features:Option<HashMap<String, FeatureConfig>>,

	/// Binary configuration
	pub binary:Option<BinaryConfig>,

	/// Build profiles (debug, production, release, etc.)
	pub profiles:HashMap<String, Profile>,

	/// Default template values
	pub templates:Option<Templates>,

	/// Environment variable prefixes per crate
	#[serde(rename = "env_prefixes")]
	pub env_prefixes:Option<HashMap<String, String>>,

	/// Build command templates
	#[serde(rename = "build_commands")]
	pub build_commands:Option<HashMap<String, String>>,

	/// Environment variable inventory
	#[serde(rename = "environment_variables")]
	pub environment_variables:Option<EnvironmentVariableInventory>,

	/// CLI configuration
	pub cli:Option<CliConfig>,
}

/// CLI configuration settings
#[derive(Debug, Deserialize, Clone)]
pub struct CliConfig {

	/// Default profile to use
	#[serde(rename = "default_profile")]
	pub default_profile:Option<String>,

	/// Configuration file path
	#[serde(rename = "config_file")]
	pub config_file:Option<String>,

	/// Log format
	#[serde(rename = "log_format")]
	pub log_format:Option<String>,

	/// Enable colors
	pub colors:Option<bool>,

	/// Show progress
	pub progress:Option<bool>,

	/// Dry run default
	#[serde(rename = "dry_run_default")]
	pub dry_run_default:Option<bool>,

	/// Profile aliases
	#[serde(rename = "profile_aliases")]
	pub profile_aliases:HashMap<String, String>,
}

/// Environment variable inventory structure
#[derive(Debug, Deserialize, Clone)]
pub struct EnvironmentVariableInventory {

	/// Build flags
	#[serde(rename = "build_flags")]
	pub build_flags:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Build configuration
	#[serde(rename = "build_config")]
	pub build_config:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Node.js configuration
	pub node:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Rust configuration
	pub rust:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Mountain configuration
	pub mountain:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Tauri configuration
	pub tauri:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Apple signing configuration
	pub apple:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Android configuration
	pub android:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// CI/CD configuration
	pub ci:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// API configuration
	pub api:Option<HashMap<String, EnvironmentVariableInfo>>,

	/// Other configuration
	pub other:Option<HashMap<String, EnvironmentVariableInfo>>,
}

/// Environment variable information
#[derive(Debug, Deserialize, Clone)]
pub struct EnvironmentVariableInfo {

	/// Variable type
	#[serde(rename = "type")]
	pub var_type:Option<String>,

	/// Description
	pub description:Option<String>,

	/// Allowed values
	pub values:Option<Vec<String>>,

	/// Default value
	pub default:Option<String>,

	/// Configuration path
	#[serde(rename = "config_path")]
	pub config_path:Option<String>,

	/// Whether this is sensitive
	pub sensitive:Option<bool>,
}

/// Workbench configuration
#[derive(Debug, Deserialize, Clone)]
pub struct WorkbenchConfig {

	/// Default workbench type
	pub default:Option<String>,

	/// Available workbench types
	pub available:Option<Vec<String>>,

	/// Feature sets per workbench
	pub features:Option<HashMap<String, WorkbenchFeatures>>,
}

/// Features for a specific workbench
#[derive(Debug, Deserialize, Clone)]
pub struct WorkbenchFeatures {

	/// Human-readable description
	pub description:Option<String>,

	/// Feature coverage percentage
	pub coverage:Option<String>,

	/// Complexity level
	pub complexity:Option<String>,

	/// Whether this workbench requires polyfills
	pub polyfills:Option<bool>,

	/// Whether this workbench uses Mountain providers
	#[serde(rename = "mountain_providers")]
	pub mountain_providers:Option<bool>,

	/// Whether this workbench uses Wind services
	#[serde(rename = "wind_services")]
	pub wind_services:Option<bool>,

	/// Whether this workbench uses Electron APIs
	#[serde(rename = "electron_apis")]
	pub electron_apis:Option<bool>,

	/// Whether this workbench is recommended
	pub recommended:Option<bool>,

	/// Recommended use cases
	#[serde(rename = "recommended_for")]
	pub recommended_for:Option<Vec<String>>,
}

/// Feature flag configuration
#[derive(Debug, Deserialize, Clone)]
pub struct FeatureConfig {

	/// Human-readable description
	pub description:Option<String>,

	/// Default value
	pub default:Option<bool>,

	/// Dependencies
	#[serde(rename = "depends_on")]
	pub depends_on:Option<Vec<String>>,
}

/// Binary configuration
#[derive(Debug, Deserialize, Clone)]
pub struct BinaryConfig {

	/// Binary name template
	#[serde(rename = "name_template")]
	pub name_template:Option<String>,

	/// Binary identifier template
	#[serde(rename = "identifier_template")]
	pub identifier_template:Option<String>,

	/// Version format
	#[serde(rename = "version_format")]
	pub version_format:Option<String>,

	/// Signing configuration
	pub sign:Option<SignConfig>,

	/// Notarization configuration
	pub notarize:Option<NotarizeConfig>,

	/// Updater configuration
	pub updater:Option<UpdaterConfig>,
}

/// Signing configuration
#[derive(Debug, Deserialize, Clone)]
pub struct SignConfig {

	/// macOS signing settings
	pub macos:Option<MacOSSignConfig>,

	/// Windows signing settings
	pub windows:Option<WindowsSignConfig>,

	/// Linux signing settings
	pub linux:Option<LinuxSignConfig>,
}

/// macOS signing configuration
#[derive(Debug, Deserialize, Clone)]
pub struct MacOSSignConfig {

	/// Signing identity
	pub identity:Option<String>,

	/// Entitlements file path
	pub entitlements:Option<String>,

	/// Enable hardened runtime
	#[serde(rename = "hardenedRuntime")]
	pub hardened_runtime:Option<bool>,

	/// Gatekeeper assessment
	#[serde(rename = "gatekeeper_assess")]
	pub gatekeeper_assess:Option<bool>,
}

/// Windows signing configuration
#[derive(Debug, Deserialize, Clone)]
pub struct WindowsSignConfig {

	/// Certificate path
	pub certificate:Option<String>,

	/// Timestamp server
	#[serde(rename = "timestamp_server")]
	pub timestamp_server:Option<String>,

	/// TSA URL restrictions
	#[serde(rename = "tsa_can_only_access_urls")]
	pub tsa_can_only_access_urls:Option<Vec<String>>,
}

/// Linux signing configuration
#[derive(Debug, Deserialize, Clone)]
pub struct LinuxSignConfig {

	/// GPG key
	#[serde(rename = "gpg_key")]
	pub gpg_key:Option<String>,

	/// GPG passphrase environment variable
	#[serde(rename = "gpg_passphrase_env")]
	pub gpg_passphrase_env:Option<String>,
}

/// Notarization configuration
#[derive(Debug, Deserialize, Clone)]
pub struct NotarizeConfig {

	/// macOS notarization settings
	pub macos:Option<MacOSNotarizeConfig>,
}

/// macOS notarization configuration
#[derive(Debug, Deserialize, Clone)]
pub struct MacOSNotarizeConfig {

	/// Apple ID
	#[serde(rename = "apple_id")]
	pub apple_id:Option<String>,

	/// Password environment variable
	#[serde(rename = "password_env")]
	pub password_env:Option<String>,

	/// Team ID
	#[serde(rename = "team_id")]
	pub team_id:Option<String>,
}

/// Updater configuration
#[derive(Debug, Deserialize, Clone)]
pub struct UpdaterConfig {

	/// Enable updater
	pub enabled:Option<bool>,

	/// Update endpoints
	pub endpoints:Option<Vec<String>>,

	/// Public key
	pub pubkey:Option<String>,
}

/// A build profile configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Profile {

	/// Human-readable description
	pub description:Option<String>,

	/// Workbench type for this profile
	pub workbench:Option<String>,

	/// Static environment variables for this profile
	pub env:Option<HashMap<String, String>>,

	/// Feature flags for this profile
	pub features:Option<HashMap<String, bool>>,

	/// Path to Rhai script for this profile
	#[serde(rename = "rhai_script")]
	pub rhai_script:Option<String>,
}

/// Default template values used across profiles
#[derive(Debug, Deserialize, Clone)]
pub struct Templates {

	/// Default environment variables
	pub env:HashMap<String, String>,
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
pub fn load(workspace_root:&str) -> Result<LandConfig, String> {

	let config_path = Path::new(workspace_root).join(".vscode").join("land-config.json");

	load_config(&config_path)
}

/// Loads the land-config.json file from a specific path.
///
/// # Arguments
///
/// * `config_path` - Path to the configuration file
///
/// # Returns
///
/// Result containing the parsed LandConfig or an error
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::Rhai::load_config;
/// let config = load_config(".vscode/land-config.json")?;
/// let debug_profile = config.profiles.get("debug");
/// ```
pub fn load_config(config_path:&Path) -> Result<LandConfig, String> {

	if !config_path.exists() {

		return Err(format!("Configuration file not found: {}", config_path.display()));
	}

	let content = std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config file: {}", e))?;

	// Parse JSON5 (using json5 crate for comment support)
	let config:LandConfig = json5::from_str(&content).map_err(|e| format!("Failed to parse config JSON: {}", e))?;

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
pub fn get_profile<'a>(config:&'a LandConfig, profile_name:&str) -> Option<&'a Profile> {

	config.profiles.get(profile_name)
}

/// Gets the workbench type for a profile.
///
/// # Arguments
///
/// * `config` - The loaded configuration
/// * `profile_name` - Name of the profile
///
/// # Returns
///
/// The workbench type for the profile, or the default workbench
pub fn get_workbench_type(config:&LandConfig, profile_name:&str) -> String {

	if let Some(profile) = config.profiles.get(profile_name) {

		if let Some(workbench) = &profile.workbench {

			return workbench.clone();
		}
	}

	// Return default workbench from config
	if let Some(workbench_config) = &config.workbench {

		if let Some(default) = &workbench_config.default {

			return default.clone();
		}
	}

	// Fallback to Browser
	"Browser".to_string()
}

/// Gets the features for a workbench type.
///
/// # Arguments
///
/// * `config` - The loaded configuration
/// * `workbench_type` - The workbench type
///
/// # Returns
///
/// Option containing the workbench features if found
pub fn get_workbench_features<'a>(config:&'a LandConfig, workbench_type:&str) -> Option<&'a WorkbenchFeatures> {

	if let Some(workbench_config) = &config.workbench {

		if let Some(features) = &workbench_config.features {

			return features.get(workbench_type);
		}
	}

	None
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
pub fn resolve_profile_env(config:&LandConfig, profile_name:&str) -> HashMap<String, String> {

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

	// Add workbench environment variable based on profile workbench type
	let workbench_type = get_workbench_type(config, profile_name);

	env_vars.insert(workbench_type.clone(), "true".to_string());

	env_vars
}

/// Resolves all feature flags for a profile.
///
/// This merges default feature values with profile-specific overrides.
///
/// # Arguments
///
/// * `config` - The loaded configuration
/// * `profile_name` - Name of the profile to resolve
///
/// # Returns
///
/// HashMap of all feature flags for the profile
pub fn resolve_profile_features(config:&LandConfig, profile_name:&str) -> HashMap<String, bool> {

	let mut features = HashMap::new();

	// Start with default feature values
	if let Some(feature_config) = &config.features {

		for (name, config) in feature_config {

			features.insert(name.clone(), config.default.unwrap_or(false));
		}
	}

	// Apply profile-specific feature overrides
	if let Some(profile) = config.profiles.get(profile_name) {

		if let Some(profile_features) = &profile.features {

			for (key, value) in profile_features {

				features.insert(key.clone(), *value);
			}
		}
	}

	features
}

/// Generates environment variables from feature flags.
///
/// Converts feature flags to FEATURE_* environment variables.
///
/// # Arguments
///
/// * `features` - HashMap of feature flags
///
/// # Returns
///
/// HashMap of FEATURE_* environment variables
pub fn features_to_env(features:&HashMap<String, bool>) -> HashMap<String, String> {

	let mut env_vars = HashMap::new();

	for (name, value) in features {

		let env_key = format!("FEATURE_{}", name.to_uppercase().replace("-", "_"));

		env_vars.insert(env_key, value.to_string());
	}

	env_vars
}

/// Gets the build command for a profile.
///
/// # Arguments
///
/// * `config` - The loaded configuration
/// * `profile_name` - Name of the profile
///
/// # Returns
///
/// Option containing the build command if found
pub fn get_build_command(config:&LandConfig, profile_name:&str) -> Option<String> {

	config.build_commands.as_ref()?.get(profile_name).cloned()
}

//=============================================================================
// Tests
//=============================================================================

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_load_config() {

		// This test would require a test fixture config file
		// For now, we just verify the types compile correctly
	}

	#[test]
	fn test_features_to_env() {

		let mut features = HashMap::new();

		features.insert("tauri_ipc".to_string(), true);

		features.insert("wind_services".to_string(), false);

		let env = features_to_env(&features);

		assert_eq!(env.get("FEATURE_TAURI_IPC"), Some(&"true".to_string()));

		assert_eq!(env.get("FEATURE_WIND_SERVICES"), Some(&"false".to_string()));
	}
}
