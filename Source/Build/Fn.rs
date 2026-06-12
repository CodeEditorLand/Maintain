//! Main entry point for the build orchestrator.
//!
//! Initializes the logger, parses command-line arguments, and executes the
//! build orchestration process, handling success and error conditions.

use clap::Parser;
use log::{error, info};

use crate::Build::{Definition::Argument, Logger::Logger, Process::Process};

/// The main entry point of the binary.
///
/// This function serves as the primary entry point for the build orchestrator.
/// It performs three main steps:
///
/// 1. **Initialization**: Sets up the global logger with appropriate formatting
/// and log level based on the `RUST_LOG` environment variable.
///
/// 2. **Argument Parsing**: Parses command-line arguments and environment
/// variables using the `Argument` struct, which includes automatic validation
/// and help text generation.
///
/// 3. **Orchestration**: Delegates to the `Process` function to execute the
/// complete build orchestration workflow, including:
/// - Validating project directory and configuration files
/// - Creating file guards for backup and restoration
/// - Generating product names and bundle identifiers
/// - Modifying configuration files for specific build flavors
/// - Optionally bundling Node.js sidecar binaries
/// - Executing the final build command
/// - Cleaning up temporary files
///
/// # Behavior
///
/// On successful build completion:
/// - Logs an informational message: "Build process completed successfully."
/// - Exits with status code 0 (implicit)
///
/// On build failure:
/// - Logs an error message with the error details
/// - Exits with status code 1 via `std::process::exit(1)`
///
/// # Example Invocation
///
/// Basic build with debug flag:
/// ```sh
/// ./build-orchestrator --dir Element/Mountain --debug pnpm tauri build
/// ```
///
/// Build with Node.js sidecar:
/// ```sh
/// export NODE_ENV=production
/// export NODE_VERSION=22
/// ./build-orchestrator --name Mountain --prefix land.editor.binary pnpm tauri build
/// ```
///
/// Build with dependency flavor:
/// ```sh
/// ./build-orchestrator --dependency tauri-apps/tauri --clean true pnpm tauri build
/// ```
///
/// # Error Handling
///
/// The function uses Rust's question mark operator (`?`) to propagate errors
/// from the `Process` function. When an error occurs:
///
/// 1. The error is logged with `error!()` macro
/// 2. The program exits with status code 1
/// 3. File guards automatically restore original configuration files
///
/// This ensures that:
/// - Build failures are clearly reported
/// - The workspace is left in a consistent state
/// - Original configuration files are preserved
///
/// # Logging
///
/// The function requires the logger to be initialized first (via `Logger()`).
/// All subsequent operations use structured logging with targets:
///
/// - `Build` - General build orchestration messages
/// - `Build::Guard` - File backup and restoration
/// - `Build::Toml` - TOML editing operations
/// - `Build::Json` - JavaScriptObjectNotation editing operations
/// - `Build::Exec` - Command execution
///
/// Control log verbosity with `RUST_LOG`:
/// ```sh
/// export RUST_LOG=debug # More verbose output
/// export RUST_LOG=error # Only errors
/// ```
///
/// # Thread Safety
///
/// This function is not designed to be called concurrently. It should be
/// called exactly once at program startup. The underlying `Process` function
/// and logging infrastructure handle their own synchronization requirements.
///
/// # Implementation Notes
///
/// The function name `Fn` is intentionally short to serve as a concise
/// entry point. In Rust, `Fn` is not a reserved word when used as a
/// function name (unlike traits like `Fn`, `FnMut`, and `FnOnce`).
///
/// The function does not return a value because it either completes
/// successfully (and the program exits) or exits with an error status.
/// This pattern is common for Rust binaries that serve as command-line
/// tools or build scripts.
pub fn Fn() {
	// Step 1: Initialize the logger with colored output
	Logger();

	// Step 2: Parse command-line arguments and environment variables
	let Argument = Argument::parse();

	log::debug!("Parsed arguments: {:?}", Argument);

	// Step 3: Execute the build orchestration process
	match Process(&Argument) {
		Ok(_) => info!("Build process completed successfully."),

		Err(e) => {
			error!("Build process failed: {}", e);

			std::process::exit(1);
		},
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	// Note: Actual tests require integration test setup
	// because Fn() initializes the logger and processes arguments.
	// Unit tests should focus on the underlying Process function
	// instead.

	#[test]
	fn test_fn_exists() {
		// Verify the function compiles and is callable
		// (actual execution would be integration tests)
		let _ = Fn as fn();
	}
}
