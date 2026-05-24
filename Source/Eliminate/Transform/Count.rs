//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Count.rs
//=============================================================================//
// Module: Count - Reference counting for let-binding candidates
//
// Counts how many times a named identifier is referenced in a slice of
// statements, respecting:
//   - Top-level shadowing: a second `let <target> = …` at the same scope depth
//     stops the count.
//   - Inner-block shadowing: if an inner block re-introduces the name, all
//     references inside that block are excluded (conservative; may under-count
//     but never over-counts).
//   - Macro token streams: identifiers inside `json!(…)`, `dev_log!(…)`, and
//     other macro invocations are counted via raw token-tree scanning.  This is
//     critical for correctness: without it, a variable used in both a macro and
//     a regular expression would be miscounted as single-use.
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
pub fn CountReferences(Target:&str, Stmts:&[Stmt]) -> (usize, bool) {
	let mut TotalCount = 0usize;
	let mut InClosure = false;

	for Stmt in Stmts {
		// A top-level shadow terminates the count for the outer binding.
		if IsTopLevelShadow(Stmt, Target) {
			break;
		}

		let mut Counter = ExprCounter { Target, Count:0, InClosure:false };

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
	/// True when a reference to `Target` was found inside a closure body.
	InClosure:bool,
}

impl<'ast> Visit<'ast> for ExprCounter<'ast> {
	// Count plain identifier path expressions that match Target.
	fn visit_expr_path(&mut self, Node:&'ast syn::ExprPath) {
		if let Some(Ident) = Node.path.get_ident() {
			if Ident == self.Target {
				self.Count += 1;
			}
		}
	}

	// Count identifier occurrences inside macro token streams (e.g. json!(…),
	// dev_log!(…), format!(…)).  The default syn visitor does NOT recurse into
	// Macro::tokens, so we do it manually here.
	//
	// This covers macros that appear in expression position, e.g.:
	//   emit(json!({ "data": X }))
	fn visit_expr_macro(&mut self, Node:&'ast syn::ExprMacro) {
		self.Count += CountIdentsInTokenStream(&Node.mac.tokens, self.Target);
	}

	// syn v2 has a SEPARATE `Stmt::Macro` variant for top-level macro
	// invocation statements (e.g. `dev_log!("{}", URI);`).  These are NOT
	// represented as `Stmt::Expr(Expr::Macro, semi)` and therefore the
	// `visit_expr_macro` override above is never reached for them.
	fn visit_stmt_macro(&mut self, Node:&'ast syn::StmtMacro) {
		self.Count += CountIdentsInTokenStream(&Node.mac.tokens, self.Target);
	}

	// Skip inner blocks that would shadow Target - conservative, avoids
	// counting references that actually belong to the inner binding.
	fn visit_block(&mut self, Node:&'ast syn::Block) {
		if BlockShadowsTarget(&Node.stmts, self.Target) {
			return; // skip entire inner block
		}

		visit_block(self, Node);
	}

	// References inside closure bodies are flagged so callers can conservatively
	// decline inlining (move vs. capture semantics differ).
	//
	// IMPORTANT: only set InClosure = true when Target was ACTUALLY found inside
	// this closure body.  Setting it unconditionally would taint variables that
	// appear *outside* the closure in the same expression (e.g. the `URI` in
	// `Url::parse(URI).map_err(|E| /* no URI here */ ...)`).
	fn visit_expr_closure(&mut self, Node:&'ast syn::ExprClosure) {
		// If the closure itself shadows Target via a parameter, skip the body.
		if ClosureParamShadows(Node, self.Target) {
			return;
		}

		let CountBefore = self.Count;

		visit_expr_closure(self, Node);

		// Only mark InClosure if Target was actually referenced inside THIS body.
		if self.Count > CountBefore {
			self.InClosure = true;
		}
	}
}

/// Recursively count occurrences of `Target` as an `Ident` token inside a
/// raw `TokenStream`.  This covers macro arguments that are otherwise opaque
/// to syn's AST visitor.
///
/// Also handles Rust 1.58+ implicit format-string captures: in
/// `format!("{Target}")` the identifier does NOT appear as a separate
/// `TokenTree::Ident` - it is embedded in the string literal `"{Target}"`.
/// Scanning the literal prevents the mixed-usage bug where
/// `let X = 5; println!("{}", X); println!("{X}")` would be counted as
/// single-use (X as a bare token once) and incorrectly inlined.
pub fn CountIdentsInTokenStream(Tokens:&TokenStream, Target:&str) -> usize {
	let mut Count = 0;

	for Tree in Tokens.clone() {
		match Tree {
			// Use .to_string() explicitly - proc_macro2::Ident's PartialEq<str>
			// has subtleties around &str vs str deref coercion that produce
			// incorrect results in non-proc-macro (library) contexts.
			TokenTree::Ident(I) if I.to_string() == Target => Count += 1,

			TokenTree::Group(G) => Count += CountIdentsInTokenStream(&G.stream(), Target),

			// Scan string literals for `{Target}` / `{Target:…}` implicit
			// format-string captures.  These appear as a single Literal token
			// rather than a separate Ident token, so the arm above misses them.
			TokenTree::Literal(Lit) => Count += CountIdentInFormatLiteral(&Lit.to_string(), Target),

			_ => {},
		}
	}

	Count
}

/// Scan a string-literal token (including its surrounding quote characters)
/// for Rust 1.58+ implicit-capture patterns such as `{Target}` or `{Target:…}`.
///
/// Only processes double-quoted string literals; returns 0 for char literals,
/// integer/float literals, and similar non-string tokens.
pub fn CountIdentInFormatLiteral(Lit:&str, Target:&str) -> usize {
	// Double-quoted string literals start with '"'.
	// Raw strings start with 'r' (e.g. `r"..."` or `r#"..."#`).
	// Char literals start with '\'' - skip those.
	// All other literal kinds (numbers) are irrelevant.
	if !Lit.starts_with('"') && !Lit.starts_with('r') {
		return 0;
	}

	// Strip the outermost quotes/hashes to get the inner content.
	let Inner:&str = if Lit.starts_with('"') {
		// Normal string: strip leading `"` and trailing `"`.
		&Lit[1..Lit.len().saturating_sub(1)]
	} else {
		// Raw string r"..." or r#"..."# - just scan the whole token;
		// the literal braces won't be mistaken for format specifiers.
		Lit
	};

	// Pattern: `{Target` immediately followed by `}`, `:`, or `!`.
	// Examples that match: `{X}`, `{X:.2f}`, `{X!r:}`.
	// Examples that don't match (false captures in double `{{`):
	//   `{{X}}` - the leading `{{` would produce `{X` at position 1, but
	//   the preceding char is `{` not a word boundary; we guard against
	//   this by checking that the character *before* our match is not `{`.
	let SearchFor = format!("{{{Target}");

	let mut Count = 0;
	let Bytes = Inner.as_bytes();
	let PatBytes = SearchFor.as_bytes();

	let mut Pos = 0usize;

	while Pos + PatBytes.len() <= Bytes.len() {
		if Bytes[Pos..].starts_with(PatBytes) {
			// Guard: the `{` we matched must not itself be an escaped `{{`.
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

	/// Variable used twice inside the same macro invocation: count=2, not
	/// eligible.
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

	// -------------------------------------------------------------------------
	// CountIdentInFormatLiteral unit tests
	// -------------------------------------------------------------------------

	/// `{X}` in a format string is one implicit capture of X.
	#[test]
	fn FormatLiteralBasicCapture() {
		assert_eq!(CountIdentInFormatLiteral("\"{X}\"", "X"), 1);
	}

	/// `{X:.2}` - capture with a format specifier.
	#[test]
	fn FormatLiteralWithSpec() {
		assert_eq!(CountIdentInFormatLiteral("\"{X:.2}\"", "X"), 1);
	}

	/// `{X!r:}` - debug alternate.
	#[test]
	fn FormatLiteralWithAlt() {
		assert_eq!(CountIdentInFormatLiteral("\"{X!r:}\"", "X"), 1);
	}

	/// Two `{X}` in one string = count 2.
	#[test]
	fn FormatLiteralTwoCaptures() {
		assert_eq!(CountIdentInFormatLiteral("\"{X} and {X}\"", "X"), 2);
	}

	/// `{{X}}` - double braces escape; the inner X is not a capture.
	#[test]
	fn FormatLiteralEscapedBraceNotCounted() {
		assert_eq!(CountIdentInFormatLiteral("\"{{X}}\"", "X"), 0);
	}

	/// A numeric literal has no captures.
	#[test]
	fn FormatLiteralNumericNotCounted() {
		assert_eq!(CountIdentInFormatLiteral("42", "X"), 0);
	}

	/// Substring match: `{XY}` should NOT count as a use of `X`.
	#[test]
	fn FormatLiteralSubstringNotCounted() {
		assert_eq!(CountIdentInFormatLiteral("\"{XY}\"", "X"), 0);
	}

	// -------------------------------------------------------------------------
	// Implicit-capture integration with CountReferences
	// -------------------------------------------------------------------------

	/// `println!("{X}")` - X used only via implicit capture, counted as 1.
	#[test]
	fn ImplicitCaptureSingleUse() {
		let S = Stmts(r#"fn f() { let X = 5; println!("{X}"); }"#);

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 1, "implicit capture {{X}} must count as one use");
	}

	/// `println!("{}", X); println!("{X}")` - mixed old-style + implicit = 2
	/// uses.  Without this fix, the second use is missed and the binding is
	/// incorrectly inlined, leaving an undefined variable.
	#[test]
	fn MixedImplicitAndExplicitCounts() {
		let S = Stmts(
			r#"fn f() {
                let X = 5;
                println!("{}", X);
                println!("{X}");
            }"#,
		);

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 2, "old-style + implicit capture must count as 2");
	}

	/// Two implicit captures in different macros = 2.
	#[test]
	fn TwoImplicitCaptures() {
		let S = Stmts(
			r#"fn f() {
                let X = 5;
                log!("{X}");
                log!("{X}");
            }"#,
		);

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 2);
	}

	/// A binding used both as a direct identifier AND in an implicit format
	/// capture: multi-use, must not be inlined.
	#[test]
	fn ImplicitCaptureWithPlainUseIsMulti() {
		let S = Stmts(
			r#"fn f() {
                let URI = compute_uri();
                log!("{URI}");
                Url::parse(URI);
            }"#,
		);

		let (Count, _) = CountReferences("URI", &S[1..]);

		assert_eq!(Count, 2);
	}

	/// A binding used only in a loop body: count = 1 (we do not model loops).
	#[test]
	fn LoopBodyCountedAsOne() {
		let S = Stmts(
			r#"fn f() {
                let X = 5;
                for _ in &v { println!("{}", X); }
            }"#,
		);

		let (Count, _) = CountReferences("X", &S[1..]);

		// Tool sees one textual reference; it has no loop-awareness.
		assert_eq!(Count, 1);
	}

	/// A binding used in two separate statements in the same block: 2.
	#[test]
	fn TwoSeparateStmtsMultiUse() {
		let S = Stmts("fn f() { let X = foo(); bar(X); baz(X); }");

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 2);
	}

	/// A binding used twice as arguments in a single call: count = 2.
	#[test]
	fn TwoUsesInSameCall() {
		let S = Stmts("fn f() { let X = foo(); bar(X, X); }");

		let (Count, _) = CountReferences("X", &S[1..]);

		assert_eq!(Count, 2);
	}
}
