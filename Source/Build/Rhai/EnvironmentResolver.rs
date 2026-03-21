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
// - Support workbench and feature flag resolution
//
// Secondary:
// - Log environment variable resolution
// - Support variable expansion (e.g., ${VAR})
// - Generate feature flag environment variables
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
/// 3. Workbench environment variables
/// 4. Feature flag variables
/// 5. Rhai script output
/// 6. Current process environment
///
/// # Arguments
///
/// * `template_env` - Environment variables from templates
/// * `profile_env` - Environment variables from profile
/// * `workbench_env` - Environment variables from workbench selection
/// * `feature_env` - Environment variables from feature flags
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
///
/// let profile = HashMap::from([("NODE_ENV", "development".to_string())]);
///
/// let workbench = HashMap::from([("Mountain", "true".to_string())]);
///
/// let features = HashMap::from([("FEATURE_TAURI_IPC", "true".to_string())]);
///
/// let script = HashMap::from([("RUST_LOG", "debug".to_string())]);
///
/// let final_env =
/// 	EnvironmentResolver::resolve_full(templates, profile, workbench, features, script, true);
/// ```
pub fn ResolveFull(
    template_env:HashMap<String, String>,

    profile_env:HashMap<String, String>,

    workbench_env:HashMap<String, String>,

    feature_env:HashMap<String, String>,

    script_env:HashMap<String, String>,

    preserve_current:bool,
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

    // Apply template values (lowest priority)
    for (key, value) in template_env {
        resolved.insert(key, value);
    }

    // Apply profile values (overriding templates)
    for (key, value) in profile_env {
        resolved.insert(key, value);
    }

    // Apply workbench values (overriding profile)
    for (key, value) in workbench_env {
        resolved.insert(key, value);
    }

    // Apply feature flag values
    for (key, value) in feature_env {
        resolved.insert(key, value);
    }

    // Apply script values (highest priority)
    for (key, value) in script_env {
        resolved.insert(key, value);
    }

    // Apply environment variable prefixes per crate
    ApplyPrefixes(&mut resolved);

    // Expand variable references
    ExpandVariables(&mut resolved);

    resolved
}

/// Resolves the final set of environment variables (simplified version).
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
pub fn Resolve(
    template_env:HashMap<String, String>,

    profile_env:HashMap<String, String>,

    script_env:HashMap<String, String>,

    preserve_current:bool,
) -> HashMap<String, String> {
    ResolveFull(
        template_env,
        profile_env,
        HashMap::new(),
        HashMap::new(),
        script_env,
        preserve_current,
    )
}

/// Applies environment variables to the current process.
///
/// # Arguments
///
/// * `env_vars` - Environment variables to apply
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::Rhai::EnvironmentResolver;
///
/// let env = HashMap::from([
/// 	("NODE_ENV".to_string(), "production".to_string()),
/// 	("RUST_LOG".to_string(), "info".to_string()),
/// ]);
///
/// EnvironmentResolver::apply(&env);
/// ```
pub fn Apply(env_vars:&HashMap<String, String>) {
    for (key, value) in env_vars {
        // Safety: set_var is now unsafe in recent Rust versions
        // Setting environment variables during build orchestration is acceptable
        // as it doesn't violate memory safety.
        unsafe {
            std::env::set_var(key, value);
        }
    }
}

/// Converts environment variables to a formatted string for logging.
///
/// # Arguments
///
/// * `env_vars` - Environment variables to format
///
/// # Returns
///
/// Formatted string representation
pub fn format_env(env_vars:&HashMap<String, String>) -> String {
	let mut entries:Vec<_> = env_vars.iter().collect();

	entries.sort_by_key(|(k, _)| *k);

	entries
		.iter()
		.map(|(k, v)| format!("  {}={}", k, v))
		.collect::<Vec<_>>()
		.join("\n")
}

/// Validates that required environment variables are set.
///
/// # Arguments
///
/// * `env_vars` - Current environment variables
/// * `required` - List of required variable names
///
/// # Returns
///
/// Result indicating success or list of missing variables
pub fn validate_required(env_vars:&HashMap<String, String>, required:&[&str]) -> Result<(), Vec<String>> {
	let missing:Vec<String> = required
		.iter()
		.filter(|var| !env_vars.contains_key(&var.to_string()))
		.map(|s| s.to_string())
		.collect();

	if missing.is_empty() { Ok(()) } else { Err(missing) }
}

/// Generates workbench-specific environment variables.
///
/// # Arguments
///
/// * `workbench_type` - The selected workbench type
///
/// # Returns
///
/// HashMap of workbench environment variables
pub fn generate_workbench_env(workbench_type:&str) -> HashMap<String, String> {
	let mut env = HashMap::new();

	// Set the workbench type as an environment variable
	env.insert(workbench_type.to_string(), "true".to_string());

	// Set WORKBENCH_TYPE for use in build scripts
	env.insert("WORKBENCH_TYPE".to_string(), workbench_type.to_string());

	env
}

/// Generates feature flag environment variables from a feature map.
///
/// # Arguments
///
/// * `features` - HashMap of feature name to enabled status
///
/// # Returns
///
/// HashMap of FEATURE_* environment variables
pub fn generate_feature_env(features:&HashMap<String, bool>) -> HashMap<String, String> {
	features
		.iter()
		.map(|(name, value)| {
			let env_key = format!("FEATURE_{}", name.to_uppercase().replace('-', "_"));

			(env_key, value.to_string())
		})
		.collect()
}

//=============================================================================
// Private Helper Functions
//=============================================================================

/// Applies environment variable prefixes per crate.
fn ApplyPrefixes(_env_vars:&mut HashMap<String, String>) {
    // Define known crate prefixes
    let Prefixes = [
        ("air", "AIR_"),
        ("cocoon", "MOUNTAIN_"),
        ("grove", "VSCODE_"),
        ("maintain", "LAND_"),
    ];

    // For now, this is a no-op as prefixes are handled in the config
    // Future: could auto-prefix variables based on their names
    let _ = Prefixes;
}

/// Expands variable references in environment variable values.
///
/// Supports ${VAR} syntax for variable expansion.
fn ExpandVariables(env_vars:&mut HashMap<String, String>) {
    // Collect all current values for reference
    let Original:HashMap<String, String> = env_vars.clone();

    // Expand ${VAR} references in each value
    for Value in env_vars.values_mut() {
        // Simple expansion - replace ${VAR} with the value from Original
        let mut Expanded = Value.clone();

        let mut Start = 0;

        while let Some(Open) = Expanded[Start..].find("${") {
            let AbsOpen = Start + Open;

            if let Some(Close) = Expanded[AbsOpen..].find('}') {
                let VarName = &Expanded[AbsOpen + 2..AbsOpen + Close];

                if let Some(Replacement) = Original.get(VarName) {
                    Expanded.replace_range(AbsOpen..AbsOpen + Close + 1, Replacement);

                    // Continue from after the replacement
                    Start = AbsOpen + Replacement.len();
                } else {
                    // Variable not found, skip past this reference
                    Start = AbsOpen + Close + 1;
                }
            } else {
                break;
            }
        }

        *Value = Expanded;
    }
}

//=============================================================================
// Tests
//=============================================================================

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_resolve() {
		let template = HashMap::from([("A", "1".to_string()), ("B", "2".to_string())]);

		let profile = HashMap::from([("B", "3".to_string()), ("C", "4".to_string())]);

		let script = HashMap::from([("C", "5".to_string())]);

		let Result = Resolve(template, profile, script, false);

		assert_eq!(result.get("A"), Some(&"1".to_string())); // From template
		assert_eq!(result.get("B"), Some(&"3".to_string())); // Profile overrides template
		assert_eq!(result.get("C"), Some(&"5".to_string())); // Script overrides profile
	}

	#[test]
	fn test_generate_workbench_env() {
		let env = generate_workbench_env("Mountain");

		assert_eq!(env.get("Mountain"), Some(&"true".to_string()));

		assert_eq!(env.get("WORKBENCH_TYPE"), Some(&"Mountain".to_string()));
	}

	#[test]
	fn test_generate_feature_env() {
		let mut features = HashMap::new();

		features.insert("tauri-ipc".to_string(), true);

		features.insert("wind-services".to_string(), false);

		let env = generate_feature_env(&features);

		assert_eq!(env.get("FEATURE_TAURI_IPC"), Some(&"true".to_string()));

		assert_eq!(env.get("FEATURE_WIND_SERVICES"), Some(&"false".to_string()));
	}

	#[test]
	fn test_validate_required() {
		let env = HashMap::from([("A".to_string(), "1".to_string()), ("B".to_string(), "2".to_string())]);

		assert!(validate_required(&env, &["A", "B"]).is_ok());

		assert!(validate_required(&env, &["A", "C"]).is_err());
	}

	#[test]
	fn test_expand_variables() {
		let mut env = HashMap::new();

		env.insert("BASE".to_string(), "/path/to/base".to_string());

		env.insert("FULL".to_string(), "${BASE}/sub".to_string());

		ExpandVariables(&mut env);

		assert_eq!(env.get("FULL"), Some(&"/path/to/base/sub".to_string()));
	}
}
