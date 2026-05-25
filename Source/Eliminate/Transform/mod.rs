//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/mod.rs
//=============================================================================//
// Module: Transform - AST transformation pipeline
//
// Entry point: `Run(source, options)` parses the Rust source with `syn`,
// applies iterative single-use-variable elimination, and re-formats the result
// with `prettyplease`. Returns `Ok(None)` when the source is already minimal.
//=============================================================================//

pub mod Collect;
pub mod Count;
pub mod Inline;
pub mod Safe;

use super::{Definition, Error};

/// Parse `Source`, run up to [`super::Constant::MaxIterations`] elimination
/// passes, then format and return the result.
///
/// Returns `Ok(None)` when no bindings were eliminated (caller can skip the
/// write-back).
pub fn Run(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let mut Ast:syn::File = syn::parse_str(Source).map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

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

/// Preserve-layout variant: inline single-use bindings with minimal textual
/// rewriting. Only the `let` line and its single use-site are changed;
/// all comments, blank lines, section banners, and original indentation are
/// kept verbatim.
///
/// Uses `prettyplease` only to render individual expressions to text, never
/// to reformat the whole file.
///
/// Returns `Ok(None)` when no bindings were eliminated.
pub fn RunPreserve(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	use std::fmt::Write as _;

	/// Render a `syn::Expr` to canonical text via prettyplease by wrapping it
	/// in a throwaway function body, pretty-printing, then stripping the
	/// wrapper. This avoids any span/proc-macro2 feature flags.
	fn ExprText(E:&syn::Expr) -> Option<String> {
		let Dummy = format!("fn __d() {{ let __v = {}; }}", quote::quote!(#E));

		let Ast:syn::File = syn::parse_str(&Dummy).ok()?;

		let Pretty = prettyplease::unparse(&Ast);

		// Extract the initialiser from `    let __v = <expr>;\n`
		let Start = Pretty.find("let __v = ")? + "let __v = ".len();

		let End = Pretty[Start..].find(';').map(|I| Start + I)?;

		Some(Pretty[Start..End].trim().to_string())
	}

	/// Render a `let <ident> = <init>;` binding to canonical text the same way.
	fn LetText(Ident:&str, E:&syn::Expr) -> Option<String> {
		let Dummy = format!("fn __d() {{ let {} = {}; }}", Ident, quote::quote!(#E));

		let Ast:syn::File = syn::parse_str(&Dummy).ok()?;

		let Pretty = prettyplease::unparse(&Ast);

		let Marker = format!("let {} = ", Ident);

		let Start = Pretty.find(&Marker)?;

		let End = Pretty[Start..].find(';').map(|I| Start + I + 1)?;

		Some(Pretty[Start..End].trim().to_string())
	}

	let mut Working = Source.to_string();

	let mut AnyChanged = false;

	'outer: loop {
		let Ast:syn::File = syn::parse_str(&Working)
			.map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

		// Walk every function body looking for single-use let bindings.
		for Item in &Ast.items {
			let Blocks = CollectBlocks(Item);

			for Block in Blocks {
				let Candidates = super::Transform::Collect::Collect(Block, Options.InlineComments);

				for Candidate in &Candidates {
					if !super::Transform::Safe::IsSafe(&Candidate.Init, Options.MaxSize) {
						continue;
					}

					let (RefCount, InClosure, InLoop) = super::Transform::Count::CountReferences(
						&Candidate.Ident,
						&Block.stmts[Candidate.StmtIndex + 1..],
					);

					if RefCount != 1 || InClosure || InLoop {
						continue;
					}

					let LetStr = match LetText(&Candidate.Ident, &Candidate.Init) {
						Some(S) => S,
						None => continue,
					};

					let InitStr = match ExprText(&Candidate.Init) {
						Some(S) => S,
						None => continue,
					};

					let IdentStr = &Candidate.Ident;

					// Find and remove the let line, then replace the use-site.
					if let Some(LetPos) = Working.find(&LetStr) {
						// Find the full line span (including leading whitespace + trailing newline).
						let LineStart = Working[..LetPos].rfind('\n').map(|I| I + 1).unwrap_or(0);

						let LineEnd = Working[LetPos..]
							.find('\n')
							.map(|I| LetPos + I + 1)
							.unwrap_or(Working.len());

						// Find the use-site of the identifier after the let line.
						let SearchFrom = LineEnd;

						if let Some(RelPos) = find_word(&Working[SearchFrom..], IdentStr) {
							let UsePos = SearchFrom + RelPos;
							let UseEnd = UsePos + IdentStr.len();

							// Replace use-site first (later in file, so offsets of let line unaffected).
							Working.replace_range(UsePos..UseEnd, &InitStr);

							// Now remove the let line.
							Working.replace_range(LineStart..LineEnd, "");

							AnyChanged = true;

							continue 'outer;
						}
					}
				}
			}
		}

		// No more candidates found in this pass.
		break;
	}

	if AnyChanged { Ok(Some(Working)) } else { Ok(None) }
}

// ---------------------------------------------------------------------------
// Helpers for RunPreserve
// ---------------------------------------------------------------------------

/// Word-boundary-aware substring search: returns the byte offset of the first
/// occurrence of `Word` in `Haystack` where the match is not immediately
/// preceded or followed by an alphanumeric character or underscore.
fn find_word(Haystack:&str, Word:&str) -> Option<usize> {
	let Bytes = Haystack.as_bytes();
	let Pat = Word.as_bytes();

	let mut Pos = 0usize;

	while Pos + Pat.len() <= Bytes.len() {
		if Bytes[Pos..].starts_with(Pat) {
			let Before = Pos > 0 && (Bytes[Pos - 1].is_ascii_alphanumeric() || Bytes[Pos - 1] == b'_');

			let After = Bytes
				.get(Pos + Pat.len())
				.map_or(false, |&B| B.is_ascii_alphanumeric() || B == b'_');

			if !Before && !After {
				return Some(Pos);
			}
		}

		Pos += 1;
	}

	None
}

/// Collect all `syn::Block` references reachable from a top-level `Item`.
/// Only descends into function bodies (free functions and impl methods).
fn CollectBlocks(Item:&syn::Item) -> Vec<&syn::Block> {
	let mut Out = Vec::new();

	match Item {
		syn::Item::Fn(F) => Out.push(F.block.as_ref()),

		syn::Item::Impl(Impl) => {
			for ImplItem in &Impl.items {
				if let syn::ImplItem::Fn(M) = ImplItem {
					Out.push(&M.block);
				}
			}
		},

		_ => {},
	}

	Out
}
