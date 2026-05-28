//! Integration test: bindings local to a closure body are each eliminated,
//! collapsing the closure to a direct struct expression.
//! The function return type anchors type inference for `.collect()`.

#[test]
fn ClosureLocalBindingsInlined() {
	super::assert_eliminates_and_compiles(
		r#"fn f(items: &[u32]) -> Vec<Item> {
            items
                .iter()
                .map(|val| {
                    let Handle = val.to_string();

                    let Label = format!("item-{}", val);

                    Item { handle: Handle, label: Label }
                })
                .collect()
        }"#,
		r#"fn f(items: &[u32]) -> Vec<Item> {
            items
                .iter()
                .map(|val| {
                    Item {
                        handle: val.to_string(),
                        label: format!("item-{}", val),
                    }
                })
                .collect()
        }"#,
		"struct Item { handle: String, label: String }",
		"ClosureLocal__ClosureLocalBindingsInlined",
	);
}
