//! # Process - Build Orchestration
//!
//! Coordinates the entire build workflow: product name generation, config file
//! modification, Node.js sidecar staging, and build command execution.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`super::BuildPlistEnvironment`] | Collects LSEnvironment variables for Info.plist |
//! | [`super::Process`] | Main build orchestration entry point |

#[path = "Process.rs"]
pub mod Process;

#[path = "BuildPlistEnvironment.rs"]
pub mod BuildPlistEnvironment;
