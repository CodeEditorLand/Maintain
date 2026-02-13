//=============================================================================//
// Standalone Test Program: RHAI Configuration Testing
//=============================================================================//
//
// This program tests:
// 1. Loading the .vscode/land-config.json file using ConfigLoader
// 2. Validating the profile definitions (debug, production, release, etc.)
// 3. Testing each Rhai script compilation by loading them
// 4. Executing GetEnvVars() function from each script
// 5. Verifying the returned environment variables match expectations
//
//=============================================================================//

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use rhai::{Engine, AST, Scope, Dynamic};

// Configuration types (mirrored from ConfigLoader.rs)
#[derive(Debug, Deserialize, Clone)]
struct LandConfig {
    pub version: String,
    pub profiles: HashMap<String, Profile>,
    pub templates: Option<Templates>,
    #[serde(rename = "env_prefixes")]
    pub env_prefixes: Option<HashMap<String, String>>,
    #[serde(rename = "build_commands")]
    pub build_commands: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Clone)]
struct Profile {
    pub description: Option<String>,
    pub env: Option<HashMap<String, String>>,
    #[serde(rename = "rhai_script")]
    pub rhai_script: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
struct Templates {
    pub env: HashMap<String, String>,
}

// Expected environment variables for each profile
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

fn load_config(workspace_root: &str) -> Result<LandConfig, String> {
    let config_path = Path::new(workspace_root)
        .join(".vscode")
        .join("land-config.json");

    println!("📄 Loading config from: {}", config_path.display());

    if !config_path.exists() {
        return Err(format!(
            "Configuration file not found: {}",
            config_path.display()
        ));
    }

    let content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config file: {}", e))?;

    let config: LandConfig = json5::from_str(&content)
        .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

    Ok(config)
}

fn load_and_compile_script(engine: &Engine, script_path: &str) -> Result<AST, String> {
    println!("  📜 Loading script: {}", script_path);

    if !Path::new(script_path).exists() {
        return Err(format!("Script file not found: {}", script_path));
    }

    let content = std::fs::read_to_string(script_path)
        .map_err(|e| format!("Failed to read script: {}", e))?;

    let ast = engine.compile(&content)
        .map_err(|e| format!("Failed to compile script: {}", e))?;

    println!("  ✅ Script compiled successfully");
    Ok(ast)
}

fn execute_get_env_vars(engine: &Engine, ast: &AST) -> Result<HashMap<String, String>, String> {
    let scope = Scope::new();

    let result = engine.call_fn(scope, ast, "get_env_vars", ());

    match result {
        Ok(dynamic) => {
            let env_map = extract_env_map(dynamic);
            println!("  ✅ get_env_vars() executed successfully");
            println!("  📦 Returned {} environment variable(s)", env_map.len());
            Ok(env_map)
        }
        Err(e) => {
            Err(format!("Failed to execute get_env_vars(): {}", e))
        }
    }
}

fn extract_env_map(dynamic: Dynamic) -> HashMap<String, String> {
    let mut env_map = HashMap::new();

    // Try to convert to a map
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

fn validate_expected_vars(
    actual: &HashMap<String, String>,
    expected: &[(&'static str, &'static str)],
    profile_name: &str,
) -> Vec<String> {
    println!("  🔍 Validating expected environment variables...");
    let mut issues = Vec::new();

    for (key, expected_val) in expected {
        match actual.get(*key) {
            Some(actual_val) if actual_val == *expected_val => {
                println!("    ✅ {} = \"{}\"", key, actual_val);
            }
            Some(actual_val) => {
                let msg = format!("  ⚠️  {} = \"{}\" (expected \"{}\")", key, actual_val, expected_val);
                println!("{}", msg);
                issues.push(msg);
            }
            None => {
                let msg = format!("  ❌ {} is missing (expected \"{}\")", key, expected_val);
                println!("{}", msg);
                issues.push(msg);
            }
        }
    }

    // Check for extra variables
    for key in actual.keys() {
        if !expected.iter().any(|(k, _)| *k == key.as_str()) {
            let val = actual.get(key).unwrap();
            println!("  ➕ {} = \"{}\" (extra variable)", key, val);
        }
    }

    issues
}

fn run_tests() {
    println!("╔═════════════════════════════════════════════════════════════════════╗");
    println!("║     RHAI Configuration Test Suite                                    ║");
    println!("╚═════════════════════════════════════════════════════════════════════╝");
    println!();

    let workspace_root = ".";
    let profiles_to_test = vec!["debug", "production", "release", "bundler-preparation", "swc-bundle", "oxc-bundle"];

    // Step 1: Load configuration
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Step 1: Loading Configuration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let config = match load_config(workspace_root) {
        Ok(cfg) => {
            println!("✅ Configuration loaded successfully");
            println!("   Version: {}", cfg.version);
            println!("   Profiles found: {}", cfg.profiles.len());
            println!();
            cfg
        }
        Err(e) => {
            println!("❌ Failed to load configuration: {}", e);
            println!();
            return;
        }
    };

    // Step 2: Validate profiles in config
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Step 2: Validating Profile Definitions");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let mut missing_scripts = Vec::new();
    for profile_name in &profiles_to_test {
        println!("🔍 Profile: {}", profile_name);

        match config.profiles.get(*profile_name) {
            Some(profile) => {
                println!("  ✅ Profile found");

                if let Some(desc) = &profile.description {
                    println!("  📝 Description: {}", desc);
                }

                if let Some(env) = &profile.env {
                    println!("  🔧 Static env vars: {}", env.len());
                }

                if let Some(script_path) = &profile.rhai_script {
                    let full_path = Path::new(workspace_root)
                        .join(".vscode")
                        .join(script_path);

                    if full_path.exists() {
                        println!("  📜 Script path: {} ✅", script_path);
                    } else {
                        println!("  📜 Script path: {} ❌ (not found)", script_path);
                        missing_scripts.push(profile_name.to_string());
                    }
                } else {
                    println!("  📜 No Rhai script defined");
                    missing_scripts.push(profile_name.to_string());
                }
            }
            None => {
                println!("  ❌ Profile not found in configuration");
                missing_scripts.push(profile_name.to_string());
            }
        }
        println!();
    }

    // Step 3: Test Rhai scripts
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📋 Step 3: Testing Rhai Script Compilation and Execution");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let engine = Engine::new();

    for profile_name in &profiles_to_test {
        println!("🔍 Testing Profile: {}", profile_name);

        if !config.profiles.contains_key(*profile_name) {
            println!("  ⏭️  Skipping - profile not in config");
            println!();
            continue;
        }

        let profile = config.profiles.get(*profile_name).unwrap();

        if let Some(script_path) = &profile.rhai_script {
            let full_path = Path::new(workspace_root)
                .join(".vscode")
                .join(script_path);

            if !full_path.exists() {
                println!("  ⏭️  Skipping - script file not found at: {}", script_path);
                println!();
                continue;
            }

            // Load and compile script
            match load_and_compile_script(&engine, full_path.to_str().unwrap()) {
                Ok(ast) => {
                    // Execute get_env_vars
                    match execute_get_env_vars(&engine, &ast) {
                        Ok(env_vars) => {
                            // Validate expected variables
                            let expected = get_expected_env_vars(profile_name);

                            if !expected.is_empty() {
                                let issues = validate_expected_vars(&env_vars, &expected, profile_name);

                                if issues.is_empty() {
                                    println!("  ✅ All expected variables match");
                                } else {
                                    println!("  ⚠️  {} issue(s) found", issues.len());
                                }
                            } else {
                                println!("  ℹ️  No validation rules defined for this profile");
                            }
                        }
                        Err(e) => {
                            println!("  ❌ {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ {}", e);
                }
            }
        } else {
            println!("  ⏭️  Skipping - no Rhai script defined");
        }

        println!();
    }

    // Test summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Test Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    let profiles_count = config.profiles.len();
    println!("📁 Configuration loaded: ✅");
    println!("   Version: {}", config.version);
    println!("   Total profiles: {}", profiles_count);
    println!();

    let scripts_available: Vec<_> = config.profiles.iter()
        .filter(|(_, p)| {
            if let Some(script) = &p.rhai_script {
                Path::new(workspace_root).join(".vscode").join(script).exists()
            } else {
                false
            }
        })
        .collect();

    println!("📜 Scripts available: {}/{}", scripts_available.len(), profiles_count);
    for (name, _) in &scripts_available {
        println!("   ✅ {}", name);
    }
    if missing_scripts.len() > 0 {
        println!();
        println!("⚠️  Profiles with missing scripts:");
        for name in &missing_scripts {
            println!("   ❌ {}", name);
        }
    }

    println!();
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Test suite completed!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}

fn main() {
    run_tests();
}
