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

use crate::Run::Definition::{Argument, RunConfig};
use crate::Run::Environment;
use crate::Run::Logger::{log_run_start, log_run_complete, log_hot_reload_status, log_watch_status};
use crate::Run::Error::{Error, Result};

use log::{debug, error, info};
use std::process::{Command, Stdio};

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
pub fn Process(arg: &Argument) -> Result<()> {
    // Resolve environment variables
    let env_vars = crate::Run::Environment::resolve(
        &crate::Run::Definition::Profile {
            name: arg.Profile.clone(),
            description: None,
            workbench: arg.Workbench.clone(),
            env: None,
            run_config: None,
        },
        true, // merge_shell
        &arg.env_override,
    )?;

    // Validate environment
    let validation_errors = Environment::validate(&env_vars);
    if !validation_errors.is_empty() {
        for err in validation_errors {
            error!("Environment validation: {}", err);
        }
        return Err(Error::InvalidConfig("Environment validation failed".to_string()));
    }

    // Create run configuration
    let config = RunConfig::new(arg, env_vars.clone());

    // Log run header
    log_run_header(&config);

    // Log hot-reload and watch status
    log_hot_reload_status(config.hot_reload, config.live_reload_port);
    log_watch_status(config.watch);

    // Dry run mode
    if arg.DryRun {
        info!("Dry run mode - showing configuration without executing");
        debug!("Configuration: {:?}", config);
        return Ok(());
    }

    // Determine the run command
    let command = determine_run_command(&config);

    // Start the run process
    execute_run(&command, &env_vars)
}

/// Logs the run header.
///
/// # Arguments
///
/// * `config` - The run configuration
fn log_run_header(config: &RunConfig) {
    info!("========================================");
    info!("Land Run: {}", config.profile_name);
    info!("========================================");

    if let Some(workbench) = config.get_workbench() {
        info!("Workbench: {}", workbench);
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
fn determine_run_command(config: &RunConfig) -> Vec<String> {
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
fn execute_run(command: &[String], env_vars: &std::collections::HashMap<String, String>) -> Result<()> {
    if command.is_empty() {
        return Err(Error::ProcessStart("Empty command".to_string()));
    }

    let (program, args) = command.split_first().unwrap();

    log_run_start(&command.join(" "));

    debug!("Executing: {} {:?}", program, args);

    let mut cmd = Command::new(program);
    cmd.args(args);
    cmd.stdin(Stdio::inherit());
    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    // Set all environment variables
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    // Set run mode indicators
    cmd.env("MAINTAIN_RUN_MODE", "true");

    // Execute the command
    let status = cmd.status()
        .map_err(|e| Error::ProcessStart(format!("Failed to start {}: {}", program, e)))?;

    if status.success() {
        log_run_complete(true);
        Ok(())
    } else {
        let code = status.code().unwrap_or(-1);
        log_run_complete(false);
        Err(Error::ProcessExit(code))
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
#[allow(dead_code)]
fn start_hot_reload_watcher(
    watch_dirs: &[String],
    _callback: impl Fn() + Send + 'static,
) -> Result<()> {
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
