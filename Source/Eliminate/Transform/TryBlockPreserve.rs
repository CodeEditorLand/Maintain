//! Try to preserve within a single block by finding an inlinable binding,
//! applying the text edit, and returning `Some(new_text)`.

use crate::Eliminate::{Definition, Error};
use super::{Collect, Count, Inline, Safe, StmtNestedBlock, StmtToText, TryBlockPreserve};

pub fn Fn(Block:&syn::Block, Working:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
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
		let LetText = StmtToText::Fn(&Block.stmts[Candidate.StmtIndex]);

		let UseOrigText = StmtToText::Fn(&Block.stmts[Candidate.StmtIndex + 1 + SubstSiteOffset]);

		let UseNewText = StmtToText::Fn(&UseStmts[SubstSiteOffset]);

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
		if let Some(Nested) = StmtNestedBlock::Fn(Stmt) {
			if let Some(R) = TryBlockPreserve::Fn(Nested, Working, Options)? {
				return Ok(Some(R));
			}
		}
	}

	Ok(None)
}
