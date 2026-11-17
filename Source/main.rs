//! Maintain binary entry point
//!
//! This is the sole entry point for the Maintain orchestrator. It owns CLI
//! dispatch (run / build / eliminate / legacy), so the library target holds
//! only the `Build`, `Run`, `Eliminate` and `Architecture` modules and cannot
//! drift from the binary's argument handling.

use clap::Parser;

use Maintain::{Build, Eliminate, Run};

/// The primary dispatch routine for the Maintain Orchestrator binary.
///
/// Supports three modes:
///
/// ## Mode 1: Run Mode (`--run` flag)
///
/// ```bash
/// cargo run --bin Maintain -- --run --profile debug-mountain
/// cargo run --bin Maintain -- --run --list-profiles
/// ```
///
/// ## Mode 2: Build Mode (default or `build` subcommand)
///
/// ```bash
/// cargo run --bin Maintain -- --profile debug-mountain
/// cargo run --bin Maintain -- --list-profiles
/// ```
///
/// ## Mode 3: Legacy Mode (environment variable based)
///
/// ```bash
/// ./Target/release/Maintain -- pnpm tauri build --debug
/// ./Target/release/Maintain -- pnpm tauri dev
/// ```
fn dispatch() {
	use std::env;

	// Collect all arguments
	let mut args:Vec<String> = env::args().collect();

	// Determine the mode based on arguments:
	// - Run mode: --run/-r flag, --dev flag, or 'run' subcommand
	// - Build mode: Direct flags like --list-profiles, --profile,
	//   --show-profile, or 'build' subcommand
	// - Legacy mode: -- followed by a build command (like pnpm, cargo, npm)
	// - No args: Show help

	if args.len() == 1 {
		// No arguments - show build help (default)
		let _ = Build::CLI::Cli::Cli::try_parse();

		return;
	}

	// Check if first arg after binary is a run-specific flag or subcommand
	let first_arg = args.get(1).map(|s| s.as_str()).unwrap_or("");

	// Check if we're in legacy mode (-- followed by a command)
	let is_legacy_mode = first_arg == "--";

	// Check for eliminate mode
	let is_eliminate_mode = first_arg == "eliminate" || first_arg == "--eliminate";

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
	let is_build_cli_mode = if !is_run_mode && !is_legacy_mode && !is_build_subcommand && !is_eliminate_mode {
		build_cli_flags
			.iter()
			.any(|flag| first_arg == *flag || first_arg.starts_with(&format!("{}=", flag)))
			|| (!first_arg.starts_with('-') && !is_build_subcommand)
	} else {
		false
	};

	if is_run_mode {
		// Strip the --run/--dev/-r flag or 'run' subcommand before passing to
		// Run CLI This allows Run::CLI to parse the remaining arguments
		// correctly
		if is_run_flag {
			// Remove the flag (and its position) from args
			args.remove(1);
		} else if is_run_subcommand {
			// Replace 'run' subcommand with arguments that Run CLI expects
			args.remove(1);
		}

		// Use Run mode (development workflow)
		// Use try_parse_from with our modified args, not try_parse() which
		// reads from env::args()
		match Run::CLI::Cli::Cli::try_parse_from(args) {
			Ok(cli) => {
				if let Err(e) = cli.execute() {
					eprintln!("Error: {}", e);

					std::process::exit(1);
				}
			},

			Err(e) => {
				// If parsing fails, it might be a --help or --version request
				// or invalid arguments - let clap handle it
				e.print().expect("Failed to print error");

				std::process::exit(e.exit_code());
			},
		}
	} else if is_build_subcommand {
		// Handle 'build' subcommand - strip it and pass to Build CLI
		args.remove(1);

		// Use try_parse_from with our modified args
		match Build::CLI::Cli::Cli::try_parse_from(args) {
			Ok(cli) => {
				if let Err(e) = cli.execute() {
					eprintln!("Error: {}", e);

					std::process::exit(1);
				}
			},

			Err(e) => {
				e.print().expect("Failed to print error");

				std::process::exit(e.exit_code());
			},
		}
	} else if is_build_cli_mode {
		// Use Build CLI mode (configuration based)
		match Build::CLI::Cli::Cli::try_parse() {
			Ok(cli) => {
				if let Err(e) = cli.execute() {
					eprintln!("Error: {}", e);

					std::process::exit(1);
				}
			},

			Err(e) => {
				// If parsing fails, it might be a --help or --version request
				// or invalid arguments - let clap handle it
				e.print().expect("Failed to print error");

				std::process::exit(e.exit_code());
			},
		}
	} else if is_eliminate_mode {
		// Inline single-use Rust let-bindings across a file or directory tree.
		args.remove(1);

		match Eliminate::CLI::Cli::try_parse_from(args) {
			Ok(Cli) => {
				if let Err(E) = Cli.execute() {
					eprintln!("Error: {}", E);

					std::process::exit(1);
				}
			},

			Err(E) => {
				E.print().expect("Failed to print error");

				std::process::exit(E.exit_code());
			},
		}
	} else {
		// Use legacy build mode (environment variable based)
		// This handles: ./Maintain -- pnpm tauri build
		Build::Fn::Fn();
	}
}

fn main() {
	dispatch();
}
