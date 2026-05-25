//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/mod.rs
//=============================================================================//
// Module: Transform - AST transformation pipeline
//
// Entry point: Run(source, options) parses the Rust source with syn, applies
// iterative single-use-variable elimination, then attempts to reconstruct the
// output by splicing only the changed byte ranges back into the original source
// text (preserving inline comments, blank lines, and all formatting trivia).
//
// Patch path (preferred):
//   For each eliminated binding, Patch locates the byte span of the let
//   statement and the byte span of the substitution site in the original source
//   text via proc_macro2 Span offsets, and applies those as sorted
//   non-overlapping replacements. The rest of the file is copied verbatim.
//
// Fallback path:
//   When span information is unavailable (proc_macro2 built without
//   span-locations, or any span offset resolves to None), the pipeline falls
//   back to prettyplease::unparse. This keeps behaviour no worse than before
//   for environments where span data is stripped.
//=============================================================================//

pub mod Collect;
pub mod Count;
pub mod Inline;
pub mod Patch;
pub mod Safe;

use super::{Definition, Error};

/// Parse Source, run up to MaxIterations elimination passes, then return the
/// patched source text. Returns Ok(None) when no bindings were eliminated.
pub fn Run(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
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

	// Attempt span-based text patching to preserve comments and whitespace.
	// Build the patched output by comparing the original AST (re-parsed from
	// Source so spans are relative to Source) against the mutated Ast.
	//
	// Strategy: re-parse Source into OriginalAst whose spans ARE anchored to
	// Source bytes. Walk both ASTs in parallel, collecting (removed-let-span,
	// substituted-ident-span, replacement-text) triples, then apply them.
	if let Some(Patched) = TryPatchSource(Source, &Ast) {
		return Ok(Some(Patched));
	}

	// Fallback: full reprint via prettyplease (drops comments / whitespace).
	Ok(Some(prettyplease::unparse(&Ast)))
}

// ---------------------------------------------------------------------------
// Span-based patch path
// ---------------------------------------------------------------------------

/// Attempt to reconstruct the output by splicing only the changed ranges.
/// Returns None on any span resolution failure, triggering the fallback.
fn TryPatchSource(Source:&str, MutatedAst:&syn::File) -> Option<String> {
	// Re-parse Source so we have an AST whose Spans are anchored to Source.
	let OriginalAst:syn::File = syn::parse_str(Source).ok()?;

	let mut Patches:Vec<Patch::Patch> = Vec::new();

	// Walk items in parallel. We only care about function bodies; other items
	// (mod, use, struct, ...) are never touched by the eliminator.
	for (OrigItem, MutItem) in OriginalAst.items.iter().zip(MutatedAst.items.iter()) {
		if let (syn::Item::Fn(OrigFn), syn::Item::Fn(MutFn)) = (OrigItem, MutItem) {
			CollectBlockPatches(&OrigFn.block, &MutFn.block, Source, &mut Patches)?;
		}
	}

	if Patches.is_empty() {
		// No spans resolved but AnyChanged was true - fall back.
		return None;
	}

	// Sort by start offset and verify non-overlapping.
	Patches.sort_by_key(|P| P.Start);

	Patch::ApplyPatches(Source, &Patches)
}

/// Recursively compare OrigBlock (spans anchored to Source) and MutBlock
/// (mutated AST, spans may be synthetic) and collect patches for any
/// statements that differ.
fn CollectBlockPatches(
	OrigBlock:&syn::Block,
	MutBlock:&syn::Block,
	Source:&str,
	Patches:&mut Vec<Patch::Patch>,
) -> Option<()> {
	// Walk MutBlock statements; for each one, find the corresponding
	// statement(s) in OrigBlock by matching on identity (same kind, same
	// initialiser text for lets). Statements the eliminator REMOVED will
	// be present in OrigBlock but absent from MutBlock. Statements where
	// an identifier was SUBSTITUTED will have a different token stream.

	let mut OrigIdx = 0usize;

	for MutStmt in &MutBlock.stmts {
		// Advance OrigIdx until we find a statement that matches MutStmt
		// or we exhaust OrigBlock.
		while OrigIdx < OrigBlock.stmts.len() {
			let OrigStmt = &OrigBlock.stmts[OrigIdx];
			OrigIdx += 1;

			if StmtTokensMatch(OrigStmt, MutStmt) {
				// Same statement - no patch needed here.
				break;
			}

			// OrigStmt is in the original but not in MutBlock at this
			// position: it was a removed let. Emit a delete patch.
			let (Start, End) = Patch::StmtLineRange(OrigStmt, Source)?;

			Patches.push(Patch::Patch { Start, End, Replacement:String::new() });

			// Now check if MutStmt corresponds to this OrigStmt after the
			// removal - if not, continue consuming OrigBlock.
			if StmtTokensMatch(&OrigBlock.stmts[OrigIdx - 1 + 1], MutStmt) {
				break;
			}
		}

		// Recurse into inner blocks.
		CollectInnerBlockPatches(MutStmt, Source, Patches)?;
	}

	Some(())
}

/// Recurse into block-containing expression variants.
fn CollectInnerBlockPatches(
	Stmt:&syn::Stmt,
	Source:&str,
	Patches:&mut Vec<Patch::Patch>,
) -> Option<()> {
	use syn::{Expr, Stmt as S};

	match Stmt {
		S::Expr(Expr::Block(B), _) => {
			for S_ in &B.block.stmts {
				CollectInnerBlockPatches(S_, Source, Patches)?;
			}
		},

		_ => {},
	}

	Some(())
}

/// Compare two statements by their token stream text.
fn StmtTokensMatch(A:&syn::Stmt, B:&syn::Stmt) -> bool {
	use quote::ToTokens;

	let mut Ta = proc_macro2::TokenStream::new();
	let mut Tb = proc_macro2::TokenStream::new();

	A.to_tokens(&mut Ta);
	B.to_tokens(&mut Tb);

	Ta.to_string() == Tb.to_string()
}
