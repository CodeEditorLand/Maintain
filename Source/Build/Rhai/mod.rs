//! # Rhai - Dynamic Script Configuration
//!
//! Integrates the Rhai scripting language for dynamic environment variable
//! configuration, allowing build processes to be customized without
//! recompiling the Rust maintain crate.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`crate::Build::Rhai::ConfigLoader`] | Loads and parses `land-config.json` configuration |
//! | [`crate::Build::Rhai::ScriptRunner`] | Executes Rhai scripts for dynamic configuration |
//! | [`crate::Build::Rhai::EnvironmentResolver`] | Resolves final environment variables |

pub mod ConfigLoader;

pub mod ScriptRunner;

pub mod EnvironmentResolver;

use rhai::Engine;

//=============================================================================
// Public API
//=============================================================================

/// Creates and configures a new Rhai engine with all necessary modules and
/// functions.
pub fn create_engine() -> Engine {
	let mut engine = Engine::new();

	// Optimize engine for script execution
	engine.set_max_expr_depths(0, 0);

	engine.set_max_operations(0);

	engine.set_allow_shadowing(true);

	// Register utility functions for scripts
	register_utility_functions(&mut engine);

	engine
}

/// Registers utility functions that can be called from Rhai scripts.
fn register_utility_functions(engine:&mut Engine) {
	// System information
	engine.register_fn("get_os_type", || std::env::consts::OS.to_string());

	engine.register_fn("get_arch", || std::env::consts::ARCH.to_string());

	engine.register_fn("get_family", || std::env::consts::FAMILY.to_string());

	// Environment access (read-only for safety)
	engine.register_fn("get_env", |name:&str| -> String { std::env::var(name).unwrap_or_default() });

	// File system utilities
	engine.register_fn("path_exists", |path:&str| -> bool { std::path::Path::new(path).exists() });

	// Time utilities
	engine.register_fn("timestamp", || -> i64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs() as i64
	});

	// Logging functions
	engine.register_fn("print", |s:&str| {
		println!("[Rhai] {}", s);
	});
}

//=============================================================================
// Tests
//=============================================================================

#[cfg(test)]
mod tests {

	use std::collections::HashMap;

	use super::*;

	/// Expected environment variables for each profile
	fn get_expected_env_vars(profile_name:&str) -> Vec<(&'static str, &'static str)> {
		match profile_name {
			"debug" => {
				vec![
					("Debug", "true"),
					("Browser", "true"),
					("Bundle", "true"),
					("Clean", "true"),
					("Compile", "false"),
					("NODE_ENV", "development"),
					("NODE_VERSION", "24"),
					("NODE_OPTIONS", "--max-old-space-size=16384"),
					("RUST_LOG", "debug"),
					("AIR_LOG_JSON", "false"),
					("AIR_LOG_FILE", ""),
					("Dependency", "Microsoft/VSCode"),
				]
			},

			"production" => {
				vec![
					("Debug", "false"),
					("Browser", "false"),
					("Bundle", "true"),
					("Clean", "true"),
					("Compile", "true"),
					("NODE_ENV", "production"),
					("NODE_VERSION", "24"),
					("NODE_OPTIONS", "--max-old-space-size=8192"),
					("RUST_LOG", "info"),
					("AIR_LOG_JSON", "false"),
					("Dependency", "Microsoft/VSCode"),
				]
			},

			"release" => {
				vec![
					("Debug", "false"),
					("Browser", "false"),
					("Bundle", "true"),
					("Clean", "true"),
					("Compile", "true"),
					("NODE_ENV", "production"),
					("NODE_VERSION", "24"),
					("NODE_OPTIONS", "--max-old-space-size=8192"),
					("RUST_LOG", "warn"),
					("AIR_LOG_JSON", "false"),
					("Dependency", "Microsoft/VSCode"),
				]
			},

			_ => vec![],
		}
	}

	#[test]
	fn test_config_loader_load() {
		let result = ConfigLoader::load(".");

		assert!(
			result.is_ok(),
			"ConfigLoader::load() should succeed but got error: {:?}",
			result.err()
		);

		let config = result.unwrap();

		assert_eq!(config.version, "1.0.0", "Configuration version should be 1.0.0");

		assert!(!config.profiles.is_empty(), "Configuration should have at least one profile");

		assert!(config.profiles.contains_key("debug"), "Debug profile should exist");

		assert!(config.profiles.contains_key("production"), "Production profile should exist");

		assert!(config.profiles.contains_key("release"), "Release profile should exist");

		assert!(config.templates.is_some(), "Configuration should have templates defined");
	}

	#[test]
	fn test_config_loader_get_profile_debug() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let profile = ConfigLoader::get_profile(&config, "debug");

		assert!(profile.is_some(), "Debug profile should exist in configuration");

		let debug_profile = profile.unwrap();

		assert!(debug_profile.description.is_some(), "Debug profile should have a description");

		assert!(
			debug_profile.env.is_some(),
			"Debug profile should have environment variables defined"
		);

		assert!(
			debug_profile.rhai_script.is_some(),
			"Debug profile should have a Rhai script defined"
		);

		let debug_env = debug_profile.env.as_ref().unwrap();

		assert_eq!(debug_env.get("Debug"), Some(&"true".to_string()));

		assert_eq!(debug_env.get("NODE_ENV"), Some(&"development".to_string()));

		assert_eq!(debug_env.get("RUST_LOG"), Some(&"debug".to_string()));
	}

	#[test]
	fn test_resolve_profile_env_debug() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let env_vars = ConfigLoader::resolve_profile_env(&config, "debug");

		assert_eq!(env_vars.get("Debug"), Some(&"true".to_string()));

		assert_eq!(env_vars.get("NODE_ENV"), Some(&"development".to_string()));

		assert!(env_vars.contains_key("MOUNTAIN_DIR"), "Template variable should be present");
	}

	#[test]
	fn test_execute_profile_script_debug() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let profile = ConfigLoader::get_profile(&config, "debug").expect("Profile 'debug' not found");

		let script_path = profile.rhai_script.as_ref().expect("No Rhai script defined for debug profile");

		let full_script_path = std::path::Path::new(".").join(".vscode").join(script_path);

		if !full_script_path.exists() {
			panic!("Script file not found: {}", full_script_path.display());
		}

		let engine = create_engine();

		let context = ScriptRunner::ScriptContext {
			profile_name:"debug".to_string(),

			cwd:".".to_string(),

			manifest_dir:".".to_string(),

			target_triple:None,

			workbench_type:None,

			features:HashMap::new(),
		};

		let result = ScriptRunner::ExecuteProfileScript(&engine, full_script_path.to_str().unwrap(), &context);

		assert!(
			result.is_ok(),
			"Script execution should succeed but got error: {:?}",
			result.err()
		);

		let script_result = result.unwrap();

		assert!(script_result.success, "Script execution should report success");

		assert!(script_result.error.is_none(), "Script execution should not have errors");

		assert!(!script_result.env_vars.is_empty(), "Script should return environment variables");

		let expected = get_expected_env_vars("debug");

		for (key, expected_val) in expected {
			let actual_val = script_result.env_vars.get(key).map(|s| s.as_str());

			assert_eq!(
				actual_val,
				Some(expected_val),
				"Env var '{}' should be '{}', got {:?}",
				key,
				expected_val,
				actual_val
			);
		}
	}

	#[test]
	fn test_execute_profile_script_production() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let profile = ConfigLoader::get_profile(&config, "production").expect("Profile 'production' not found");

		let script_path = profile
			.rhai_script
			.as_ref()
			.expect("No Rhai script defined for production profile");

		let full_script_path = std::path::Path::new(".").join(".vscode").join(script_path);

		if !full_script_path.exists() {
			panic!("Script file not found: {}", full_script_path.display());
		}

		let engine = create_engine();

		let context = ScriptRunner::ScriptContext {
			profile_name:"production".to_string(),

			cwd:".".to_string(),

			manifest_dir:".".to_string(),

			target_triple:None,

			workbench_type:None,

			features:HashMap::new(),
		};

		let result = ScriptRunner::ExecuteProfileScript(&engine, full_script_path.to_str().unwrap(), &context);

		assert!(
			result.is_ok(),
			"Script execution should succeed but got error: {:?}",
			result.err()
		);

		let script_result = result.unwrap();

		assert!(script_result.success, "Script execution should report success");

		assert!(script_result.error.is_none(), "Script execution should not have errors");

		let expected = get_expected_env_vars("production");

		for (key, expected_val) in expected {
			let actual_val = script_result.env_vars.get(key).map(|s| s.as_str());

			assert_eq!(
				actual_val,
				Some(expected_val),
				"Env var '{}' should be '{}', got {:?}",
				key,
				expected_val,
				actual_val
			);
		}
	}

	#[test]
	fn test_execute_profile_script_release() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let profile = ConfigLoader::get_profile(&config, "release").expect("Profile 'release' not found");

		let script_path = profile
			.rhai_script
			.as_ref()
			.expect("No Rhai script defined for release profile");

		let full_script_path = std::path::Path::new(".").join(".vscode").join(script_path);

		if !full_script_path.exists() {
			panic!("Script file not found: {}", full_script_path.display());
		}

		let engine = create_engine();

		let context = ScriptRunner::ScriptContext {
			profile_name:"release".to_string(),

			cwd:".".to_string(),

			manifest_dir:".".to_string(),

			target_triple:None,

			workbench_type:None,

			features:HashMap::new(),
		};

		let result = ScriptRunner::ExecuteProfileScript(&engine, full_script_path.to_str().unwrap(), &context);

		assert!(
			result.is_ok(),
			"Script execution should succeed but got error: {:?}",
			result.err()
		);

		let script_result = result.unwrap();

		assert!(script_result.success, "Script execution should report success");

		assert!(script_result.error.is_none(), "Script execution should not have errors");

		let expected = get_expected_env_vars("release");

		for (key, expected_val) in expected {
			let actual_val = script_result.env_vars.get(key).map(|s| s.as_str());

			assert_eq!(
				actual_val,
				Some(expected_val),
				"Env var '{}' should be '{}', got {:?}",
				key,
				expected_val,
				actual_val
			);
		}
	}

	#[test]
	fn test_execute_profile_script_bundler_preparation() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let profile = ConfigLoader::get_profile(&config, "bundler-preparation")
			.expect("Profile 'bundler-preparation' not found in configuration");

		let script_path = profile
			.rhai_script
			.as_ref()
			.expect("No Rhai script defined for bundler-preparation profile");

		let full_script_path = std::path::Path::new(".").join(".vscode").join(script_path);

		if !full_script_path.exists() {
			eprintln!("Skipping test - script file not found: {}", full_script_path.display());

			return;
		}

		let engine = create_engine();

		let context = ScriptRunner::ScriptContext {
			profile_name:"bundler-preparation".to_string(),

			cwd:".".to_string(),

			manifest_dir:".".to_string(),

			target_triple:None,

			workbench_type:None,

			features:HashMap::new(),
		};

		let result = ScriptRunner::ExecuteProfileScript(&engine, full_script_path.to_str().unwrap(), &context);

		assert!(
			result.is_ok(),
			"Script execution should succeed but got error: {:?}",
			result.err()
		);

		let script_result = result.unwrap();

		assert!(script_result.success, "Script execution should report success");

		assert!(!script_result.env_vars.is_empty(), "Script should return environment variables");

		// Check for bundler-specific variables
		assert_eq!(script_result.env_vars.get("BUNDLER_TYPE"), Some(&"swc".to_string()));

		assert_eq!(script_result.env_vars.get("SWC_TARGET"), Some(&"esnext".to_string()));
	}

	#[test]
	fn test_execute_profile_script_swc_bundle() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let profile =
			ConfigLoader::get_profile(&config, "swc-bundle").expect("Profile 'swc-bundle' not found in configuration");

		let script_path = profile
			.rhai_script
			.as_ref()
			.expect("No Rhai script defined for swc-bundle profile");

		let full_script_path = std::path::Path::new(".").join(".vscode").join(script_path);

		if !full_script_path.exists() {
			eprintln!("Skipping test - script file not found: {}", full_script_path.display());

			return;
		}

		let engine = create_engine();

		let context = ScriptRunner::ScriptContext {
			profile_name:"swc-bundle".to_string(),

			cwd:".".to_string(),

			manifest_dir:".".to_string(),

			target_triple:None,

			workbench_type:None,

			features:HashMap::new(),
		};

		let result = ScriptRunner::ExecuteProfileScript(&engine, full_script_path.to_str().unwrap(), &context);

		assert!(
			result.is_ok(),
			"Script execution should succeed but got error: {:?}",
			result.err()
		);

		let script_result = result.unwrap();

		assert!(script_result.success, "Script execution should report success");

		assert!(!script_result.env_vars.is_empty(), "Script should return environment variables");

		assert_eq!(script_result.env_vars.get("BUNDLER_TYPE"), Some(&"swc".to_string()));

		assert_eq!(script_result.env_vars.get("NODE_ENV"), Some(&"production".to_string()));
	}

	#[test]
	fn test_execute_profile_script_oxc_bundle() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		let profile =
			ConfigLoader::get_profile(&config, "oxc-bundle").expect("Profile 'oxc-bundle' not found in configuration");

		let script_path = profile
			.rhai_script
			.as_ref()
			.expect("No Rhai script defined for oxc-bundle profile");

		let full_script_path = std::path::Path::new(".").join(".vscode").join(script_path);

		if !full_script_path.exists() {
			eprintln!("Skipping test - script file not found: {}", full_script_path.display());

			return;
		}

		let engine = create_engine();

		let context = ScriptRunner::ScriptContext {
			profile_name:"oxc-bundle".to_string(),

			cwd:".".to_string(),

			manifest_dir:".".to_string(),

			target_triple:None,

			workbench_type:None,

			features:HashMap::new(),
		};

		let result = ScriptRunner::ExecuteProfileScript(&engine, full_script_path.to_str().unwrap(), &context);

		assert!(
			result.is_ok(),
			"Script execution should succeed but got error: {:?}",
			result.err()
		);

		let script_result = result.unwrap();

		assert!(script_result.success, "Script execution should report success");

		assert!(!script_result.env_vars.is_empty(), "Script should return environment variables");

		assert_eq!(script_result.env_vars.get("BUNDLER_TYPE"), Some(&"oxc".to_string()));

		assert_eq!(script_result.env_vars.get("NODE_ENV"), Some(&"production".to_string()));
	}

	#[test]
	fn test_env_vars_match_static_config() {
		let config = ConfigLoader::load(".").expect("Failed to load configuration");

		for profile_name in &["debug", "production", "release"] {
			let profile = ConfigLoader::get_profile(&config, profile_name)
				.expect(&format!("Profile '{}' not found", profile_name));

			let script_path = profile
				.rhai_script
				.as_ref()
				.expect(&format!("No Rhai script defined for {}", profile_name));

			let full_script_path = std::path::Path::new(".").join(".vscode").join(script_path);

			if !full_script_path.exists() {
				continue;
			}

			let engine = create_engine();

			let context = ScriptRunner::ScriptContext {
				profile_name:profile_name.to_string(),

				cwd:".".to_string(),

				manifest_dir:".".to_string(),

				target_triple:None,

				workbench_type:None,

				features:HashMap::new(),
			};

			let script_result =
				ScriptRunner::ExecuteProfileScript(&engine, full_script_path.to_str().unwrap(), &context)
					.expect(&format!("Failed to execute script for profile '{}'", profile_name));

			let static_env = ConfigLoader::resolve_profile_env(&config, profile_name);

			// Verify that Rhai script returns values that match static config where
			// appropriate
			if let Some(static_debug) = static_env.get("Debug") {
				let dynamic_debug = script_result.env_vars.get("Debug");

				assert_eq!(
					dynamic_debug,
					Some(static_debug),
					"Debug value should match between static config and Rhai script for profile '{}'",
					profile_name
				);
			}
		}
	}
}
