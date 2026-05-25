//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/mod.rs
//=============================================================================//
// Module: Transform - AST transformation pipeline
//
// Two entry points:
//
//   Run(source, options)
//     Original behaviour: parse with syn, run the VisitMut eliminator,
//     then attempt span-based text patching to preserve comments and
//     whitespace. Falls back to prettyplease::unparse when span data is
//     unavailable. Used by the Reformat path and by all existing unit tests
//     in Inline.rs (which compare output against prettyplease-normalised
//     expected values).
//
//   RunPreserve(source, options)
//     Preserve-layout behaviour (default when Options.Reformat == false):
//     identifies inlinable bindings via the same Collect/Safe/Count pipeline,
//     then applies targeted text substitutions to the original source string
//     without touching anything outside the affected lines.  Comments, blank
//     lines, section banners, and the original indentation style survive
//     unchanged.  Uses NO proc_macro2 span APIs so no extra Cargo features
//     are required.
//
// Returns `Ok(None)` when no bindings were eliminated.
//=============================================================================//

pub mod Collect;
pub mod Count;
pub mod Inline;
pub mod Patch;
pub mod Safe;

use super::{Definition, Error};

// ---------------------------------------------------------------------------
// Original entry point - span-based patch path with prettyplease fallback
// (signature identical to Current; all Inline.rs tests call this function)
// ---------------------------------------------------------------------------

/// Parse `Source`, run up to [`super::Constant::MaxIterations`] elimination
/// passes, then return the patched source text.
///
/// Preferred path: span-based text patching via `TryPatchSource` preserves
/// inline comments and blank lines. Falls back to `prettyplease::unparse`
/// when span-location data is unavailable.
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
fn CollectInnerBlockPatches(Stmt:&syn::Stmt, Source:&str, Patches:&mut Vec<Patch::Patch>) -> Option<()> {
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

// ---------------------------------------------------------------------------
// Preserve-layout entry point - span-free text substitution
// ---------------------------------------------------------------------------

/// Identify inlinable bindings via the same AST pipeline as `Run`, but apply
/// the substitutions as targeted text edits so that every character outside
/// the affected `let` binding and its single use-site is preserved verbatim.
///
/// Uses no `proc_macro2` span APIs; works on stable Rust with the dependency
/// set already declared in `Cargo.toml`.
///
/// Returns `Ok(None)` when no bindings were eliminated.
pub fn RunPreserve(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let mut Working = Source.to_owned();
	let mut AnyChanged = false;

	for _ in 0..super::Constant::MaxIterations {
		match PreservePass(&Working, Options)? {
			Some(Next) => {
				Working = Next;
				AnyChanged = true;
			},

			None => break,
		}
	}

	if AnyChanged { Ok(Some(Working)) } else { Ok(None) }
}

/// One pass: parse `Working`, find the first inlinable binding, apply the
/// text edit, return `Some(new_text)`. Returns `None` when nothing changed.
fn PreservePass(Working:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let Ast:syn::File = syn::parse_str(Working).map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

	for Item in &Ast.items {
		if let Some(Result) = TryItemPreserve(Item, Working, Options)? {
			return Ok(Some(Result));
		}
	}

	Ok(None)
}

fn TryItemPreserve(Item:&syn::Item, Working:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	match Item {
		syn::Item::Fn(F) => TryBlockPreserve(&F.block, Working, Options),

		syn::Item::Impl(I) => {
			for ImplItem in &I.items {
				if let syn::ImplItem::Fn(M) = ImplItem {
					if let Some(R) = TryBlockPreserve(&M.block, Working, Options)? {
						return Ok(Some(R));
					}
				}
			}

			Ok(None)
		},

		_ => Ok(None),
	}
}

fn TryBlockPreserve(Block:&syn::Block, Working:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let Candidates = Collect::Collect(Block, Options.InlineComments);

	for Candidate in &Candidates {
		if !Safe::IsSafe(&Candidate.Init, Options.MaxSize) {
			continue;
		}

		let (RefCount, InClosure, InLoop) =
			Count::CountReferences(&Candidate.Ident, &Block.stmts[Candidate.StmtIndex + 1..]);

		if RefCount != 1 || InClosure || InLoop {
			continue;
		}

		let StmtsAfter = &Block.stmts[Candidate.StmtIndex + 1..];
		let SubstSiteOffset = Inline::FindSubstSite(StmtsAfter, &Candidate.Ident);
		if !Safe::IsFreeVarSafe(&Candidate.Init, &StmtsAfter[..SubstSiteOffset]) {
			continue;
		}

		// Clone downstream statements; substitute in-memory.
		let mut UseStmts:Vec<syn::Stmt> = Block.stmts[Candidate.StmtIndex + 1..].to_vec();

		if !Inline::SubstituteRef(&mut UseStmts, &Candidate.Ident, &Candidate.Init) {
			continue;
		}

		// Render the ORIGINAL let-stmt and use-stmt to canonical text so we
		// can locate them in `Working` by plain string search.
		let LetText = StmtToText(&Block.stmts[Candidate.StmtIndex]);
		let UseOrigText = StmtToText(&Block.stmts[Candidate.StmtIndex + 1 + SubstSiteOffset]);
		let UseNewText = StmtToText(&UseStmts[SubstSiteOffset]);

		// Locate the original let-stmt text in Working.
		let Some(LetPos) = Working.find(&LetText) else {
			continue;
		};

		// Locate the original use-stmt text - must appear AFTER the let.
		let SearchFrom = LetPos + LetText.len();
		let Some(UseOffset) = Working[SearchFrom..].find(&UseOrigText) else {
			continue;
		};

		let UsePos = SearchFrom + UseOffset;

		// Apply edits in reverse order (use comes later, so edit it first so
		// the let-stmt byte positions remain valid).
		let mut Out = Working.to_owned();

		// 1. Replace the use-stmt with the substituted version.
		Out.replace_range(UsePos..UsePos + UseOrigText.len(), &UseNewText);

		// 2. Remove the let-stmt line (expand to include trailing newline).
		let LetEnd = LetPos + LetText.len();
		let ExpandedLetEnd = if LetEnd < Out.len() && Out.as_bytes()[LetEnd] == b'\n' {
			LetEnd + 1
		} else {
			LetEnd
		};

		Out.replace_range(LetPos..ExpandedLetEnd, "");

		return Ok(Some(Out));
	}

	// Recurse into directly nested blocks.
	for Stmt in &Block.stmts {
		if let Some(Nested) = StmtNestedBlock(Stmt) {
			if let Some(R) = TryBlockPreserve(Nested, Working, Options)? {
				return Ok(Some(R));
			}
		}
	}

	Ok(None)
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Render a single `syn::Stmt` to its canonical text representation by
/// wrapping it in a dummy function body and extracting the inner line(s).
/// The wrapper indentation (one tab or 4 spaces from prettyplease) is
/// stripped so the result is indentation-relative.
fn StmtToText(Stmt:&syn::Stmt) -> String {
	use quote::quote;

	let Wrapped:syn::File = syn::parse_quote! { fn __d() { #Stmt } };
	let Full = prettyplease::unparse(&Wrapped);

	// Full looks like "fn __d() {\n    <stmt>\n}\n".
	// Extract between first '{' and last '}'.
	if let (Some(Open), Some(Close)) = (Full.find('{'), Full.rfind('}')) {
		let Inner = Full[Open + 1..Close].trim_matches('\n');

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
