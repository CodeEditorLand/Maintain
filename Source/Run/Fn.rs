//=============================================================================//
// File Path: Element/Maintain/Source/Run/Fn.rs
//=============================================================================//
// Module: Fn
//
// Brief Description: The main entry point for the run orchestrator.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Initialize the logger
// - Parse command-line arguments
// - Execute the run orchestration process
// - Handle success and error conditions appropriately
//
// Secondary:
// - Provide a clean interface for calling the run system
// - Exit with appropriate status codes
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Interface/Entry point layer
// - Main public API
//
// Dependencies (What this module requires):
// - External crates: log
// - Internal modules: Definition::Argument, Process, Logger
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Application entry point (main function)
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Entry point pattern
// - Early exit on error pattern
//
// Performance Considerations:
// - Complexity: O(1) - delegates to Process function
// - Memory usage patterns: Minimal overhead
// - Hot path optimizations: None
//
// Thread Safety:
// - Thread-safe: No (called only once at program start)
// - Synchronization mechanisms used: None
//
// Error Handling:
// - Error types returned: None (logs and exits)
// - Recovery strategies: Exit with status code 1 on error
//
// EXAMPLES:
// =========
//
// Example 1: Direct invocation
/// ```rust
/// use crate::Maintain::Source::Run::Fn;
/// Fn();
/// ```
//
// Example 2: Usage in main function
/// ```rust
/// fn main() {
///     crate::Maintain::Source::Run::Fn();
/// }
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

use crate::Run::Definition::Argument;
use crate::Run::Process;
use crate::Run::Logger;

use clap::Parser;
use log::{error, info};

/// The main entry point of the run binary.
///
/// This function serves as the primary entry point for the run orchestrator.
/// It performs three main steps:
///
/// 1. **Initialization**: Sets up the global logger with appropriate formatting
///    and log level based on the `RUST_LOG` environment variable.
///
/// 2. **Argument Parsing**: Parses command-line arguments and environment
///    variables using the `Argument` struct, which includes automatic validation
///    and help text generation.
///
/// 3. **Orchestration**: Delegates to the `Process` function to execute the
///    complete run orchestration workflow, including:
///    - Validating project directory and configuration files
///    - Resolving environment variables from profiles
///    - Starting the development server with hot-reload
///    - Managing watch mode for file changes
///    - Executing the run command
///    - Handling graceful shutdown
///
/// # Behavior
///
/// On successful run completion:
/// - Logs an informational message: "Run process completed successfully."
/// - Exits with status code 0 (implicit)
///
/// On run failure:
/// - Logs an error message with the error details
/// - Exits with status code 1 via `std::process::exit(1)`
///
/// # Example Invocation
///
/// Basic run with debug profile:
/// ```sh
/// cargo run --bin Maintain -- --run --profile debug
/// ```
///
/// Run with hot-reload disabled:
/// ```sh
/// cargo run --bin Maintain -- --run --profile debug --no-hot-reload
/// ```
///
/// Run with custom workbench:
/// ```sh
/// cargo run --bin Maintain -- --run --profile debug --workbench Wind
/// ```
///
/// # Error Handling
///
/// The function uses Rust's question mark operator (`?`) to propagate errors
/// from the `Process` function. When an error occurs:
///
/// 1. The error is logged with `error!()` macro
/// 2. The program exits with status code 1
///
/// This ensures that:
/// - Run failures are clearly reported
/// - The workspace is left in a consistent state
///
/// # Logging
///
/// The function requires the logger to be initialized first (via `Logger()`).
/// All subsequent operations use structured logging with targets:
///
/// - `Run` - General run orchestration messages
/// - `Run::Process` - Process management messages
/// - `Run::Environment` - Environment variable resolution
/// - `Run::HotReload` - Hot-reload related messages
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
/// entry point, following the naming convention of the Build module.
///
/// The function does not return a value because it either completes
/// successfully (and the program exits) or exits with an error status.
/// This pattern is common for Rust binaries that serve as command-line
/// tools or development scripts.
pub fn Fn() {
    // Step 1: Initialize the logger with colored output
    Logger::Logger();

    // Step 2: Parse command-line arguments and environment variables
    let argument = Argument::parse();

    log::debug!("Parsed arguments: {:?}", argument);

    // Step 3: Execute the run orchestration process
    match Process::Process(&argument) {
        Ok(_) => info!("Run process completed successfully."),

        Err(e) => {
            error!("Run process failed: {}", e);

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
