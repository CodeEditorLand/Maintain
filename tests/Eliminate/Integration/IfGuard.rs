//! Integration test: binding inlined into an `if` guard condition.
//! Async compile test proves rustc accepts `.await?` in that position.

#[test]
fn InlineIntoIfGuard() {
	super::assert_eliminates_and_compiles(
		r#"async fn f(val: u32) -> Result<(), String> {
            let Status = check(val).await.map_err(|e: String| e)?;
            if Status {
                Ok(())
            } else {
                Err("failed".into())
            }
        }"#,
		r#"async fn f(val: u32) -> Result<(), String> {
            if check(val).await.map_err(|e: String| e)? {
                Ok(())
            } else {
                Err("failed".into())
            }
        }"#,
		"async fn check(_: u32) -> Result<bool, String> { Ok(true) }",
		"IfGuard__InlineIntoIfGuard",
	);
}
