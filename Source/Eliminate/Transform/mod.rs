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
// Public entry point
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
		RunReformat(Source, Options)
	} else {
		RunPreserve(Source, Options)
	}
}

// ---------------------------------------------------------------------------
// Reformat path (opt-in) — original behaviour, unchanged
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
// Preserve path (default) — text-level substitution
// ---------------------------------------------------------------------------
//
// Strategy:
//   Parse the working text into an AST solely to *identify* which let
//   bindings are eligible (Collect) and safe (Safe/Count).  Once a candidate
//   is confirmed for inlining we locate its byte range in the original source
//   using proc_macro2 span information and splice the replacement text in,
//   leaving everything else character-for-character identical.
//
//   We iterate up to MaxIterations so that chains (A→B→C) converge.
//   Each iteration re-parses the *current working text* so span offsets
//   stay accurate after prior edits.
//
// Span byte offsets:
//   proc_macro2 provides `Span::byte_range()` (a `std::ops::Range<usize>`)
//   when the "span-locations" feature is active (it is, because syn enables
//   it via the proc-macro2 dependency).  This gives us the exact half-open
//   byte range [start, end) of any token or node within the *last string
//   passed to proc_macro2::SourceFile* — which, because we call
//   `syn::parse_str`, is our working text.

fn RunPreserve(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let mut Working = Source.to_owned();
	let mut AnyChanged = false;

	'outer: for _ in 0..super::Constant::MaxIterations {
		// Parse the current working text. We keep the Ast immutable after
		// parse because we only need it for candidate discovery; actual edits
		// are applied to `Working` as strings.
		let Ast:syn::File = syn::parse_str(&Working)
			.map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

		// Walk every function/closure body looking for a single inlinable
		// binding. We apply at most one substitution per iteration then
		// restart, so indices stay valid.
		for Item in &Ast.items {
			if let Some(NewWorking) = TryInlineInItem(Item, &Working, Options)? {
				Working = NewWorking;
				AnyChanged = true;
				continue 'outer;
			}
		}

		// Nothing changed in this pass — converged.
		break;
	}

	if AnyChanged { Ok(Some(Working)) } else { Ok(None) }
}

/// Attempt one text-level inline substitution inside `Item`.
/// Returns `Some(new_working_text)` on the first successful substitution.
fn TryInlineInItem(
	Item:&syn::Item,
	Working:&str,
	Options:&Definition::Options,
) -> Error::Result<Option<String>> {
	match Item {
		syn::Item::Fn(F) => TryInlineInBlock(&F.block, Working, Options),

		syn::Item::Impl(I) => {
			for ImplItem in &I.items {
				if let syn::ImplItem::Fn(Method) = ImplItem {
					if let Some(Result) = TryInlineInBlock(&Method.block, Working, Options)? {
						return Ok(Some(Result));
					}
				}
			}

			Ok(None)
		},

		_ => Ok(None),
	}
}

/// Attempt one text-level inline substitution inside `Block`.
fn TryInlineInBlock(
	Block:&syn::Block,
	Working:&str,
	Options:&Definition::Options,
) -> Error::Result<Option<String>> {
	let Candidates = Collect::Collect(Block, Options.InlineComments);

	for Candidate in &Candidates {
		if !Safe::IsSafe(&Candidate.Init, Options.MaxSize) {
			continue;
		}

		let (RefCount, InClosure) =
			Count::CountReferences(&Candidate.Ident, &Block.stmts[Candidate.StmtIndex + 1..]);

		if RefCount != 1 || InClosure {
			continue;
		}

		// We have a confirmed single-use candidate.  Clone the downstream
		// statements into a mutable copy so SubstituteRef can mutate them
		// in memory — we use the resulting AST node only to render the
		// replacement text via prettyplease, not to rewrite the whole file.
		let mut UseStmts:Vec<syn::Stmt> = Block.stmts[Candidate.StmtIndex + 1..].to_vec();

		let Substituted = Inline::SubstituteRef(&mut UseStmts, &Candidate.Ident, &Candidate.Init);

		if !Substituted {
			continue;
		}

		// Obtain the byte range of the `let` statement in Working.
		let LetStmt = &Block.stmts[Candidate.StmtIndex];
		let LetRange = SpanByteRange(LetStmt);

		// Obtain the byte range of the single use statement in Working.
		// After substitution, render the mutated statement back to source.
		let UseStmt = &Block.stmts[Candidate.StmtIndex + 1];
		let UseRange = SpanByteRange(UseStmt);
		let UseReplacement = StmtToSource(&UseStmts[0]);

		// Apply edits in reverse order (use first because it comes later in
		// the file, so the let-statement range is unaffected).
		let mut Out = Working.to_owned();

		if UseRange.start <= Out.len() && UseRange.end <= Out.len() && UseRange.start <= UseRange.end {
			Out.replace_range(UseRange.clone(), &UseReplacement);
		}

		// Remove the let statement line.  After splicing UseReplacement the
		// let-stmt offsets in `Out` may have shifted; recalculate from the
		// delta.
		let Delta:i64 = UseReplacement.len() as i64 - (UseRange.end - UseRange.start) as i64;

		let LetStart = (LetRange.start as i64) as usize;
		let LetEnd = (LetRange.end as i64) as usize;

		if LetStart <= Out.len() && LetEnd <= Out.len() && LetStart <= LetEnd {
			// Expand the removal to include any trailing newline so we don't
			// leave a blank line in place of the let statement.
			let ExpandedEnd = if LetEnd < Out.len() && Out.as_bytes()[LetEnd] == b'\n' {
				LetEnd + 1
			} else {
				LetEnd
			};

			Out.replace_range(LetStart..ExpandedEnd, "");
		}

		// Recurse into nested blocks within the statements we just modified.
		// (Handled by the outer 'outer loop re-parsing Working.)
		return Ok(Some(Out));
	}

	// Recurse into nested blocks (e.g. if/match arms, nested fns).
	for Stmt in &Block.stmts {
		if let Some(NestedBlock) = StmtNestedBlock(Stmt) {
			if let Some(Result) = TryInlineInBlock(NestedBlock, Working, Options)? {
				return Ok(Some(Result));
			}
		}
	}

	Ok(None)
}

// ---------------------------------------------------------------------------
// Span helpers
// ---------------------------------------------------------------------------

/// Return the `[start, end)` byte range of a `syn::Stmt` within the source
/// string that was most recently parsed by proc_macro2.
fn SpanByteRange(Stmt:&syn::Stmt) -> std::ops::Range<usize> {
	use quote::ToTokens;

	let mut Tokens = proc_macro2::TokenStream::new();

	Stmt.to_tokens(&mut Tokens);

	let Spans: Vec<proc_macro2::Span> = Tokens.into_iter().map(|T| T.span()).collect();

	if Spans.is_empty() {
		return 0..0;
	}

	let First = Spans.first().unwrap().byte_range();
	let Last = Spans.last().unwrap().byte_range();

	First.start..Last.end
}

/// Render a single `syn::Stmt` to a source string via prettyplease,
/// by wrapping it in a dummy function body and extracting the inner text.
fn StmtToSource(Stmt:&syn::Stmt) -> String {
	use quote::quote;

	// Wrap in a function so prettyplease can parse the file.
	let Wrapped:syn::File = syn::parse_quote! {
		fn __dummy__() { #Stmt }
	};

	let Full = prettyplease::unparse(&Wrapped);

	// Extract the inner statement text: everything between the first `{\n`
	// and the last `\n}`, then strip one level of leading whitespace.
	if let Some(Start) = Full.find('{') {
		if let Some(End) = Full.rfind('}') {
			let Inner = Full[Start + 1..End].trim_matches('\n');

			// Strip one leading tab or 4 spaces added by the dummy wrapper.
			return Inner
				.lines()
				.map(|L| L.strip_prefix('\t').or_else(|| L.strip_prefix("    ")).unwrap_or(L))
				.collect::<Vec<_>>()
				.join("\n");
		}
	}

	// Fallback: return the pretty-printed full file (should not happen).
	Full
}

/// Extract a directly nested `Block` from a statement, if any, so the
/// outer loop can recurse into it.
fn StmtNestedBlock(Stmt:&syn::Stmt) -> Option<&syn::Block> {
	if let syn::Stmt::Expr(Expr, _) = Stmt {
		match Expr {
			syn::Expr::Block(B) => return Some(&B.block),
			syn::Expr::If(I) => return Some(&I.then_branch),
			syn::Expr::Loop(L) => return Some(&L.body),
			syn::Expr::While(W) => return Some(&W.body),
			syn::Expr::ForLoop(F) => return Some(&F.body),
			syn::Expr::Unsafe(U) => return Some(&U.block),
			_ => {},
		}
	}

	None
}
