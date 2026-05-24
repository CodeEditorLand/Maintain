//! Integration test: shadowed bindings each get inlined independently.

#[test]
fn ShadowFirstThenInline() {
	// Rename the outer function to avoid a recursive call to itself.
	super::assert_eliminates_and_compiles(
		"fn f() { let X = 1i32; f2(X); let X = 2i32; g(X); }",
		"fn f() { f2(1i32); g(2i32); }",
		"fn f2(_: i32) {} fn g(_: i32) {}",
		"Shadow__ShadowFirstThenInline",
	);
}
