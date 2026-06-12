//! # Run - Development Run Module with Hot-Reload Support
//!
//! Orchestrates the development run process with hot-reload, managing the
//! development server lifecycle and supporting profile-based configurations.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`crate::Run::CLI`] | CLI interface for profile-based runs |
//! | [`crate::Run::Constant`] | Run module constants and configuration |
//! | [`crate::Run::Definition`] | Type definitions and data structures |
//! | [`crate::Run::Environment`] | Environment variable management |
//! | [`crate::Run::Error`] | Error types |
//! | [`crate::Run::Fn`] | Main entry point |
//! | [`crate::Run::Logger`] | Logging utilities |
//! | [`crate::Run::Process`] | Process management |
//! | [`crate::Run::Profile`] | Profile resolution and management |

pub mod CLI;

pub mod Constant;

pub mod Definition;

pub mod Environment;

pub mod Error;

pub mod Fn;

pub mod Logger;

pub mod Process;

pub mod Profile;
