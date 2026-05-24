//! Integration test: `as` cast inlined at call site (no extra parens) and as
//! a binary operand (parens required to preserve precedence).

#[test]
fn CastExprInlined() {
	super::assert_eliminates_and_compiles(
		"fn f() { let Code = 42u64 as i32; app_exit(Code); }",
		"fn f() { app_exit(42u64 as i32); }",
		"fn app_exit(_: i32) {}",
		"CastExpr__CastExprInlined",
	);
}

#[test]
fn CastExprNeedsParen() {
	// `(10u64 as u16) + 1` - the cast wraps the whole left operand.
	super::assert_eliminates_and_compiles(
		"fn f() { let X = 10u64 as u16; let _ = X + 1; }",
		"fn f() { let _ = (10u64 as u16) + 1; }",
		"",
		"CastExpr__CastExprNeedsParen",
	);
}
