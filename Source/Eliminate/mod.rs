//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/mod.rs
//=============================================================================//
// Module: Eliminate - Rust Single-Use Variable Inliner
//
// Brief Description:
//   Analyses Rust source files and inlines `let` bindings that are used exactly
//   once, are non-mutated, and have no closure capture semantics. Equivalent to
//   the TypeScript Eliminate project at ~/Developer/Application/PlayForm/NPM/Eliminate
//   but targeting Rust ASTs via `syn` + `prettyplease`.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
//   - Parse Rust source with syn::parse_file
//   - Iteratively inline single-use let bindings
//   - Re-format output with prettyplease (rustfmt-compatible)
//
// Secondary:
//   - CLI interface for batch file processing
//   - Dry-run preview mode
//   - Verbose statistics reporting
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
//   - Peer module to Build and Run inside Maintain
//   - Invoked via `Maintain eliminate [--path …] [--glob …]`
//
// Dependencies:
//   - syn v2 (full + visit + visit-mut + extra-traits features)
//   - prettyplease v0.2
//   - walkdir (workspace)
//
//=============================================================================//

pub mod CLI;
pub mod Constant;
pub mod Definition;
pub mod Error;
pub mod Fn;
pub mod Logger;
pub mod Process;
pub mod Transform;
