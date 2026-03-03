//=============================================================================//
// File Path: Element/Maintain/Source/Run/mod.rs
//=============================================================================//
// Module: Run
//
// Brief Description: Development Run Module with Hot-Reload Support.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Orchestrate the development run process with hot-reload
// - Manage development server lifecycle
// - Support profile-based development configurations
// - Enable rapid iteration during development
//
// Secondary:
// - Provide comprehensive logging for run operations
// - Handle various workbench types (Browser, Wind, Mountain, Electron)
// - Support environment-specific run configurations
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Development orchestration layer
// - Development-time tooling
//
// Dependencies (What this module requires):
// - External crates: clap, colored, env_logger, json5, log, serde, serde_json
// - Internal modules: None
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Element/Maintain/Source/Library.rs
// - Maintain/Debug/Run.sh
// - Development workflow scripts
//
// IMPLEMENTATION DETAILS:
// =======================
//
// Design Patterns:
// - Configuration object pattern (Argument struct)
// - Error handling with thiserror
// - Builder pattern (via clap)
//
// Performance Considerations:
// - Complexity: O(n) - file I/O and process management
// - Memory usage patterns: Moderate (stores configuration in memory)
// - Hot path optimizations: None needed (development time)
//
// Thread Safety:
// - Thread-safe: No (not designed for concurrent execution)
// - Synchronization mechanisms used: None
//
// Error Handling:
// - Error types returned: Error (comprehensive error enum)
// - Recovery strategies: Process cleanup on error
//
// EXAMPLES:
// =========
//
// Example 1: Basic run orchestration
/// ```rust
/// use crate::Maintain::Source::Run::Fn;
/// Fn();
/// ```
//
// Example 2: Using the run orchestrator from command line
/// ```sh
/// # Debug run with hot-reload
/// cargo run --bin Run -- --profile debug-mountain
///
/// # Run with specific workbench
/// ./Maintain/Debug/Run.sh --profile debug-mountain
/// ```
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

// Module declarations - flattened structure
pub mod CLI;
pub mod Constant;
pub mod Definition;
pub mod Environment;
pub mod Error;
pub mod Fn;
pub mod Logger;
pub mod Process;
pub mod Profile;
