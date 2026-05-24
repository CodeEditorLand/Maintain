//! Integration test: plain literal inlined at its single use site.
//! Compile target: fn f() { println!("{}", 5); }

#[test]
fn SimpleInline() {
	super::assert_eliminates_and_compiles(
		r#"fn f() { let X = 5; println!("{}", X); }"#,
		r#"fn f() { println!("{}", 5); }"#,
		"",
		"Simple__SimpleInline",
	);
}
