//! Integration test: `?` operator in the initialiser is safely inlined.
//! Async functions compile under --crate-type=lib without an executor.

#[test]
fn QuestionMarkInlined() {
	super::assert_eliminates_and_compiles(
		r#"
            async fn f() -> Result<(), String> {
                let X = foo().map_err(|e: String| e)?;
                bar(X);
                Ok(())
            }
        "#,
		r#"
            async fn f() -> Result<(), String> {
                bar(foo().map_err(|e: String| e)?);
                Ok(())
            }
        "#,
		"fn foo() -> Result<u32, String> { Ok(0) } fn bar(_: u32) {}",
		"QuestionMark__QuestionMarkInlined",
	);
}
