//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Collect.rs
//=============================================================================//
// Module: Collect - Candidate let-binding discovery
//
// Scans a block's statement list for `let` bindings that are structurally
// eligible for inlining: simple (non-destructured) identifier pattern, no
// `mut` / `ref` qualifiers, has an initialiser, and no `else` branch.
//=============================================================================//

use syn::{Block, Pat, Stmt};

/// A `let` binding that may be inlinable.
#[derive(Debug, Clone)]
pub struct Candidate {
	/// The identifier name (e.g. `"URI"`).
	pub Ident:String,

	/// Index of this `Stmt::Local` within `block.stmts`.
	pub StmtIndex:usize,

	/// Cloned copy of the initialiser expression.
	/// Stored here so that we can borrow the expression independently while
	/// mutating `block.stmts`.
	pub Init:syn::Expr,
}

/// Collect all structurally eligible `let` bindings in `Block`.
///
/// Bindings are excluded when:
/// - The pattern is not a plain identifier (destructuring, tuple, …).
/// - The binding is `let mut` or `let ref`.
/// - The binding has a `@ subpat`.
/// - There is no initialiser (`let x;`).
/// - There is a diverging `else` branch (`let … = … else { … }`).
/// - The let statement carries attributes AND `InlineComments` is `false`.
/// - The binding is the last statement in the block (nothing to substitute
///   into).
pub fn Collect(Block:&Block, InlineComments:bool) -> Vec<Candidate> {
	let Len = Block.stmts.len();

	Block
		.stmts
		.iter()
		.enumerate()

		// Must have at least one subsequent statement to substitute into.
		.filter(|(Index, _)| *Index + 1 < Len)
		.filter_map(|(Index, Stmt)| {
			let Stmt::Local(Local) = Stmt else {
				return None;
			};

			// Skip attributed lets (doc-comments / derive-like attrs) unless
			// the caller explicitly opts in.
			if !InlineComments && !Local.attrs.is_empty() {
				return None;
			}

			// Must have an initialiser.
			let Some(Init) = &Local.init else {
				return None;
			};

			// No `let … = … else { … }` (diverging pattern).
			if Init.diverge.is_some() {
				return None;
			}

			// Pattern must be a plain identifier (optionally with a type
			// annotation).  `let X: i32 = 5` has Pat::Type(Pat::Ident).
			// No destructuring, no `mut`, no `ref`, no `@ subpat`.
			let PatIdent = match &Local.pat {
				Pat::Ident(P) => P,

				Pat::Type(syn::PatType { pat, .. }) => {
					if let Pat::Ident(P) = pat.as_ref() {
						P
					} else {
						return None;
					}
				},

				_ => return None,
			};

			if PatIdent.by_ref.is_some()

				|| PatIdent.mutability.is_some()

				|| PatIdent.subpat.is_some()

			{
				return None;
			}

			Some(Candidate {
				Ident: PatIdent.ident.to_string(),
				StmtIndex: Index,
				Init: *Init.expr.clone(),
			})
		})
		.collect()
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {

	use super::*;

	fn CollectFrom(Src:&str) -> Vec<Candidate> {
		let File:syn::File = syn::parse_str(Src).expect("parse");

		// Grab the first function body.
		for Item in &File.items {
			if let syn::Item::Fn(F) = Item {
				return Collect(&F.block, false);
			}
		}

		vec![]
	}

	#[test]
	fn SimpleLetIsCandidate() {
		let Candidates = CollectFrom("fn f() { let X = 5; f(X); }");

		assert_eq!(Candidates.len(), 1);

		assert_eq!(Candidates[0].Ident, "X");
	}

	#[test]
	fn MutLetExcluded() {
		let Candidates = CollectFrom("fn f() { let mut X = 5; f(X); }");

		assert!(Candidates.is_empty());
	}

	#[test]
	fn DestructuringExcluded() {
		let Candidates = CollectFrom("fn f() { let (A, B) = pair; f(A); }");

		assert!(Candidates.is_empty());
	}

	#[test]
	fn NoInitExcluded() {
		let Candidates = CollectFrom("fn f() { let X: i32; X = 5; f(X); }");

		assert!(Candidates.is_empty());
	}

	#[test]
	fn LastStmtExcluded() {
		// X is the last statement - nothing to substitute into.
		let Candidates = CollectFrom("fn f() { f(1); let X = 5; }");

		assert!(Candidates.is_empty());
	}

	#[test]
	fn LetElseExcluded() {
		let Candidates = CollectFrom(
			r#"fn f() {
                let Ok(X) = foo() else { return; };
                f(X);
            }"#,
		);

		assert!(Candidates.is_empty());
	}

	#[test]
	fn TwoConsecutiveCandidates() {
		let Candidates = CollectFrom(
			r#"fn f() {
                let A = 1;
                let B = A + 1;
                g(B);
            }"#,
		);

		assert_eq!(Candidates.len(), 2);

		assert_eq!(Candidates[0].Ident, "A");

		assert_eq!(Candidates[1].Ident, "B");
	}

	#[test]
	fn AttributedLetExcludedByDefault() {
		let Candidates = CollectFrom(
			r#"fn f() {
                #[allow(unused)]
                let X = 5;
                g(X);
            }"#,
		);

		assert!(Candidates.is_empty());
	}

	#[test]
	fn AttributedLetIncludedWhenOptIn() {
		let File:syn::File = syn::parse_str(
			r#"fn f() {
                #[allow(unused)]
                let X = 5;
                g(X);
            }"#,
		)
		.unwrap();

		let Block = match &File.items[0] {
			syn::Item::Fn(F) => &F.block,

			_ => panic!(),
		};

		let Candidates = Collect(Block, true);

		assert_eq!(Candidates.len(), 1);
	}
}
