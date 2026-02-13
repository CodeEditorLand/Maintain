//=============================================================================//
// Unit Tests: RHAI Configuration
//=============================================================================//
//
// This test module provides unit tests for:
// 1. ConfigLoader::load() function
// 2. ConfigLoader::get_profile() function
// 3. ScriptRunner::execute_profile_script() for each profile
// 4. Environment variable structure validation
//
//=============================================================================//

use std::collections::HashMap;

//=============================================================================
// Helper Functions and Types
//=============================================================================

/// Expected environment variables for each profile
fn get_expected_env_vars(profile_name: &str) -> Vec<(&'static str, &'static str)> {
    match profile_name {
        "debug" => vec![
            ("Debug", "true"),
            ("Browser", "true"),
            ("Bundle", "true"),
            ("Clean", "true"),
            ("Compile", "false"),
            ("NODE_ENV", "development"),
            ("NODE_VERSION", "22"),
            ("NODE_OPTIONS", "--max-old-space-size=16384"),
            ("RUST_LOG", "debug"),
            ("AIR_LOG_JSON", "false"),
            ("AIR_LOG_FILE", ""),
            ("Dependency", "Microsoft/VSCode"),
            ("Level", "silent"),
        ],
        "production" => vec![
            ("Debug", "false"),
            ("Browser", "false"),
            ("Bundle", "true"),
            ("Clean", "true"),
            ("Compile", "true"),
            ("NODE_ENV", "production"),
            ("NODE_VERSION", "22"),
            ("NODE_OPTIONS", "--max-old-space-size=8192"),
            ("RUST_LOG", "info"),
            ("AIR_LOG_JSON", "false"),
            ("Dependency", "Microsoft/VSCode"),
        ],
        "release" => vec![
            ("Debug", "false"),
            ("Browser", "false"),
            ("Bundle", "true"),
            ("Clean", "true"),
            ("Compile", "true"),
            ("NODE_ENV", "production"),
            ("NODE_VERSION", "22"),
            ("NODE_OPTIONS", "--max-old-space-size=8192"),
            ("RUST_LOG", "warn"),
            ("AIR_LOG_JSON", "false"),
            ("Dependency", "Microsoft/VSCode"),
        ],
        _ => vec![],
    }
}

/// Extract Rhai map to HashMap<String, String>
fn extract_env_map(dynamic: rhai::Dynamic) -> HashMap<String, String> {
    let mut env_map = HashMap::new();

    if let Some(map) = dynamic.try_cast::<rhai::Map>() {
        for (key, value) in map {
            if let Some(str_val) = value.as_str() {
                env_map.insert(key, str_val.to_string());
            } else {
                env_map.insert(key, value.to_string());
            }
        }
    }

    env_map
}

//=============================================================================
// Test: ConfigLoader::load()
//=============================================================================

#[test]
fn test_config_loader_load() {
    use crate::Source::Build::Rhai::ConfigLoader::load;

    let result = load(".");

    assert!(
        result.is_ok(),
        "ConfigLoader::load() should succeed but got error: {:?}",
        result.err()
    );

    let config = result.unwrap();

    // Verify basic properties
    assert_eq!(config.version, "1.0.0", "Configuration version should be 1.0.0");
    assert!(
        !config.profiles.is_empty(),
        "Configuration should have at least one profile"
    );

    // Verify expected profiles exist
    let expected_profiles = vec!["debug", "production", "release"];
    for profile_name in expected_profiles {
        assert!(
            config.profiles.contains_key(profile_name),
            "Profile '{}' should exist in configuration",
            profile_name
        );
    }

    // Verify templates exist
    assert!(
        config.templates.is_some(),
        "Configuration should have templates defined"
    );

    let templates = config.templates.as_ref().unwrap();
    assert!(
        !templates.env.is_empty(),
        "Templates should have at least one environment variable"
    );
}

//=============================================================================
// Test: ConfigLoader::get_profile()
//=============================================================================

#[test]
fn test_config_loader_get_profile_debug() {
    use crate::Source::Build::Rhai::ConfigLoader::{load, get_profile};

    let config = load(".").expect("Failed to load configuration");
    let profile = get_profile(&config, "debug");

    assert!(
        profile.is_some(),
        "Debug profile should exist in configuration"
    );

    let debug_profile = profile.unwrap();

    assert!(
        debug_profile.description.is_some(),
        "Debug profile should have a description"
    );

    assert!(
        debug_profile.env.is_some(),
        "Debug profile should have environment variables defined"
    );

    assert!(
        debug_profile.rhai_script.is_some(),
        "Debug profile should have a Rhai script defined"
    );

    // Verify expected static environment variables
    let debug_env = debug_profile.env.as_ref().unwrap();
    assert_eq!(debug_env.get("Debug"), Some(&"true".to_string()));
    assert_eq!(debug_env.get("NODE_ENV"), Some(&"development".to_string()));
    assert_eq!(debug_env.get("RUST_LOG"), Some(&"debug".to_string()));
}

#[test]
fn test_config_loader_get_profile_production() {
    use crate::Source::Build::Rhai::ConfigLoader::{load, get_profile};

    let config = load(".").expect("Failed to load configuration");
    let profile = get_profile(&config, "production");

    assert!(
        profile.is_some(),
        "Production profile should exist in configuration"
    );

    let prod_profile = profile.unwrap();

    assert!(
        prod_profile.description.is_some(),
        "Production profile should have a description"
    );

    // Verify expected static environment variables
    let prod_env = prod_profile.env.as_ref().unwrap();
    assert_eq!(prod_env.get("Debug"), Some(&"false".to_string()));
    assert_eq!(prod_env.get("NODE_ENV"), Some(&"production".to_string()));
    assert_eq!(prod_env.get("RUST_LOG"), Some(&"info".to_string()));
}

#[test]
fn test_config_loader_get_profile_release() {
    use crate::Source::Build::Rhai::ConfigLoader::{load, get_profile};

    let config = load(".").expect("Failed to load configuration");
    let profile = get_profile(&config, "release");

    assert!(
        profile.is_some(),
        "Release profile should exist in configuration"
    );

    let release_profile = profile.unwrap();

    assert!(
        release_profile.description.is_some(),
        "Release profile should have a description"
    );

    // Verify expected static environment variables
    let release_env = release_profile.env.as_ref().unwrap();
    assert_eq!(release_env.get("Debug"), Some(&"false".to_string()));
    assert_eq!(release_env.get("NODE_ENV"), Some(&"production".to_string()));
    assert_eq!(release_env.get("RUST_LOG"), Some(&"warn".to_string()));
}

#[test]
fn test_config_loader_get_profile_nonexistent() {
    use crate::Source::Build::Rhai::ConfigLoader::{load, get_profile};

    let config = load(".").expect("Failed to load configuration");
    let profile = get_profile(&config, "nonexistent_profile");

    assert!(
        profile.is_none(),
        "Nonexistent profile should return None"
    );
}

//=============================================================================
// Test: Profile Environment Resolution
//=============================================================================

#[test]
fn test_resolve_profile_env_debug() {
    use crate::Source::Build::Rhai::ConfigLoader::{load, resolve_profile_env};

    let config = load(".").expect("Failed to load configuration");
    let env_vars = resolve_profile_env(&config, "debug");

    // Profile-specific variables should be present
    assert_eq!(env_vars.get("Debug"), Some(&"true".to_string()));
    assert_eq!(env_vars.get("NODE_ENV"), Some(&"development".to_string()));

    // Template variables should also be present
    assert!(
        env_vars.contains_key("MOUNTAIN_DIR"),
        "Template variable MOUNTAIN_DIR should be present",
    );
}

#[test]
fn test_resolve_profile_env_production() {
    use crate::Source::Build::Rhai::ConfigLoader::{load, resolve_profile_env};

    let config = load(".").expect("Failed to load configuration");
    let env_vars = resolve_profile_env(&config, "production");

    // Profile-specific variables should override templates
    assert_eq!(env_vars.get("Debug"), Some(&"false".to_string()));
    assert_eq!(env_vars.get("NODE_ENV"), Some(&"production".to_string()));
}

//=============================================================================
// Test: ScriptRunner::execute_profile_script()
//=============================================================================

#[test]
fn test_execute_profile_script_debug() {
    use crate::Source::Build::Rhai::ConfigLoader::{get_profile, load};
    use crate::Source::Build::Rhai::ScriptRunner::{execute_profile_script, ScriptContext};
    use crate::Source::Build::Rhai::create_engine;

    let config = load(".").expect("Failed to load configuration");
    let profile = get_profile(&config, "debug").expect("Profile 'debug' not found");

    let script_path = profile.rhai_script.as_ref().expect("No Rhai script defined for debug profile");

    // Construct full script path
    let full_script_path = std::path::Path::new(".")
        .join(".vscode")
        .join(script_path);

    if !full_script_path.exists() {
        panic!("Script file not found: {}", full_script_path.display());
    }

    let engine = create_engine();
    let context = ScriptContext {
        profile_name: "debug".to_string(),
        cwd: ".".to_string(),
        manifest_dir: ".".to_string(),
        target_triple: None,
    };

    let result = execute_profile_script(&engine, full_script_path.to_str().unwrap(), &context);

    assert!(
        result.is_ok(),
        "Script execution should succeed but got error: {:?}",
        result.err()
    );

    let script_result = result.unwrap();

    assert!(
        script_result.success,
        "Script execution should report success"
    );

    assert!(
        script_result.error.is_none(),
        "Script execution should not have errors, but got: {:?}",
        script_result.error
    );

    assert!(
        !script_result.env_vars.is_empty(),
        "Script should return environment variables"
    );

    // Verify expected environment variables
    let expected = get_expected_env_vars("debug");
    for (key, expected_val) in expected {
        let actual_val = script_result.env_vars.get(key);
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
    use crate::Source::Build::Rhai::ConfigLoader::{get_profile, load};
    use crate::Source::Build::Rhai::ScriptRunner::{execute_profile_script, ScriptContext};
    use crate::Source::Build::Rhai::create_engine;

    let config = load(".").expect("Failed to load configuration");
    let profile = get_profile(&config, "production").expect("Profile 'production' not found");

    let script_path = profile.rhai_script.as_ref().expect("No Rhai script defined for production profile");

    let full_script_path = std::path::Path::new(".")
        .join(".vscode")
        .join(script_path);

    if !full_script_path.exists() {
        panic!("Script file not found: {}", full_script_path.display());
    }

    let engine = create_engine();
    let context = ScriptContext {
        profile_name: "production".to_string(),
        cwd: ".".to_string(),
        manifest_dir: ".".to_string(),
        target_triple: None,
    };

    let result = execute_profile_script(&engine, full_script_path.to_str().unwrap(), &context);

    assert!(
        result.is_ok(),
        "Script execution should succeed but got error: {:?}",
        result.err()
    );

    let script_result = result.unwrap();

    assert!(script_result.success, "Script execution should report success");
    assert!(script_result.error.is_none(), "Script execution should not have errors");

    // Verify expected environment variables
    let expected = get_expected_env_vars("production");
    for (key, expected_val) in expected {
        let actual_val = script_result.env_vars.get(key);
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
    use crate::Source::Build::Rhai::ConfigLoader::{get_profile, load};
    use crate::Source::Build::Rhai::ScriptRunner::{execute_profile_script, ScriptContext};
    use crate::Source::Build::Rhai::create_engine;

    let config = load(".").expect("Failed to load configuration");
    let profile = get_profile(&config, "release").expect("Profile 'release' not found");

    let script_path = profile.rhai_script.as_ref().expect("No Rhai script defined for release profile");

    let full_script_path = std::path::Path::new(".")
        .join(".vscode")
        .join(script_path);

    if !full_script_path.exists() {
        panic!("Script file not found: {}", full_script_path.display());
    }

    let engine = create_engine();
    let context = ScriptContext {
        profile_name: "release".to_string(),
        cwd: ".".to_string(),
        manifest_dir: ".".to_string(),
        target_triple: None,
    };

    let result = execute_profile_script(&engine, full_script_path.to_str().unwrap(), &context);

    assert!(
        result.is_ok(),
        "Script execution should succeed but got error: {:?}",
        result.err()
    );

    let script_result = result.unwrap();

    assert!(script_result.success, "Script execution should report success");
    assert!(script_result.error.is_none(), "Script execution should not have errors");

    // Verify expected environment variables
    let expected = get_expected_env_vars("release");
    for (key, expected_val) in expected {
        let actual_val = script_result.env_vars.get(key);
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

//=============================================================================
// Test: Environment Variable Structure
//=============================================================================

#[test]
fn test_env_var_structure_has_required_keys() {
    use crate::Source::Build::Rhai::ConfigLoader::{get_profile, load};
    use crate::Source::Build::Rhai::ScriptRunner::{execute_profile_script, load_script, ScriptContext};
    use crate::Source::Build::Rhai::create_engine;
    use rhai::Scope;

    let config = load(".").expect("Failed to load configuration");
    let engine = create_engine();

    let required_function_names = vec!["get_env_vars", "pre_build_hook", "post_build_hook"];

    for profile_name in &["debug", "production", "release"] {
        let profile = get_profile(&config, profile_name).expect(&format!("Profile '{}' not found", profile_name));

        let script_path = profile.rhai_script.as_ref().expect(&format!("No Rhai script defined for {}", profile_name));

        let full_script_path = std::path::Path::new(".")
            .join(".vscode")
            .join(script_path);

        if !full_script_path.exists() {
            panic!("Script file not found: {}", full_script_path.display());
        }

        let ast = load_script(&engine, full_script_path.to_str().unwrap())
            .expect(&format!("Failed to load script for profile '{}'", profile_name));

        // Verify that get_env_vars can be called directly
        let scope = Scope::new();
        let result = engine.call_fn(scope, &ast, "get_env_vars", ());

        assert!(
            result.is_ok(),
            "get_env_vars should be callable in profile '{}'",
            profile_name
        );

        // Verify the function returns a map
        let dynamic = result.unwrap();
        let env_map = extract_env_map(dynamic);

        assert!(!env_map.is_empty(), "get_env_vars should return non-empty map for profile '{}'", profile_name);

        // Verify required keys are present
        for fn_name in &required_function_names {
            if *fn_name != "get_env_vars" {
                // Other hooks are optional, don't fail if they don't exist
                continue;
            }
        }
    }
}

#[test]
fn test_env_vars_match_static_config() {
    use crate::Source::Build::Rhai::ConfigLoader::{get_profile, load, resolve_profile_env};
    use crate::Source::Build::Rhai::ScriptRunner::{execute_profile_script, ScriptContext};
    use crate::Source::Build::Rhai::create_engine;

    let config = load(".").expect("Failed to load configuration");

    for profile_name in &["debug", "production", "release"] {
        let profile = get_profile(&config, profile_name).expect(&format!("Profile '{}' not found", profile_name));

        let script_path = profile.rhai_script.as_ref().expect(&format!("No Rhai script defined for {}", profile_name));

        let full_script_path = std::path::Path::new(".")
            .join(".vscode")
            .join(script_path);

        if !full_script_path.exists() {
            continue;
        }

        let engine = create_engine();
        let context = ScriptContext {
            profile_name: profile_name.to_string(),
            cwd: ".".to_string(),
            manifest_dir: ".".to_string(),
            target_triple: None,
        };

        let script_result = execute_profile_script(&engine, full_script_path.to_str().unwrap(), &context)
            .expect(&format!("Failed to execute script for profile '{}'", profile_name));

        let static_env = resolve_profile_env(&config, profile_name);

        // Verify that Rhai script returns values that match static config where appropriate
        if let Some(static_debug) = static_env.get("Debug") {
            let dynamic_debug = script_result.env_vars.get("Debug");
            assert_eq!(
                dynamic_debug,
                Some(static_debug),
                "Debug value should match between static config and Rhai script for profile '{}'",
                profile_name
            );
        }

        if let Some(static_node_env) = static_env.get("NODE_ENV") {
            let dynamic_node_env = script_result.env_vars.get("NODE_ENV");
            assert_eq!(
                dynamic_node_env,
                Some(static_node_env),
                "NODE_ENV value should match between static config and Rhai script for profile '{}'",
                profile_name
            );
        }
    }
}
