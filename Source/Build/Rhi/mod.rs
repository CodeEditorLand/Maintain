//=============================================================================//
// File Path: Element/Maintain/Source/Build/Rhai/mod.rs
//=============================================================================//
// Module: Rhai
//
// Brief Description: Rhai scripting engine integration for dynamic configuration.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Initialize and configure the Rhai scripting engine
// - Load and execute Rhai scripts for environment configuration
// - Bridge JSON5 configuration with Rhai script execution
// - Provide safe script execution environment
//
// Secondary:
// - Cache compiled scripts for performance
// - Handle script errors gracefully
// - Provide utility functions for scripts to call
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Scripting layer
// - Dynamic configuration engine
//
// Dependencies (What this module requires):
// - External crates: rhai
// - Internal modules: Error, ConfigLoader, ScriptRunner
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Build orchestration functions
// - Environment resolution logic
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Engine pattern (Rhai engine configuration)
// - Registry pattern (function registration)
// - Cache pattern (compiled script caching)
//
// Performance Considerations:
// - Complexity: O(n) - script compilation time
// - Memory usage patterns: Moderate (engine + cached scripts)
// - Hot path optimizations: Script caching
//
// Thread Safety:
// - Thread-safe: Yes (Rhai engine can be cloned)
// - Synchronization mechanisms used: Arc<Engine> if needed
// - Interior mutability considerations: None for read-only scripts
//
// Error Handling:
// - Error types returned: Box<EvalAltResult>, RhaiError
// - Recovery strategies: Use default values on script failure
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

pub mod ConfigLoader;
pub mod ScriptRunner;
pub mod EnvironmentResolver;

// Re-export commonly used items
pub use ConfigLoader::*;
pub use ScriptRunner::*;
pub use EnvironmentResolver::*;

use rhai::{Engine, Module, ImmutableString, Shared};
use std::sync::Arc;

//=============================================================================
// Types
//=============================================================================

/// Represents the result of executing a Rhai script.
#[derive(Debug, Clone)]
pub struct ScriptResult {
	/// Map of environment variables returned by the script
	pub env_vars: std::collections::HashMap<String, String>,
	/// Whether the script executed successfully
	pub success: bool,
	/// Error message if script failed
	pub error: Option<String>,
}

/// Context information passed to Rhai scripts.
#[derive(Debug, Clone)]
pub struct ScriptContext {
	/// Profile name being executed
	pub profile_name: String,
	/// Current working directory
	pub cwd: String,
	/// Cargo manifest directory
	pub manifest_dir: String,
	/// Target triple for the build
	pub target_triple: Option<String>,
}

//=============================================================================
// Engine Setup
//=============================================================================

/// Creates and configures a new Rhai engine with all necessary modules and functions.
///
/// This function:
/// - Creates a new Rhai engine
/// - Registers standard modules
/// - Registers custom utility functions
/// - Configures engine settings for build scripts
///
/// # Returns
///
/// A configured Rhai Engine ready for script execution.
///
/// # Example
///
/// ```no_run
/// use crate::Maintain::Source::Build::Rhai;
/// let engine = Rhai::create_engine();
/// let result = engine.eval_file::<String>("script.rhai")?;
/// ```
pub fn create_engine() -> Engine {
	let mut engine = Engine::new();

	// Optimize engine for script execution
	engine.set_max_expr_depths(0, 0); // No limit on recursion
	engine.set_max_operations(0); // No limit on operations for now
	engine.set_allow_shadowing(true); // Allow variable shadowing

	// Register utility functions for scripts
	register_utility_functions(&mut engine);

	engine
}

/// Registers utility functions that can be called from Rhai scripts.
///
/// # Arguments
///
/// * `engine` - Mutable reference to the Rhai engine
fn register_utility_functions(engine: &mut Engine) {
	// System information
	engine.register_fn("get_os_type", || {
		std::env::consts::OS.to_string()
	});
	engine.register_fn("get_arch", || {
		std::env::consts::ARCH.to_string()
	});
	engine.register_fn("get_family", || {
		std::env::consts::FAMILY.to_string()
	});

	// Environment access (read-only for safety)
	engine.register_fn("get_env", |name: &str| -> String {
		std::env::var(name).unwrap_or_default()
	});

	// File system utilities (read-only paths)
	engine.register_fn("path_exists", |path: &str| -> bool {
		std::path::Path::new(path).exists()
	});

	// Time utilities
	engine.register_fn("timestamp", || -> i64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs() as i64
	});

	// Logging functions
	engine.register_fn("print", |s: &str| {
		println!("[Rhai]{