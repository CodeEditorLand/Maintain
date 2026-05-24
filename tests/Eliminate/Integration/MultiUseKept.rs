//! Integration test: multi-use bindings are kept unchanged AND compile.

#[test]
fn MultiUseKept() {
	super::assert_unchanged_and_compiles(
		"fn f() { let X = 42i32; bar(X); baz(X); }",
		"fn bar(_: i32) {} fn baz(_: i32) {}",
		"MultiUseKept__MultiUseKept",
	);
}

#[test]
fn MutKept() {
	super::assert_unchanged_and_compiles(
		"fn f() { let mut X = 5i32; X += 1; println!(\"{}\", X); }",
		"",
		"MultiUseKept__MutKept",
	);
}

#[test]
fn TwoUsesInSameCallKept() {
	super::assert_unchanged_and_compiles(
		"fn f() { let X = 7u32; double(X, X); }",
		"fn double(_: u32, _: u32) {}",
		"MultiUseKept__TwoUsesInSameCallKept",
	);
}

#[test]
fn RangeDoubleBoundKept() {
	// StartPort used in both bounds of `StartPort..StartPort + 100`.
	super::assert_unchanged_and_compiles(
		r#"fn f() -> u16 {
            let StartPort = 9000u16;
            for Port in StartPort..StartPort + 100 {
                if Port > 9050 { return Port; }
            }
            0
        }"#,
		"",
		"MultiUseKept__RangeDoubleBoundKept",
	);
}
