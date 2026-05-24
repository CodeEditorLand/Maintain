//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Safe.rs
//=============================================================================//
// Module: Safe - Safety predicate for inlinable initialisers
//
// An initialiser is "safe to inline" when:
//   1. Its AST node count does not exceed `MaxSize` (avoids blowing up short
//      call-site lines with enormous expressions).
//   2. It does not contain an `unsafe { … }` block.
//
// Notable inclusions (expressions that ARE safe in Rust):
//   - `?` operator  (`Expr::Try`)     - propagates the error, context unchanged.
//   - `.await`      (`Expr::Await`)   - async context is caller-determined.
//   - Closures      (`Expr::Closure`) - safe when the closure does not capture
//     the candidate binding (that is checked by Count, not here).
//=============================================================================//

use syn::{
	Expr,
	visit::{Visit, visit_expr},
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns `true` when `E` is safe to substitute at its single use site.
pub fn IsSafe(E: &Expr, MaxSize: usize) -> bool {
	NodeCount(E) <= MaxSize && !ContainsUnsafe(E)
}

// ---------------------------------------------------------------------------
// Node counting
// ---------------------------------------------------------------------------

struct NodeCounter {
	pub Count: usize,
}

impl<'ast> Visit<'ast> for NodeCounter {
	fn visit_expr(&mut self, Node: &'ast Expr) {
		self.Count += 1;

		visit_expr(self, Node);
	}
}

pub fn NodeCount(E: &Expr) -> usize {
	let mut Counter = NodeCounter { Count: 0 };

	Counter.visit_expr(E);

	Counter.Count
}

// ---------------------------------------------------------------------------
// Unsafe detection
// ---------------------------------------------------------------------------

struct UnsafeDetector {
	pub Found: bool,
}

impl<'ast> Visit<'ast> for UnsafeDetector {
	fn visit_expr_unsafe(&mut self, _Node: &'ast syn::ExprUnsafe) {
		self.Found = true;

		// Do not recurse - one match is enough.
	}
}

pub fn ContainsUnsafe(E: &Expr) -> bool {
	let mut Detector = UnsafeDetector { Found: false };

	Detector.visit_expr(E);

	Detector.Found
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {
	use super::*;

	fn ParseExpr(Src: &str) -> Expr {
		syn::parse_str(Src).expect("parse expression")
	}

	#[test]
	fn LiteralIsSafe() {
		assert!(IsSafe(&ParseExpr("42"), 100));
	}

	#[test]
	fn UnsafeBlockIsNotSafe() {
		assert!(!IsSafe(&ParseExpr("unsafe { *ptr }"), 100));
	}

	#[test]
	fn QuestionMarkIsSafe() {
		assert!(IsSafe(&ParseExpr("foo().map_err(|e| e)?"), 100));
	}

	#[test]
	fn AwaitIsSafe() {
		assert!(IsSafe(&ParseExpr("service.call().await"), 100));
	}

	#[test]
	fn ClosureIsSafe() {
		assert!(IsSafe(&ParseExpr("|| { 1 + 1 }"), 100));
	}

	#[test]
	fn StructLiteralIsSafe() {
		assert!(IsSafe(&ParseExpr("Opts { a: 1, b: 2 }"), 100));
	}

	#[test]
	fn MacroCallIsSafe() {
		assert!(IsSafe(&ParseExpr(r#"json!({ "key": val })"#), 100));
	}

	#[test]
	fn OversizedExprFails() {
		// Build a deeply nested binary expression exceeding MaxSize=5.
		let Big = ParseExpr("a + b + c + d + e + f + g + h");

		assert!(!IsSafe(&Big, 5));
	}

	#[test]
	fn NodeCountLiteral() {
		assert_eq!(NodeCount(&ParseExpr("1")), 1);
	}

	#[test]
	fn NodeCountBinary() {
		// `A + B` has one Binary node wrapping two Path nodes = 3 total.
		assert_eq!(NodeCount(&ParseExpr("A + B")), 3);
	}

	#[test]
	fn NodeCountCall() {
		// `f(A, B)` = Call + Path(f) + Path(A) + Path(B) = 4.
		assert_eq!(NodeCount(&ParseExpr("f(A, B)")), 4);
	}
}
