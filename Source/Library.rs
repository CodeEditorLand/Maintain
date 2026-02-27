/*=============================================================================*/
/* File Path: Element/Maintain/Source/Library.rs */
/*=============================================================================*/
/* Module: Library */
/* */
/* Brief Description: Entry point for the Maintain binary supporting both */
/*                    Build and Run workflows. */
/* */
/* RESPONSIBILITIES: */
/* ================ */
/* */
/* Primary: */
/* - Serve as the entry point for the Maintain orchestrator */
/* - Initialize Build and Run modules based on CLI arguments */
/* - Route commands to appropriate workflow handlers */
/* */
/* Secondary: */
/* - Provide unified CLI interface for build and run operations */
/* - Support legacy mode for backward compatibility */
/* */
/* ARCHITECTURAL ROLE: */
/* =================== */
/* */
/* Position: */
/* - Entry point layer */
/* - Binary initialization and routing */
/* */
/* Dependencies (What this module requires): */
/* - External crates: clap, std */
/* - Internal modules: Build, Run */
/* - Traits implemented: None */
/* */
/* Dependents (What depends on this module): */
/* - Cargo.toml (binary definition) */
/* - Build system entry point */
/* - Development run entry point */
/* */
/*=============================================================================*/
/* IMPLEMENTATION */
/*=============================================================================*/

// Disable Windows console for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Allow PascalCase naming for function names
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

/*=============================================================================*/
/* MAIN ENTRY POINT */
/*=============================================================================*/

/// The primary entry point for the Maintain Orchestrator binary.
///
/// This function serves as the bridge between the Cargo binary definition
/// and the Build/Run modules' orchestration logic. It supports three modes:
///
/// ## Mode 1: Run Mode (--run flag)
///
/// When called with the `--run` flag, uses the development run workflow:
/// ```bash
/// cargo run --bin Maintain -- --run --profile debug-mountain
/// cargo run --bin Maintain -- --run --list-profiles
/// ```
///
/// ## Mode 2: Build Mode (default or --build flag)
///
/// When called with CLI arguments (without --run), uses the build workflow:
/// ```bash
/// cargo run --bin Maintain -- --profile debug-mountain
/// cargo run --bin Maintain -- --list-profiles
/// ```
///
/// ## Mode 3: Legacy Mode (environment variable based)
///
/// When called with a `--` separator followed by a build command, uses the
/// traditional environment variable-based build system:
/// ```bash
/// ./Target/release/Maintain -- pnpm tauri build --debug
/// ./Target/release/Maintain -- pnpm tauri dev
/// ```
///
/// The function is marked as `#[allow(dead_code)]` because when this file
/// is used as a library module, the main function may not be called directly.
/// However, when compiled as a binary, this main function is the entry point.
/// TODO: Move this function to main.rs in a future refactor
#[allow(dead_code)]
pub fn main() {
    use clap::Parser;
    use std::env;

    // Collect all arguments
    let mut args: Vec<String> = env::args().collect();

    // Determine the mode based on arguments:
    // - Run mode: --run/-r flag, --dev flag, or 'run' subcommand
    // - Build mode: Direct flags like --list-profiles, --profile, --show-profile, or 'build' subcommand
    // - Legacy mode: -- followed by a build command (like pnpm, cargo, npm)
    // - No args: Show help

    if args.len() == 1 {
        // No arguments - show build help (default)
        let _ = Build::CLI::Cli::try_parse();
        return;
    }

    // Check if first arg after binary is a run-specific flag or subcommand
    let first_arg = args.get(1).map(|s| s.as_str()).unwrap_or("");

    // Check if we're in legacy mode (-- followed by a command)
    let is_legacy_mode = first_arg == "--";

    // Check for run mode indicators
    let is_run_flag = first_arg == "--run" || first_arg == "--dev" || first_arg == "-r";
    let is_run_subcommand = first_arg == "run";
    let is_run_mode = is_run_flag || is_run_subcommand;

    // Check for build mode indicators (subcommand or flags)
    let is_build_subcommand = first_arg == "build";

    // CLI flags that indicate we should use the build CLI mode
    let build_cli_flags = [
        "--list-profiles",
        "--show-profile",
        "--validate-profile",
        "--profile",
        "--dry-run",
        "--help",
        "-h",
        "--version",
        "-V",
        "list-profiles",
        "show-profile",
        "validate-profile",
        "resolve",
    ];

    // Check if first arg is a build CLI flag
    let is_build_cli_mode = if !is_run_mode && !is_legacy_mode && !is_build_subcommand {
        build_cli_flags.iter().any(|flag| {
            first_arg == *flag || first_arg.starts_with(&format!("{}=", flag))
        }) || (!first_arg.starts_with('-') && !is_build_subcommand)
    } else {
        false
    };

    if is_run_mode {
        // Strip the --run/--dev/-r flag or 'run' subcommand before passing to Run CLI
        // This allows Run::CLI to parse the remaining arguments correctly
        if is_run_flag {
            // Remove the flag (and its position) from args
            args.remove(1);
        } else if is_run_subcommand {
            // Replace 'run' subcommand with arguments that Run CLI expects
            args.remove(1);
        }

        // Use Run mode (development workflow)
        // Use try_parse_from with our modified args, not try_parse() which reads from env::args()
        match Run::CLI::Cli::try_parse_from(args) {
            Ok(cli) => {
                if let Err(e) = cli.execute() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                // If parsing fails, it might be a --help or --version request
                // or invalid arguments - let clap handle it
                e.print().expect("Failed to print error");
                std::process::exit(e.exit_code());
            }
        }
    } else if is_build_subcommand {
        // Handle 'build' subcommand - strip it and pass to Build CLI
        args.remove(1);

        // Use try_parse_from with our modified args
        match Build::CLI::Cli::try_parse_from(args) {
            Ok(cli) => {
                if let Err(e) = cli.execute() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                e.print().expect("Failed to print error");
                std::process::exit(e.exit_code());
            }
        }
    } else if is_build_cli_mode {
        // Use Build CLI mode (configuration based)
        match Build::CLI::Cli::try_parse() {
            Ok(cli) => {
                if let Err(e) = cli.execute() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                // If parsing fails, it might be a --help or --version request
                // or invalid arguments - let clap handle it
                e.print().expect("Failed to print error");
                std::process::exit(e.exit_code());
            }
        }
    } else {
        // Use legacy build mode (environment variable based)
        // This handles: ./Maintain -- pnpm tauri build
        Build::Fn::Fn();
    }
}

/*=============================================================================*/
/* MODULE DECLARATIONS */
/*=============================================================================*/

/// Build Orchestrator Module.
///
/// This module contains all the build orchestration logic, including:
///
/// - **CLI**: Command-line interface for configuration-based builds
/// - **Constant**: File paths, delimiters, and environment variable names
/// - **Definition**: Data structures for arguments, manifests, and file guards
/// - **Error**: Comprehensive error types for build operations
/// - **Fn**: Main build orchestration function
/// - **GetTauriTargetTriple**: Target triple detection
/// - **JsonEdit**: JSON configuration editing
/// - **Logger**: Logging utilities
/// - **Pascalize**: PascalCase conversion utilities
/// - **Process**: Process management
/// - **Rhai**: Rhai scripting support
/// - **TomlEdit**: TOML configuration editing
/// - **WordsFromPascal**: Extract words from PascalCase strings
///
/// See the Build module documentation for detailed information about the
/// build system's capabilities and usage.
pub mod Build;

/// Development Run Module.
///
/// This module contains all the development run orchestration logic, including:
///
/// - **CLI**: Command-line interface for configuration-based runs
/// - **Constant**: File paths, delimiters, and environment variable names
/// - **Definition**: Data structures for arguments and run configuration
/// - **Environment**: Environment variable resolution and management
/// - **Error**: Comprehensive error types for run operations
/// - **Fn**: Main run orchestration function
/// - **Logger**: Logging utilities
/// - **Process**: Process management for development servers
/// - **Profile**: Profile resolution and management
///
/// See the Run module documentation for detailed information about the
/// development run system's capabilities and usage.
pub mod Run;
