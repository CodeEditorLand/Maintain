/*=============================================================================*/
/* File Path: Element/Maintain/Source/Library.rs                                             */
/*=============================================================================*/
/* Module: Library                                                            */
/*                                                                            */
/* Brief Description: Entry point for the Build Orchestrator binary.          */
/*                                                                            */
/* RESPONSIBILITIES:                                                          */
/* ================                                                          */
/*                                                                            */
/* Primary:                                                                   */
/* - Serve as the entry point for the build orchestrator                       */
/* - Initialize the Build module for the binary                                */
/*                                                                            */
/* Secondary:                                                                 */
/* - None                                                                     */
/*                                                                            */
/* ARCHITECTURAL ROLE:                                                        */
/* ===================                                                        */
/*                                                                            */
/* Position:                                                                  */
/* - Entry point layer                                                         */
/* - Binary initialization                                                     */
/*                                                                            */
/* Dependencies (What this module requires):                                  */
/* - External crates: None                                                    */
/* - Internal modules: Build                                                  */
/* - Traits implemented: None                                                 */
/*                                                                            */
/* Dependents (What depends on this module):                                  */
/* - Cargo.toml (binary definition)                                           */
/* - Build system entry point                                                 */
/*                                                                            */
/*=============================================================================*/
/* IMPLEMENTATION                                                             */
/*=============================================================================*/

// Disable Windows console for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// Allow PascalCase naming for function names
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

/*=============================================================================*/
/* MAIN ENTRY POINT                                                            */
/*=============================================================================*/

/*=============================================================================*/
/* MAIN ENTRY POINT                                                            */
/*=============================================================================*/

/// The primary entry point for the Build Orchestrator binary.
///
/// This function serves as the bridge between the Cargo binary definition
/// and the Build module's orchestration logic. It supports two modes:
///
/// ## Mode 1: CLI Mode (cargo-first builds)
///
/// When called with CLI arguments, uses the new configuration-based build system:
/// ```bash
/// cargo run --package maintain -- --profile debug-mountain
/// cargo run --package maintain -- --list-profiles
/// ```
///
/// ## Mode 2: Legacy Mode (environment variable based)
///
/// When called with a `--` separator followed by a build command, uses the
/// traditional environment variable-based build system:
/// ```bash
/// ./Target/release/Maintain -- pnpm tauri build --debug
/// ```
///
/// The function is marked as `#[allow(dead_code)]` because when this file
/// is used as a library module, the main function may not be called directly.
/// However, when compiled as a binary, this main function is the entry point.
#[allow(dead_code)]
fn main() {
	use clap::Parser;
	use std::env;

	// Collect all arguments
	let args: Vec<String> = env::args().collect();

	// Determine the mode based on arguments:
	// - CLI mode: Direct flags like --list-profiles, --profile, --show-profile
	// - Legacy mode: -- followed by a build command (like pnpm, cargo, npm)
	// - No args: Show help

	if args.len() == 1 {
		// No arguments - show help
		let _ = Build::Cli::try_parse();
		return;
	}

	// Check if we're in CLI mode (first arg after binary is a flag starting with --)
	// or if we need legacy mode (-- followed by a command)
	let first_arg = args.get(1).map(|s| s.as_str()).unwrap_or("");

	// CLI flags that indicate we should use the new CLI mode
	let cli_flags = [
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
		"build",
	];

	// Check if first arg is a CLI flag or subcommand
	let is_cli_mode = cli_flags.iter().any(|flag| {
		first_arg == *flag || first_arg.starts_with(&format!("{}=", flag))
	}) || first_arg.starts_with("--");

	if is_cli_mode {
		// Use new CLI mode (configuration based)
		match Build::Cli::try_parse() {
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
		Build::Fn();
	}
}

/*=============================================================================*/
/* MODULE DECLARATIONS                                                        */
/*=============================================================================*/

/// Build Orchestrator Module.
///
/// This module contains all the build orchestration logic, including:
///
/// - **Constant**: File paths, delimiters, and environment variable names
/// - **Definition**: Data structures for arguments, manifests, and file guards
/// - **Error**: Comprehensive error types for build operations
/// - **Function**: Build orchestration functions and utilities
///
/// See the Build module documentation for detailed information about the
/// build system's capabilities and usage.
pub mod Build;
