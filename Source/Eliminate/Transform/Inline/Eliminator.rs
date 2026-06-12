//! VisitMut transformer that eliminates single-use bindings - Eliminator.
//!
//! Algorithm (per block, bottom-up):
//! 1. Collect structurally eligible let-binding candidates.
//! 2. For each candidate (in declaration order): count references, skip if
//!    count != 1, used-in-closure, used-in-loop-body, or initialiser is
//!    unsafe/large.
//! 3. Substitute the single reference with the initialiser.
//! 4. Remove the let statement.
//! 5. Wrap substituted expressions in parentheses when needed (precedence
//!    safety).

use syn::{
	visit_mut::{VisitMut, visit_block_mut},
};

use super::{Collect::Collect, Count::CountReferences, Safe, SubstituteRef, FindSubstSite};

// ---------------------------------------------------------------------------
// Public: Eliminator
// ---------------------------------------------------------------------------

pub struct Eliminator<'a> {
	pub Changed:bool,

	Options:&'a crate::Eliminate::Definition::Options,
}

impl<'a> Eliminator<'a> {
	pub fn new(Options:&'a crate::Eliminate::Definition::Options) -> Self { Self { Changed:false, Options } }

	fn EliminateBlock(&mut self, Block:&mut syn::Block) {
		loop {
			let Candidates = Collect(Block, self.Options.InlineComments);

			let mut DidChange = false;

			for Candidate in &Candidates {
				if !Safe::IsSafe(&Candidate.Init, self.Options.MaxSize) {
					continue;
				}

				let StmtsAfter = &Block.stmts[Candidate.StmtIndex + 1..];

				let (RefCount, InClosure, InLoop) = CountReferences(&Candidate.Ident, StmtsAfter);

				if RefCount != 1 || InClosure || InLoop {
					continue;
				}

				// Find the index of the substitution site within StmtsAfter
				// (the first statement that contains a reference to Candidate).
				// We need the slice of statements that come BEFORE that site
				// to check whether any free variable in Init is moved there.
				let SubstSiteOffset = FindSubstSite(StmtsAfter, &Candidate.Ident);

				let StmtsBetween = &StmtsAfter[..SubstSiteOffset];

				if !Safe::IsFreeVarSafe(&Candidate.Init, StmtsBetween) {
					continue;
				}

				let Substituted =
					SubstituteRef(&mut Block.stmts[Candidate.StmtIndex + 1..], &Candidate.Ident, &Candidate.Init);

				if Substituted {
					Block.stmts.remove(Candidate.StmtIndex);

					self.Changed = true;

					DidChange = true;

					break;
				}
			}

			if !DidChange {
				break;
			}
		}
	}
}

impl<'a> VisitMut for Eliminator<'a> {
	fn visit_block_mut(&mut self, Block:&mut syn::Block) {
		// Bottom-up: process inner blocks before this one.
		visit_block_mut(self, Block);

		self.EliminateBlock(Block);
	}
}
