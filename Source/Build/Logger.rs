//=============================================================================//
// File Path: Element/Maintain/Source/Build/Logger.rs
//=============================================================================//
// Module: Logger
//
// Brief Description: Initializes the global logger for the build system.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Initialize the environment logger with colored output
// - Configure log level based on environment variable
// - Format log messages consistent with the build system style
//
// Secondary:
// - Support module-level log filtering
// - Provide color-coded log levels
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Logging layer
// - Logging system initialization
//
// Dependencies (What this module requires):
// - External crates: env_logger, log, colored
// - Internal modules: Constant::LogEnvironmentConstant
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Main entry point
// - Initialize function
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Singleton pattern (global logger)
// - Initialization function pattern
//
// Performance Considerations:
// - Complexity: O(1) - single initialization
// - Memory usage patterns: Minimal overhead for logging infrastructure
// - Hot path optimizations: None needed (logging is infrequent)
//
// Thread Safety:
// - Thread-safe: Yes (env_logger is thread-safe)
// - Synchronization mechanisms used: Internal to env_logger
// - Interior mutability considerations: None
//
// Error Handling:
// - Error types returned: None (panics are handled by env_logger)
// - Recovery strategies: Not applicable (logger must be initialized once)
//
// EXAMPLES:
// =========
//
// Example 1: Basic logger initialization
use std::{env, io::Write};

use colored::*;
use env_logger::Builder;
use log::LevelFilter;

/// ```rust
/// use crate::Maintain::Source::Build::Logger;
/// Logger();
/// log::info!("Logger initialized");
/// ```
// Example 2: Setting custom log level
/// ```sh
/// export RUST_LOG=debug
/// ./build-script --name MyApp pnpm build
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//
use crate::Build::Constant::LogEnv;

/// Sets up the global logger for the application.
///
/// This function initializes the `env_logger` crate with custom formatting
/// and configuration for the build system. It:
///
/// - Reads the `RUST_LOG` environment variable to set the log level
/// - Defaults to `info` level if the variable is not set
/// - Provides color-coded output for different log levels
/// - Formats messages with a consistent `[Build]` prefix
///
/// # Log Levels
///
/// Supported log levels (in increasing verbosity):
/// - `error` - Only error messages
/// - `warn` - Warning messages and errors
/// - `info` - Informational messages, warnings, and errors (default)
/// - `debug` - Debug messages and above
/// - `trace` - Trace messages and above (very verbose)
///
/// # Log Format
///
/// Messages are formatted as:
/// ```
/// [Build] [LEVEL]: message content
/// ```
///
/// Where `LEVEL` is color-coded:
/// - `[ERROR]` - Red, bold
/// - `[WARN]` - Yellow, bold
/// - `[INFO]` - Green
/// - `[DEBUG]` - Blue
/// - `[TRACE]` - Magenta
///
/// # Environment Configuration
///
/// The log level is controlled by the `RUST_LOG` environment variable:
///
/// ```sh
/// Set minimum log level to debug
/// export RUST_LOG=debug
///
/// Enable debug logs only for the Build module
/// export RUST_LOG=Build=debug
///
/// Enable trace logs for Build::Toml module
/// export RUST_LOG=Build::Toml=trace
/// ```
///
/// # Usage Example
///
/// ```no_run
/// use log::{error, info};
///
/// use crate::Maintain::Source::Build::Logger;
///
/// // Initialize logger first
/// Logger();
///
/// // Now logging is available throughout the application
/// info!("Build process started");
/// error!("Build process failed");
/// ```
///
/// # Module-Specific Logging
///
/// The build system uses the following logging targets:
/// - `Build` - General build orchestration messages
/// - `Build::Guard` - File backup and restoration messages
/// - `Build::Toml` - TOML editing messages
/// - `Build::Json` - JavaScriptObjectNotation editing messages
/// - `Build::Exec` - Command execution messages
///
/// You can set different log levels for each target:
/// ```sh
/// export RUST_LOG=Build=info,Build::Toml=debug,Build::Exec=warn
/// ```
///
/// # Thread Safety
///
/// The global logger initialized by this function is thread-safe and can be
/// used from multiple threads concurrently without additional synchronization.
///
/// # Implementation Notes
///
/// This function uses `env_logger::Builder` to construct the logger with
/// custom formatting. The formatter uses the `colored` crate to apply
/// color coding based on the log level.
///
/// The logger is typically called once at program startup, before any other
/// operations that might generate log messages.
pub fn Logger() {
	let LevelText = env::var(LogEnv).unwrap_or_else(|_| "info".to_string());

	let LogLevel = LevelText.parse::<LevelFilter>().unwrap_or(LevelFilter::Info);

	Builder::new()
		.filter_level(LogLevel)
		.format(|Buffer, Record| {
			let LevelStyle = match Record.level() {
				log::Level::Error => "ERROR".red().bold(),

				log::Level::Warn => "WARN".yellow().bold(),

				log::Level::Info => "INFO".green(),

				log::Level::Debug => "DEBUG".blue(),

				log::Level::Trace => "TRACE".magenta(),
			};

			writeln!(Buffer, "[{}] [{}]: {}", "Build".red(), LevelStyle, Record.args())
		})
		.parse_default_env()
		.init();
}
