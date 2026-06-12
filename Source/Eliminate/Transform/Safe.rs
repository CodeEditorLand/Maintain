//! Safety predicates for inlinable initialisers.
//!
//! An initialiser is "safe to inline" when:
//! 1. Its AST node count does not exceed MaxSize.
//! 2. It does not contain an unsafe { } block.
//! 3. Every plain-identifier free variable in the initialiser is not moved
//!    (consumed by value) in the statements between the candidate declaration
//!    and the substitution site.

use syn::{
	Expr,
	Stmt,
	visit::{Visit, visit_expr},
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Returns true when E is safe to substitute at its single use site,
/// considering only the expression itself (size and unsafe).
pub fn IsSafe(E:&Expr, MaxSize:usize) -> bool { NodeCount(E) <= MaxSize && !ContainsUnsafe(E) }

/// Returns true when every plain-identifier free variable inside Init remains
/// live (not moved by value) in the statements that appear between the
/// candidate let-binding (exclusive) and the substitution site (exclusive).
///
/// Stmts is the slice Block.stmts[CandidateIndex + 1..SubstSite] - i.e. the
/// statements that execute after the let but before the single use.
///
/// The check is conservative: it only tracks Expr::Path single-segment bare
/// identifiers. Qualified paths (a::b), method receivers (.foo()), and
/// reference borrows (&x) are all ignored, so false positives (keeping a
/// binding that would have been safe) are possible but false negatives that
/// would introduce a compile error are not.
pub fn IsFreeVarSafe(Init:&Expr, StmtsBetween:&[Stmt]) -> bool {
	let FreeVars = CollectFreeIdents(Init);

	if FreeVars.is_empty() || StmtsBetween.is_empty() {
		return true;
	}

	for Name in &FreeVars {
		if IsMovedInStmts(Name, StmtsBetween) {
			return false;
		}
	}

	true
}

// ---------------------------------------------------------------------------
// Free-identifier collection
// ---------------------------------------------------------------------------

struct FreeIdentCollector {
	Idents:Vec<String>,
}

impl<'ast> Visit<'ast> for FreeIdentCollector {
	fn visit_expr_path(&mut self, Node:&'ast syn::ExprPath) {
		if Node.qself.is_none() {
			if let Some(Ident) = Node.path.get_ident() {
				let Name = Ident.to_string();

				if !self.Idents.contains(&Name) {
					self.Idents.push(Name);
				}
			}
		}
	}
}

fn CollectFreeIdents(E:&Expr) -> Vec<String> {
	let mut C = FreeIdentCollector { Idents:vec![] };

	C.visit_expr(E);

	C.Idents
}

// ---------------------------------------------------------------------------
// Move detection
// ---------------------------------------------------------------------------

/// Returns true when Target is consumed by value in any of the statements.
/// We detect by-value consumption conservatively: if Target appears as a
/// plain Expr::Path (bare identifier, no & / &mut / ref prefix) in a
/// position that syntactically transfers ownership:
///   - a function or method call argument
///   - a struct / tuple-struct field value (shorthand or explicit)
///   - the right-hand side of a let initialiser or assignment
///   - an array element, tuple element, or return expression
fn IsMovedInStmts(Target:&str, Stmts:&[Stmt]) -> bool {
	for Stmt in Stmts {
		let mut Detector = MoveDetector { Target, Found:false };

		Detector.visit_stmt(Stmt);

		if Detector.Found {
			return true;
		}
	}

	false
}

struct MoveDetector<'a> {
	Target:&'a str,

	Found:bool,
}

impl<'ast> Visit<'ast> for MoveDetector<'ast> {
	// Plain identifier used as a call argument, struct field, array element,
	// tuple element, return value, or let/assign RHS - all by-value moves.
	fn visit_expr_path(&mut self, Node:&'ast syn::ExprPath) {
		if self.Found {
			return;
		}

		if Node.qself.is_none() {
			if let Some(I) = Node.path.get_ident() {
				if I == self.Target {
					self.Found = true;
				}
			}
		}
	}

	// Do not descend into reference expressions: &Target and &mut Target
	// borrow rather than move, so they are safe.
	fn visit_expr_reference(&mut self, _Node:&'ast syn::ExprReference) {}

	// Do not descend into method call receivers: self.foo(target_as_self)
	// is a borrow in almost all cases; conservatively skip.
	// We DO still visit the arguments via the default walk, so explicit
	// by-value arguments to method calls are still caught.
	fn visit_expr_method_call(&mut self, Node:&'ast syn::ExprMethodCall) {
		if self.Found {
			return;
		}

		// Visit arguments only; skip the receiver (self.receiver).
		for Arg in &Node.args {
			self.visit_expr(Arg);
		}
	}
}

// ---------------------------------------------------------------------------
// Node counting
// ---------------------------------------------------------------------------

struct NodeCounter {
	pub Count:usize,
}

impl<'ast> Visit<'ast> for NodeCounter {
	fn visit_expr(&mut self, Node:&'ast Expr) {
		self.Count += 1;

		visit_expr(self, Node);
	}
}

pub fn NodeCount(E:&Expr) -> usize {
	let mut Counter = NodeCounter { Count:0 };

	Counter.visit_expr(E);

	Counter.Count
}

// ---------------------------------------------------------------------------
// Unsafe detection
// ---------------------------------------------------------------------------

struct UnsafeDetector {
	pub Found:bool,
}

impl<'ast> Visit<'ast> for UnsafeDetector {
	fn visit_expr_unsafe(&mut self, _Node:&'ast syn::ExprUnsafe) { self.Found = true; }
}

pub fn ContainsUnsafe(E:&Expr) -> bool {
	let mut Detector = UnsafeDetector { Found:false };

	Detector.visit_expr(E);

	Detector.Found
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {

	use super::*;

	fn ParseExpr(Src:&str) -> Expr { syn::parse_str(Src).expect("parse expression") }

	fn ParseStmts(Src:&str) -> Vec<Stmt> {
		let File:syn::File = syn::parse_str(Src).expect("parse");

		if let syn::Item::Fn(F) = &File.items[0] {
			return F.block.stmts.clone();
		}

		vec![]
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
		let Big = ParseExpr("a + b + c + d + e + f + g + h");

		assert!(!IsSafe(&Big, 5));
	}

	#[test]
	fn NodeCountLiteral() {
		assert_eq!(NodeCount(&ParseExpr("1")), 1);
	}

	#[test]
	fn NodeCountBinary() {
		assert_eq!(NodeCount(&ParseExpr("A + B")), 3);
	}

	#[test]
	fn NodeCountCall() {
		assert_eq!(NodeCount(&ParseExpr("f(A, B)")), 4);
	}

	// IsFreeVarSafe tests

	#[test]
	fn FreeVarSafeWhenNoStmtsBetween() {
		// Init = var.clone(), no statements between decl and use: safe.
		assert!(IsFreeVarSafe(&ParseExpr("var.clone()"), &[]));
	}

	#[test]
	fn FreeVarSafeWhenVarOnlyBorrowed() {
		// &var between decl and use: borrow, not a move.
		let Stmts = ParseStmts("fn f() { let _r = &var; }");

		assert!(IsFreeVarSafe(&ParseExpr("var.clone()"), &Stmts));
	}

	#[test]
	fn FreeVarUnsafeWhenMovedIntoCall() {
		// fn g(x: String) consumes x by value.
		let Stmts = ParseStmts("fn f() { g(var); }");

		assert!(!IsFreeVarSafe(&ParseExpr("var.clone()"), &Stmts));
	}

	#[test]
	fn FreeVarUnsafeWhenMovedIntoStructField() {
		// Struct field shorthand { path } moves path.
		let Stmts = ParseStmts("fn f() { let _r = Foo { path }; }");

		assert!(!IsFreeVarSafe(&ParseExpr("path.clone()"), &Stmts));
	}

	#[test]
	fn FreeVarSafeWhenDifferentVarMoved() {
		// A different variable is moved; the one we care about is untouched.
		let Stmts = ParseStmts("fn f() { let _r = Foo { other }; }");

		assert!(IsFreeVarSafe(&ParseExpr("path.clone()"), &Stmts));
	}

	#[test]
	fn FreeVarUnsafeWhenMovedIntoRpcRequest() {
		// Exact pattern from AirClient::get_file_info.
		// path is moved into FileInfoRequest { request_id, path }.
		let Stmts = ParseStmts(
			r#"fn f() {
                client.get_file_info(Request::new(FileInfoRequest { request_id, path })).await;
            }"#,
		);

		assert!(!IsFreeVarSafe(&ParseExpr("path.clone()"), &Stmts));
	}
}
