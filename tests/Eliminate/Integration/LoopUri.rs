//! Integration test: loop-local binding inlined into an `if let` scrutinee.
//! Uses std::str::strip_prefix (same structural pattern as url::Url::parse).

#[test]
fn LoopUriIntoIfLetParse() {
	super::assert_eliminates_and_compiles(
		r#"fn f(additions: &[String], folders: &mut Vec<String>) {
            for addition in additions {
                let URI = addition.as_str();
                if let Some(Parsed) = URI.strip_prefix("file://") {
                    folders.push(Parsed.to_string());
                }
            }
        }"#,
		r#"fn f(additions: &[String], folders: &mut Vec<String>) {
            for addition in additions {
                if let Some(Parsed) = addition.as_str().strip_prefix("file://") {
                    folders.push(Parsed.to_string());
                }
            }
        }"#,
		"",
		"LoopUri__LoopUriIntoIfLetParse",
	);
}
