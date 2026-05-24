//! Integration test: binding inlined as match scrutinee.

#[test]
fn OptionBindingIntoMatchScrutinee() {
	super::assert_eliminates_and_compiles(
		r#"fn f(items: &[u64]) -> Result<u64, String> {
            let TermId = items.iter().copied().find(|x| *x > 0);
            match TermId {
                Some(Id) if Id > 0 => Ok(Id),
                _ => Err("invalid id".into()),
            }
        }"#,
		r#"fn f(items: &[u64]) -> Result<u64, String> {
            match items.iter().copied().find(|x| *x > 0) {
                Some(Id) if Id > 0 => Ok(Id),
                _ => Err("invalid id".into()),
            }
        }"#,
		"",
		"MatchScrutinee__OptionBindingIntoMatchScrutinee",
	);
}

#[test]
fn MatchArmWithEarlyReturnInlined() {
	// A `return` inside a match arm that is a sub-expression of `Ok(…)`.
	super::assert_eliminates_and_compiles(
		r#"fn f(x: Option<i32>) -> Result<i32, ()> {
            let Key = match x {
                Some(K) => K,
                None => return Ok(0),
            };
            Ok(Key * 2)
        }"#,
		r#"fn f(x: Option<i32>) -> Result<i32, ()> {
            Ok(match x {
                Some(K) => K,
                None => return Ok(0),
            } * 2)
        }"#,
		"",
		"MatchScrutinee__MatchArmWithEarlyReturnInlined",
	);
}
