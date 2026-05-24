//! Integration test: precedence parentheses inserted when a binary expression
//! becomes a binary operand; no parens in function-argument position.

#[test]
fn BinaryExprParens() {
	super::assert_eliminates_and_compiles(
		"fn f() { let X = A + B; let _ = Y * X; }",
		"fn f() { let _ = Y * (A + B); }",
		"static A: i32 = 1; static B: i32 = 2; static Y: i32 = 3;",
		"BinaryParens__BinaryExprParens",
	);
}

#[test]
fn BinaryExprNoParensInFnArg() {
	super::assert_eliminates_and_compiles(
		"fn f() { let X = A + B; foo(X); }",
		"fn f() { foo(A + B); }",
		"static A: i32 = 1; static B: i32 = 2; fn foo(_: i32) {}",
		"BinaryParens__NoParensInFnArg",
	);
}
