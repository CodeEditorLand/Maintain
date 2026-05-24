//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Constant.rs
//=============================================================================//
// Module: Constant - Named literals for the Eliminate module
//=============================================================================//

/// Glob pattern used when no explicit pattern is supplied.
pub const DefaultGlob: &str = "**/*.rs";

/// Maximum number of iterative elimination passes per file.
pub const MaxIterations: usize = 100;

/// Default maximum AST node count for an inlinable initialiser expression.
/// Initialisers whose node count exceeds this value are left as-is.
pub const DefaultMaxSize: usize = 100;
