//! Creates and configures a new Rhai engine with all necessary modules and
//! functions for dynamic script configuration.

use rhai::Engine;

use super::RegisterUtilityFunctions::Fn as RegisterUtilityFunctions;

/// Creates a new Rhai engine pre-configured with utility functions.
///
/// The engine is optimized for script execution with:
/// - Maximum expression depths unlimited (0)
/// - Maximum operations unlimited (0)
/// - Variable shadowing enabled
/// - Utility functions registered for OS info, env access, filesystem checks,
///   timestamps, and logging
pub fn Fn() -> Engine {
	let mut engine = Engine::new();

	// Optimize engine for script execution
	engine.set_max_expr_depths(0, 0);

	engine.set_max_operations(0);

	engine.set_allow_shadowing(true);

	// Register utility functions for scripts
	RegisterUtilityFunctions(&mut engine);

	engine
}
