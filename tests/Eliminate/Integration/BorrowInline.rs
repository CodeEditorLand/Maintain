//! Integration test: borrow initialisers and borrows of inline temporaries.
//! The critical compile check: rustc confirms the temporary lifetime is
//! sufficient in the `BorrowOfInlineExpr` case.

#[test]
fn BorrowInlined() {
	super::assert_eliminates_and_compiles(
		"fn f() { let X = &FOO; bar(X); }",
		"fn f() { bar(&FOO); }",
		"static FOO: i32 = 0; fn bar(_: &i32) {}",
		"BorrowInline__BorrowInlined",
	);
}

/// The transformed code creates a temporary `PathBuf` via `.join()` and
/// immediately borrows it.  The temporary lives for the entire statement,
/// which is long enough for `create_dir_all`.  This COMPILE test proves that
/// rustc agrees with the transform.
#[test]
fn BorrowOfInlineExprCompiles() {
	super::assert_eliminates_and_compiles(
		r#"
            use std::path::PathBuf;
            pub fn Fn(Base: &PathBuf) -> std::io::Result<()> {
                let Dir = Base.join("window1");

                std::fs::create_dir_all(&Dir)?;

                Ok(())
            }
        "#,
		r#"
            use std::path::PathBuf;
            pub fn Fn(Base: &PathBuf) -> std::io::Result<()> {
                std::fs::create_dir_all(&Base.join("window1"))?;

                Ok(())
            }
        "#,
		// No preamble - the snippet already imports PathBuf at the top.
		"",
		"BorrowInline__BorrowOfInlineExprCompiles",
	);
}
