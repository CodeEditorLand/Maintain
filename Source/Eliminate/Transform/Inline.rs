//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Inline.rs
//=============================================================================//
// Module: Inline - VisitMut transformer that eliminates single-use bindings
//
// Algorithm (per block, bottom-up):
//   1. Collect structurally eligible let-binding candidates (Collect).
//   2. For each candidate (in declaration order):
//      a. Count references in subsequent statements (Count). This includes
//         references inside macro token streams (json!, dev_log!, format!,
//         etc.) so that multi-use variables are never misidentified as
//         single-use.
//      b. Skip if count != 1, used-in-closure, used-in-loop-body, or
//         initialiser is unsafe/large.
//      c. Skip if any free variable inside the initialiser is moved by value
//         in the statements between the candidate declaration and the
//         substitution site (IsFreeVarSafe). This prevents E0382 borrow-of-
//         moved-value errors introduced by inlining clone() helpers.
//      d. Substitute the single reference with the initialiser (SubstituteRef).
//         Handles both plain expression positions AND macro token streams.
//      e. Remove the let statement.
//      f. Set Changed = true and restart candidate collection.
//   3. Wrap substituted binary/range expressions in parentheses when placed
//      as a direct operand of a binary or unary expression (precedence
//      safety).
//=============================================================================//

use proc_macro2::{Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
	Expr,
	Stmt,
	visit_mut::{VisitMut, visit_block_mut, visit_expr_mut},
};

use super::{Collect, Count, Safe};

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
			let Candidates = Collect::Collect(Block, self.Options.InlineComments);

			let mut DidChange = false;

			for Candidate in &Candidates {
				if !Safe::IsSafe(&Candidate.Init, self.Options.MaxSize) {
					continue;
				}

				let StmtsAfter = &Block.stmts[Candidate.StmtIndex + 1..];

				let (RefCount, InClosure, InLoop) =
					Count::CountReferences(&Candidate.Ident, StmtsAfter);

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

// ---------------------------------------------------------------------------
// Public: SubstituteRef
// ---------------------------------------------------------------------------

/// Replace the first occurrence of Target (as a plain identifier expression
/// OR as an identifier token inside a macro's token stream) in Stmts with
/// Replacement. Returns true when the substitution was performed.
pub fn SubstituteRef(Stmts:&mut [Stmt], Target:&str, Replacement:&Expr) -> bool {
	let mut Sub = Substitutor { Target, Replacement, Substituted:false, InBinaryOperandPosition:false };

	for Stmt in Stmts.iter_mut() {
		if Sub.Substituted {
			break;
		}

		Sub.visit_stmt_mut(Stmt);
	}

	Sub.Substituted
}

// ---------------------------------------------------------------------------
// Internal: FindSubstSite
// ---------------------------------------------------------------------------

/// Return the index within Stmts of the first statement that contains a
/// reference to Target. Returns Stmts.len() (one past end) when not found,
/// which causes StmtsBetween to be the full slice - the conservative safe
/// choice.
fn FindSubstSite(Stmts:&[Stmt], Target:&str) -> usize {
	for (I, Stmt) in Stmts.iter().enumerate() {
		let (Count, _, _) = Count::CountReferences(Target, std::slice::from_ref(Stmt));

		if Count > 0 {
			return I;
		}
	}

	Stmts.len()
}

// ---------------------------------------------------------------------------
// Internal: Substitutor
// ---------------------------------------------------------------------------

struct Substitutor<'a> {
	Target:&'a str,
	Replacement:&'a Expr,
	Substituted:bool,
	/// True when the current AST position is a direct operand of a binary or
	/// unary expression - used to decide whether to wrap Replacement.
	InBinaryOperandPosition:bool,
}

impl<'a> VisitMut for Substitutor<'a> {
	fn visit_expr_mut(&mut self, Node:&mut Expr) {
		if self.Substituted {
			return;
		}

		if IsTargetIdent(Node, self.Target) {
			let NeedsWrapping = self.InBinaryOperandPosition && NeedsParen(self.Replacement);

			*Node = if NeedsWrapping {
				Expr::Paren(syn::ExprParen {
					attrs:vec![],
					paren_token:Default::default(),
					expr:Box::new(self.Replacement.clone()),
				})
			} else {
				self.Replacement.clone()
			};

			self.Substituted = true;

			return;
		}

		// Propagate binary-operand context for children.
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

	/// Substitute inside macro token streams (e.g. json!(), dev_log!()).
	/// The default VisitMut does NOT recurse into Macro::tokens.
	fn visit_expr_macro_mut(&mut self, Node:&mut syn::ExprMacro) {
		if self.Substituted {
			return;
		}

		let ReplacementTokens = ExprToTokenStream(self.Replacement);

		let (NewTokens, Found) = SubstituteInTokenStream(Node.mac.tokens.clone(), self.Target, &ReplacementTokens);

		if Found {
			Node.mac.tokens = NewTokens;

			self.Substituted = true;
		}
	}

	/// syn v2 separates top-level macro statements (dev_log!("{}", X);) into
	/// Stmt::Macro(StmtMacro), which is never routed through
	/// visit_expr_macro_mut. Mirror the same substitution here.
	fn visit_stmt_macro_mut(&mut self, Node:&mut syn::StmtMacro) {
		if self.Substituted {
			return;
		}

		let ReplacementTokens = ExprToTokenStream(self.Replacement);

		let (NewTokens, Found) = SubstituteInTokenStream(Node.mac.tokens.clone(), self.Target, &ReplacementTokens);

		if Found {
			Node.mac.tokens = NewTokens;

			self.Substituted = true;
		}
	}

	// Skip inner blocks that shadow Target - mirrors Count logic.
	fn visit_block_mut(&mut self, Block:&mut syn::Block) {
		if BlockShadowsTarget(&Block.stmts, self.Target) {
			return;
		}

		syn::visit_mut::visit_block_mut(self, Block);
	}

	// Skip closures whose parameter shadows Target.
	fn visit_expr_closure_mut(&mut self, Node:&mut syn::ExprClosure) {
		if ClosureParamShadows(Node, self.Target) {
			return;
		}

		syn::visit_mut::visit_expr_closure_mut(self, Node);
	}
}

// ---------------------------------------------------------------------------
// Token-stream helpers
// ---------------------------------------------------------------------------

fn ExprToTokenStream(E:&Expr) -> TokenStream {
	let mut Tokens = TokenStream::new();

	E.to_tokens(&mut Tokens);

	Tokens
}

fn SubstituteInTokenStream(Tokens:TokenStream, Target:&str, Replacement:&TokenStream) -> (TokenStream, bool) {
	let mut Result:Vec<TokenTree> = Vec::new();

	let mut Found = false;

	for Tree in Tokens {
		if Found {
			Result.push(Tree);

			continue;
		}

		match Tree {
			TokenTree::Ident(ref I) if I.to_string() == Target => {
				Result.extend(Replacement.clone());

				Found = true;
			},

			TokenTree::Group(G) => {
				let (NewStream, F) = SubstituteInTokenStream(G.stream(), Target, Replacement);

				if F {
					Found = true;
				}

				Result.push(TokenTree::Group(Group::new(G.delimiter(), NewStream)));
			},

			Other => Result.push(Other),
		}
	}

	(Result.into_iter().collect(), Found)
}

// ---------------------------------------------------------------------------
// Expression helpers
// ---------------------------------------------------------------------------

fn IsTargetIdent(E:&Expr, Target:&str) -> bool {
	if let Expr::Path(ExprPath) = E {
		if ExprPath.qself.is_none() {
			if let Some(Ident) = ExprPath.path.get_ident() {
				return Ident == Target;
			}
		}
	}

	false
}

fn NeedsParen(E:&Expr) -> bool { matches!(E, Expr::Binary(_) | Expr::Range(_) | Expr::Closure(_) | Expr::Cast(_)) }

fn BlockShadowsTarget(Stmts:&[Stmt], Target:&str) -> bool {
	Stmts.iter().any(|S| {
		if let Stmt::Local(L) = S {
			if let syn::Pat::Ident(P) = &L.pat {
				return P.ident == Target;
			}
		}

		false
	})
}

fn ClosureParamShadows(Closure:&syn::ExprClosure, Target:&str) -> bool {
	Closure
		.inputs
		.iter()
		.any(|P| if let syn::Pat::Ident(P) = P { P.ident == Target } else { false })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {
	use super::*;

	fn Transform(Src:&str) -> String {
		let Opts = crate::Eliminate::Definition::Options::default();

		crate::Eliminate::Transform::Run(Src, &Opts)
			.expect("transform failed")
			.unwrap_or_else(|| {
				let Ast:syn::File = syn::parse_str(Src).unwrap();

				prettyplease::unparse(&Ast)
			})
	}

	fn Normalise(Src:&str) -> String {
		let Ast:syn::File = syn::parse_str(Src).unwrap();

		prettyplease::unparse(&Ast)
	}

	fn AssertEliminates(Input:&str, Expected:&str) {
		assert_eq!(Transform(Input), Normalise(Expected));
	}

	fn AssertUnchanged(Input:&str) {
		let Opts = crate::Eliminate::Definition::Options::default();

		let Result = crate::Eliminate::Transform::Run(Input, &Opts).expect("transform");

		assert!(Result.is_none(), "expected no change but got:\n{}", Result.unwrap());
	}

	// --- Simple inline tests ------------------------------------------------

	#[test]
	fn SimpleInline() {
		AssertEliminates(
			r#"fn f() { let X = 5; println!("{}", X); }"#,
			r#"fn f() { println!("{}", 5); }"#,
		);
	}

	#[test]
	fn ChainInline() { AssertEliminates("fn f() { let A = 1; let B = A + 1; g(B); }", "fn f() { g(1 + 1); }"); }

	#[test]
	fn BinaryExprParens() {
		AssertEliminates("fn f() { let X = A + B; let _ = Y * X; }", "fn f() { let _ = Y * (A + B); }");
	}

	#[test]
	fn BinaryExprNoParensInFnArg() { AssertEliminates("fn f() { let X = A + B; foo(X); }", "fn f() { foo(A + B); }"); }

	#[test]
	fn QuestionMarkInlined() {
		AssertEliminates(
			"async fn f() -> Result<(), E> { let X = foo().map_err(|e| e)?; bar(X); Ok(()) }",
			"async fn f() -> Result<(), E> { bar(foo().map_err(|e| e)?); Ok(()) }",
		);
	}

	// --- Macro substitution tests -------------------------------------------

	#[test]
	fn InlineIntoJsonMacro() {
		AssertEliminates(
			r#"fn f() {
                let DataString = compute_data();
                emit(json!({ "data": DataString }));
            }"#,
			r#"fn f() {
                emit(json!({ "data": compute_data() }));
            }"#,
		);
	}

	#[test]
	fn MacroAndExprMultiUseKept() {
		AssertUnchanged(
			r#"fn f() {
                let URI = compute_uri();
                dev_log!("{}", URI);
                let _ = Url::parse(URI);
            }"#,
		);
	}

	#[test]
	fn MacroDoubleUseKept() {
		AssertUnchanged(
			r#"fn f() {
                let X = val();
                emit(json!({ "a": X, "b": X }));
            }"#,
		);
	}

	// --- Loop-body tests (#58) ----------------------------------------------

	/// Binding used only inside a for loop body must not be inlined.
	/// Inlining would move the initialiser inside the loop, running it
	/// N times instead of once and potentially changing semantics or
	/// introducing a compile error for move-only types.
	#[test]
	fn LoopBodyBindingKept() {
		AssertUnchanged(
			r#"fn f() {
                let X = expensive();
                for item in &collection { process(item, X); }
            }"#,
		);
	}

	#[test]
	fn WhileBodyBindingKept() {
		AssertUnchanged(
			r#"fn f() {
                let X = expensive();
                while cond { process(X); }
            }"#,
		);
	}

	#[test]
	fn LoopExprBindingKept() {
		AssertUnchanged(
			r#"fn f() {
                let X = expensive();
                loop { process(X); break; }
            }"#,
		);
	}

	/// A binding used outside any loop must still be inlined normally.
	#[test]
	fn OutsideLoopStillInlined() {
		AssertEliminates(
			"fn f() { let X = compute(); g(X); }",
			"fn f() { g(compute()); }",
		);
	}

	// --- Free-variable move-safety tests (regression for #56) ---------------

	/// display = path.clone() must NOT be inlined when path is moved into a
	/// struct field between the declaration and the devlog use site.
	/// This is the exact pattern from AirClient::get_file_info that produced
	/// E0382 after eliminate ran on Source/Air.
	#[test]
	fn DisplayCloneKeptWhenOriginalMovedFirst() {
		AssertUnchanged(
			r#"
				pub async fn get_file_info(request_id: String, path: String) -> Result<(), E> {
					let path_display = path.clone();
					client
						.get_file_info(Request::new(FileInfoRequest { request_id, path }))
						.await?;
					devlog!("{}", path_display, path.clone());
					Ok(())
				}
			"#,
		);
	}

	/// Same pattern with section instead of path (AirClient::get_configuration).
	#[test]
	fn SectionDisplayCloneKeptWhenOriginalMovedFirst() {
		AssertUnchanged(
			r#"
				pub async fn get_configuration(request_id: String, section: String) -> Result<(), E> {
					let section_display = section.clone();
					client
						.get_configuration(Request::new(ConfigurationRequest { request_id, section }))
						.await?;
					devlog!("{}", section_display, section.clone());
					Ok(())
				}
			"#,
		);
	}

	/// When path is NOT moved between decl and use, the clone helper should
	/// still be inlined (no false positive that would block legitimate inlines).
	#[test]
	fn DisplayCloneInlinedWhenOriginalNotMoved() {
		AssertEliminates(
			r#"
				pub async fn log_path(path: String) -> Result<(), E> {
					let path_display = path.clone();
					devlog!("{}", path_display);
					Ok(())
				}
			"#,
			r#"
				pub async fn log_path(path: String) -> Result<(), E> {
					devlog!("{}", path.clone());
					Ok(())
				}
			"#,
		);
	}

	// --- URL-parsing pattern ------------------------------------------------

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
                        .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?,
                    )
                    .await
                {
                    Ok(_) => Ok(Response::new(ProvideCodeLensesResponse::default())),
                    Err(Error) => Err(Status::internal(format!("Code lenses failed: {}", Error))),
                }
            }
        "#;

		let Got = Transform(Input);
		let Norm = Normalise(Expected);
		assert_eq!(Got, Norm, "URL pattern not inlined as expected");
	}

	// --- Kept-as-is tests ---------------------------------------------------

	#[test]
	fn MultiUseKept() { AssertUnchanged("fn f() { let X = foo(); bar(X); baz(X); }"); }

	#[test]
	fn MutKept() { AssertUnchanged("fn f() { let mut X = 5; X += 1; g(X); }"); }

	#[test]
	fn ClosureCaptureKept() {
		AssertEliminates(
			"fn f() { let X = heavy(); let F = move || X; call(F); }",
			"fn f() { let X = heavy(); call(move || X); }",
		);
	}

	// --- Idempotency --------------------------------------------------------

	#[test]
	fn Idempotent() {
		let Opts = crate::Eliminate::Definition::Options::default();

		let Src = r#"fn f() { let X = 5; println!("{}", X); }"#;

		let First = crate::Eliminate::Transform::Run(Src, &Opts)
			.unwrap()
			.expect("first pass should change");

		let Second = crate::Eliminate::Transform::Run(&First, &Opts).unwrap();

		assert!(Second.is_none(), "second pass must be a no-op:\n{}", First);
	}
}
