#![allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	unused_imports,
	unused_variables
)]
//=============================================================================//
// File Path: Element/Maintain/tests/Eliminate.rs
//=============================================================================//
// Integration tests for the Eliminate module.
//
// Each test exercises the public `Maintain::Eliminate::Transform::Run` API with
// representative Rust source snippets.  Tests are grouped by behaviour:
//
//   [INLINED]   - the binding should be eliminated.
//   [KEPT]      - the binding must remain unchanged.
//   [SPECIAL]   - edge cases, options, or behavioural contracts.
//=============================================================================//

use Maintain::Eliminate::{Definition::Options, Transform};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Run the elimination transform and return the output (or the pretty-printed
/// original if nothing changed).
fn transform(Src: &str) -> String {
	transform_with(Src, Options::default())
}

fn transform_with(Src: &str, Opts: Options) -> String {
	Transform::Run(Src, &Opts)
		.expect("transform failed")
		.unwrap_or_else(|| {
			let Ast: syn::File = syn::parse_str(Src).unwrap();

			prettyplease::unparse(&Ast)
		})
}

/// Normalise `Src` through prettyplease so whitespace differences are ignored.
fn norm(Src: &str) -> String {
	let Ast: syn::File = syn::parse_str(Src).unwrap();

	prettyplease::unparse(&Ast)
}

fn assert_eliminates(Input: &str, Expected: &str) {
	assert_eq!(transform(Input), norm(Expected));
}

fn assert_unchanged(Input: &str) {
	let Opts = Options::default();

	let Result = Transform::Run(Input, &Opts).expect("transform");

	assert!(Result.is_none(), "expected no change but got:\n{}", Result.unwrap());
}

// ---------------------------------------------------------------------------
// [INLINED] tests
// ---------------------------------------------------------------------------

/// (1) A plain literal is inlined at its single use site.
#[test]
fn SimpleInline() {
	assert_eliminates(
		r#"fn f() { let X = 5; println!("{}", X); }"#,
		r#"fn f() { println!("{}", 5); }"#,
	);
}

/// (2) Chained single-use bindings collapse over two passes.
#[test]
fn ChainInline() {
	assert_eliminates(
		"fn f() { let A = 1; let B = A + 1; g(B); }",
		"fn f() { g(1 + 1); }",
	);
}

/// (7) Shadow: first binding used once before the shadow; second used once
///     after.  Both should be inlined independently.
#[test]
fn ShadowFirstThenInline() {
	assert_eliminates(
		"fn f() { let X = 1; f(X); let X = 2; g(X); }",
		"fn f() { f(1); g(2); }",
	);
}

/// (9) Binary expression gets parentheses when placed as a binary operand.
#[test]
fn BinaryExprParens() {
	assert_eliminates(
		"fn f() { let X = A + B; let _ = Y * X; }",
		"fn f() { let _ = Y * (A + B); }",
	);
}

/// Function-argument position does NOT get extra parentheses.
#[test]
fn BinaryExprNoParensInFnArg() {
	assert_eliminates(
		"fn f() { let X = A + B; foo(X); }",
		"fn f() { foo(A + B); }",
	);
}

/// (10) The `?` operator in the initialiser is safely inlined.
#[test]
fn QuestionMarkInlined() {
	assert_eliminates(
		"async fn f() -> Result<(), E> { let X = foo().map_err(|e| e)?; bar(X); Ok(()) }",
		"async fn f() -> Result<(), E> { bar(foo().map_err(|e| e)?); Ok(()) }",
	);
}

/// (11) Struct literal initialiser is inlined.
#[test]
fn StructInlined() {
	assert_eliminates(
		"fn f() { let Opts = MyOpts { a: 1, b: 2 }; call(Opts); }",
		"fn f() { call(MyOpts { a: 1, b: 2 }); }",
	);
}

/// (12) `json!` macro call is inlined.
#[test]
fn JsonMacroInlined() {
	assert_eliminates(
		r#"fn f() { let Dto = json!({ "k": val }); call(Dto); }"#,
		r#"fn f() { call(json!({ "k": val })); }"#,
	);
}

/// (13) URL-parsing two-step pattern - the primary motivating example.
///      Pass 1: URI (str) → inlined into Url::parse.
///      Pass 2: DocumentURI (Url) → inlined into ProvideCodeLenses call.
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

	assert_eq!(transform(Input), norm(Expected));
}

/// (14) `match` expression initialiser is inlined.
#[test]
fn MatchExprInlined() {
	assert_eliminates(
		"fn f() { let R = match x { 1 => true, _ => false }; use_result(R); }",
		"fn f() { use_result(match x { 1 => true, _ => false }); }",
	);
}

/// (15) Block expression initialiser is inlined.
#[test]
fn BlockExprInlined() {
	assert_eliminates(
		"fn f() { let V = { compute() }; g(V); }",
		"fn f() { g({ compute() }); }",
	);
}

/// (16) Type annotation on the let binding is dropped when inlined (the
///      compiler infers the type at the use site).
#[test]
fn TypeAnnotationDropped() {
	assert_eliminates(
		"fn f() { let X: i32 = 5; g(X); }",
		"fn f() { g(5); }",
	);
}

/// (19) Inner-scope binding is inlined without touching outer scope.
#[test]
fn NestedScopeInlined() {
	assert_eliminates(
		r#"fn f() {
            let Outer = outer_val();
            {
                let Inner = 42;
                use_inner(Inner);
            }
            use_outer(Outer);
        }"#,
		r#"fn f() {
            let Outer = outer_val();
            { use_inner(42); }
            use_outer(Outer);
        }"#,
	);
}

/// (20) Borrow (`&expr`) initialiser is inlined.
#[test]
fn BorrowInlined() {
	assert_eliminates(
		"fn f() { let X = &foo; bar(X); }",
		"fn f() { bar(&foo); }",
	);
}

/// (21) `PositionDTO_` struct construction inlined into async method call.
#[test]
fn PositionDtoInlined() {
	assert_eliminates(
		r#"async fn f() -> Result<(), E> {
            let DocURI = parse_uri()?;
            let PositionDTO_ = PositionDTO { LineNumber: 0, Column: 0 };
            env.ProvideHover(DocURI, PositionDTO_).await?;
            Ok(())
        }"#,
		r#"async fn f() -> Result<(), E> {
            let DocURI = parse_uri()?;
            env.ProvideHover(DocURI, PositionDTO { LineNumber: 0, Column: 0 }).await?;
            Ok(())
        }"#,
	);
}

// ---------------------------------------------------------------------------
// [KEPT] tests
// ---------------------------------------------------------------------------

/// (3) Multi-use binding is kept.
#[test]
fn MultiUseKept() {
	assert_unchanged("fn f() { let X = foo(); bar(X); baz(X); }");
}

/// (4) Mutable binding is kept.
#[test]
fn MutKept() {
	assert_unchanged("fn f() { let mut X = 5; X += 1; g(X); }");
}

/// (5) Destructuring pattern is kept.
#[test]
fn DestructuringKept() {
	assert_unchanged("fn f() { let (A, B) = pair; g(A); }");
}

/// (6) Identifier referenced inside a macro at a second site is kept.
#[test]
fn MacroDoubleReferenceKept() {
	assert_unchanged(
		r#"fn f() {
            let URI = "x";
            dev_log!("{}", URI);
            let _ = Url::parse(URI);
        }"#,
	);
}

/// (8) Binding used inside a closure body is kept (conservative: move
///     semantics may change).
#[test]
fn ClosureCaptureKept() {
	assert_unchanged("fn f() { let X = heavy(); let F = move || X; call(F); }");
}

/// (22) Initialiser exceeding `MaxSize` is kept.
#[test]
fn SizeThresholdKept() {
	// Build a source with a very large initialiser (> 5 nodes) and MaxSize=5.
	let Input =
		"fn f() { let X = a + b + c + d + e + f + g + h + i + j; use_x(X); }";

	let Opts = Options { MaxSize: 5, ..Options::default() };

	let Result = Transform::Run(Input, &Opts).expect("transform");

	assert!(Result.is_none(), "oversized initialiser should not be inlined");
}

/// (23) Binding whose initialiser is `unsafe { … }` is kept.
#[test]
fn UnsafeNotInlined() {
	assert_unchanged("fn f() { let X = unsafe { *ptr }; g(X); }");
}

/// (24) `let … = … else { … }` (diverging let) is kept.
#[test]
fn LetElseKept() {
	assert_unchanged(
		r#"fn f() {
            let Ok(X) = foo() else { return; };
            g(X);
        }"#,
	);
}

// ---------------------------------------------------------------------------
// [SPECIAL] tests
// ---------------------------------------------------------------------------

/// (17) A file with no single-use variables returns `None` (no write needed).
#[test]
fn AlreadyMinimalReturnsNone() {
	let Opts = Options::default();

	let Src = "fn f() { let X = foo(); bar(X); baz(X); }";

	let Result = Transform::Run(Src, &Opts).unwrap();

	assert!(Result.is_none());
}

/// (18) Empty function body does not panic.
#[test]
fn EmptyFnNoCrash() {
	assert_unchanged("fn foo() {}");
}

/// (25) Applying the transform twice produces the same output (idempotent).
#[test]
fn Idempotent() {
	let Opts = Options::default();

	let Src = r#"fn f() { let X = 5; println!("{}", X); }"#;

	let First = Transform::Run(Src, &Opts)
		.unwrap()
		.expect("first pass should change");

	let Second = Transform::Run(&First, &Opts).unwrap();

	assert!(Second.is_none(), "second pass must be a no-op:\n{}", First);
}

/// `InlineComments = true` allows a binding with an attribute to be inlined.
#[test]
fn AttributedLetInlinedWhenOptIn() {
	let Input = r#"fn f() {
        #[allow(unused)]
        let X = 5;
        g(X);
    }"#;

	let Expected = "fn f() { g(5); }";

	let Opts = Options { InlineComments: true, ..Options::default() };

	assert_eq!(transform_with(Input, Opts), norm(Expected));
}

/// `InlineComments = false` (default) keeps a binding with an attribute.
#[test]
fn AttributedLetKeptByDefault() {
	let Input = r#"fn f() {
        #[allow(unused)]
        let X = 5;
        g(X);
    }"#;

	let Opts = Options::default();

	let Result = Transform::Run(Input, &Opts).unwrap();

	assert!(Result.is_none(), "attributed let should not be inlined by default");
}
