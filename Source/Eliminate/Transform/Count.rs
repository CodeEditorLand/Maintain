//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Count.rs
//=============================================================================//
// Module: Count - Reference counting for let-binding candidates
//
// Counts how many times a named identifier is referenced in a slice of
// statements, respecting:
//   - Top-level shadowing: a second `let <target> = ...` at the same scope
//     depth stops the count.
//   - Inner-block shadowing: if an inner block re-introduces the name, all
//     references inside that block are excluded (conservative).
//   - Macro token streams: identifiers inside json!(), dev_log!(), and other
//     macro invocations are counted via raw token-tree scanning.
//   - Closure captures: references inside a closure body set InClosure.
//   - Loop bodies: references inside for/while/loop bodies set InLoop.
//     Callers treat such bindings as non-inlinable because inlining would
//     move the initialiser expression inside the loop, changing evaluation
//     semantics (runs N times instead of once).
//=============================================================================//

use proc_macro2::{TokenStream, TokenTree};
use syn::{
	Pat,
	Stmt,
	visit::{Visit, visit_block, visit_expr_closure, visit_expr_for_loop, visit_expr_loop, visit_expr_while},
};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Count references to Target in Stmts.
///
/// Returns (count, in_closure, in_loop).
///
/// - count      : number of times the identifier is referenced (plain
///                expressions and macro token streams).
/// - in_closure : true when at least one reference occurs inside a closure
///                body (even if count == 1).
/// - in_loop    : true when at least one reference occurs inside the body of
///                a for, while, or loop expression (even if count == 1).
///
/// Counting stops when a top-level `let <target> = ...` shadow is encountered.
pub fn CountReferences(Target:&str, Stmts:&[Stmt]) -> (usize, bool, bool) {
	let mut TotalCount = 0usize;
	let mut InClosure = false;
	let mut InLoop = false;

	for Stmt in Stmts {
		if IsTopLevelShadow(Stmt, Target) {
			break;
		}

		let mut Counter = ExprCounter { Target, Count:0, InClosure:false, InLoop:false, InsideLoop:false };

		Counter.visit_stmt(Stmt);

		TotalCount += Counter.Count;

		if Counter.InClosure {
			InClosure = true;
		}

		if Counter.InLoop {
			InLoop = true;
		}
	}

	(TotalCount, InClosure, InLoop)
}

// ---------------------------------------------------------------------------
// Internals
// ---------------------------------------------------------------------------

fn IsTopLevelShadow(Stmt:&Stmt, Target:&str) -> bool {
	if let Stmt::Local(Local) = Stmt {
		if let Pat::Ident(P) = &Local.pat {
			return P.ident == Target;
		}
	}

	false
}

struct ExprCounter<'a> {
	Target:&'a str,
	Count:usize,
	/// True when a reference to Target was found inside a closure body.
	InClosure:bool,
	/// True when a reference to Target was found inside a loop body.
	InLoop:bool,
	/// Internal flag: true when the visitor is currently descending inside
	/// a for/while/loop body.
	InsideLoop:bool,
}

impl<'ast> Visit<'ast> for ExprCounter<'ast> {
	fn visit_expr_path(&mut self, Node:&'ast syn::ExprPath) {
		if let Some(Ident) = Node.path.get_ident() {
			if Ident == self.Target {
				self.Count += 1;

				if self.InsideLoop {
					self.InLoop = true;
				}
			}
		}
	}

	fn visit_expr_macro(&mut self, Node:&'ast syn::ExprMacro) {
		let Found = CountIdentsInTokenStream(&Node.mac.tokens, self.Target);

		if Found > 0 {
			self.Count += Found;

			if self.InsideLoop {
				self.InLoop = true;
			}
		}
	}

	fn visit_stmt_macro(&mut self, Node:&'ast syn::StmtMacro) {
		let Found = CountIdentsInTokenStream(&Node.mac.tokens, self.Target);

		if Found > 0 {
			self.Count += Found;

			if self.InsideLoop {
				self.InLoop = true;
			}
		}
	}

	fn visit_block(&mut self, Node:&'ast syn::Block) {
		if BlockShadowsTarget(&Node.stmts, self.Target) {
			return;
		}

		visit_block(self, Node);
	}

	fn visit_expr_closure(&mut self, Node:&'ast syn::ExprClosure) {
		if ClosureParamShadows(Node, self.Target) {
			return;
		}

		let CountBefore = self.Count;

		visit_expr_closure(self, Node);

		if self.Count > CountBefore {
			self.InClosure = true;
		}
	}

	// Set InsideLoop before descending into the three loop-body variants.

	fn visit_expr_for_loop(&mut self, Node:&'ast syn::ExprForLoop) {
		let Saved = self.InsideLoop;

		self.InsideLoop = true;

		visit_expr_for_loop(self, Node);

		self.InsideLoop = Saved;
	}

	fn visit_expr_while(&mut self, Node:&'ast syn::ExprWhile) {
		let Saved = self.InsideLoop;

		self.InsideLoop = true;

		visit_expr_while(self, Node);

		self.InsideLoop = Saved;
	}

	fn visit_expr_loop(&mut self, Node:&'ast syn::ExprLoop) {
		let Saved = self.InsideLoop;

		self.InsideLoop = true;

		visit_expr_loop(self, Node);

		self.InsideLoop = Saved;
	}
}

pub fn CountIdentsInTokenStream(Tokens:&TokenStream, Target:&str) -> usize {
	let mut Count = 0;

	for Tree in Tokens.clone() {
		match Tree {
			TokenTree::Ident(I) if I.to_string() == Target => Count += 1,

			TokenTree::Group(G) => Count += CountIdentsInTokenStream(&G.stream(), Target),

			TokenTree::Literal(Lit) => Count += CountIdentInFormatLiteral(&Lit.to_string(), Target),

			_ => {},
		}
	}

	Count
}

pub fn CountIdentInFormatLiteral(Lit:&str, Target:&str) -> usize {
	if !Lit.starts_with('"') && !Lit.starts_with('r') {
		return 0;
	}

	let Inner:&str = if Lit.starts_with('"') {
		&Lit[1..Lit.len().saturating_sub(1)]
	} else {
		Lit
	};

	let SearchFor = format!("{{{Target}");

	let mut Count = 0;
	let Bytes = Inner.as_bytes();
	let PatBytes = SearchFor.as_bytes();
	let mut Pos = 0usize;

	while Pos + PatBytes.len() <= Bytes.len() {
		if Bytes[Pos..].starts_with(PatBytes) {
			let IsEscaped = Pos > 0 && Bytes[Pos - 1] == b'{';

			if !IsEscaped {
				let After = &Bytes[Pos + PatBytes.len()..];

				if After.first().map_or(false, |&B| B == b'}' || B == b':' || B == b'!') {
					Count += 1;
				}
			}
		}

		Pos += 1;
	}

	Count
}

fn BlockShadowsTarget(Stmts:&[Stmt], Target:&str) -> bool { Stmts.iter().any(|S| IsTopLevelShadow(S, Target)) }

fn ClosureParamShadows(Closure:&syn::ExprClosure, Target:&str) -> bool {
	Closure
		.inputs
		.iter()
		.any(|P| if let Pat::Ident(PIdent) = P { PIdent.ident == Target } else { false })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {
	use super::*;

	fn Stmts(Src:&str) -> Vec<Stmt> {
		let File:syn::File = syn::parse_str(Src).expect("parse");

		if let syn::Item::Fn(F) = &File.items[0] {
			return F.block.stmts.clone();
		}

		vec![]
	}

	#[test]
	fn SingleUse() {
		let S = Stmts("fn f() { let X = 1; g(X); }");
		let (Count, InClosure, InLoop) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 1);
		assert!(!InClosure);
		assert!(!InLoop);
	}

	#[test]
	fn MultiUse() {
		let S = Stmts("fn f() { let X = foo(); bar(X); baz(X); }");
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 2);
	}

	#[test]
	fn ZeroUse() {
		let S = Stmts("fn f() { let X = 1; g(1); }");
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 0);
	}

	#[test]
	fn ShadowStops() {
		let S = Stmts("fn f() { let X = 1; f(X); let X = 2; g(X); }");
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 1);
	}

	#[test]
	fn MacroCountsAsUse() {
		let S = Stmts(r#"fn f() { let URI = "x"; dev_log!("{}", URI); }"#);
		let (Count, _, _) = CountReferences("URI", &S[1..]);
		assert_eq!(Count, 1);
	}

	#[test]
	fn MacroAndExprBothCounted() {
		let S = Stmts(
			r#"fn f() {
                let URI = "x";
                dev_log!("{}", URI);
                let _ = Url::parse(URI);
            }"#,
		);
		let (Count, _, _) = CountReferences("URI", &S[1..]);
		assert_eq!(Count, 2, "URI used in macro + expression must count as 2");
	}

	#[test]
	fn MacroOnlyUse() {
		let S = Stmts(
			r#"fn f() {
                let DataString = compute();
                emit(json!({ "data": DataString }));
            }"#,
		);
		let (Count, _, _) = CountReferences("DataString", &S[1..]);
		assert_eq!(Count, 1);
	}

	#[test]
	fn MacroDoubleUse() {
		let S = Stmts(
			r#"fn f() {
                let X = val();
                json!({ "a": X, "b": X });
            }"#,
		);
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 2);
	}

	#[test]
	fn ClosureCaptureDetected() {
		let S = Stmts("fn f() { let X = heavy(); let F = move || X; call(F); }");
		let (Count, InClosure, _) = CountReferences("X", &S[1..]);
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
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 0);
	}

	#[test]
	fn ClosureParamShadowSkipped() {
		let S = Stmts("fn f() { let X = 5; let _ = |X| X + 1; g(0); }");
		let (Count, InClosure, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 0);
		assert!(!InClosure);
	}

	// Loop-body tests

	/// X used only inside a for loop body: count=1 but InLoop=true.
	/// The binding must not be inlined because that would move the initialiser
	/// expression inside the loop, changing evaluation semantics.
	#[test]
	fn LoopBodySetsInLoop() {
		let S = Stmts(
			r#"fn f() {
                let X = expensive();
                for _ in &v { process(X); }
            }"#,
		);
		let (Count, _, InLoop) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 1);
		assert!(InLoop, "X used inside for body must set InLoop");
	}

	#[test]
	fn WhileBodySetsInLoop() {
		let S = Stmts(
			r#"fn f() {
                let X = expensive();
                while cond { process(X); }
            }"#,
		);
		let (Count, _, InLoop) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 1);
		assert!(InLoop, "X used inside while body must set InLoop");
	}

	#[test]
	fn LoopExprBodySetsInLoop() {
		let S = Stmts(
			r#"fn f() {
                let X = expensive();
                loop { process(X); break; }
            }"#,
		);
		let (Count, _, InLoop) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 1);
		assert!(InLoop, "X used inside loop body must set InLoop");
	}

	/// X used outside any loop: InLoop must be false.
	#[test]
	fn OutsideLoopNotFlagged() {
		let S = Stmts("fn f() { let X = 1; g(X); }");
		let (_, _, InLoop) = CountReferences("X", &S[1..]);
		assert!(!InLoop);
	}

	/// X used both inside and outside a loop: count=2, InLoop=true.
	/// Still ineligible (count != 1).
	#[test]
	fn UsedInsideAndOutsideLoop() {
		let S = Stmts(
			r#"fn f() {
                let X = val();
                g(X);
                for _ in &v { h(X); }
            }"#,
		);
		let (Count, _, InLoop) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 2);
		assert!(InLoop);
	}

	// Format literal tests (unchanged)

	#[test]
	fn FormatLiteralBasicCapture() { assert_eq!(CountIdentInFormatLiteral("\"{X}\"", "X"), 1); }

	#[test]
	fn FormatLiteralWithSpec() { assert_eq!(CountIdentInFormatLiteral("\"{X:.2}\"", "X"), 1); }

	#[test]
	fn FormatLiteralWithAlt() { assert_eq!(CountIdentInFormatLiteral("\"{X!r:}\"", "X"), 1); }

	#[test]
	fn FormatLiteralTwoCaptures() { assert_eq!(CountIdentInFormatLiteral("\"{X} and {X}\"", "X"), 2); }

	#[test]
	fn FormatLiteralEscapedBraceNotCounted() { assert_eq!(CountIdentInFormatLiteral("\"{{X}}\"", "X"), 0); }

	#[test]
	fn FormatLiteralNumericNotCounted() { assert_eq!(CountIdentInFormatLiteral("42", "X"), 0); }

	#[test]
	fn FormatLiteralSubstringNotCounted() { assert_eq!(CountIdentInFormatLiteral("\"{XY}\"", "X"), 0); }

	#[test]
	fn ImplicitCaptureSingleUse() {
		let S = Stmts(r#"fn f() { let X = 5; println!("{X}"); }"#);
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 1, "implicit capture {X} must count as one use");
	}

	#[test]
	fn MixedImplicitAndExplicitCounts() {
		let S = Stmts(
			r#"fn f() {
                let X = 5;
                println!("{}", X);
                println!("{X}");
            }"#,
		);
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 2, "old-style + implicit capture must count as 2");
	}

	#[test]
	fn TwoImplicitCaptures() {
		let S = Stmts(
			r#"fn f() {
                let X = 5;
                log!("{X}");
                log!("{X}");
            }"#,
		);
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 2);
	}

	#[test]
	fn ImplicitCaptureWithPlainUseIsMulti() {
		let S = Stmts(
			r#"fn f() {
                let URI = compute_uri();
                log!("{URI}");
                Url::parse(URI);
            }"#,
		);
		let (Count, _, _) = CountReferences("URI", &S[1..]);
		assert_eq!(Count, 2);
	}

	#[test]
	fn TwoSeparateStmtsMultiUse() {
		let S = Stmts("fn f() { let X = foo(); bar(X); baz(X); }");
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 2);
	}

	#[test]
	fn TwoUsesInSameCall() {
		let S = Stmts("fn f() { let X = foo(); bar(X, X); }");
		let (Count, _, _) = CountReferences("X", &S[1..]);
		assert_eq!(Count, 2);
	}
}
