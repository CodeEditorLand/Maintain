//! Recursively compare OrigBlock (spans anchored to Source) and MutBlock
//! (mutated AST, spans may be synthetic) and collect patches for any
//! statements that differ.

use super::{CollectInnerBlockPatches, Patch, StmtTokensMatch};

pub fn Fn(OrigBlock:&syn::Block, MutBlock:&syn::Block, Source:&str, Patches:&mut Vec<Patch::Patch>) -> Option<()> {
	let mut OrigIdx = 0usize;

	for MutStmt in &MutBlock.stmts {
		while OrigIdx < OrigBlock.stmts.len() {
			let OrigStmt = &OrigBlock.stmts[OrigIdx];

			OrigIdx += 1;

			if StmtTokensMatch::Fn(OrigStmt, MutStmt) {
				break;
			}

			let (Start, End) = Patch::StmtLineRange(OrigStmt, Source)?;

			Patches.push(Patch::Patch { Start, End, Replacement:String::new() });

			if OrigIdx < OrigBlock.stmts.len() && StmtTokensMatch::Fn(&OrigBlock.stmts[OrigIdx], MutStmt) {
				break;
			}
		}

		CollectInnerBlockPatches::Fn(MutStmt, Source, Patches)?;
	}

	Some(())
}
