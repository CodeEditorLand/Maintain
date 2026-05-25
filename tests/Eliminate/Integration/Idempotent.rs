//! Integration test: running the transform twice produces no further change,
//! and the first-pass output compiles.

use Maintain::Eliminate::{Definition::Options, Transform};

#[test]
fn Idempotent() {
	let Opts = Options::default();

	let Src = r#"fn f() { let X = 5i32; println!("{}", X); }"#;

	let First = Transform::Run(Src, &Opts).unwrap().expect("first pass should produce a change");

	let Second = Transform::Run(&First, &Opts).unwrap();

	assert!(Second.is_none(), "second pass must be a no-op:\n{}", First);

	// Compile the first-pass output to prove it is valid Rust.
	let Code = format!("#![allow(unused, dead_code, non_snake_case)]\n{}", First);

	super::compile(&Code, "Idempotent__Idempotent");
}
