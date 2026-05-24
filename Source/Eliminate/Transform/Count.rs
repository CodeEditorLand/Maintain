//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Count.rs
//=============================================================================//
// Module: Count - Reference counting for let-binding candidates
//
// Counts how many times a named identifier is referenced in a slice of
// statements, respecting:
//   - Top-level shadowing: a second `let <target> = …` at the same scope
//     depth stops the count.
//   - Inner-block shadowing: if an inner block re-introduces the name, all
//     references inside that block are excluded (conservative; may under-count
//     but will never over-count).
//   - Closure captures: references inside a closure body set `InClosure`.
//     Callers treat such bindings as non-inlinable (move semantics may differ).
//=============================================================================//

use syn::{
	Pat,
	Stmt,
	visit::{Visit, visit_block, visit_expr_closure},
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Count references to `Target` in `Stmts`.
///
/// Returns `(count, in_closure)`.
///
/// - `count` is the number of times the identifier is referenced.
/// - `in_closure` is true when at least one reference occurs inside a closure
///   body (even if `count == 1`).
///
/// Counting stops when a top-level `let <target> = …` shadow is encountered.
pub fn CountReferences(Target: &str, Stmts: &[Stmt]) -> (usize, bool) {
	let mut TotalCount = 0usize;
	let mut InClosure = false;

	for Stmt in Stmts {
		// A top-level shadow terminates the count for the outer binding.
		if IsTopLevelShadow(Stmt, Target) {
			break;
		}

		let mut Counter = ExprCounter { Target, Count: 0, InClosure: false };

		Counter.visit_stmt(Stmt);

		TotalCount += Counter.Count;

		if Counter.InClosure {
			InClosure = true;
		}
	}

	(TotalCount, InClosure)
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn IsTopLevelShadow(Stmt: &Stmt, Target: &str) -> bool {
	if let Stmt::Local(Local) = Stmt {
		if let Pat::Ident(P) = &Local.pat {
			return P.ident == Target;
		}
	}

	false
}

struct ExprCounter<'a> {
	Target: &'a str,
	Count: usize,
	/// True when a reference to `Target` was found inside a closure body.
	InClosure: bool,
}

impl<'ast> Visit<'ast> for ExprCounter<'ast> {
	// Count plain identifier path expressions that match Target.
	fn visit_expr_path(&mut self, Node: &'ast syn::ExprPath) {
		if let Some(Ident) = Node.path.get_ident() {
			if Ident == self.Target {
				self.Count += 1;
			}
		}
	}

	// Skip inner blocks that would shadow Target - conservative, avoids
	// counting references that actually belong to the inner binding.
	fn visit_block(&mut self, Node: &'ast syn::Block) {
		if BlockShadowsTarget(&Node.stmts, self.Target) {
			return; // skip entire inner block
		}

		visit_block(self, Node);
	}

	// References inside closure bodies are flagged so callers can conservatively
	// decline inlining (move vs. capture semantics differ).
	fn visit_expr_closure(&mut self, Node: &'ast syn::ExprClosure) {
		// If the closure itself shadows Target via a parameter, skip the body.
		if ClosureParamShadows(Node, self.Target) {
			return;
		}

		let WasInClosure = self.InClosure;

		self.InClosure = true;

		visit_expr_closure(self, Node);

		// Once in-closure is set, keep it set even after returning from this
		// recursive call so that the caller sees the flag.
		if !WasInClosure {
			// propagate upward; do NOT reset to false
		}
	}
}

fn BlockShadowsTarget(Stmts: &[Stmt], Target: &str) -> bool {
	Stmts.iter().any(|S| IsTopLevelShadow(S, Target))
}

fn ClosureParamShadows(Closure: &syn::ExprClosure, Target: &str) -> bool {
	Closure.inputs.iter().any(|P| {
		if let Pat::Ident(PIdent) = P {
			PIdent.ident == Target
		} else {
			false
		}
	})
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {
	use super::*;

	fn Stmts(Src: &str) -> Vec<Stmt> {
		let File: syn::File = syn::parse_str(Src).expect("parse");

		if let syn::Item::Fn(F) = &File.items[0] {
			return F.block.stmts.clone();
		}

		vec![]
	}

	#[test]
	fn SingleUse() {
		// Stmts after the `let X` declaration are stmts[1..]
		let S = Stmts("fn f() { let X = 1; g(X); }");

		let (Count, InClosure) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 1);

		assert!(!InClosure);
	}

	#[test]
	fn MultiUse() {
		let S = Stmts("fn f() { let X = foo(); bar(X); baz(X); }");

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 2);
	}

	#[test]
	fn ZeroUse() {
		let S = Stmts("fn f() { let X = 1; g(1); }");

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 0);
	}

	#[test]
	fn ShadowStops() {
		// let X = 1; f(X); let X = 2; g(X);
		// When counting after the first let X, we see f(X) (count 1) then
		// encounter `let X = 2` (shadow) and stop.
		let S = Stmts("fn f() { let X = 1; f(X); let X = 2; g(X); }");

		// Stmts are: [let X=1, f(X), let X=2, g(X)]
		// Count refs for the FIRST X, starting from index 1.
		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 1); // only f(X) counted; shadow stops before g(X)
	}

	#[test]
	fn MacroCountsAsUse() {
		let S = Stmts(r#"fn f() { let URI = "x"; dev_log!("{}", URI); }"#);

		let (Count, _) = CountReferences("URI", &S[1..]);

		assert_eq!(Count, 1);
	}

	#[test]
	fn ClosureCaptureDetected() {
		let S = Stmts("fn f() { let X = heavy(); let F = move || X; call(F); }");

		let (Count, InClosure) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 1);

		assert!(InClosure, "X used inside closure should set InClosure");
	}

	#[test]
	fn InnerBlockShadowSkipped() {
		// Inner block re-introduces X; its use should NOT be counted.
		let S = Stmts(
			r#"fn f() {
                let X = 1;
                { let X = 2; g(X); }
            }"#,
		);

		let (Count, _) = CountReferences("X", &S[1..]);

		// X inside { let X = 2; g(X); } is the inner X, not counted.
		assert_eq!(Count, 0);
	}

	#[test]
	fn ClosureParamShadowSkipped() {
		// |X| uses X as a parameter name - the outer X is not captured.
		let S = Stmts("fn f() { let X = 5; let _ = |X| X + 1; g(0); }");

		let (Count, InClosure) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 0);

		assert!(!InClosure);
	}
}
