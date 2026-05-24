//! Integration test: block expression inlined as the scrutinee of `if let`.
//! The block drops a Vec before the if-let body runs - borrow-safety must hold.

#[test]
fn BlockExprIntoIfLetScrutinee() {
	super::assert_eliminates_and_compiles(
		r#"fn f(key: &str) -> Option<String> {
            let MaybePrimary = {
                let v = vec!["alpha".to_string(), "beta".to_string()];
                v.iter().find(|s| s.as_str() == key).cloned()
            };
            if let Some(PrimaryHandle) = MaybePrimary {
                Some(PrimaryHandle)
            } else {
                None
            }
        }"#,
		r#"fn f(key: &str) -> Option<String> {
            if let Some(PrimaryHandle) = {
                vec!["alpha".to_string(), "beta".to_string()]
                    .iter()
                    .find(|s| s.as_str() == key)
                    .cloned()
            } {
                Some(PrimaryHandle)
            } else {
                None
            }
        }"#,
		"",
		"BlockIntoIfLet__BlockExprIntoIfLetScrutinee",
	);
}
