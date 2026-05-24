//! Integration test: type annotation on the let binding is dropped when
//! inlined; the compiler infers the type at the use site.

#[test]
fn TypeAnnotationDropped() {
	// `g` is declared to accept `i32` so the compiler can infer the type
	// of the literal `5` after the annotation is removed.
	super::assert_eliminates_and_compiles(
		"fn f() { let X: i32 = 5; g(X); }",
		"fn f() { g(5); }",
		"fn g(_: i32) {}",
		"TypeAnnotation__TypeAnnotationDropped",
	);
}

#[test]
fn TypeAnnotatedArrayInlined() {
	// Type annotation with a non-trivial type ([u8; 12]) is dropped; the
	// compiler infers from the use site.
	super::assert_eliminates_and_compiles(
		"fn f(B: &[u8]) { let Nonce: [u8; 4] = B[..4].try_into().unwrap(); use_nonce(Nonce); }",
		"fn f(B: &[u8]) { use_nonce(B[..4].try_into().unwrap()); }",
		"fn use_nonce(_: [u8; 4]) {}",
		"TypeAnnotation__TypeAnnotatedArrayInlined",
	);
}
