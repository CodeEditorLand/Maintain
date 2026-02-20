//=============================================================================//
// File Path: Element/Maintain/Source/Build/CLI/mod.rs
//=============================================================================//
// Module: CLI - Command Line Interface for Configuration-Based Builds
//
// This module provides the cargo-first CLI interface that enables triggering
// builds directly with the Cargo utility instead of shell scripts.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Parse command-line arguments for profile-based builds
// - Load and validate configuration from land-config.json
// - Resolve environment variables from configuration
// - Execute builds with resolved configuration
//
// Secondary:
// - Provide utility commands (--list-profiles, --show-profile)
// - Support dry-run mode for configuration preview
// - Enable profile aliases for quick access
//
// USAGE:
// ======
//
// Basic usage:
// ```bash
// cargo run --bin Maintain -- --profile debug-mountain
// ```
//
// List profiles:
// ```bash
// cargo run --bin Maintain -- --list-profiles
// ```
//
// Dry run:
// ```bash
// cargo run --bin Maintain -- --profile debug --dry-run
// ```
//
//===================================================================================

use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

use crate::Build::Rhai::{load_config, LandConfig, Profile};

//=============================================================================
// CLI Argument Definitions
//=============================================================================

/// Land Build System - Configuration-based builds via Cargo
#[derive(Parser, Debug, Clone)]
#[clap(
    name = "maintain",
    author,
    version,
    about = "Land Build System - Configuration-based builds",
    long_about = "A configuration-driven build system that enables triggering builds \
                  directly with Cargo instead of shell scripts. Reads configuration \
                  from .vscode/land-config.json and supports multiple build profiles."
)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Option<Commands>,

    /// Build profile to use (shortcut for 'build' subcommand)
    #[clap(long, short = 'p', value_parser = parse_profile_name)]
    pub profile: Option<String>,

    /// Configuration file path (default: .vscode/land-config.json)
    #[clap(long, short = 'c', global = true)]
    pub config: Option<PathBuf>,

    /// Override workbench type
    #[clap(long, short = 'w', global = true)]
    pub workbench: Option<String>,

    /// Override Node.js version
    #[clap(long, short = 'n', global = true)]
    pub node_version: Option<String>,

    /// Override Node.js environment
    #[clap(long, short = 'e', global = true)]
    pub environment: Option<String>,

    /// Override dependency source
    #[clap(long, short = 'd', global = true)]
    pub dependency: Option<String>,

    /// Override environment variables (key=value pairs)
    #[clap(long = "env", value_parser = parse_key_val::<String, String>, global = true, action = clap::ArgAction::Append)]
    pub env_override: Vec<(String, String)>,

    /// Enable dry-run mode (show config without building)
    #[clap(long, global = true)]
    pub dry_run: bool,

    /// Enable verbose output
    #[clap(long, short = 'v', global = true)]
    pub verbose: bool,

    /// Merge with shell environment (default: true)
    #[clap(long, default_value = "true", global = true)]
    pub merge_env: bool,

    /// Additional build arguments (passed through to build command)
    #[clap(last = true)]
    pub build_args: Vec<String>,
}

/// Available subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Execute a build with the specified profile
    Build {
        /// Build profile to use
        #[clap(long, short = 'p', value_parser = parse_profile_name)]
        profile: String,

        /// Enable dry-run mode
        #[clap(long)]
        dry_run: bool,
    },

    /// List all available build profiles
    ListProfiles {
        /// Show detailed information for each profile
        #[clap(long, short = 'v')]
        verbose: bool,
    },

    /// Show details for a specific profile
    ShowProfile {
        /// Profile name to show
        profile: String,
    },

    /// Validate a build profile
    ValidateProfile {
        /// Profile name to validate
        profile: String,
    },

    /// Show current environment variable resolution
    Resolve {
        /// Profile name to resolve
        #[clap(long, short = 'p')]
        profile: String,

        /// Output format
        #[clap(long, short = 'f', default_value = "table")]
        format: OutputFormat,
    },
}

/// Output format options
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
    Env,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Table => write!(f, "table"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Env => write!(f, "env"),
        }
    }
}

//=============================================================================
// CLI Implementation
//=============================================================================//

impl Cli {
    /// Execute the CLI command
    pub fn execute(&self) -> Result<(), String> {
        let config_path = self.config.clone().unwrap_or_else(|| {
            PathBuf::from(".vscode/land-config.json")
        });

        // Load configuration
        let config = load_config(&config_path)
            .map_err(|e| format!("Failed to load configuration: {}", e))?;

        // Handle subcommands
        if let Some(command) = &self.command {
            return self.execute_command(command, &config);
        }

        // Handle direct profile argument
        if let Some(profile_name) = &self.profile {
            return self.execute_build(profile_name, &config, self.dry_run);
        }

        // Default: show help
        Err("No command specified. Use --profile <name> to build or --help for usage.".to_string())
    }

    /// Execute a subcommand
    fn execute_command(&self, command: &Commands, config: &LandConfig) -> Result<(), String> {
        match command {
            Commands::Build { profile, dry_run } => {
                self.execute_build(profile, config, *dry_run)
            }
            Commands::ListProfiles { verbose } => {
                self.execute_list_profiles(config, *verbose)
            }
            Commands::ShowProfile { profile } => {
                self.execute_show_profile(profile, config)
            }
            Commands::ValidateProfile { profile } => {
                self.execute_validate_profile(profile, config)
            }
            Commands::Resolve { profile, format } => {
                self.execute_resolve(profile, config, Some(format.to_string()))
            }
        }
    }

    /// Execute a build with the specified profile
    fn execute_build(
        &self,
        profile_name: &str,
        config: &LandConfig,
        dry_run: bool,
    ) -> Result<(), String> {
        // Resolve profile name (handle aliases)
        let resolved_profile = resolve_profile_name(profile_name, config);

        // Get profile from config
        let profile = config.profiles.get(&resolved_profile).ok_or_else(|| {
            format!(
                "Profile '{}' not found. Available profiles: {}",
                resolved_profile,
                config.profiles.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?;

        // Print build header
        print_build_header(&resolved_profile, profile);

        // Resolve environment variables with dual-path merge
        let env_vars = resolve_environment_dual_path(
            profile,
            config,
            self.merge_env,
            &self.env_override,
        );

        // Apply CLI overrides for explicit flags
        let env_vars = apply_overrides(
            env_vars,
            &self.workbench,
            &self.node_version,
            &self.environment,
            &self.dependency,
        );

        // Print resolved configuration
        if self.verbose || dry_run {
            print_resolved_environment(&env_vars);
        }

        // Dry run: stop here
        if dry_run {
            println!("\n{}", "Dry run complete. No changes made.");
            return Ok(());
        }

        // Execute build
        execute_build_command(&resolved_profile, config, &env_vars, &self.build_args)
    }

    /// List all available profiles
    fn execute_list_profiles(&self, config: &LandConfig, verbose: bool) -> Result<(), String> {
        println!("\n{}", "Land Build System - Available Profiles");
        println!("{}\n", "=".repeat(50));

        // Group profiles by type
        let mut debug_profiles: Vec<_> = config
            .profiles
            .iter()
            .filter(|(k, _)| k.starts_with("debug"))
            .collect();
        let mut release_profiles: Vec<_> = config
            .profiles
            .iter()
            .filter(|(k, _)| {
                k.starts_with("production") || k.starts_with("release") || k.starts_with("web")
            })
            .collect();
        let mut bundler_profiles: Vec<_> = config
            .profiles
            .iter()
            .filter(|(k, _)| k.contains("bundler") || k.contains("swc") || k.contains("oxc"))
            .collect();

        // Sort profiles
        debug_profiles.sort_by_key(|(k, _)| k.as_str());
        release_profiles.sort_by_key(|(k, _)| k.as_str());
        bundler_profiles.sort_by_key(|(k, _)| k.as_str());

        // Print debug profiles
        println!("{}:", "Debug Profiles".yellow());
        println!();
        for (name, profile) in &debug_profiles {
            let default_profile = config.cli.as_ref()
                .and_then(|cli| cli.default_profile.as_ref())
                .map(|s| s.as_str())
                .unwrap_or("");
            let recommended = default_profile == name.as_str();
            let marker = if recommended { " [RECOMMENDED]" } else { "" };
            println!(
                "  {:<20} - {}{}",
                name.green(),
                profile.description.as_ref().map(|d| d.as_str()).unwrap_or("No description"),
                marker.bright_magenta()
            );
            if verbose {
                if let Some(workbench) = &profile.workbench {
                    println!("    Workbench: {}", workbench);
                }
                if let Some(features) = &profile.features {
                    for (feature, enabled) in features {
                            let status = if *enabled { "[X]" } else { "[ ]" };
                            println!("    {:>20} {} = {}", feature.cyan(), status, enabled);
                    }
                }
            }
        }

        // Print release profiles
        println!("\n{}:", "Release Profiles");
        for (name, profile) in &release_profiles {
            println!(
                "  {:<20} - {}",
                name.green(),
                profile.description.as_ref().map(|d| d.as_str()).unwrap_or("No description")
            );
            if verbose {
                if let Some(workbench) = &profile.workbench {
                    println!("    Workbench: {}", workbench);
                }
            }
        }

        // Print bundler profiles
        if !bundler_profiles.is_empty() {
            println!("\n{}:", "Bundler Profiles");
            for (name, profile) in &bundler_profiles {
                println!(
                    "  {:<20} - {}",
                    name.green(),
                    profile.description.as_ref().map(|d| d.as_str()).unwrap_or("No description")
                );
            }
        }

        // Print CLI aliases if available
        if let Some(cli_config) = &config.cli {
            if !cli_config.profile_aliases.is_empty() {
                println!("\n{}:", "Profile Aliases");
                for (alias, target) in &cli_config.profile_aliases {
                    println!("  {:<10} -> {}", alias.cyan(), target);
                }
            }
        }

        println!();
        Ok(())
    }

    /// Show details for a specific profile
    fn execute_show_profile(&self, profile_name: &str, config: &LandConfig) -> Result<(), String> {
        let resolved_profile = resolve_profile_name(profile_name, config);

        let profile = config.profiles.get(&resolved_profile).ok_or_else(|| {
            format!("Profile '{}' not found.", resolved_profile)
        })?;

        println!("\n{}: {}", "Profile:", resolved_profile.green());
        println!("{}\n", "=".repeat(50));

        // Description
        if let Some(desc) = &profile.description {
            println!("Description: {}", desc);
        }

        // Workbench
        if let Some(workbench) = &profile.workbench {
            println!("\nWorkbench:");
            println!("  Type: {}", workbench);

            if let Some(wb_config) = &config.workbench {
                if let Some(features) = &wb_config.features {
                    if let Some(wb_features) = features.get(workbench) {
                        if let Some(coverage) = &wb_features.coverage {
                            println!("  Coverage: {}", coverage);
                        }
                        if let Some(complexity) = &wb_features.complexity {
                            println!("  Complexity: {}", complexity);
                        }
                        if wb_features.polyfills.unwrap_or(false) {
                            println!("  Polyfills: enabled");
                        }
                        if wb_features.mountain_providers.unwrap_or(false) {
                            println!("  Mountain Providers: enabled");
                        }
                        if wb_features.wind_services.unwrap_or(false) {
                            println!("  Wind Services: enabled");
                        }
                    }
                }
            }
        }

        // Environment Variables
        println!("\nEnvironment Variables:");
        if let Some(env) = &profile.env {
            let mut sorted_env: Vec<_> = env.iter().collect();
            sorted_env.sort_by_key(|(k, _)| k.as_str());
            for (key, value) in sorted_env {
                println!("  {:<25} = {}", key, value);
            }
        }

        // Features
        if let Some(features) = &profile.features {
            println!("\nFeatures:");
            println!("\n  Enabled:");
            let mut sorted_features: Vec<_> = features.iter()
                .filter(|(_, enabled)| **enabled)
                .collect();
            sorted_features.sort_by_key(|(k, _)| k.as_str());
            for (feature, _) in &sorted_features {
                println!("  {:<30}", feature.green());
            }
        }

        // Build Command
        let build_cmd = get_build_command(&resolved_profile, &config);
        println!("\nBuild Command: {}", build_cmd);

        // Rhai Script
        if let Some(script) = &profile.rhai_script {
            println!("\nRhai Script: {}", script);
        }

        println!();
        Ok(())
    }

    /// Validate a profile's configuration
    fn execute_validate_profile(&self, profile_name: &str, config: &LandConfig) -> Result<(), String> {
        let resolved_profile = resolve_profile_name(profile_name, config);

        let profile = config.profiles.get(&resolved_profile).ok_or_else(|| {
            format!("Profile '{}' not found.", resolved_profile)
        })?;

        println!("\n{}: {}", "Validating Profile:", resolved_profile.green());
        println!("{}\n", "=".repeat(50));

        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check description
        if profile.description.is_none() {
            warnings.push("Profile has no description".to_string());
        }

        // Check workbench
        if profile.workbench.is_none() {
            issues.push("Profile has no workbench type specified".to_string());
        } else if let Some(workbench) = &profile.workbench {
            if let Some(wb_config) = &config.workbench {
                if let Some(available) = &wb_config.available {
                    if !available.contains(workbench) {
                        issues.push(format!("Workbench '{}' not defined in workbench configuration", workbench));
                    }
                }
            }
        }

        // Check environment variables
        if profile.env.is_none() || profile.env.as_ref().unwrap().is_empty() {
            warnings.push("Profile has no environment variables defined".to_string());
        }

        // Display results
        if issues.is_empty() && warnings.is_empty() {
            println!("{}", "Profile is valid!".green());
        } else {
            if !warnings.is_empty() {
                println!("\n{} Warnings:", warnings.len().to_string().yellow());
                for warning in &warnings {
                    println!("  - {}", warning.yellow());
                }
            }
            if !issues.is_empty() {
                println!("\n{} Issues:", issues.len().to_string().red());
                for issue in &issues {
                    println!("  - {}", issue.red());
                }
            }
        }

        println!();
        Ok(())
    }

    /// Resolve a profile to its resolved configuration
    fn execute_resolve(&self, profile_name: &str, config: &LandConfig, _format: Option<String>) -> Result<(), String> {
        let resolved_profile = resolve_profile_name(profile_name, config);

        let profile = config.profiles.get(&resolved_profile).ok_or_else(|| {
            format!("Profile '{}' not found.", resolved_profile)
        })?;

        println!("\n{}: {}", "Resolved Profile:", resolved_profile.green());
        println!("{}\n", "=".repeat(50));

        // Profile information
        if let Some(desc) = &profile.description {
            println!("Description: {}", desc);
        }

        if let Some(workbench) = &profile.workbench {
            println!("Workbench: {}", workbench);
        }

        // Environment Variables
        if let Some(env) = &profile.env {
            println!("\nEnvironment Variables ({}):", env.len());
            for (key, value) in env {
                println!("  {} = {}", key.green(), value);
            }
        }

        // Features
        if let Some(features) = &profile.features {
            println!("\nFeatures ({}):", features.len());
            for (feature, enabled) in features {
                let status = if *enabled { "[X]" } else { "[ ]" };
                println!("  {} {}", status, feature);
            }
        }

        println!();
        Ok(())
    }
}

//=============================================================================
// Helper Functions (standalone functions, not methods)
//=============================================================================

/// Print build header
fn print_build_header(profile_name: &str, profile: &Profile) {
    println!("\n{}", "========================================");
    println!("Land Build: {}", profile_name);
    println!("========================================");

    if let Some(desc) = &profile.description {
        println!("Description: {}", desc);
    }

    if let Some(workbench) = &profile.workbench {
        println!("Workbench: {}", workbench);
    }
}

/// Print resolved environment variables
fn print_resolved_environment(env: &HashMap<String, String>) {
    println!("\nResolved Environment:");
    let mut sorted_env: Vec<_> = env.iter().collect();
    sorted_env.sort_by_key(|(k, _)| k.as_str());

    for (key, value) in sorted_env {
        let display_value = if value.is_empty() {
            "(empty)"
        } else {
            value
        };
        println!("  {:<25} = {}", key, display_value);
    }
}

/// Parse and validate profile name
fn parse_profile_name(s: &str) -> Result<String, String> {
    let name = s.trim().to_lowercase();

    if name.is_empty() {
        return Err("Profile name cannot be empty".to_string());
    }

    if name.contains(' ') {
        return Err("Profile name cannot contain spaces".to_string());
    }

    Ok(name)
}

/// Resolve profile name (handle aliases)
fn resolve_profile_name(name: &str, config: &LandConfig) -> String {
    if let Some(cli_config) = &config.cli {
        if let Some(resolved) = cli_config.profile_aliases.get(name) {
            return resolved.clone();
        }
    }
    name.to_string()
}

/// Get build command for a profile
fn get_build_command(profile_name: &str, config: &LandConfig) -> String {
    // Determine the command from NODE_ENV
    let command_key = if profile_name.starts_with("production")
        || profile_name.starts_with("release")
        || profile_name.starts_with("web")
    {
        "production"
    } else {
        profile_name.split('-').next().unwrap_or("debug")
    };

    if let Some(build_commands) = &config.build_commands {
        build_commands.get(command_key).cloned().unwrap_or_else(|| {
            if profile_name.starts_with("production") {
                "pnpm tauri build".to_string()
            } else {
                "pnpm tauri build --debug".to_string()
            }
        })
    } else {
        // Fallback command
        if profile_name.starts_with("production") {
            "pnpm tauri build".to_string()
        } else {
            "pnpm tauri build --debug".to_string()
        }
    }
}

/// Resolve environment variables with dual-path merging.
///
/// This function implements the dual-path environment resolution:
/// - Path A: Shell environment variables (from process)
/// - Path B: CLI profile configuration (from land-config.json)
///
/// Merge priority (lowest to highest):
/// 1. Template defaults
/// 2. Shell environment variables (if merge_env is true)
/// 3. Profile environment variables
/// 4. CLI --env overrides
///
/// # Arguments
///
/// * `profile` - The profile configuration
/// * `config` - The land configuration
/// * `merge_env` - Whether to merge with shell environment
/// * `cli_overrides` - CLI --env override pairs
///
/// # Returns
///
/// Merged HashMap of environment variables
fn resolve_environment_dual_path(
    profile: &Profile,
    config: &LandConfig,
    merge_env: bool,
    cli_overrides: &[(String, String)],
) -> HashMap<String, String> {
    let mut env = HashMap::new();

    // Layer 1: Start with template defaults (lowest priority)
    if let Some(templates) = &config.templates {
        for (key, value) in &templates.env {
            env.insert(key.clone(), value.clone());
        }
    }

    // Layer 2: Merge shell environment variables (if enabled)
    if merge_env {
        for (key, value) in std::env::vars() {
            // Only merge relevant environment variables
            // that are part of our build system
            if is_build_env_var(&key) {
                env.insert(key, value);
            }
        }
    }

    // Layer 3: Apply profile environment (overrides shell)
    if let Some(profile_env) = &profile.env {
        for (key, value) in profile_env {
            env.insert(key.clone(), value.clone());
        }
    }

    // Layer 4: Apply CLI --env overrides (highest priority)
    for (key, value) in cli_overrides {
        env.insert(key.clone(), value.clone());
    }

    env
}

/// Check if an environment variable is a build system variable.
fn is_build_env_var(key: &str) -> bool {
    matches!(
        key,
        "Browser"
            | "Bundle"
            | "Clean"
            | "Compile"
            | "Debug"
            | "Dependency"
            | "Mountain"
            | "Wind"
            | "Electron"
            | "BrowserProxy"
            | "NODE_ENV"
            | "NODE_VERSION"
            | "NODE_OPTIONS"
            | "RUST_LOG"
            | "AIR_LOG_JSON"
            | "AIR_LOG_FILE"
            | "Level"
            | "Name"
            | "Prefix"
    )
}

/// Parse a key=value pair from command line.
fn parse_key_val<K, V>(s: &str) -> Result<(K, V), String>
where
    K: std::str::FromStr,
    V: std::str::FromStr,
    K::Err: std::fmt::Display,
    V::Err: std::fmt::Display,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((
        s[..pos].parse().map_err(|e| format!("key parse error: {e}"))?,
        s[pos + 1..].parse().map_err(|e| format!("value parse error: {e}"))?,
    ))
}

/// Apply CLI overrides to environment
fn apply_overrides(
    mut env: HashMap<String, String>,
    workbench: &Option<String>,
    node_version: &Option<String>,
    environment: &Option<String>,
    dependency: &Option<String>,
) -> HashMap<String, String> {
    if let Some(workbench) = workbench {
        // Clear all workbench flags
        env.remove("Browser");
        env.remove("Wind");
        env.remove("Mountain");
        env.remove("Electron");
        env.remove("BrowserProxy");

        // Set the selected workbench
        env.insert(workbench.clone(), "true".to_string());
    }

    if let Some(version) = node_version {
        env.insert("NODE_VERSION".to_string(), version.clone());
    }

    if let Some(environment) = environment {
        env.insert("NODE_ENV".to_string(), environment.clone());
    }

    if let Some(dependency) = dependency {
        env.insert("Dependency".to_string(), dependency.clone());
    }

    env
}

/// Execute the build command with dual-path environment injection.
///
/// This function:
/// 1. Calls the Maintain binary in legacy mode with merged environment variables
/// 2. The Maintain binary's Process() function generates the extensive product name
/// 3. The Process() function updates tauri.conf.json with the generated name
/// 4. The actual tauri build command is executed
///
/// # Arguments
///
/// * `profile_name` - The resolved profile name
/// * `config` - Land configuration
/// * `env_vars` - Merged environment variables from all sources
/// * `build_args` - Additional build arguments
///
/// # Returns
///
/// Result indicating success or failure
fn execute_build_command(
    profile_name: &str,
    config: &LandConfig,
    env_vars: &HashMap<String, String>,
    build_args: &[String],
) -> Result<(), String> {
    use std::process::Command as StdCommand;

    // Get the build command from config
    let build_command = get_build_command(profile_name, config);

    // Determine if this is a debug build
    let is_debug = build_command.to_lowercase().contains("--debug");

    // Build the command arguments for the Maintain binary
    // The Maintain binary expects: -- <build_command> [args...]
    let mut maintain_args = vec!["--".to_string()];

    // Add pnpm tauri build command
    maintain_args.push("pnpm".to_string());
    maintain_args.push("tauri".to_string());
    maintain_args.push("build".to_string());

    if is_debug {
        maintain_args.push("--debug".to_string());
    }

    // Add any additional build arguments
    maintain_args.extend(build_args.iter().cloned());

    println!("Executing: {}", maintain_args.join(" "));
    println!("With environment variables:");
    for (key, value) in env_vars.iter().take(10) {
        println!("  {}={}", key, value);
    }
    if env_vars.len() > 10 {
        println!("  ... and {} more", env_vars.len() - 10);
    }

    // Get the path to the Maintain binary
    // Try to find it in the target directory
    let maintain_binary = find_maintain_binary();

    // Execute the Maintain binary with merged environment variables
    // The Maintain binary will:
    // 1. Parse environment variables via clap (Argument struct)
    // 2. Generate the extensive product name in Process()
    // 3. Update tauri.conf.json with productName and identifier
    // 4. Execute the actual build command
    let mut cmd = StdCommand::new(&maintain_binary);
    cmd.args(&maintain_args);

    // Pass ALL resolved environment variables to the Maintain binary
    // This is critical for the dual-path merge to work
    cmd.envs(env_vars.iter());

    // Also set the MERGED_ENV_INDICATOR to show that env was merged
    cmd.env("MAINTAIN_CLI_MERGED", "true");

    cmd.stderr(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit());

    let status = cmd.status()
        .map_err(|e| format!("Failed to execute Maintain binary ({}): {}", maintain_binary, e))?;

    if status.success() {
        println!("\n{}", "Build completed successfully!".green());
        Ok(())
    } else {
        Err(format!("Build failed with exit code: {:?}", status.code()))
    }
}

/// Find the Maintain binary path.
///
/// Tries multiple locations:
/// 1. ./Target/release/Maintain
/// 2. ./Target/debug/Maintain
/// 3. maintain (from PATH)
/// 4. cargo run --bin Maintain (fallback)
fn find_maintain_binary() -> String {
    use std::path::Path;

    // Try release build first
    let release_path = "./Target/release/Maintain";
    if Path::new(release_path).exists() {
        return release_path.to_string();
    }

    // Try debug build
    let debug_path = "./Target/debug/Maintain";
    if Path::new(debug_path).exists() {
        return debug_path.to_string();
    }

    // Fallback to "maintain" in PATH
    "maintain".to_string()
}

/// List all available profiles (avoids Self)
pub fn get_all_profiles(config: &LandConfig) -> Vec<&str> {
    let mut profiles: Vec<&str> = config.profiles.keys().map(|s| s.as_str()).collect();
    profiles.sort();
    profiles
}
