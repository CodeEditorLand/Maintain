//! Integration test: long method-chain binding inlined into a struct field.
//! Mirrors Stat.rs: MTime used only as the `mtime` field; Metadata stays
//! because it is used four times.

#[test]
fn MtimeChainIntoStructField() {
	super::assert_eliminates_and_compiles(
		r#"fn stat(data: &[u64]) -> Report {
            let MTime = data
                .iter()
                .copied()
                .filter(|x| *x > 0)
                .max()
                .unwrap_or(0);
            Report {
                count: data.len() as u64,
                mtime: MTime,
            }
        }"#,
		r#"fn stat(data: &[u64]) -> Report {
            Report {
                count: data.len() as u64,
                mtime: data.iter().copied().filter(|x| *x > 0).max().unwrap_or(0),
            }
        }"#,
		"struct Report { count: u64, mtime: u64 }",
		"MtimeChain__MtimeChainIntoStructField",
	);
}

/// `Metadata` is multi-use (count + is_file + mtime chain); only `MTime`
/// is single-use and gets inlined.
#[test]
fn MultiUseDataKept() {
	super::assert_unchanged_and_compiles(
		r#"fn stat2(data: &[u64]) -> Report2 {
            Report2 {
                count: data.len() as u64,
                max:   data.iter().copied().max().unwrap_or(0),
                sum:   data.iter().copied().sum(),
            }
        }"#,
		"struct Report2 { count: u64, max: u64, sum: u64 }",
		"MtimeChain__MultiUseDataKept",
	);
}
