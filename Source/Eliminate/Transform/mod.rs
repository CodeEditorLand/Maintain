//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/mod.rs
//=============================================================================//
// Module: Transform - AST transformation pipeline
//
// Entry point: `Run(source, options)` parses the Rust source with `syn`,
// identifies single-use-variable bindings, then applies the substitutions
// either as targeted text edits on the original source (default, preserving
// all comments, blank lines, and indentation) or as a full `prettyplease`
// reformat (opt-in via `Options.Reformat = true`).
//
// Returns `Ok(None)` when no bindings were eliminated (caller can skip the
// write-back).
//=============================================================================//

pub mod Collect;
pub mod Count;
pub mod Inline;
pub mod Safe;

use super::{Definition, Error};

// ---------------------------------------------------------------------------
// Text-edit helpers (used when Options.Reformat == false)
// ---------------------------------------------------------------------------

/// A single text substitution: replace the byte range `Start..End` in the
/// original source string with `Text`.
#[derive(Debug, Clone)]
pub struct TextEdit {
	/// Byte offset of the first character to replace (inclusive).
	pub Start:usize,
	/// Byte offset one past the last character to replace (exclusive).
	pub End:usize,
	/// Replacement text (does **not** have to be the same length).
	pub Text:String,
}

/// Apply a list of [`TextEdit`]s to `Source`.
///
/// Edits **must** be sorted in ascending `Start` order and must not overlap.
/// They are applied in *reverse* order so that earlier byte offsets stay valid
/// as later regions are replaced.
pub fn ApplyEdits(Source:&str, mut Edits:Vec<TextEdit>) -> String {
	Edits.sort_by_key(|E| E.Start);

	let mut Result = Source.to_owned();

	for Edit in Edits.iter().rev() {
		Result.replace_range(Edit.Start..Edit.End, &Edit.Text);
	}

	Result
}

// ---------------------------------------------------------------------------
// Main entry point
// ---------------------------------------------------------------------------

/// Parse `Source`, run up to [`super::Constant::MaxIterations`] elimination
/// passes, then return the result.
///
/// When `Options.Reformat` is `false` (the default) the output preserves every
/// comment, blank line, section banner, and the original indentation style —
/// only the specific `let` binding lines that were inlined are rewritten.
///
/// When `Options.Reformat` is `true` the entire file is reformatted with
/// `prettyplease` after inlining (the previous unconditional behaviour).
///
/// Returns `Ok(None)` when no bindings were eliminated.
pub fn Run(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	if Options.Reformat {
		return RunReformat(Source, Options);
	}

	RunPreserve(Source, Options)
}

// ---------------------------------------------------------------------------
// Reformat path (old behaviour, opt-in)
// ---------------------------------------------------------------------------

fn RunReformat(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let mut Ast:syn::File =
		syn::parse_str(Source).map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

	let mut AnyChanged = false;

	for _ in 0..super::Constant::MaxIterations {
		let mut Eliminator = Inline::Eliminator::new(Options);

		syn::visit_mut::visit_file_mut(&mut Eliminator, &mut Ast);

		if Eliminator.Changed {
			AnyChanged = true;
		} else {
			break;
		}
	}

	if !AnyChanged {
		return Ok(None);
	}

	Ok(Some(prettyplease::unparse(&Ast)))
}

// ---------------------------------------------------------------------------
// Preserve path (default): text-level substitution
// ---------------------------------------------------------------------------

/// Run elimination and apply substitutions as targeted byte-range edits on
/// `Source`, leaving everything outside the inlined binding sites unchanged.
fn RunPreserve(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	// We run up to MaxIterations.  Each iteration:
	//   1. Parse the *current* working text (starts as `Source`, then becomes
	//      the result of the previous iteration's edits).
	//   2. Collect text edits from the AST visitor.
	//   3. Apply them to the working text.
	//
	// This lets multi-step chains (A→B→C) converge correctly while still only
	// touching the exact character positions that changed.

	let mut Working = Source.to_owned();
	let mut AnyChanged = false;

	for _ in 0..super::Constant::MaxIterations {
		let Ast:syn::File = syn::parse_str(&Working)
			.map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

		let mut Eliminator = Inline::Eliminator::new_text_mode(Options);

		syn::visit_mut::visit_file_mut(&mut Eliminator, &mut {
			// visit_file_mut requires &mut; clone the Ast so the visitor can
			// explore freely without us keeping the mutated (and now
			// comment-free) tree.
			let mut A = Ast.clone();
			A
		});

		if Eliminator.Edits.is_empty() {
			break;
		}

		AnyChanged = true;
		Working = ApplyEdits(&Working, Eliminator.Edits);
	}

	if !AnyChanged {
		return Ok(None);
	}

	Ok(Some(Working))
}
