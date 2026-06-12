//! Recurse into block-containing expression variants to collect patches
//! from inner scopes.

use super::Patch;

pub fn Fn(Stmt:&syn::Stmt, Source:&str, Patches:&mut Vec<Patch::Patch>) -> Option<()> {
	use syn::{Expr, Stmt as S};

	match Stmt {
		S::Expr(Expr::Block(B), _) => {
			for S_ in &B.block.stmts {
				super::CollectInnerBlockPatches::Fn(S_, Source, Patches)?;
			}
		},

		_ => {},
	}

	Some(())
}
