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
fn transform(Src:&str) -> String { transform_with(Src, Options::default()) }

fn transform_with(Src:&str, Opts:Options) -> String {
	Transform::Run(Src, &Opts).expect("transform failed").unwrap_or_else(|| {
		let Ast:syn::File = syn::parse_str(Src).unwrap();

		prettyplease::unparse(&Ast)
	})
}

/// Normalise `Src` through prettyplease so whitespace differences are ignored.
fn norm(Src:&str) -> String {
	let Ast:syn::File = syn::parse_str(Src).unwrap();

	prettyplease::unparse(&Ast)
}

fn assert_eliminates(Input:&str, Expected:&str) {
	assert_eq!(transform(Input), norm(Expected));
}

fn assert_unchanged(Input:&str) {
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
fn ChainInline() { assert_eliminates("fn f() { let A = 1; let B = A + 1; g(B); }", "fn f() { g(1 + 1); }"); }

/// (7) Shadow: first binding used once before the shadow; second used once
///     after.  Both should be inlined independently.
#[test]
fn ShadowFirstThenInline() {
	assert_eliminates("fn f() { let X = 1; f(X); let X = 2; g(X); }", "fn f() { f(1); g(2); }");
}

/// (9) Binary expression gets parentheses when placed as a binary operand.
#[test]
fn BinaryExprParens() {
	assert_eliminates("fn f() { let X = A + B; let _ = Y * X; }", "fn f() { let _ = Y * (A + B); }");
}

/// Function-argument position does NOT get extra parentheses.
#[test]
fn BinaryExprNoParensInFnArg() { assert_eliminates("fn f() { let X = A + B; foo(X); }", "fn f() { foo(A + B); }"); }

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
fn BlockExprInlined() { assert_eliminates("fn f() { let V = { compute() }; g(V); }", "fn f() { g({ compute() }); }"); }

/// (16) Type annotation on the let binding is dropped when inlined (the
///      compiler infers the type at the use site).
#[test]
fn TypeAnnotationDropped() { assert_eliminates("fn f() { let X: i32 = 5; g(X); }", "fn f() { g(5); }"); }

/// (19) Both inner and outer single-use bindings are eliminated: Inner in the
///      nested block, Outer in the outer scope.
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
            { use_inner(42); }
            use_outer(outer_val());
        }"#,
	);
}

/// (20) Borrow (`&expr`) initialiser is inlined.
#[test]
fn BorrowInlined() { assert_eliminates("fn f() { let X = &foo; bar(X); }", "fn f() { bar(&foo); }"); }

/// (21) Both `DocURI` and `PositionDTO_` are single-use - both are eliminated.
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
            env.ProvideHover(parse_uri()?, PositionDTO { LineNumber: 0, Column: 0 }).await?;
            Ok(())
        }"#,
	);
}

// ---------------------------------------------------------------------------
// [KEPT] tests
// ---------------------------------------------------------------------------

/// (3) Multi-use binding is kept.
#[test]
fn MultiUseKept() { assert_unchanged("fn f() { let X = foo(); bar(X); baz(X); }"); }

/// (4) Mutable binding is kept.
#[test]
fn MutKept() { assert_unchanged("fn f() { let mut X = 5; X += 1; g(X); }"); }

/// (5) Destructuring pattern is kept.
#[test]
fn DestructuringKept() { assert_unchanged("fn f() { let (A, B) = pair; g(A); }"); }

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

/// (8) `F` (the closure itself, single-use) is inlined; `X` (captured inside
///     the closure body) is conservatively kept.
#[test]
fn ClosureCaptureKept() {
	assert_eliminates(
		"fn f() { let X = heavy(); let F = move || X; call(F); }",
		"fn f() { let X = heavy(); call(move || X); }",
	);
}

/// (22) Initialiser exceeding `MaxSize` is kept.
#[test]
fn SizeThresholdKept() {
	// Build a source with a very large initialiser (> 5 nodes) and MaxSize=5.
	let Input = "fn f() { let X = a + b + c + d + e + f + g + h + i + j; use_x(X); }";

	let Opts = Options { MaxSize:5, ..Options::default() };

	let Result = Transform::Run(Input, &Opts).expect("transform");

	assert!(Result.is_none(), "oversized initialiser should not be inlined");
}

/// (23) Binding whose initialiser is `unsafe { … }` is kept.
#[test]
fn UnsafeNotInlined() { assert_unchanged("fn f() { let X = unsafe { *ptr }; g(X); }"); }

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
fn EmptyFnNoCrash() { assert_unchanged("fn foo() {}"); }

/// (25) Applying the transform twice produces the same output (idempotent).
#[test]
fn Idempotent() {
	let Opts = Options::default();

	let Src = r#"fn f() { let X = 5; println!("{}", X); }"#;

	let First = Transform::Run(Src, &Opts).unwrap().expect("first pass should change");

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

	let Opts = Options { InlineComments:true, ..Options::default() };

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

// ---------------------------------------------------------------------------
// [MOUNTAIN] real-world patterns harvested from Mountain source files
// ---------------------------------------------------------------------------

/// AcceptTerminalProcessData.rs: `DataString` is used only inside a `json!`
/// macro - must be inlined into the macro token stream.
#[test]
fn MountainDataStringIntoJsonMacro() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: AcceptTerminalProcessDataRequest,
        ) -> Result<Response<AcceptTerminalProcessDataResponse>, Status> {
            let DataString = String::from_utf8_lossy(&Request.data).to_string();

            let _ = Service
                .environment
                .ApplicationHandle
                .emit("sky://terminal/data", json!({ "id": Request.terminal_id, "data": DataString }));

            Ok(Response::new(AcceptTerminalProcessDataResponse {}))
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: AcceptTerminalProcessDataRequest,
        ) -> Result<Response<AcceptTerminalProcessDataResponse>, Status> {
            let _ = Service
                .environment
                .ApplicationHandle
                .emit("sky://terminal/data", json!({ "id": Request.terminal_id, "data": String::from_utf8_lossy(&Request.data).to_string() }));

            Ok(Response::new(AcceptTerminalProcessDataResponse {}))
        }"#,
	);
}

/// ProvideReferences.rs: all three single-use bindings inlined - `DocumentURI`,
/// `PositionDTO_`, `ContextDTO`.
#[test]
fn MountainContextDtoIntoFnArg() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideReferencesRequest,
        ) -> Result<Response<ProvideReferencesResponse>, Status> {
            let DocumentURI = parse_uri(&Request)?;

            let PositionDTO_ = build_position(&Request);

            let ContextDTO = json!({ "includeDeclaration": true });

            match Service.environment.ProvideReferences(DocumentURI, PositionDTO_, ContextDTO).await {
                Ok(_) => Ok(Response::new(ProvideReferencesResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideReferencesRequest,
        ) -> Result<Response<ProvideReferencesResponse>, Status> {
            match Service.environment.ProvideReferences(
                parse_uri(&Request)?,

                build_position(&Request),

                json!({ "includeDeclaration": true }),
            ).await {
                Ok(_) => Ok(Response::new(ProvideReferencesResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
	);
}

/// ProvideDefinition.rs: `PositionDTO_` struct literal used exactly once.
#[test]
fn MountainPositionDtoInlined() {
	// All single-use bindings inlined: DocumentURI + PositionDTO_.  Position_
	// is multi-use (two field reads) so it stays.
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideDefinitionRequest,
        ) -> Result<Response<ProvideDefinitionResponse>, Status> {
            let Position_ = Request.position.as_ref();

            let DocumentURI = parse_uri(&Request)?;

            let PositionDTO_ = PositionDTO {
                LineNumber: Position_.map(|P| P.line).unwrap_or(0),

                Column: Position_.map(|P| P.character).unwrap_or(0),
            };

            match Service.environment.ProvideDefinition(DocumentURI, PositionDTO_).await {
                Ok(_) => Ok(Response::new(ProvideDefinitionResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideDefinitionRequest,
        ) -> Result<Response<ProvideDefinitionResponse>, Status> {
            let Position_ = Request.position.as_ref();

            match Service.environment.ProvideDefinition(
                parse_uri(&Request)?,

                PositionDTO {
                    LineNumber: Position_.map(|P| P.line).unwrap_or(0),
                    Column: Position_.map(|P| P.character).unwrap_or(0),
                },
            ).await {
                Ok(_) => Ok(Response::new(ProvideDefinitionResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
	);
}

/// FileWatch.rs: `Root = PathBuf::from(&Path)` is a single-use
/// `PathBuf` construction that feeds directly into `RegisterWatcher`.
#[test]
fn MountainPathBufInlined() {
	assert_eliminates(
		r#"pub async fn Fn(Path: String) -> Result<(), String> {
            let Root = PathBuf::from(&Path);
            RunTime
                .Environment
                .RegisterWatcher(Handle.clone(), Root, IsRecursive, Pattern)
                .await
                .map_err(|E| format!("file:watch: {E}"))?;
            Ok(())
        }"#,
		r#"pub async fn Fn(Path: String) -> Result<(), String> {
            RunTime
                .Environment
                .RegisterWatcher(Handle.clone(), PathBuf::from(&Path), IsRecursive, Pattern)
                .await
                .map_err(|E| format!("file:watch: {E}"))?;
            Ok(())
        }"#,
	);
}

/// Encrypt.rs: `UnboundK` used once - inlined with `?` propagation.
#[test]
fn MountainUnboundKeyInlined() {
	assert_eliminates(
		r#"fn encrypt() -> Result<(), String> {
            let UnboundK = UnboundKey::new(&AES_256_GCM, &KeyBytes)
                .map_err(|E| format!("encrypt key: {E:?}"))?;
            let Key = LessSafeKey::new(UnboundK);
            Ok(())
        }"#,
		r#"fn encrypt() -> Result<(), String> {
            let Key = LessSafeKey::new(
                UnboundKey::new(&AES_256_GCM, &KeyBytes)
                    .map_err(|E| format!("encrypt key: {E:?}"))?,
            );
            Ok(())
        }"#,
	);
}

/// Decrypt.rs: `NonceBytes` array inlined directly into
/// `Nonce::assume_unique_for_key`.
#[test]
fn MountainNonceBytesInlined() {
	assert_eliminates(
		r#"fn decrypt() -> Result<(), String> {
            let NonceBytes: [u8; 12] = Blob[..12].try_into().unwrap();
            let NonceVal = Nonce::assume_unique_for_key(NonceBytes);
            Ok(())
        }"#,
		r#"fn decrypt() -> Result<(), String> {
            let NonceVal = Nonce::assume_unique_for_key(Blob[..12].try_into().unwrap());
            Ok(())
        }"#,
	);
}

/// GitExec.rs: `StdoutString` used once - Cow from `from_utf8_lossy` inlined.
#[test]
fn MountainStdoutStringInlined() {
	assert_eliminates(
		r#"fn process_output(Output: Output) {
            let StdoutString = String::from_utf8_lossy(&Output.stdout);
            let mut OutputLines: Vec<String> = StdoutString.lines().map(|L| L.to_string()).collect();
            drop(OutputLines);
        }"#,
		r#"fn process_output(Output: Output) {
            let mut OutputLines: Vec<String> = String::from_utf8_lossy(&Output.stdout).lines().map(|L| L.to_string()).collect();
            drop(OutputLines);
        }"#,
	);
}

/// UpdateScmGroup.rs: `ResourceStates` Vec inlined into a `json!` macro arg.
#[test]
fn MountainResourceStatesIntoJsonMacro() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: UpdateScmGroupRequest,
        ) -> Result<Response<UpdateScmGroupResponse>, Status> {
            let ResourceStates: Vec<serde_json::Value> = Request
                .resource_states
                .iter()
                .map(|RS| json!({ "uri": RS.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("") }))
                .collect();

            let _ = Service.environment.ApplicationHandle.emit(
                "sky://scm/updateGroup",

                json!({ "groupId": Request.group_id, "resourceStates": ResourceStates }),
            );

            Ok(Response::new(UpdateScmGroupResponse {}))
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: UpdateScmGroupRequest,
        ) -> Result<Response<UpdateScmGroupResponse>, Status> {
            let _ = Service.environment.ApplicationHandle.emit(
                "sky://scm/updateGroup",

                json!({ "groupId": Request.group_id, "resourceStates": Request
                    .resource_states
                    .iter()
                    .map(|RS| json!({ "uri": RS.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("") }))
                    .collect() }),
            );

            Ok(Response::new(UpdateScmGroupResponse {}))
        }"#,
	);
}

/// ProvideHover.rs: `URI` is multi-use (dev_log! + Url::parse), so it stays.
/// `DocumentURI` is single-use and is inlined into `drop(...)`.
#[test]
fn MountainUriDoubleUseKept() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideHoverRequest,
        ) -> Result<Response<ProvideHoverResponse>, Status> {
            let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

            dev_log!("hover URI={}", URI);

            let DocumentURI = Url::parse(URI)
                .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

            drop(DocumentURI);

            Ok(Response::new(ProvideHoverResponse::default()))
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideHoverRequest,
        ) -> Result<Response<ProvideHoverResponse>, Status> {
            let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

            dev_log!("hover URI={}", URI);

            drop(Url::parse(URI).map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?);

            Ok(Response::new(ProvideHoverResponse::default()))
        }"#,
	);
}

/// ShowQuickPick.rs: `SelectedIndices` Vec produced by iterator chain inlined
/// directly into the `ShowQuickPickResponse` struct literal.
#[test]
fn MountainSelectedIndicesInlined() {
	// All single-use bindings inlined: SelectedIndices into the response,
	// then Selected (which itself appears only once in the inlined chain).
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ShowQuickPickRequest,
        ) -> Result<Response<ShowQuickPickResponse>, Status> {
            let Selected = vec!["option_a".to_string()];

            let SelectedIndices: Vec<u32> = Selected
                .iter()
                .filter_map(|Label| {
                    Request
                        .items
                        .iter()
                        .position(|Item| &Item.label == Label)
                        .map(|Index| Index as u32)
                })
                .collect();

            Ok(Response::new(ShowQuickPickResponse { selected_indices: SelectedIndices }))
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ShowQuickPickRequest,
        ) -> Result<Response<ShowQuickPickResponse>, Status> {
            Ok(Response::new(ShowQuickPickResponse {
                selected_indices: vec!["option_a".to_string()]
                    .iter()
                    .filter_map(|Label| {
                        Request
                            .items
                            .iter()
                            .position(|Item| &Item.label == Label)
                            .map(|Index| Index as u32)
                    })
                    .collect(),
            }))
        }"#,
	);
}

/// GetTreeChildren.rs: `Parameters` json! literal inlined as the third argument
/// to `SendRequest`.
#[test]
fn MountainParametersIntoSendRequest() {
	// Handle inlined into json!, then Parameters inlined into SendRequest,
	// then Reply inlined into Ok(...).
	assert_eliminates(
		r#"pub async fn Fn() -> Result<String, String> {
            let Handle = get_handle();
            let Parameters = json!({
                "viewId": "explorer",
                "treeItemHandle": "item_1",
                "handle": Handle,
            });
            let Reply = SendRequest("cocoon-main", "$provideTreeChildren".to_string(), Parameters, 5000).await?;
            Ok(Reply)
        }"#,
		r#"pub async fn Fn() -> Result<String, String> {
            Ok(SendRequest(
                "cocoon-main",

                "$provideTreeChildren".to_string(),

                json!({
                    "viewId": "explorer",
                    "treeItemHandle": "item_1",
                    "handle": get_handle(),
                }),

                5000,
            ).await?)
        }"#,
	);
}

/// ProvideCodeActions.rs: `ContextDTO` inlined, while `RangeDTO` (which itself
/// depends on a multi-use `R` borrow) is left in place.
#[test]
fn MountainContextDtoWithKeptRangeDto() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideCodeActionsRequest,
        ) -> Result<Response<ProvideCodeActionsResponse>, Status> {
            let R = Request.range.as_ref();

            let RangeDTO = json!({
                "startLine": R.and_then(|R| R.start.as_ref()).map(|P| P.line).unwrap_or(0),
                "endLine": R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
            });

            let ContextDTO = json!({ "diagnostics": [], "only": null });

            match Service.environment.ProvideCodeActions(RangeDTO, ContextDTO).await {
                Ok(_) => Ok(Response::new(ProvideCodeActionsResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideCodeActionsRequest,
        ) -> Result<Response<ProvideCodeActionsResponse>, Status> {
            let R = Request.range.as_ref();

            match Service.environment.ProvideCodeActions(
                json!({
                    "startLine": R.and_then(|R| R.start.as_ref()).map(|P| P.line).unwrap_or(0),
                    "endLine": R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
                }),

                json!({ "diagnostics": [], "only": null }),
            ).await {
                Ok(_) => Ok(Response::new(ProvideCodeActionsResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
	);
}

// ===========================================================================
// [EDGE-CASES] - format-string implicit captures, loop bodies, branch inits
// ===========================================================================

/// `format!("{X}")` - X is used only via implicit capture.  The binding must
/// NOT be inlined because the substitution engine operates on token trees, not
/// string-literal content; removing the let would leave {X} undefined.
#[test]
fn ImplicitFormatCaptureKept() {
	// X appears only in a format-string literal (no bare Ident token).
	// Count = 1 (found by format-literal scanner), but SubstituteRef finds no
	// TokenTree::Ident(X) to replace, so Substituted = false and the let stays.
	assert_unchanged(r#"fn f() { let X = 5; println!("{X}"); }"#);
}

/// Mixed old-style + implicit: `println!("{}", X); println!("{X}")` - count =
/// 2, must NOT be inlined.  Without the format-literal scanner this was a
/// correctness bug where count came back as 1 and the binding was removed.
#[test]
fn MixedImplicitAndExplicitKept() {
	assert_unchanged(
		r#"fn f() {
            let X = 5;
            println!("{}", X);
            println!("{X}");
        }"#,
	);
}

/// Two explicit uses in different `println!` calls - multi-use, kept.
#[test]
fn TwoMacroUsesKept() {
	assert_unchanged(
		r#"fn f() {
            let X = 5;
            println!("{}", X);
            println!("{}", X);
        }"#,
	);
}

/// Two uses of X as arguments to the same function call: `bar(X, X)` - count =
/// 2, kept.
#[test]
fn TwoUsesInSameCallKept() { assert_unchanged("fn f() { let X = foo(); bar(X, X); }"); }

/// `let mut X = 5; g(X)` - mut binding, conservatively kept even though X is
/// never actually mutated.
#[test]
fn MutNeverMutatedKept() { assert_unchanged("fn f() { let mut X = 5; g(X); }"); }

/// A `for` loop iteration variable is not a `let` binding - the loop body is
/// unchanged regardless of how often the var appears.
#[test]
fn ForLoopVarUntouched() {
	assert_unchanged(
		r#"fn f() {
            for X in items {
                println!("{}", X);
            }
        }"#,
	);
}

/// An `if let Some(X) = foo()` binding is not a plain `let` - unchanged.
#[test]
fn IfLetBindingUntouched() {
	assert_unchanged(
		r#"fn f() {
            if let Some(X) = foo() {
                bar(X);
            }
        }"#,
	);
}

/// A `let X` used inside a `for` loop body: count = 1 (tool has no loop
/// awareness - it counts textual occurrences, not execution frequency).
/// For cheap/Copy initialisers this is semantically safe; the test documents
/// the current behavior.
#[test]
fn LoopBodySingleUseInlined() {
	assert_eliminates(
		r#"fn f(v: &[i32]) {
            let Label = "item";
            for _ in v {
                println!("{}", Label);
            }
        }"#,
		r#"fn f(v: &[i32]) {
            for _ in v {
                println!("{}", "item");
            }
        }"#,
	);
}

// ===========================================================================
// [EDGE-CASES] - chain inlining depths, block / if expressions as initialisers
// ===========================================================================

/// Chain of three bindings: A→B→C - inlined in three iterative passes.
#[test]
fn ChainOfThreeInlined() {
	assert_eliminates(
		"fn f() { let A = 1; let B = A + 1; let C = B * 2; use_it(C); }",
		"fn f() { use_it((1 + 1) * 2); }",
	);
}

/// Block expression as initialiser: `let X = { foo() }; use(X)`.
#[test]
fn BlockExprAsInitInlined() {
	assert_eliminates(
		"fn f() { let X = { compute() }; consume(X); }",
		"fn f() { consume({ compute() }); }",
	);
}

/// `if` expression as initialiser: `let X = if cond { A } else { B }; use(X)`.
#[test]
fn IfExprAsInitInlined() {
	assert_eliminates(
		"fn f() { let X = if ready { 1 } else { 0 }; set(X); }",
		"fn f() { set(if ready { 1 } else { 0 }); }",
	);
}

/// Match expression as initialiser: `let R = match x { … }; consume(R)`.
#[test]
fn MatchExprAsInitInlined() {
	assert_eliminates(
		r#"fn f() {
            let R = match x {
                0 => "zero",
                _ => "other",
            };
            println!("{}", R);
        }"#,
		r#"fn f() {
            println!("{}", match x {
                0 => "zero",
                _ => "other",
            });
        }"#,
	);
}

/// Match arm containing an early `return` - the return is valid inside a
/// sub-expression once the binding is inlined (Decrypt.rs `UnboundK` pattern).
#[test]
fn MatchArmWithEarlyReturnInlined() {
	assert_eliminates(
		r#"fn decrypt() -> Result<(), String> {
            let Key = match derive_key() {
                Ok(K) => K,
                Err(_) => return Ok(()),
            };
            use_key(Key);
            Ok(())
        }"#,
		r#"fn decrypt() -> Result<(), String> {
            use_key(match derive_key() {
                Ok(K) => K,
                Err(_) => return Ok(()),
            });
            Ok(())
        }"#,
	);
}

// ===========================================================================
// [EDGE-CASES] - borrow of inline temporary expressions
// ===========================================================================

/// `&inline_expr` - borrow of an expression that becomes a temporary.
/// In Rust, the temporary lives to the end of the enclosing statement, which
/// is long enough for the function call to complete.
#[test]
fn BorrowOfInlineExprInlined() {
	assert_eliminates(
		r#"pub fn Fn(Base: &PathBuf) -> std::io::Result<()> {
            let Dir = Base.join("window1");
            std::fs::create_dir_all(&Dir)?;
            Ok(())
        }"#,
		r#"pub fn Fn(Base: &PathBuf) -> std::io::Result<()> {
            std::fs::create_dir_all(&Base.join("window1"))?;
            Ok(())
        }"#,
	);
}

/// `.as_bytes()` call on an inline `format!` result - the temporary `String`
/// lives for the statement duration, so the borrow is valid.
#[test]
fn AsMethodOnInlineTemporaryInlined() {
	assert_eliminates(
		r#"fn f() -> Vec<u8> {
            let Input = format!("prefix-{}", id);
            digest(Input.as_bytes())
        }"#,
		r#"fn f() -> Vec<u8> {
            digest(format!("prefix-{}", id).as_bytes())
        }"#,
	);
}

// ===========================================================================
// [EDGE-CASES] - ? and .await in inline position
// ===========================================================================

/// `let Status = cmd.status()?; if Status.success() { … }` - inlined into the
/// `if` condition.
#[test]
fn InlineIntoIfGuard() {
	assert_eliminates(
		r#"async fn f() -> Result<(), E> {
            let Status = cmd().status().await.map_err(|e| e)?;
            if Status.success() {
                Ok(())
            } else {
                Err(e)
            }
        }"#,
		r#"async fn f() -> Result<(), E> {
            if cmd().status().await.map_err(|e| e)?.success() {
                Ok(())
            } else {
                Err(e)
            }
        }"#,
	);
}

// ===========================================================================
// [MOUNTAIN] - additional patterns from Key.rs / TerminalProvider.rs
// ===========================================================================

/// Key.rs pattern: `Input = format!(…)` → inlined into
/// `digest(Input.as_bytes())`, then `Hash` → inlined into
/// `Key.copy_from_slice(Hash.as_ref())`.
#[test]
fn MountainKeyDerivationChain() {
	assert_eliminates(
		r#"fn derive_key(MachineId: &str) -> [u8; 32] {
            let Input = format!("Land-Encryption-v1{}", MachineId);
            let Hash = digest(&SHA256, Input.as_bytes());
            let mut Key = [0u8; 32];
            Key.copy_from_slice(Hash.as_ref());
            Key
        }"#,
		r#"fn derive_key(MachineId: &str) -> [u8; 32] {
            let mut Key = [0u8; 32];
            Key.copy_from_slice(
                digest(&SHA256, format!("Land-Encryption-v1{}", MachineId).as_bytes()).as_ref(),
            );
            Key
        }"#,
	);
}

/// TerminalProvider.rs: `Payload = json!([Term, Data.clone()])` - single-use
/// json! array literal inlined into a function call.
#[test]
fn MountainJsonArrayPayloadInlined() {
	assert_eliminates(
		r#"async fn f(Term: u32, Data: String) -> Result<(), E> {
            let Payload = json!([Term, Data.clone()]);
            SendNotification("main", "$accept", Payload).await?;
            Ok(())
        }"#,
		r#"async fn f(Term: u32, Data: String) -> Result<(), E> {
            SendNotification("main", "$accept", json!([Term, Data.clone()])).await?;
            Ok(())
        }"#,
	);
}

/// ProvideDocumentSymbols chain: `URI` and `DocumentURI` both single-use,
/// inlined in two passes.
#[test]
fn MountainDocumentSymbolsChain() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideDocumentSymbolsRequest,
        ) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
            let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

            let DocumentURI = Url::parse(URI)
                .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

            match Service.environment.ProvideDocumentSymbols(DocumentURI).await {
                Ok(_) => Ok(Response::new(ProvideDocumentSymbolsResponse::default())),

                Err(E) => Err(Status::internal(format!("Symbols failed: {}", E))),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideDocumentSymbolsRequest,
        ) -> Result<Response<ProvideDocumentSymbolsResponse>, Status> {
            match Service.environment.ProvideDocumentSymbols(
                Url::parse(Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""))
                    .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?,
            ).await {
                Ok(_) => Ok(Response::new(ProvideDocumentSymbolsResponse::default())),

                Err(E) => Err(Status::internal(format!("Symbols failed: {}", E))),
            }
        }"#,
	);
}

/// ProvideSignatureHelp.rs: `ContextDTO = json!({…})` inlined alongside
/// `DocumentURI` and `PositionDTO_`.
#[test]
fn MountainSignatureHelpContextDto() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideSignatureHelpRequest,
        ) -> Result<Response<ProvideSignatureHelpResponse>, Status> {
            let DocumentURI = parse_uri(&Request)?;

            let PositionDTO_ = build_position(&Request);

            let ContextDTO = json!({ "triggerKind": 1, "isRetrigger": false });

            match Service.environment.ProvideSignatureHelp(DocumentURI, PositionDTO_, ContextDTO).await {
                Ok(_) => Ok(Response::new(ProvideSignatureHelpResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideSignatureHelpRequest,
        ) -> Result<Response<ProvideSignatureHelpResponse>, Status> {
            match Service.environment.ProvideSignatureHelp(
                parse_uri(&Request)?,

                build_position(&Request),

                json!({ "triggerKind": 1, "isRetrigger": false }),
            ).await {
                Ok(_) => Ok(Response::new(ProvideSignatureHelpResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
	);
}

/// URI/Line/Character are multi-use (dev_log! + method call), kept.
/// DocumentURI and PositionDTO_ are single-use, inlined.
#[test]
fn MountainUriAndLineMultiUseKept() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideHoverRequest,
        ) -> Result<Response<ProvideHoverResponse>, Status> {
            let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

            let Line = Request.position.as_ref().map(|P| P.line).unwrap_or(0);

            let Character = Request.position.as_ref().map(|P| P.character).unwrap_or(0);

            dev_log!("hover pos={}:{} uri={}", Line, Character, URI);

            let DocumentURI = Url::parse(URI)
                .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

            let PositionDTO_ = PositionDTO { LineNumber: Line, Column: Character };

            match Service.environment.ProvideHover(DocumentURI, PositionDTO_).await {
                Ok(_) => Ok(Response::new(ProvideHoverResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideHoverRequest,
        ) -> Result<Response<ProvideHoverResponse>, Status> {
            let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

            let Line = Request.position.as_ref().map(|P| P.line).unwrap_or(0);

            let Character = Request.position.as_ref().map(|P| P.character).unwrap_or(0);

            dev_log!("hover pos={}:{} uri={}", Line, Character, URI);

            match Service.environment.ProvideHover(
                Url::parse(URI)
                    .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?,

                PositionDTO { LineNumber: Line, Column: Character },
            ).await {
                Ok(_) => Ok(Response::new(ProvideHoverResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
	);
}

/// Decrypt.rs pattern: both `UnboundK` and `Key` are single-use - inlined in
/// two passes to produce the fully collapsed form.
#[test]
fn MountainDecryptUnboundKeyMatchReturn() {
	assert_eliminates(
		r#"fn decrypt(KeyBytes: &[u8]) -> Result<Vec<u8>, String> {
            let UnboundK = match UnboundKey::new(&AES_256_GCM, KeyBytes) {
                Ok(K) => K,
                Err(_) => return Ok(vec![]),
            };
            let Key = LessSafeKey::new(UnboundK);
            Ok(Key.open())
        }"#,
		r#"fn decrypt(KeyBytes: &[u8]) -> Result<Vec<u8>, String> {
            Ok(LessSafeKey::new(match UnboundKey::new(&AES_256_GCM, KeyBytes) {
                Ok(K) => K,
                Err(_) => return Ok(vec![]),
            }).open())
        }"#,
	);
}

// ===========================================================================
// [EDGE-CASES] - idempotency of format-string-aware transform
// ===========================================================================

/// Re-running on already-minimal code containing `{X}` format strings must
/// return None (no change).
#[test]
fn IdempotentWithImplicitCapture() { assert_unchanged(r#"fn f() { let X = 5; println!("{X}"); }"#); }

// ===========================================================================
// [BATCH-2] cast · deref · negated-bool · double-let-in-json · match-scrutinee
// ===========================================================================

/// Cast expression inlined directly into the call site: `val as i32` stays
/// as-is (no extra parentheses needed inside a function argument).
#[test]
fn CastExprInlined() {
	assert_eliminates(
		"fn f() { let Code = raw_val() as i32; app_exit(Code); }",
		"fn f() { app_exit(raw_val() as i32); }",
	);
}

/// `as` cast used as the RHS of a binary expression DOES need parentheses,
/// because `(a as T) + b` and `a as (T + b)` are different.
#[test]
fn CastExprNeedsParen() {
	assert_eliminates(
		"fn f() { let X = val() as u16; let _ = X + 1; }",
		"fn f() { let _ = (val() as u16) + 1; }",
	);
}

/// `*deref` of an OnceLock result inlined into a `json!` macro argument.
#[test]
fn DerefOnceLockIntoJsonMacro() {
	assert_eliminates(
		r#"fn f() {
            let Dark = *DARK_MODE.get_or_init(detect_dark_mode);
            Ok(json!({ "dark": Dark, "highContrast": false }))
        }"#,
		r#"fn f() {
            Ok(json!({ "dark": *DARK_MODE.get_or_init(detect_dark_mode), "highContrast": false }))
        }"#,
	);
}

/// Negated bool guard: `let Allowed = set.contains(&x); if !Allowed { … }`.
#[test]
fn NegatedBoolGuardInlined() {
	assert_eliminates(
		r#"fn f() {
            let IsKnown = ALLOWED.contains(&Scheme.as_str());
            if !IsKnown {
                return Err("unknown scheme".into());
            }
        }"#,
		r#"fn f() {
            if !ALLOWED.contains(&Scheme.as_str()) {
                return Err("unknown scheme".into());
            }
        }"#,
	);
}

/// Two separate single-use bindings that both feed into the same `json!` macro
/// are inlined in two passes.
#[test]
fn TwoSeparateJsonFieldsInlined() {
	assert_eliminates(
		r#"fn f(Sys: &System) -> Value {
            let TotalMem = Sys.total_memory();
            let FreeMem = Sys.available_memory();
            json!({ "total": TotalMem, "free": FreeMem })
        }"#,
		r#"fn f(Sys: &System) -> Value {
            json!({ "total": Sys.total_memory(), "free": Sys.available_memory() })
        }"#,
	);
}

/// `let Opt = expr; match Opt { Some(x) if … => …, _ => … }` - binding inlined
/// as the match scrutinee.
#[test]
fn OptionBindingIntoMatchScrutinee() {
	assert_eliminates(
		r#"fn f(Response: &Value) -> Result<u64, String> {
            let TermId = Response.get("id").and_then(Value::as_u64);
            match TermId {
                Some(Id) if Id > 0 => Ok(Id),
                _ => Err("invalid id".into()),
            }
        }"#,
		r#"fn f(Response: &Value) -> Result<u64, String> {
            match Response.get("id").and_then(Value::as_u64) {
                Some(Id) if Id > 0 => Ok(Id),
                _ => Err("invalid id".into()),
            }
        }"#,
	);
}

/// Method chain into match discriminant: `let Kind = match
/// dialog_type.as_str()`. Two-pass: first inline `Kind`, second inline
/// `DialogType`.
#[test]
fn ChainIntoMatchDiscriminant() {
	assert_eliminates(
		r#"fn f(Options: &Value) -> MessageKind {
            let DialogType = Options
                .get("type")
                .and_then(Value::as_str)
                .map(|S| S.to_lowercase())
                .unwrap_or_default();
            let Kind = match DialogType.as_str() {
                "warning" => MessageKind::Warning,
                "error"   => MessageKind::Error,
                _         => MessageKind::Info,
            };
            Kind
        }"#,
		r#"fn f(Options: &Value) -> MessageKind {
            match Options
                .get("type")
                .and_then(Value::as_str)
                .map(|S| S.to_lowercase())
                .unwrap_or_default()
                .as_str()
            {
                "warning" => MessageKind::Warning,
                "error"   => MessageKind::Error,
                _         => MessageKind::Info,
            }
        }"#,
	);
}

// ===========================================================================
// [BATCH-2] async result · struct-field · elapsed-time inlines
// ===========================================================================

/// Async result inlined into a struct-literal field:
/// `let Content = fs::read(p).await?; Ok(Response { content: Content, … })`.
#[test]
fn AsyncResultIntoStructField() {
	assert_eliminates(
		r#"pub async fn read(Path: &str) -> Result<Resp, E> {
            let Content = tokio::fs::read(Path).await.map_err(|E| to_status(E))?;
            Ok(Response::new(FileReadResponse { content: Content, encoding: "utf-8".into() }))
        }"#,
		r#"pub async fn read(Path: &str) -> Result<Resp, E> {
            Ok(Response::new(FileReadResponse {
                content: tokio::fs::read(Path).await.map_err(|E| to_status(E))?,
                encoding: "utf-8".into(),
            }))
        }"#,
	);
}

/// `ElapsedMs` used only in a `dev_log!` - count = 1, inlined.
/// `Timer` is a parameter (not a let binding), so only `ElapsedMs` is a
/// candidate.
#[test]
fn ElapsedIntoLogMacro() {
	assert_eliminates(
		r#"fn f(Timer: std::time::Instant) {
            let ElapsedMs = Timer.elapsed().as_millis();
            dev_log!("elapsed ms={}", ElapsedMs);
        }"#,
		r#"fn f(Timer: std::time::Instant) {
            dev_log!("elapsed ms={}", Timer.elapsed().as_millis());
        }"#,
	);
}

/// `MTime` chain (Stat.rs pattern) inlined into a struct field.
#[test]
fn MtimeChainIntoStructField() {
	assert_eliminates(
		r#"fn stat(Meta: &Metadata) -> StatResponse {
            let MTime = Meta
                .modified()
                .ok()
                .and_then(|T| T.duration_since(UNIX_EPOCH).ok())
                .map(|D| D.as_millis() as u64)
                .unwrap_or(0);
            StatResponse {
                is_file:      Meta.is_file(),
                is_directory: Meta.is_dir(),
                size:         Meta.len(),
                mtime:        MTime,
            }
        }"#,
		r#"fn stat(Meta: &Metadata) -> StatResponse {
            StatResponse {
                is_file:      Meta.is_file(),
                is_directory: Meta.is_dir(),
                size:         Meta.len(),
                mtime:        Meta
                    .modified()
                    .ok()
                    .and_then(|T| T.duration_since(UNIX_EPOCH).ok())
                    .map(|D| D.as_millis() as u64)
                    .unwrap_or(0),
            }
        }"#,
	);
}

// ===========================================================================
// [BATCH-2] closure-local bindings inlined inside map()
// ===========================================================================

/// Inside a `.map(|Item| { let Handle = …; let Label = …; Struct { Handle,
/// Label } })` closure, `Handle` and `Label` are each single-use.  The tool
/// inlines them in two passes, collapsing the closure to a struct expression.
#[test]
fn ClosureLocalBindingsInlined() {
	assert_eliminates(
		r#"fn f(Items: &[Value]) -> Vec<TreeItem> {
            Items
                .iter()
                .map(|Item| {
                    let Handle = Item.get("handle").and_then(Value::as_str).unwrap_or("").to_string();

                    let Label  = Item.get("label").and_then(Value::as_str).unwrap_or("").to_string();

                    TreeItem { handle: Handle, label: Label }
                })
                .collect()
        }"#,
		r#"fn f(Items: &[Value]) -> Vec<TreeItem> {
            Items
                .iter()
                .map(|Item| {
                    TreeItem {
                        handle: Item.get("handle").and_then(Value::as_str).unwrap_or("").to_string(),
                        label: Item.get("label").and_then(Value::as_str).unwrap_or("").to_string(),
                    }
                })
                .collect()
        }"#,
	);
}

// ===========================================================================
// [BATCH-2] long-chain inlines: urlenc · canonical name · workspace patterns
// ===========================================================================

/// `EncodedPath` builder chain inlined into `format!` argument.
#[test]
fn UrlEncodedPathInlined() {
	assert_eliminates(
		r#"fn f(Origin: &str, PathStr: &str) -> String {
            let EncodedPath = url::form_urlencoded::Serializer::new(String::new())
                .append_pair("folder", PathStr)
                .finish();
            format!("{}/?{}", Origin, EncodedPath)
        }"#,
		r#"fn f(Origin: &str, PathStr: &str) -> String {
            format!(
                "{}/?{}",

                Origin,

                url::form_urlencoded::Serializer::new(String::new())
                    .append_pair("folder", PathStr)
                    .finish()
            )
        }"#,
	);
}

/// `Name = path.file_name()…unwrap_or_else(display)` inlined as argument to
/// a struct constructor (PickFolder.rs pattern).
#[test]
fn CanonicalFileNameInlined() {
	assert_eliminates(
		r#"fn f(Canonical: &PathBuf, Uri: Url) -> Option<WorkspaceFolder> {
            let Name = Canonical
                .file_name()
                .and_then(|N| N.to_str())
                .map(str::to_string)
                .unwrap_or_else(|| Canonical.display().to_string());
            Some(WorkspaceFolder::new(Uri, Name, 0))
        }"#,
		r#"fn f(Canonical: &PathBuf, Uri: Url) -> Option<WorkspaceFolder> {
            Some(WorkspaceFolder::new(
                Uri,

                Canonical
                    .file_name()
                    .and_then(|N| N.to_str())
                    .map(str::to_string)
                    .unwrap_or_else(|| Canonical.display().to_string()),

                0,
            ))
        }"#,
	);
}

/// `RemovalURIs` is referenced inside the `retain` closure body - the tool
/// detects `InClosure = true` and conservatively keeps the binding.
#[test]
fn RemovalUrisClosureCaptureKept() {
	assert_unchanged(
		r#"fn f(Removals: &[Removal], Folders: &mut Vec<Folder>) {
            let RemovalURIs: Vec<String> = Removals
                .iter()
                .filter_map(|R| R.uri.as_ref().map(|U| U.value.clone()))
                .collect();
            Folders.retain(|F| !RemovalURIs.contains(&F.uri.to_string()));
        }"#,
	);
}

/// `ExternalUri` optional chain inlined into `unwrap_or_else`
/// (FileWriteNative.rs).
#[test]
fn ExternalUriChainInlined() {
	assert_eliminates(
		r#"fn build_uri(Resource: &Value, Path: &str) -> String {
            let ExternalUri = Resource
                .as_object()
                .and_then(|O| O.get("external"))
                .and_then(|V| V.as_str())
                .map(|S| S.to_string());
            ExternalUri.unwrap_or_else(|| format!("file://{}", Path))
        }"#,
		r#"fn build_uri(Resource: &Value, Path: &str) -> String {
            Resource
                .as_object()
                .and_then(|O| O.get("external"))
                .and_then(|V| V.as_str())
                .map(|S| S.to_string())
                .unwrap_or_else(|| format!("file://{}", Path))
        }"#,
	);
}

/// `URI` binding (loop body) inlined into `Url::parse` inside an `if let`
/// guard (UpdateWorkspaceFolders.rs loop pattern).
#[test]
fn LoopUriIntoIfLetParse() {
	assert_eliminates(
		r#"fn f(Additions: &[Addition], Folders: &mut Vec<Folder>) {
            for Addition in Additions {
                let URI = Addition.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

                if let Ok(Parsed) = url::Url::parse(URI) {
                    Folders.push(Folder::new(Parsed));
                }
            }
        }"#,
		r#"fn f(Additions: &[Addition], Folders: &mut Vec<Folder>) {
            for Addition in Additions {
                if let Ok(Parsed) = url::Url::parse(
                    Addition.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""),
                ) {
                    Folders.push(Folder::new(Parsed));
                }
            }
        }"#,
	);
}

/// Block expression in `if let` scrutinee position (MaybePrimary pattern from
/// FileWatcherProvider.rs): the block acquires a mutex, removes an entry, and
/// releases the guard on the closing brace.
#[test]
fn BlockExprIntoIfLetScrutinee() {
	assert_eliminates(
		r#"fn f(Handle: String, State: &State) -> Option<String> {
            let MaybePrimary = {
                let mut Map = State.handle_map.lock().unwrap();

                Map.remove(&Handle)
            };
            if let Some(PrimaryHandle) = MaybePrimary {
                Some(PrimaryHandle)
            } else {
                None
            }
        }"#,
		r#"fn f(Handle: String, State: &State) -> Option<String> {
            if let Some(PrimaryHandle) = {
                let mut Map = State.handle_map.lock().unwrap();

                Map.remove(&Handle)
            } {
                Some(PrimaryHandle)
            } else {
                None
            }
        }"#,
	);
}

/// `EditsJSON` type-annotated collect into `json!` macro (ApplyEdit.rs).
#[test]
fn TypeAnnotatedCollectIntoJsonMacro() {
	assert_eliminates(
		r#"async fn f(Request: ApplyEditRequest, URI: &str, Handle: &AppHandle) {
            let EditsJSON: Vec<serde_json::Value> = Request
                .edits
                .iter()
                .map(|E| json!({ "newText": E.new_text }))
                .collect();
            let _ = Handle.emit("sky://editor/applyEdits", json!({ "uri": URI, "edits": EditsJSON }));
        }"#,
		r#"async fn f(Request: ApplyEditRequest, URI: &str, Handle: &AppHandle) {
            let _ = Handle.emit(
                "sky://editor/applyEdits",

                json!({
                    "uri": URI,
                    "edits": Request
                        .edits
                        .iter()
                        .map(|E| json!({ "newText": E.new_text }))
                        .collect()
                }),
            );
        }"#,
	);
}

/// `OptionsDTO` simple json! literal inlined as a format-function argument
/// (ProvideDocumentFormatting.rs).
#[test]
fn HardcodedOptionsDtoInlined() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            DocumentURI: Url,
        ) -> Result<Response<FormatResponse>, Status> {
            let OptionsDTO = json!({ "tabSize": 4, "insertSpaces": true });

            match Service.environment.ProvideDocumentFormattingEdits(DocumentURI, OptionsDTO).await {
                Ok(_) => Ok(Response::new(FormatResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            DocumentURI: Url,
        ) -> Result<Response<FormatResponse>, Status> {
            match Service.environment.ProvideDocumentFormattingEdits(
                DocumentURI,

                json!({ "tabSize": 4, "insertSpaces": true }),
            ).await {
                Ok(_) => Ok(Response::new(FormatResponse::default())),

                Err(E) => Err(Status::internal(E.to_string())),
            }
        }"#,
	);
}

// ===========================================================================
// [BATCH-2] KEPT patterns - range double-use, pre-clone, content_len dedup
// ===========================================================================

/// `StartPort` is used in BOTH bounds of a range expression
/// (`Start..Start+100`)
/// - count = 2, must NOT be inlined.
#[test]
fn RangeDoubleBoundKept() {
	assert_unchanged(
		r#"fn find_free_port() -> u16 {
            let StartPort = 9000u16;
            for Port in StartPort..StartPort + 100 {
                if is_free(Port) { return Port; }
            }
            0
        }"#,
	);
}

/// A pre-clone for `move ||` closure: the clone is needed before the closure
/// captures the handle.  The variable cannot be inlined because removing the
/// binding would leave the closure body with no handle.
#[test]
fn PreCloneForMoveClosureKept() {
	assert_unchanged(
		r#"async fn f(AppHandle: AppHandle) {
            let Handle = AppHandle.clone();
            tokio::task::spawn_blocking(move || {
                let _ = Handle.dialog().message("hello").show();
            });
        }"#,
	);
}

/// A binding used in TWO different `dev_log!` calls - multi-use, must stay.
#[test]
fn TwoDevLogUsesKept() {
	assert_unchanged(
		r#"fn f(Exit: i32) {
            let Code = Exit as i32;
            dev_log!("exit code={}", Code);
            dev_log!("shutdown code={}", Code);
        }"#,
	);
}

/// The `R` borrow alias used FOUR times in a json! macro body - the tool
/// counts all four token occurrences and keeps the binding.
#[test]
fn FourUsesBorrowAliasKept() {
	assert_unchanged(
		r#"fn f(Request: &Request) -> Value {
            let R = Request.range.as_ref();
            json!({
                "startLine": R.and_then(|R| R.start.as_ref()).map(|P| P.line).unwrap_or(0),
                "startChar": R.and_then(|R| R.start.as_ref()).map(|P| P.char).unwrap_or(0),
                "endLine":   R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
                "endChar":   R.and_then(|R| R.end.as_ref()).map(|P| P.char).unwrap_or(0),
            })
        }"#,
	);
}

/// A clone used in TWO fields of the same json! - count = 2, must stay.
/// Models the pattern where a cloned value feeds multiple json! keys.
#[test]
fn MultiUseCloneKept() {
	assert_unchanged(
		r#"fn f(Data: &Snapshot) -> Value {
            let Clone = Data.state.clone();
            json!({ "id": Clone.id, "name": Clone.name })
        }"#,
	);
}

/// `Handle` fed into both `dev_log!` AND `json!` - two uses, kept.
#[test]
fn HandleUsedInLogAndJsonKept() {
	assert_unchanged(
		r#"fn f() {
            let Handle = WATCH_SEQ.fetch_add(1, Ordering::Relaxed).to_string();
            dev_log!("watch handle={}", Handle);
            Ok(json!(Handle))
        }"#,
	);
}

// ===========================================================================
// [BATCH-2] Mountain-specific patterns
// ===========================================================================

/// Exit.rs pattern: `Code = arg_i64(args, 0) as i32` inlined into `app.exit()`.
#[test]
fn MountainExitCodeInlined() {
	assert_eliminates(
		r#"fn exit_app(Arguments: &[Value], ApplicationHandle: &AppHandle) -> Value {
            let Code = arg_i64(&Arguments, 0) as i32;
            ApplicationHandle.exit(Code);
            Value::Null
        }"#,
		r#"fn exit_app(Arguments: &[Value], ApplicationHandle: &AppHandle) -> Value {
            ApplicationHandle.exit(arg_i64(&Arguments, 0) as i32);
            Value::Null
        }"#,
	);
}

/// ClipboardWriteText.rs: `Text` used only as `Cb.set_text(Text)`.
#[test]
fn MountainClipboardTextInlined() {
	assert_eliminates(
		r#"fn write_clipboard(Arguments: &[Value]) -> Value {
            let Text = arg_string(&Arguments, 0);
            if let Ok(mut Cb) = arboard::Clipboard::new() {
                let _ = Cb.set_text(Text);
            }
            Value::Null
        }"#,
		r#"fn write_clipboard(Arguments: &[Value]) -> Value {
            if let Ok(mut Cb) = arboard::Clipboard::new() {
                let _ = Cb.set_text(arg_string(&Arguments, 0));
            }
            Value::Null
        }"#,
	);
}

/// ReviveTerminalProcesses.rs: `ShellArgs` typed collect inlined into json!,
/// then `Options` json! itself inlined into `CreateTerminal` call.
#[test]
fn MountainReviveTerminalChain() {
	assert_eliminates(
		r#"pub async fn Fn(RunTime: &RunTime, Config: &Value) -> Result<u64, String> {
            let ShellArgs: Vec<Value> = Config
                .get("args")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            let Options = json!({
                "shellPath": Config.get("shell").and_then(Value::as_str).unwrap_or(""),
                "shellArgs": ShellArgs,
            });
            match RunTime.Environment.CreateTerminal(Options).await {
                Ok(Resp) => Ok(Resp.get("id").and_then(Value::as_u64).unwrap_or(0)),
                Err(E)   => Err(E.to_string()),
            }
        }"#,
		r#"pub async fn Fn(RunTime: &RunTime, Config: &Value) -> Result<u64, String> {
            match RunTime.Environment.CreateTerminal(json!({
                "shellPath": Config.get("shell").and_then(Value::as_str).unwrap_or(""),
                "shellArgs": Config
                    .get("args")
                    .and_then(Value::as_array)
                    .cloned()
                    .unwrap_or_default(),
            })).await {
                Ok(Resp) => Ok(Resp.get("id").and_then(Value::as_u64).unwrap_or(0)),
                Err(E)   => Err(E.to_string()),
            }
        }"#,
	);
}

/// Stat.rs: `Metadata` is multi-use (four field calls) and must stay;
/// only `MTime` (single-use) is inlined.
#[test]
fn MountainStatMTimeInlined() {
	assert_eliminates(
		r#"pub async fn Fn(Path: &str) -> Result<Response<StatResp>, Status> {
            let Metadata = tokio::fs::metadata(Path).await.map_err(|E| Status::not_found(E.to_string()))?;
            let MTime = Metadata
                .modified()
                .ok()
                .and_then(|T| T.duration_since(UNIX_EPOCH).ok())
                .map(|D| D.as_millis() as u64)
                .unwrap_or(0);
            Ok(Response::new(StatResp {
                is_file:      Metadata.is_file(),
                is_directory: Metadata.is_dir(),
                size:         Metadata.len(),
                mtime:        MTime,
            }))
        }"#,
		r#"pub async fn Fn(Path: &str) -> Result<Response<StatResp>, Status> {
            let Metadata = tokio::fs::metadata(Path).await.map_err(|E| Status::not_found(E.to_string()))?;
            Ok(Response::new(StatResp {
                is_file:      Metadata.is_file(),
                is_directory: Metadata.is_dir(),
                size:         Metadata.len(),
                mtime:        Metadata
                    .modified()
                    .ok()
                    .and_then(|T| T.duration_since(UNIX_EPOCH).ok())
                    .map(|D| D.as_millis() as u64)
                    .unwrap_or(0),
            }))
        }"#,
	);
}

/// ProvideInlayHints.rs: `DocumentURI`, `PositionDTO_`, and `RangeDTO` are all
/// single-use and get inlined.  `R` (used 4× inside `RangeDTO`) must stay.
#[test]
fn MountainInlayHintsFullCollapse() {
	assert_eliminates(
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideInlayHintsRequest,
        ) -> Result<Response<ProvideInlayHintsResponse>, Status> {
            let URI = Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or("");

            let DocumentURI = Url::parse(URI)
                .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?;

            let R = Request.range.as_ref();

            let RangeDTO = json!({
                "startLine": R.and_then(|R| R.start.as_ref()).map(|P| P.line).unwrap_or(0),
                "endLine":   R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
            });

            match Service.environment.ProvideInlayHints(DocumentURI, RangeDTO).await {
                Ok(_) => Ok(Response::new(ProvideInlayHintsResponse::default())),

                Err(E) => Err(Status::internal(format!("InlayHints failed: {}", E))),
            }
        }"#,
		r#"pub async fn Fn(
            Service: &CocoonServiceImpl,

            Request: ProvideInlayHintsRequest,
        ) -> Result<Response<ProvideInlayHintsResponse>, Status> {
            let R = Request.range.as_ref();

            match Service.environment.ProvideInlayHints(
                Url::parse(Request.uri.as_ref().map(|U| U.value.as_str()).unwrap_or(""))
                    .map_err(|E| Status::invalid_argument(format!("Invalid URI: {}", E)))?,

                json!({
                    "startLine": R.and_then(|R| R.start.as_ref()).map(|P| P.line).unwrap_or(0),
                    "endLine":   R.and_then(|R| R.end.as_ref()).map(|P| P.line).unwrap_or(0),
                }),
            ).await {
                Ok(_) => Ok(Response::new(ProvideInlayHintsResponse::default())),

                Err(E) => Err(Status::internal(format!("InlayHints failed: {}", E))),
            }
        }"#,
	);
}
