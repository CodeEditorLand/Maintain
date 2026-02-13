//=============================================================================//
// File Path: Element/Maintain/Source/Build/Rhai/EnvironmentResolver.rs
//=============================================================================//
// Module: EnvironmentResolver
//
// Brief Description: Resolves final environment variables from all sources.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Merge environment variables from multiple sources
// - Add environment variable prefixes per crate
// - Apply environment variables to current process
// - Provide validation of required environment variables
//
// Secondary:
// - Log environment variable resolution
// - Support variable expansion (e.g., ${VAR})
//
//=============================================================================//

use std::collections::HashMap;

//=============================================================================
// Public API
//=============================================================================

/// Resolves the final set of environment variables.
///
/// This function merges variables from:
/// 1. Template defaults
/// 2. Profile-specific static variables
/// 3. Rhai script output
/// 4. Current process environment
///
/// # Arguments
///
/// * `template_env` - Environment variables from templates
/// * `profile_env` - Environment variables from profile
/// * `script_env` - Environment variables from Rhai script
/// * `preserve_current` - Whether to preserve current process environment
///
/// # Returns
///
/// Final resolved HashMap of environment variables
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::Rhai::EnvironmentResolver;
///
/// let templates = HashMap::from([("PATH", "/usr/bin".to_string())]);
/// let profile = HashMap::from([("NODE_ENV", "development".to_string())]);
/// let script = HashMap::from([("RUST_LOG", "debug".to_string())]);
///
/// let final_env = EnvironmentResolver::resolve(templates, profile, script, true);
/// ```
pub fn resolve(
	template_env: HashMap<String, String>,
	profile_env: HashMap<String, String>,
	script_env: HashMap<String, String>,
	preserve_current: bool,
) -> HashMap<String, String> {
	let mut resolved = HashMap::new();

	// Start with current environment if requested
	if preserve_current {
		for (key, value) in std::env::vars_os() {
			if let (Ok(key_str), Ok(value_str)) = (key.into_string(), value.into_string()) {
				resolved.insert(key_str, value_str);
			}
		}
	}

	// Apply template values
	for (key, value) in template_env {
		resolved.insert(key, value);
	}

	// Apply profile values (overriding templates)
	for (key, value) in profile_env {
		resolved.insert(key, value);
	}

	// Apply script values (highest priority)
	for (key, value) in script_env {
		resolved.insert(key, value);
	}

	// Apply environment variable prefixes per crate
	apply_prefixes(&mut resolved);

	// Expand variable references
	expand_variables(&mut resolved);

	resolved
}

/// Applies environment variables to the current process.
///
/// This function sets all environment variables in the resolved HashMap
/// to the current process environment, making them available to child processes.
///
/// # Arguments
///
/// * `env_vars` - HashMap of environment variables to set
pub fn apply_to_process(env_vars: &HashMap<String, String>) {
	for (key, value) in env_vars {
		unsafe { std::env::set_var(key, value) };
	}
}

/// Validates that all required environment variables are present.
///
/// # Arguments
///
/// * `env_vars` - Environment variables to validate
/// * `required_vars` - List of required variable names
///
/// # Returns
///
/// Result containing validation result or missing variables
pub fn validate_required(
	env_vars: &HashMap<String, String>,
	required_vars: &[&str],
) -> Result<(), Vec<String>> {
	let mut missing = Vec::new();

	for var in required_vars {
		if !env_vars.contains_key(*var) {
			missing.push(var.to_string());
		}
	}

	if !missing.is_empty() {
		Err(missing)
	} else {
		Ok(())
	}
}

/// Gets build-specific environment variables with their prefixes applied.
///
/// This function returns only the environment variables that are relevant
/// to the build process, with appropriate prefixes for each crate.
///
/// # Arguments
///
/// * `env_vars` - All environment variables
///
/// # Returns
///
/// Build-specific environment variables
pub fn get_build_vars(env_vars: &HashMap<String, String>) -> HashMap<String, String> {
	let mut build_vars = HashMap::new();

	// Collect all build-relevant variables
	for (key, value) in env_vars {
		if is_build_variable(key) {
			build_vars.insert(key.clone(), value.clone());
		}
	}

	build_vars
}

//=============================================================================
// Private Helper Functions
//=============================================================================

/// Applies environment variable prefixes per crate.
fn apply_prefixes(_env_vars: &mut HashMap<String, String>) {
	// This function would apply prefixes based on configuration
	// For now, we keep the variable names as-is since they're already prefixed
	// in the configuration files

	// Example: if a variable is "HOST" and the crate is "cocoon",
	// it would become "MOUNTAIN_HOST" - but in our config files,
	// we already use the prefixed names
}

/// Expands variable references like ${VAR} in values.
fn expand_variables(env_vars: &mut HashMap<String, String>) {
	let mut changed = true;

	while changed {
		changed = false;
		let mut new_values = HashMap::new();

		for (key, value) in &*env_vars {
			let mut new_value = value.clone();

			// Find all ${VAR} patterns
			while let Some(start) = new_value.find("${") {
				if let Some(end) = new_value[start..].find('}') {
					let var_name = &new_value[start + 2..start + end];
					
					if let Some(replacement) = env_vars.get(var_name) {
						new_value.replace_range(start..start + end + 1, replacement);
						changed = true;
					} else {
						break;
					}
				} else {
					break;
				}
			}

			if new_value != *value {
				new_values.insert(key.clone(), new_value);
			}
		}

		for (key, value) in new_values {
			env_vars.insert(key, value);
		}
	}
}

/// Checks if an environment variable is relevant to the build process.
fn is_build_variable(key: &str) -> bool {
	let build_var_prefixes = [
		"Debug",
		"Browser",
		"Bundle",
		"Clean",
		"Compile",
		"Dependency",
		"NODE_ENV",
		"NODE_VERSION",
		"NODE_OPTIONS",
		"RUST_LOG",
		"MOUNTAIN_",
		"AIR_",
		"VSCODE_",
		"LAND_",
	];

	build_var_prefixes.iter().any(|prefix| key.starts_with(prefix))
}
