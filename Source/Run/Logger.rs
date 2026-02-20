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

use crate::Run::Constant::LogEnv;

use env_logger::Builder;
use log::LevelFilter;
use std::io::Write;

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

            writeln!(
                buf,
                "{} [{}] {}",
                level_str,
                module.white().bold(),
                record.args()
            )
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
pub fn log_run_header(profile: &str, workbench: Option<&str>) {
    use log::info;

    info!("========================================");
    info!("Land Run: {}", profile);
    info!("========================================");

    if let Some(wb) = workbench {
        info!("Workbench: {}", wb);
    }
}

/// Logs environment variable resolution.
///
/// # Arguments
///
/// * `env` - The resolved environment variables
pub fn log_environment(env: &std::collections::HashMap<String, String>) {
    use log::debug;

    debug!("Resolved environment variables:");
    for (key, value) in env {
        let display_value = if value.is_empty() {
            "(empty)"
        } else {
            value.as_str()
        };
        debug!("  {} = {}", key, display_value);
    }
}

/// Logs a success message.
///
/// # Arguments
///
/// * `message` - The success message
pub fn log_success(message: &str) {
    use log::info;
    use colored::Colorize;

    info!("{}", message.green());
}

/// Logs an error message.
///
/// # Arguments
///
/// * `message` - The error message
pub fn log_error(message: &str) {
    use log::error;
    use colored::Colorize;

    error!("{}", message.red());
}

/// Logs a warning message.
///
/// # Arguments
///
/// * `message` - The warning message
pub fn log_warning(message: &str) {
    use log::warn;
    use colored::Colorize;

    warn!("{}", message.yellow());
}

/// Logs the start of a run process.
///
/// # Arguments
///
/// * `command` - The command being executed
pub fn log_run_start(command: &str) {
    use log::info;

    info!("Starting run: {}", command);
}

/// Logs the completion of a run process.
///
/// # Arguments
///
/// * `success` - Whether the run completed successfully
pub fn log_run_complete(success: bool) {
    use log::info;
    use colored::Colorize;

    if success {
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
pub fn log_hot_reload_status(enabled: bool, port: u16) {
    use log::info;
    use colored::Colorize;

    if enabled {
        info!("Hot-reload enabled on port {}", port.to_string().cyan());
    } else {
        info!("Hot-reload disabled");
    }
}

/// Logs watch mode status.
///
/// # Arguments
///
/// * `enabled` - Whether watch mode is enabled
pub fn log_watch_status(enabled: bool) {
    use log::info;
    use colored::Colorize;

    if enabled {
        info!("Watch mode {}", "enabled".green());
    } else {
        info!("Watch mode disabled");
    }
}
