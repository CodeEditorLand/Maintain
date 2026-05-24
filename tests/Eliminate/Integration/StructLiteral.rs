//! Integration test: struct literal inlined at its single use site.

#[test]
fn StructInlined() {
	super::assert_eliminates_and_compiles(
		"fn f() { let Opts = MyOpts { a: 1u32, b: 2u32 }; call(Opts); }",
		"fn f() { call(MyOpts { a: 1u32, b: 2u32 }); }",
		"struct MyOpts { a: u32, b: u32 } fn call(_: MyOpts) {}",
		"StructLiteral__StructInlined",
	);
}

/// Struct field value inlined: multiple single-use bindings each go into
/// the corresponding field.
#[test]
fn StructFieldsInlined() {
	super::assert_eliminates_and_compiles(
		r#"fn make(n: u32) -> Point {
            let X = n * 2;
            let Y = n + 1;
            Point { x: X, y: Y }
        }"#,
		r#"fn make(n: u32) -> Point {
            Point { x: n * 2, y: n + 1 }
        }"#,
		"struct Point { x: u32, y: u32 }",
		"StructLiteral__StructFieldsInlined",
	);
}
