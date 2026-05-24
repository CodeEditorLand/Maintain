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
