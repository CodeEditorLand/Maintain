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
	pub MaxSize:usize,

	/// When `true`, bindings that carry leading doc-comments or `#[…]`
	/// attributes are still eligible for inlining.
	/// Default `false` - commented bindings are kept as-is.
	pub InlineComments:bool,

	/// When `true`, show what would change without writing any files.
	pub DryRun:bool,

	/// When `true`, emit per-binding log lines.
	pub Verbose:bool,

	/// When `true`, reformat the entire file with `prettyplease` after
	/// inlining (the old default behaviour).
	///
	/// Default `false` - only the inlined binding sites are rewritten;
	/// all comments, blank lines, section banners, and the original
	/// indentation style are preserved verbatim.
	pub Reformat:bool,
}

impl Default for Options {
	fn default() -> Self {
		Self {
			MaxSize:Constant::DefaultMaxSize,
			InlineComments:false,
			DryRun:false,
			Verbose:false,
			Reformat:false,
		}
	}
}

/// Aggregate statistics returned by [`super::Process::Process`].
#[derive(Debug, Default)]
pub struct Stats {
	pub FilesProcessed:usize,
	pub FilesModified:usize,
	pub BindingsInlined:usize,
}
