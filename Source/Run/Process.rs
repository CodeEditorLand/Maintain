//=============================================================================//
// File Path: Element/Maintain/Source/Run/Process.rs
//=============================================================================//
// Module: Process
//
// Brief Description: Process management for run operations.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Start and manage the development run process
// - Handle hot-reload and watch mode
// - Manage process lifecycle
//
// Secondary:
// - Provide process status reporting
// - Handle graceful shutdown
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Process management layer
// - Process orchestration
//
// Dependencies (What this module requires):
// - External crates: log, std
// - Internal modules: Definition, Environment, Logger, Error
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Run entry point (Fn)
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use std::process::{Command, Stdio};

use log::{debug, error, info};

use crate::Run::{
	Definition::{Argument, RunConfig},
	Environment,
	Error::{Error, Result},
	Logger::{LogHotReloadStatus, LogRunComplete, LogRunStart, LogWatchStatus},
};

/// Executes the run process with the provided configuration.
///
/// This function orchestrates the development run, including:
/// 1. Setting up the environment
/// 2. Starting the development server
/// 3. Managing hot-reload and watch mode
/// 4. Handling process lifecycle
///
/// # Arguments
///
/// * `arg` - The parsed command-line arguments
///
/// # Returns
///
/// Result indicating success or failure
pub fn Process(Arg:&Argument) -> Result<()> {
	// Resolve environment variables
	let EnvVars = crate::Run::Environment::Resolve(
		&crate::Run::Definition::Profile {
			name:Arg.Profile.clone(),
			description:None,
			workbench:Arg.Workbench.clone(),
			env:None,
			run_config:None,
		},
		true, // merge_shell
		&Arg.env_override,
	)?;

	// Validate environment
	let ValidationErrors = Environment::Validate(&EnvVars);

	if !ValidationErrors.is_empty() {
		for Error in ValidationErrors {
			error!("Environment validation: {}", Error);
		}

		return Err(Error::InvalidConfig("Environment validation failed".to_string()));
	}

	// Create run configuration
	let Config = RunConfig::new(Arg, EnvVars.clone());

	// Log run header
	LogRunHeader(&Config);

	// Log hot-reload and watch status
	LogHotReloadStatus(Config.hot_reload, Config.live_reload_port);

	LogWatchStatus(Config.watch);

	// Dry run mode
	if Arg.DryRun {
		info!("Dry run mode - showing configuration without executing");

		debug!("Configuration: {:?}", Config);

		return Ok(());
	}

	// Determine the run command
	let Command = DetermineRunCommand(&Config);

	// Start the run process
	ExecuteRun(&Command, &EnvVars)
}

/// Logs the run header.
///
/// # Arguments
///
/// * `config` - The run configuration
fn LogRunHeader(config:&RunConfig) {
	info!("========================================");

	info!("Land Run: {}", config.profile_name);

	info!("========================================");

	if let Some(Workbench) = config.get_workbench() {
		info!("Workbench: {}", Workbench);
	}

	info!("Hot-reload: {}", if config.hot_reload { "enabled" } else { "disabled" });

	info!("Watch mode: {}", if config.watch { "enabled" } else { "disabled" });
}

/// Determines the run command based on configuration.
///
/// # Arguments
///
/// * `config` - The run configuration
///
/// # Returns
///
/// A vector of command arguments
fn DetermineRunCommand(config:&RunConfig) -> Vec<String> {
	// If custom command is provided, use it
	if !config.command.is_empty() {
		return config.command.clone();
	}

	// Default to pnpm tauri dev for debug profiles
	if config.is_debug() {
		vec!["pnpm".to_string(), "tauri".to_string(), "dev".to_string()]
	} else {
		vec!["pnpm".to_string(), "dev".to_string()]
	}
}

/// Executes the run command.
///
/// # Arguments
///
/// * `command` - The command to execute
/// * `env_vars` - Environment variables to set
///
/// # Returns
///
/// Result indicating success or failure
fn ExecuteRun(Command:&[String], EnvVars:&std::collections::HashMap<String, String>) -> Result<()> {
	if Command.is_empty() {
		return Err(Error::ProcessStart("Empty command".to_string()));
	}

	let (Program, Args) = Command.split_first().unwrap();

	LogRunStart(&Command.join(" "));

	debug!("Executing: {} {:?}", Program, Args);

	let mut Cmd = Command::new(Program);

	Cmd.args(Args);

	Cmd.stdin(Stdio::inherit());

	Cmd.stdout(Stdio::inherit());

	Cmd.stderr(Stdio::inherit());

	// Set all environment variables
	for (Key, Value) in EnvVars {
		Cmd.env(Key, Value);
	}

	// Set run mode indicators
	Cmd.env("MAINTAIN_RUN_MODE", "true");

	// Execute the command
	let Status = Cmd
		.status()
		.map_err(|Error| Error::ProcessStart(format!("Failed to start {}: {}", Program, Error)))?;

	if Status.success() {
		LogRunComplete(true);

		Ok(())
	} else {
		let Code = Status.code().unwrap_or(-1);

		LogRunComplete(false);

		Err(Error::ProcessExit(Code))
	}
}

/// Starts a hot-reload watcher.
///
/// # Arguments
///
/// * `watch_dirs` - Directories to watch
/// * `callback` - Function to call on file changes
///
/// # Returns
///
/// Result indicating success or failure
fn start_hot_reload_watcher(watch_dirs:&[String], _callback:impl Fn() + Send + 'static) -> Result<()> {
	// Placeholder for hot-reload watcher implementation
	// In a full implementation, this would use the `notify` crate
	// to watch for file changes and trigger reloads

	info!("Hot-reload watcher would watch: {:?}", watch_dirs);

	Ok(())
}

/// Gracefully shuts down the run process.
///
/// This function handles cleanup and graceful termination
/// of any child processes.
pub fn shutdown() {
	info!("Shutting down run process...");

	// Cleanup logic would go here
}
