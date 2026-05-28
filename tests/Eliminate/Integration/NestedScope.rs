//! Integration test: inner and outer single-use bindings each eliminated in
//! their respective scopes.

#[test]
fn NestedScopeInlined() {
	super::assert_eliminates_and_compiles(
		r#"fn f() {
            let Outer = 99i32;
            {
                let Inner = 42i32;

                use_inner(Inner);
            }
            use_outer(Outer);
        }"#,
		r#"fn f() {
            { use_inner(42i32); }
            use_outer(99i32);
        }"#,
		"fn use_inner(_: i32) {} fn use_outer(_: i32) {}",
		"NestedScope__NestedScopeInlined",
	);
}
