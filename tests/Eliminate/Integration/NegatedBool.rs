//! Integration test: negated bool guard inlined into `if` condition.

#[test]
fn NegatedBoolGuardInlined() {
	super::assert_eliminates_and_compiles(
		r#"fn f(Scheme: &str) -> Result<(), String> {
            let IsKnown = ALLOWED.contains(&Scheme);
            if !IsKnown {
                return Err("unknown scheme".into());
            }
            Ok(())
        }"#,
		r#"fn f(Scheme: &str) -> Result<(), String> {
            if !ALLOWED.contains(&Scheme) {
                return Err("unknown scheme".into());
            }
            Ok(())
        }"#,
		"static ALLOWED: &[&str] = &[\"http\", \"https\"];",
		"NegatedBool__NegatedBoolGuardInlined",
	);
}
