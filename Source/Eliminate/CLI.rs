//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/CLI.rs
//=============================================================================//
// Module: CLI - Command-line interface for the Eliminate module
//
// Usage:
//   cargo run --bin Maintain -- eliminate --path ./Source --glob "**/*.rs"
//   cargo run --bin Maintain -- eliminate --path ./Source/Foo.rs --dry-run
//=============================================================================//

use std::path::PathBuf;

use clap::Parser;

use super::{Constant, Definition, Error, Process};

/// Inline single-use Rust `let` bindings across one file or a directory tree.
#[derive(Parser, Debug, Clone)]
#[clap(
	name = "eliminate",
	about = "Inline single-use Rust let-bindings (non-mutated, non-closure-captured)"
)]
pub struct Cli {
	/// File or directory to process.
	#[clap(long, short = 'p', default_value = ".")]
	pub Path:PathBuf,

	/// Glob pattern relative to Path for selecting files.
	#[clap(long, short = 'g', default_value = Constant::DefaultGlob)]
	pub Glob:String,

	/// Preview changes without writing any files.
	#[clap(long, short = 'd')]
	pub DryRun:bool,

	/// Maximum AST node count for an inlinable initialiser.
	/// Expressions with more nodes are left as-is.
	#[clap(long, default_value_t = Constant::DefaultMaxSize)]
	pub MaxSize:usize,

	/// Also inline bindings that carry leading doc-comments or attributes.
	/// Default: skip commented bindings.
	#[clap(long)]
	pub InlineComments:bool,

	/// Print a line for every binding that is inlined.
	#[clap(long, short = 'v')]
	pub Verbose:bool,
}

impl Cli {
	pub fn execute(&self) -> Error::Result<()> {
		let Options = Definition::Options {
			MaxSize:self.MaxSize,
			InlineComments:self.InlineComments,
			DryRun:self.DryRun,
			Verbose:self.Verbose,
		};

		let Stats = Process::Process(&self.Path, &self.Glob, &Options)?;

		if self.Verbose || self.DryRun {
			println!(
				"Eliminate: {} file(s) scanned, {} modified, {} binding(s) inlined{}",
				Stats.FilesProcessed,
				Stats.FilesModified,
				Stats.BindingsInlined,
				if self.DryRun { " [dry-run]" } else { "" }
			);
		}

		Ok(())
	}
}
