//! # Eliminate - Rust Single-Use Variable Inliner
//!
//! Analyses Rust source files and inlines `let` bindings that are used exactly
//! once, are non-mutated, and have no closure capture semantics. Equivalent to
//! the TypeScript Eliminate project but targeting Rust ASTs via `syn` +
//! `prettyplease`.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`crate::Eliminate::CLI`] | CLI interface for batch file processing |
//! | [`crate::Eliminate::Constant`] | Named literals and defaults |
//! | [`crate::Eliminate::Definition`] | Data structures and options |
//! | [`crate::Eliminate::Error`] | Error types |
//! | [`crate::Eliminate::Fn`] | Top-level entry point |
//! | [`crate::Eliminate::Logger`] | Colored logging initialization |
//! | [`crate::Eliminate::Process`] | File discovery and orchestration |
//! | [`crate::Eliminate::Transform`] | AST transformation pipeline |

pub mod CLI;

pub mod Constant;

pub mod Definition;

pub mod Error;

pub mod Fn;

pub mod Logger;

pub mod Process;

pub mod Transform;
