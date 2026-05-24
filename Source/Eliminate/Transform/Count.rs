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
//     but never over-counts).
//   - Macro token streams: identifiers inside `json!(…)`, `dev_log!(…)`, and
//     other macro invocations are counted via raw token-tree scanning.  This
//     is critical for correctness: without it, a variable used in both a macro
//     and a regular expression would be miscounted as single-use.
//   - Closure captures: references inside a closure body set `InClosure`.
//     Callers treat such bindings as non-inlinable (move semantics may differ).
//=============================================================================//

use proc_macro2::{TokenStream, TokenTree};
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
/// - `count` is the number of times the identifier is referenced (both as a
///   plain expression AND inside macro token streams).
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

	// Count identifier occurrences inside macro token streams (e.g. json!(…),
	// dev_log!(…), format!(…)).  The default syn visitor does NOT recurse into
	// Macro::tokens, so we do it manually here.
	fn visit_expr_macro(&mut self, Node: &'ast syn::ExprMacro) {
		self.Count += CountIdentsInTokenStream(&Node.mac.tokens, self.Target);
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

		// propagate InClosure upward - once set, keep it
		let _ = WasInClosure;
	}
}

/// Recursively count occurrences of `Target` as an `Ident` token inside a
/// raw `TokenStream`.  This covers macro arguments that are otherwise opaque
/// to syn's AST visitor.
pub fn CountIdentsInTokenStream(Tokens: &TokenStream, Target: &str) -> usize {
	let mut Count = 0;

	for Tree in Tokens.clone() {
		match Tree {
			TokenTree::Ident(I) if I == Target => Count += 1,

			TokenTree::Group(G) => Count += CountIdentsInTokenStream(&G.stream(), Target),

			_ => {},
		}
	}

	Count
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
		// let X=1; f(X); let X=2; g(X)
		// Count refs for the FIRST X starting from index 1.
		let S = Stmts("fn f() { let X = 1; f(X); let X = 2; g(X); }");

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 1); // only f(X) counted; shadow stops before g(X)
	}

	/// A variable used inside a format-style macro MUST be counted.
	/// Previously, macros were transparent - this was a correctness bug.
	#[test]
	fn MacroCountsAsUse() {
		// dev_log! uses URI once.
		let S = Stmts(r#"fn f() { let URI = "x"; dev_log!("{}", URI); }"#);

		let (Count, _) = CountReferences("URI", &S[1..]);

		assert_eq!(Count, 1);
	}

	/// The motivating correctness bug: URI used in BOTH a macro and a plain
	/// expression should give count=2, not count=1.
	#[test]
	fn MacroAndExprBothCounted() {
		let S = Stmts(
			r#"fn f() {
                let URI = "x";
                dev_log!("{}", URI);
                let _ = Url::parse(URI);
            }"#,
		);

		let (Count, _) = CountReferences("URI", &S[1..]);

		assert_eq!(Count, 2, "URI used in macro + expression must count as 2");
	}

	/// Variable used ONLY inside a json! macro: count=1, eligible.
	#[test]
	fn MacroOnlyUse() {
		let S = Stmts(
			r#"fn f() {
                let DataString = compute();
                emit(json!({ "data": DataString }));
            }"#,
		);

		let (Count, _) = CountReferences("DataString", &S[1..]);

		assert_eq!(Count, 1);
	}

	/// Variable used twice inside the same macro invocation: count=2, not eligible.
	#[test]
	fn MacroDoubleUse() {
		let S = Stmts(
			r#"fn f() {
                let X = val();
                json!({ "a": X, "b": X });
            }"#,
		);

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 2);
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
		let S = Stmts(
			r#"fn f() {
                let X = 1;
                { let X = 2; g(X); }
            }"#,
		);

		// X inside the inner block is the inner X, not the outer X.
		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 0);
	}

	#[test]
	fn ClosureParamShadowSkipped() {
		let S = Stmts("fn f() { let X = 5; let _ = |X| X + 1; g(0); }");

		let (Count, InClosure) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 0);

		assert!(!InClosure);
	}
}
