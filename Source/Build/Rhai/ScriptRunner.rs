//=============================================================================//
// Module: ScriptRunner - Executes Rhai scripts for dynamic configuration
//=============================================================================//

use rhai::{Engine, AST, Dynamic, Scope};
use std::path::Path;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ScriptResult {
	pub env_vars: HashMap<String, String>,
	pub success: bool,
	pub error: Option<String>,
	pub pre_build_continue: bool,
	pub post_build_output: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ScriptContext {
	pub profile_name: String,
	pub cwd: String,
	pub manifest_dir: String,
	pub target_triple: Option<String>,
}

pub fn execute_profile_script(
	engine: &Engine,
	script_path: &str,
	context: &ScriptContext,
) -> Result<ScriptResult, String> {
	let ast = load_script(engine, script_path)?;
	let mut scope = Scope::new();

	scope.push("profile_name", context.profile_name.clone());
	scope.push("cwd", context.cwd.clone());
	scope.push("manifest_dir", context.manifest_dir.clone());
	scope.push("target_triple", context.target_triple.clone().unwrap_or_default());

	let env_vars_result = engine.call_fn(&mut scope, &ast, "get_env_vars", ());

	let mut result = ScriptResult {
		env_vars: HashMap::new(),
		success: env_vars_result.is_ok(),
		error: None,
		pre_build_continue: true,
		post_build_output: None,
	};

	if let Ok(env_map) = env_vars_result {
		result.env_vars = extract_env_map(env_map);
	} else if let Err(e) = env_vars_result {
		result.error = Some(e.to_string());
	}

	Ok(result)
}

pub fn load_script(engine: &Engine, script_path: &str) -> Result<AST, String> {
	if !Path::new(script_path).exists() {
		return Err(format!("Script file not found: {}", script_path));
	}

	let content = std::fs::read_to_string(script_path)
		.map_err(|e| format!("Failed to read script: {}", e))?;

	let ast = engine.compile(&content)
		.map_err(|e| format!("Failed to compile script: {}", e))?;

	Ok(ast)
}

pub fn extract_env_map(dynamic: Dynamic) -> HashMap<String, String> {
	let mut env_map = HashMap::new();

	if let Some(map) = dynamic.try_cast::<rhai::Map>() {
		for (key, value) in map {
			if value.is_string() {
				env_map.insert(key.to_string(), value.to_string());
			} else if value.is_int() {
				env_map.insert(key.to_string(), value.as_int().unwrap_or(0).to_string());
			} else if value.is_bool() {
				env_map.insert(key.to_string(), value.as_bool().unwrap_or(false).to_string());
			} else {
				env_map.insert(key.to_string(), value.to_string());
			}
		}
	}

	env_map
}
