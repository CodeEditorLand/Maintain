//! Integration test: chained single-use bindings collapse across multiple
//! passes.

#[test]
fn ChainInline() {
	super::assert_eliminates_and_compiles(
		"fn f() { let A = 1u32; let B = A + 1; g(B); }",
		"fn f() { g(1u32 + 1); }",
		"fn g(_: u32) {}",
		"Chain__ChainInline",
	);
}

#[test]
fn ChainOfThreeInlined() {
	super::assert_eliminates_and_compiles(
		"fn f() { let A = 1u32; let B = A + 1; let C = B * 2; use_it(C); }",
		"fn f() { use_it((1u32 + 1) * 2); }",
		"fn use_it(_: u32) {}",
		"Chain__ChainOfThreeInlined",
	);
}
