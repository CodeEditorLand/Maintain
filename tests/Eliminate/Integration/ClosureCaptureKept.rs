//! Integration test: the closure VALUE (F) is inlined (single-use) while the
//! captured variable (X) is kept because it is referenced inside the closure body.

#[test]
fn ClosureCaptureKept() {
	super::assert_eliminates_and_compiles(
		"fn f() { let X = 42i32; let F = move || X; call(F); }",
		"fn f() { let X = 42i32; call(move || X); }",
		"fn call(_: impl Fn() -> i32) {}",
		"ClosureCaptureKept__ClosureCaptureKept",
	);
}

/// `Clone` is captured inside the closure (InClosure=true → kept).
/// `Result` is the closure call result - single-use, gets inlined.
/// The tool correctly inlines `Result` and keeps `Clone`.
#[test]
fn PreCloneClosureValueInlined() {
	super::assert_eliminates_and_compiles(
		r#"fn f(val: i32) -> i32 {
            let Clone = val.wrapping_add(1);
            let Result = (move || Clone)();
            Result
        }"#,
		r#"fn f(val: i32) -> i32 {
            let Clone = val.wrapping_add(1);
            (move || Clone)()
        }"#,
		"",
		"ClosureCaptureKept__PreCloneClosureValueInlined",
	);
}
