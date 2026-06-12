//! Unit tests for the Inline transform module.

#[cfg(test)]
mod Tests {

	use super::super::*;

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
		AssertEliminates("fn f() { let X = compute(); g(X); }", "fn f() { g(compute()); }");
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

	/// Same pattern with section instead of path
	/// (AirClient::get_configuration).
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
	/// still be inlined (no false positive that would block legitimate
	/// inlines).
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
