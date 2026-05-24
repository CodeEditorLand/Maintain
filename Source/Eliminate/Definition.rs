//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Definition.rs
//=============================================================================//
// Module: Definition - Data structures for the Eliminate module
//=============================================================================//

use super::Constant;

/// Tunable knobs forwarded from the CLI or callers.
#[derive(Debug, Clone)]
pub struct Options {
	/// Maximum AST node count for an inlinable initialiser.
	/// Initialisers larger than this are left untouched.
	pub MaxSize: usize,

	/// When `true`, bindings that carry leading doc-comments or `#[…]`
	/// attributes are still eligible for inlining.
	/// Default `false` - commented bindings are kept as-is.
	pub InlineComments: bool,

	/// When `true`, show what would change without writing any files.
	pub DryRun: bool,

	/// When `true`, emit per-binding log lines.
	pub Verbose: bool,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			MaxSize: Constant::DefaultMaxSize,
			InlineComments: false,
			DryRun: false,
			Verbose: false,
		}
	}
}

/// Aggregate statistics returned by [`super::Process::Process`].
#[derive(Debug, Default)]
pub struct Stats {
	pub FilesProcessed: usize,
	pub FilesModified: usize,
	pub BindingsInlined: usize,
}
