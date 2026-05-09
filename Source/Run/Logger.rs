//=============================================================================//
// File Path: Element/Maintain/Source/Run/Logger.rs
//=============================================================================//
// Module: Logger
//
// Brief Description: Logging utilities for the Run module.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Initialize the logging infrastructure for run operations
// - Provide colored, structured log output
// - Support configurable log levels
//
// Secondary:
// - Format log messages consistently
// - Support file and console output
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Logging layer
// - Log initialization
//
// Dependencies (What this module requires):
// - External crates: env_logger, log, colored
// - Internal modules: Constant
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Run entry point (Fn)
// - Run Process module
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use std::io::Write;

use env_logger::Builder;

use log::LevelFilter;

use crate::Run::Constant::LogEnv;

/// Initializes the logger for run operations.
///
/// This function sets up a colored, structured logger with support for
/// configurable log levels via the `RUST_LOG` environment variable.
///
/// # Log Levels
///
/// Control log verbosity with `RUST_LOG`:
/// ```sh
/// export RUST_LOG=debug # More verbose output
/// export RUST_LOG=info  # Standard output
/// export RUST_LOG=error # Only errors
/// export RUST_LOG=Run=debug # Run module only
/// ```
///
/// # Example
///
/// ```rust
/// use crate::Run::Logger;
/// Logger();
/// ```
pub fn Logger() {

	Builder::from_env(LogEnv)
		.format(|buf, record| {
			use colored::Colorize;

			let level = record.level();
			let level_str = match level {
				log::Level::Error => "ERROR".red(),
				log::Level::Warn => "WARN ".yellow(),
				log::Level::Info => "INFO ".green(),
				log::Level::Debug => "DEBUG".blue(),
				log::Level::Trace => "TRACE".magenta(),
			};

			let target = record.target();
			let module = target.split("::").last().unwrap_or("Run");

			writeln!(buf, "{} [{}] {}", level_str, module.white().bold(), record.args())
		})
		.filter(None, LevelFilter::Info)
		.init();
}

/// Logs a run header message.
///
/// # Arguments
///
/// * `profile` - The profile name
/// * `workbench` - The workbench type
pub fn LogRunHeader(Profile:&str, Workbench:Option<&str>) {

	use log::info;

	info!("========================================");

	info!("Land Run: {}", Profile);

	info!("========================================");

	if let Some(Wb) = Workbench {

		info!("Workbench: {}", Wb);
	}
}

/// Logs environment variable resolution.
///
/// # Arguments
///
/// * `env` - The resolved environment variables
pub fn LogEnvironment(Env:&std::collections::HashMap<String, String>) {

	use log::debug;

	debug!("Resolved environment variables:");

	for (Key, Value) in Env {

		let DisplayValue = if Value.is_empty() { "(empty)" } else { Value.as_str() };

		debug!(" {} = {}", Key, DisplayValue);
	}
}

/// Logs a success message.
///
/// # Arguments
///
/// * `message` - The success message
pub fn LogSuccess(Message:&str) {

	use log::info;

	use colored::Colorize;

	info!("{}", Message.green());
}

/// Logs an error message.
///
/// # Arguments
///
/// * `message` - The error message
pub fn LogError(Message:&str) {

	use log::error;

	use colored::Colorize;

	error!("{}", Message.red());
}

/// Logs a warning message.
///
/// # Arguments
///
/// * `message` - The warning message
pub fn LogWarning(Message:&str) {

	use log::warn;

	use colored::Colorize;

	warn!("{}", Message.yellow());
}

/// Logs the start of a run process.
///
/// # Arguments
///
/// * `command` - The command being executed
pub fn LogRunStart(Command:&str) {

	use log::info;

	info!("Starting run: {}", Command);
}

/// Logs the completion of a run process.
///
/// # Arguments
///
/// * `success` - Whether the run completed successfully
pub fn LogRunComplete(Success:bool) {

	use log::info;

	use colored::Colorize;

	if Success {

		info!("{}", "Run completed successfully".green());
	} else {

		info!("{}", "Run completed with errors".red());
	}
}

/// Logs hot-reload status.
///
/// # Arguments
///
/// * `enabled` - Whether hot-reload is enabled
/// * `port` - The live-reload port
pub fn LogHotReloadStatus(Enabled:bool, Port:u16) {

	use log::info;

	use colored::Colorize;

	if Enabled {

		info!("Hot-reload enabled on port {}", Port.to_string().cyan());
	} else {

		info!("Hot-reload disabled");
	}
}

/// Logs watch mode status.
///
/// # Arguments
///
/// * `enabled` - Whether watch mode is enabled
pub fn LogWatchStatus(Enabled:bool) {

	use log::info;

	use colored::Colorize;

	if Enabled {

		info!("Watch mode {}", "enabled".green());
	} else {

		info!("Watch mode disabled");
	}
}
