//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Inline.rs
//=============================================================================//
// Module: Inline - VisitMut transformer that eliminates single-use bindings
//
// Algorithm (per block, bottom-up):
//   1. Collect structurally eligible let-binding candidates (Collect).
//   2. For each candidate (in declaration order):
//      a. Count references in subsequent statements (Count).
//      b. Skip if count ≠ 1, used-in-closure, or initialiser is unsafe/large.
//      c. Substitute the single reference with the initialiser (SubstituteRef).
//      d. Remove the let statement.
//      e. Set Changed = true and restart candidate collection.
//   3. Wrap substituted expressions in parentheses when placed as a direct
//      operand of a binary or unary expression and the replacement is a
//      compound expression (preserves operator precedence).
//=============================================================================//

use syn::{
	Expr,
	Stmt,
	visit_mut::{VisitMut, visit_block_mut, visit_expr_mut},
};

use super::{Collect, Count, Safe};

// ---------------------------------------------------------------------------
// Public: Eliminator
// ---------------------------------------------------------------------------

/// `VisitMut` implementation that eliminates single-use `let` bindings in
/// every block it visits.  Bottom-up traversal ensures inner blocks are
/// processed before outer ones.
pub struct Eliminator<'a> {
	pub Changed: bool,
	Options: &'a crate::Eliminate::Definition::Options,
}

impl<'a> Eliminator<'a> {
	pub fn new(Options: &'a crate::Eliminate::Definition::Options) -> Self {
		Self { Changed: false, Options }
	}

	fn EliminateBlock(&mut self, Block: &mut syn::Block) {
		// Repeat until a full pass finds nothing to eliminate.
		loop {
			let Candidates = Collect::Collect(Block, self.Options.InlineComments);

			let mut DidChange = false;

			for Candidate in &Candidates {
				// Safety check on the initialiser.
				if !Safe::IsSafe(&Candidate.Init, self.Options.MaxSize) {
					continue;
				}

				// Count references in the statements that follow the let.
				let (RefCount, InClosure) =
					Count::CountReferences(&Candidate.Ident, &Block.stmts[Candidate.StmtIndex + 1..]);

				if RefCount != 1 || InClosure {
					continue;
				}

				// Attempt substitution.  Returns false only when the counter
				// over-counted (can happen with deeply nested inner-block shadows).
				let Substituted = SubstituteRef(
					&mut Block.stmts[Candidate.StmtIndex + 1..],
					&Candidate.Ident,
					&Candidate.Init,
				);

				if Substituted {
					Block.stmts.remove(Candidate.StmtIndex);

					self.Changed = true;

					DidChange = true;

					break; // candidates are now stale - restart
				}
			}

			if !DidChange {
				break;
			}
		}
	}
}

impl<'a> VisitMut for Eliminator<'a> {
	fn visit_block_mut(&mut self, Block: &mut syn::Block) {
		// Bottom-up: process inner blocks first.
		visit_block_mut(self, Block);

		// Then process this block.
		self.EliminateBlock(Block);
	}
}

// ---------------------------------------------------------------------------
// Public: SubstituteRef
// ---------------------------------------------------------------------------

/// Replace the first (and only expected) occurrence of `Target` in `Stmts`
/// with `Replacement`.
///
/// Returns `true` when substitution succeeded.
pub fn SubstituteRef(Stmts: &mut [Stmt], Target: &str, Replacement: &Expr) -> bool {
	let mut Sub = Substitutor {
		Target,
		Replacement,
		Substituted: false,
		InBinaryOperandPosition: false,
	};

	for Stmt in Stmts.iter_mut() {
		if Sub.Substituted {
			break;
		}

		Sub.visit_stmt_mut(Stmt);
	}

	Sub.Substituted
}

// ---------------------------------------------------------------------------
// Internal: Substitutor
// ---------------------------------------------------------------------------

struct Substitutor<'a> {
	Target: &'a str,
	Replacement: &'a Expr,
	Substituted: bool,
	/// True when the current position is a direct operand of a binary or unary
	/// expression.  Used to decide whether to wrap `Replacement` in parens.
	InBinaryOperandPosition: bool,
}

impl<'a> VisitMut for Substitutor<'a> {
	fn visit_expr_mut(&mut self, Node: &mut Expr) {
		if self.Substituted {
			return;
		}

		// Check whether this expression IS the target identifier.
		if IsTargetIdent(Node, self.Target) {
			let NeedsWrapping = self.InBinaryOperandPosition && NeedsParen(self.Replacement);

			*Node = if NeedsWrapping {
				Expr::Paren(syn::ExprParen {
					attrs: vec![],
					paren_token: Default::default(),
					expr: Box::new(self.Replacement.clone()),
				})
			} else {
				self.Replacement.clone()
			};

			self.Substituted = true;

			return;
		}

		// Recurse into children, tracking the binary-operand-position context.
		match Node {
			Expr::Binary(B) => {
				let Saved = self.InBinaryOperandPosition;

				self.InBinaryOperandPosition = true;

				self.visit_expr_mut(&mut B.left);

				if !self.Substituted {
					self.visit_expr_mut(&mut B.right);
				}

				self.InBinaryOperandPosition = Saved;
			},

			Expr::Unary(U) => {
				let Saved = self.InBinaryOperandPosition;

				self.InBinaryOperandPosition = true;

				self.visit_expr_mut(&mut U.expr);

				self.InBinaryOperandPosition = Saved;
			},

			_ => {
				let Saved = self.InBinaryOperandPosition;

				self.InBinaryOperandPosition = false;

				visit_expr_mut(self, Node);

				self.InBinaryOperandPosition = Saved;
			},
		}
	}

	// Skip inner blocks that shadow Target - mirrors the Count logic so that
	// substitution and counting are always consistent.
	fn visit_block_mut(&mut self, Block: &mut syn::Block) {
		if BlockShadowsTarget(&Block.stmts, self.Target) {
			return;
		}

		syn::visit_mut::visit_block_mut(self, Block);
	}

	// Skip closures whose parameter shadows Target.
	fn visit_expr_closure_mut(&mut self, Node: &mut syn::ExprClosure) {
		if ClosureParamShadows(Node, self.Target) {
			return;
		}

		syn::visit_mut::visit_expr_closure_mut(self, Node);
	}
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn IsTargetIdent(E: &Expr, Target: &str) -> bool {
	if let Expr::Path(ExprPath) = E {
		if ExprPath.qself.is_none() {
			if let Some(Ident) = ExprPath.path.get_ident() {
				return Ident == Target;
			}
		}
	}

	false
}

/// Returns `true` for compound expressions that need parentheses when placed
/// as a direct operand of a binary or unary expression.
fn NeedsParen(E: &Expr) -> bool {
	matches!(
		E,
		Expr::Binary(_) | Expr::Range(_) | Expr::Closure(_) | Expr::Cast(_)
	)
}

fn BlockShadowsTarget(Stmts: &[Stmt], Target: &str) -> bool {
	Stmts.iter().any(|S| {
		if let Stmt::Local(L) = S {
			if let syn::Pat::Ident(P) = &L.pat {
				return P.ident == Target;
			}
		}

		false
	})
}

fn ClosureParamShadows(Closure: &syn::ExprClosure, Target: &str) -> bool {
	Closure.inputs.iter().any(|P| {
		if let syn::Pat::Ident(P) = P {
			P.ident == Target
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

	fn Transform(Src: &str) -> String {
		let Opts = crate::Eliminate::Definition::Options::default();

		crate::Eliminate::Transform::Run(Src, &Opts)
			.expect("transform failed")
			.unwrap_or_else(|| {
				// Already minimal - return normalised version for comparison.
				let Ast: syn::File = syn::parse_str(Src).unwrap();

				prettyplease::unparse(&Ast)
			})
	}

	fn Normalise(Src: &str) -> String {
		let Ast: syn::File = syn::parse_str(Src).unwrap();

		prettyplease::unparse(&Ast)
	}

	fn AssertEliminates(Input: &str, Expected: &str) {
		assert_eq!(Transform(Input), Normalise(Expected));
	}

	fn AssertUnchanged(Input: &str) {
		let Opts = crate::Eliminate::Definition::Options::default();

		let Result = crate::Eliminate::Transform::Run(Input, &Opts).expect("transform");

		assert!(Result.is_none(), "expected no change but got:\n{}", Result.unwrap());
	}

	// --- Simple cases -------------------------------------------------------

	#[test]
	fn SimpleInline() {
		AssertEliminates(
			"fn f() { let X = 5; println!(\"{}\", X); }",
			"fn f() { println!(\"{}\", 5); }",
		);
	}

	#[test]
	fn ChainInline() {
		// Two passes: A → 1, then B → 1 + 1.
		AssertEliminates(
			"fn f() { let A = 1; let B = A + 1; g(B); }",
			"fn f() { g(1 + 1); }",
		);
	}

	#[test]
	fn StructInlined() {
		AssertEliminates(
			"fn f() { let Opts = MyOpts { a: 1 }; call(Opts); }",
			"fn f() { call(MyOpts { a: 1 }); }",
		);
	}

	#[test]
	fn QuestionMarkInlined() {
		AssertEliminates(
			"async fn f() -> Result<(), E> { let X = foo().map_err(|e| e)?; bar(X); Ok(()) }",
			"async fn f() -> Result<(), E> { bar(foo().map_err(|e| e)?); Ok(()) }",
		);
	}

	#[test]
	fn MatchExprInlined() {
		AssertEliminates(
			"fn f() { let R = match x { 1 => true, _ => false }; use_result(R); }",
			"fn f() { use_result(match x { 1 => true, _ => false }); }",
		);
	}

	#[test]
	fn BorrowInlined() {
		AssertEliminates(
			"fn f() { let X = &foo; bar(X); }",
			"fn f() { bar(&foo); }",
		);
	}

	// --- Binary expression parenthesisation --------------------------------

	#[test]
	fn BinaryExprParens() {
		// let X = A + B; Y * X  →  Y * (A + B)
		AssertEliminates(
			"fn f() { let X = A + B; let _ = Y * X; }",
			"fn f() { let _ = Y * (A + B); }",
		);
	}

	#[test]
	fn BinaryExprNoParensInFnArg() {
		// In a function-argument position, no parens needed.
		AssertEliminates(
			"fn f() { let X = A + B; foo(X); }",
			"fn f() { foo(A + B); }",
		);
	}

	// --- Kept as-is cases ---------------------------------------------------

	#[test]
	fn MultiUseKept() {
		AssertUnchanged("fn f() { let X = foo(); bar(X); baz(X); }");
	}

	#[test]
	fn MutKept() {
		AssertUnchanged("fn f() { let mut X = 5; X += 1; g(X); }");
	}

	#[test]
	fn DestructuringKept() {
		AssertUnchanged("fn f() { let (A, B) = pair; g(A); }");
	}

	#[test]
	fn ClosureCaptureKept() {
		AssertUnchanged("fn f() { let X = heavy(); let F = move || X; call(F); }");
	}

	#[test]
	fn ShadowFirstThenInline() {
		// The first X (= 1) is used once before the shadow; second X (= 2) is
		// used once after.  Both should inline.
		AssertEliminates(
			"fn f() { let X = 1; f(X); let X = 2; g(X); }",
			"fn f() { f(1); g(2); }",
		);
	}

	// --- URL-parsing pattern (the motivating example) -----------------------

	#[test]
	fn UrlPatternInlined() {
		let Input = r#"
            pub async fn Fn(
                Service: &CocoonServiceImpl,
                Request: ProvideCodeLensesRequest,
            ) -> Result<Response<ProvideCodeLensesResponse>, Status> {
                let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");
                let DocumentURI = Url::parse(URI)
                    .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;
                match Service.environment.ProvideCodeLenses(DocumentURI).await {
                    Ok(_) => Ok(Response::new(ProvideCodeLensesResponse::default())),
                    Err(Error) => Err(Status::internal(format!("Code lenses failed: {}", Error))),
                }
            }
        "#;

		let Expected = r#"
            pub async fn Fn(
                Service: &CocoonServiceImpl,
                Request: ProvideCodeLensesRequest,
            ) -> Result<Response<ProvideCodeLensesResponse>, Status> {
                match Service
                    .environment
                    .ProvideCodeLenses(
                        Url::parse(
                            Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
                        )
                        .map_err(|E| {
                            Status::invalid_argument(format!("Invalid URI: {}", E))
                        })?,
                    )
                    .await
                {
                    Ok(_) => Ok(Response::new(ProvideCodeLensesResponse::default())),
                    Err(Error) => Err(Status::internal(format!("Code lenses failed: {}", Error))),
                }
            }
        "#;

		// Compare after normalisation so whitespace differences don't matter.
		let Got = Transform(Input);
		let Norm = Normalise(Expected);
		assert_eq!(Got, Norm, "URL pattern not inlined as expected");
	}

	// --- Idempotency --------------------------------------------------------

	#[test]
	fn AlreadyMinimalReturnsNone() {
		let Opts = crate::Eliminate::Definition::Options::default();

		// A file with no single-use variables.
		let Src = "fn f() { let X = foo(); bar(X); baz(X); }";

		let Result = crate::Eliminate::Transform::Run(Src, &Opts).unwrap();

		assert!(Result.is_none());
	}

	#[test]
	fn Idempotent() {
		let Opts = crate::Eliminate::Definition::Options::default();

		let Src = "fn f() { let X = 5; println!(\"{}\", X); }";

		let First = crate::Eliminate::Transform::Run(Src, &Opts)
			.unwrap()
			.expect("first pass should change");

		let Second = crate::Eliminate::Transform::Run(&First, &Opts).unwrap();

		assert!(Second.is_none(), "second pass should be idempotent");
	}
}
