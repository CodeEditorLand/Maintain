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
//   Parse the working text into an AST solely to *identify* which let bindings
//   are eligible (Collect) and safe (Safe/Count). Once a candidate is confirmed
//   for inlining we locate its line range in the current working text using
//   proc_macro2 Span::start()/end() line/column information and splice the
//   replacement in, leaving every other byte unchanged.
//
//   We iterate up to MaxIterations so chains (A->B->C) converge. Each
//   iteration re-parses the current working text so span positions stay valid.

fn RunPreserve(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let mut Working = Source.to_owned();
	let mut AnyChanged = false;

	'outer: for _ in 0..super::Constant::MaxIterations {
		let Ast:syn::File = syn::parse_str(&Working)
			.map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

		// Build the line-start table once per parse iteration, not per stmt.
		let LineStarts = BuildLineStartTable(&Working);

		for Item in &Ast.items {
			if let Some(NewWorking) = TryInlineInItem(Item, &Working, &LineStarts, Options)? {
				Working = NewWorking;
				AnyChanged = true;
				continue 'outer;
			}
		}

		break;
	}

	if AnyChanged { Ok(Some(Working)) } else { Ok(None) }
}

// ---------------------------------------------------------------------------
// Line-start table
// ---------------------------------------------------------------------------

/// Build a table where `table[i]` is the byte offset of the first character
/// on 1-based line `i+1`.  Line 1 always starts at byte 0.
fn BuildLineStartTable(Source:&str) -> Vec<usize> {
	let mut Table = vec![0usize];

	for (Offset, Ch) in Source.char_indices() {
		if Ch == '\n' {
			Table.push(Offset + Ch.len_utf8());
		}
	}

	Table
}

/// Convert a (1-based line, 0-based column) pair from `proc_macro2::LineColumn`
/// to a byte offset within `Source`, using the pre-built `LineStarts` table.
fn LcToByte(Lc:proc_macro2::LineColumn, LineStarts:&[usize], Source:&str) -> usize {
	let LineOffset = LineStarts.get(Lc.line.saturating_sub(1)).copied().unwrap_or(0);

	// Column is 0-based character count; translate to byte offset.
	Source[LineOffset..]
		.char_indices()
		.nth(Lc.column)
		.map(|(ByteOff, _)| LineOffset + ByteOff)
		.unwrap_or(Source.len())
}

// ---------------------------------------------------------------------------
// Item / block traversal
// ---------------------------------------------------------------------------

fn TryInlineInItem(
	Item:&syn::Item,
	Working:&str,
	LineStarts:&[usize],
	Options:&Definition::Options,
) -> Error::Result<Option<String>> {
	match Item {
		syn::Item::Fn(F) => TryInlineInBlock(&F.block, Working, LineStarts, Options),

		syn::Item::Impl(I) => {
			for ImplItem in &I.items {
				if let syn::ImplItem::Fn(Method) = ImplItem {
					if let Some(R) = TryInlineInBlock(&Method.block, Working, LineStarts, Options)? {
						return Ok(Some(R));
					}
				}
			}
			Ok(None)
		},

		_ => Ok(None),
	}
}

fn TryInlineInBlock(
	Block:&syn::Block,
	Working:&str,
	LineStarts:&[usize],
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

		// Clone only the downstream statements for in-memory substitution.
		let mut UseStmts:Vec<syn::Stmt> = Block.stmts[Candidate.StmtIndex + 1..].to_vec();

		if !Inline::SubstituteRef(&mut UseStmts, &Candidate.Ident, &Candidate.Init) {
			continue;
		}

		// Locate the `let` and its use statement in the source text.
		let LetRange = StmtByteRange(&Block.stmts[Candidate.StmtIndex], LineStarts, Working);
		let UseRange = StmtByteRange(&Block.stmts[Candidate.StmtIndex + 1], LineStarts, Working);

		// Render the substituted use-statement to source text.
		let UseReplacement = StmtToSource(&UseStmts[0]);

		// Apply the two edits to `Working` in reverse offset order so the
		// earlier `let` range isn't invalidated by the later use-range edit.
		let mut Out = Working.to_owned();

		// 1. Replace the use-statement text with the inlined version.
		if UseRange.start < UseRange.end && UseRange.end <= Out.len() {
			Out.replace_range(UseRange.clone(), &UseReplacement);
		}

		// 2. Remove the let-statement line (including its trailing newline).
		//    The use-range edit happened *after* the let range in the file, so
		//    the let-range offsets inside `Out` are still valid here.
		if LetRange.start < LetRange.end && LetRange.end <= Out.len() {
			let ExpandedEnd =
				if LetRange.end < Out.len() && Out.as_bytes()[LetRange.end] == b'\n' {
					LetRange.end + 1
				} else {
					LetRange.end
				};

			Out.replace_range(LetRange.start..ExpandedEnd, "");
		}

		return Ok(Some(Out));
	}

	// Recurse into directly nested blocks (if/loop/while/for/unsafe bodies).
	for Stmt in &Block.stmts {
		if let Some(Nested) = StmtNestedBlock(Stmt) {
			if let Some(R) = TryInlineInBlock(Nested, Working, LineStarts, Options)? {
				return Ok(Some(R));
			}
		}
	}

	Ok(None)
}

// ---------------------------------------------------------------------------
// Span helpers
// ---------------------------------------------------------------------------

/// Return the `[start, end)` byte range of `Stmt` within `Working`, using
/// `proc_macro2::Span::start()`/`end()` (1-based line, 0-based column) and
/// the pre-built `LineStarts` table.
///
/// We collect the first and last token spans via `ToTokens` and take the
/// outer envelope, which is reliable for all concrete statement kinds.
fn StmtByteRange(Stmt:&syn::Stmt, LineStarts:&[usize], Source:&str) -> std::ops::Range<usize> {
	use quote::ToTokens;

	let mut Tokens = proc_macro2::TokenStream::new();
	Stmt.to_tokens(&mut Tokens);

	let AllSpans:Vec<proc_macro2::Span> = Tokens.into_iter().map(|T| T.span()).collect();

	if AllSpans.is_empty() {
		return 0..0;
	}

	let Start = LcToByte(AllSpans.first().unwrap().start(), LineStarts, Source);
	let End = LcToByte(AllSpans.last().unwrap().end(), LineStarts, Source);

	Start..End
}

/// Render a single `syn::Stmt` to a source string.
///
/// We wrap it in a dummy function, pretty-print via `prettyplease`, then strip
/// the wrapper indentation so the result matches the surrounding style.
fn StmtToSource(Stmt:&syn::Stmt) -> String {
	use quote::quote;

	let Wrapped:syn::File = syn::parse_quote! { fn __dummy__() { #Stmt } };
	let Full = prettyplease::unparse(&Wrapped);

	if let (Some(Open), Some(Close)) = (Full.find('{'), Full.rfind('}')) {
		let Inner = Full[Open + 1..Close].trim_matches('\n');

		// Strip one level of wrapper indentation (tab or 4 spaces).
		return Inner
			.lines()
			.map(|L| L.strip_prefix('\t').or_else(|| L.strip_prefix("    ")).unwrap_or(L))
			.collect::<Vec<_>>()
			.join("\n");
	}

	Full
}

/// Extract a directly nested `Block` from a statement for recursion.
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
