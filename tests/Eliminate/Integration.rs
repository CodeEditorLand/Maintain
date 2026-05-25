#![allow(
	non_snake_case,
	non_camel_case_types,
	non_upper_case_globals,
	dead_code,
	unused_imports,
	unused_variables
)]
//=============================================================================//
// File Path: Element/Maintain/tests/Eliminate/Integration.rs
//=============================================================================//
// Integration test suite for the Eliminate module.
//
// Each sub-module:
//   (a) asserts the transformed output matches the expected string (same AST
//       comparison as Syntactic), AND
//   (b) compiles the *transformed* snippet with
//           rustc --edition=2024 --crate-type=lib
//       to prove that the output is semantically valid Rust, not just
//       syntactically equivalent to the expected string.
//
// Every test snippet is self-contained: it uses only `std` / primitives and
// inline mock-type definitions so it can be handed directly to `rustc` with no
// external crate dependencies.
//
// Skip the compile step (fast / no-rustc CI):
//   SKIP_COMPILE=1 cargo test --test Integration
//
// Run both pipelines together:
//   cargo test --test Syntactic --test Integration
//   cargo test            # runs both because both are declared in Cargo.toml
//=============================================================================//

use Maintain::Eliminate::{Definition::Options, Transform};

// ---------------------------------------------------------------------------
// Sub-module declarations - one file per test or closely related test group
// ---------------------------------------------------------------------------

// When Integration.rs is the test-crate root, Rust resolves plain `mod X;`
// as a sibling file (tests/Eliminate/X.rs).  Use explicit `#[path]` to point
// into the Integration/ subdirectory instead.
#[path = "Integration/BinaryParens.rs"]
mod BinaryParens;
#[path = "Integration/BlockIntoIfLet.rs"]
mod BlockIntoIfLet;
#[path = "Integration/BorrowInline.rs"]
mod BorrowInline;
#[path = "Integration/CastExpr.rs"]
mod CastExpr;
#[path = "Integration/Chain.rs"]
mod Chain;
#[path = "Integration/ClosureCaptureKept.rs"]
mod ClosureCaptureKept;
#[path = "Integration/ClosureLocal.rs"]
mod ClosureLocal;
#[path = "Integration/Idempotent.rs"]
mod Idempotent;
#[path = "Integration/IfGuard.rs"]
mod IfGuard;
#[path = "Integration/LoopUri.rs"]
mod LoopUri;
#[path = "Integration/MatchScrutinee.rs"]
mod MatchScrutinee;
#[path = "Integration/MtimeChain.rs"]
mod MtimeChain;
#[path = "Integration/MultiUseKept.rs"]
mod MultiUseKept;
#[path = "Integration/NegatedBool.rs"]
mod NegatedBool;
#[path = "Integration/NestedScope.rs"]
mod NestedScope;
#[path = "Integration/QuestionMark.rs"]
mod QuestionMark;
#[path = "Integration/Shadow.rs"]
mod Shadow;
#[path = "Integration/Simple.rs"]
mod Simple;
#[path = "Integration/StructLiteral.rs"]
mod StructLiteral;
#[path = "Integration/TypeAnnotation.rs"]
mod TypeAnnotation;

// ---------------------------------------------------------------------------
// Shared helpers (accessed from sub-modules as `super::function_name(…)`)
// ---------------------------------------------------------------------------

/// Run the elimination transform and return the normalised output.
/// Returns the normalised *original* when no transformation was applied.
pub fn transform(Src: &str) -> String {
	transform_with(Src, Options::default())
}

pub fn transform_with(Src: &str, Opts: Options) -> String {
	Transform::Run(Src, &Opts)
		.expect("transform failed")
		.unwrap_or_else(|| {
			let Ast: syn::File = syn::parse_str(Src).unwrap();

			prettyplease::unparse(&Ast)
		})
}

/// Normalise `Src` through prettyplease so whitespace differences are ignored.
pub fn norm(Src: &str) -> String {
	let Ast: syn::File = syn::parse_str(Src).unwrap();

	prettyplease::unparse(&Ast)
}

/// Assert the transformation output matches `Expected`.
pub fn assert_eliminates(Input: &str, Expected: &str) {
	assert_eq!(transform(Input), norm(Expected));
}

/// Assert the input is not changed by the transform.
pub fn assert_unchanged(Input: &str) {
	let Opts = Options::default();

	let Result = Transform::Run(Input, &Opts).expect("transform");

	assert!(
		Result.is_none(),
		"expected no change but got:\n{}",
		Result.unwrap()
	);
}

/// Write `Code` to a temp file and invoke
/// `rustc --edition=2024 --crate-type=lib` on it.  Panics when rustc exits
/// non-zero.
///
/// `Label` identifies the test (e.g. `"Simple__SimpleInline"`) and is used as
/// the temp filename so concurrent tests do not collide.
///
/// Set `SKIP_COMPILE=1` to bypass the compile step.
pub fn compile(Code: &str, Label: &str) {
	if std::env::var("SKIP_COMPILE").as_deref() == Ok("1") {
		return;
	}

	use std::io::Write;

	let TmpDir = std::env::temp_dir();

	let SrcPath = TmpDir.join(format!("eliminate_integ_{}.rs", Label));

	{
		let mut F = std::fs::File::create(&SrcPath)
			.unwrap_or_else(|E| panic!("create temp file {}: {}", SrcPath.display(), E));

		F.write_all(Code.as_bytes())
			.unwrap_or_else(|E| panic!("write temp file {}: {}", SrcPath.display(), E));
	}

	let Out = std::process::Command::new("rustc")
		.arg("--edition=2024")
		.arg("--crate-type=lib")
		.arg("--emit=metadata")
		.arg("--out-dir")
		.arg(&TmpDir)
		.arg(&SrcPath)
		.output()
		.unwrap_or_else(|E| {
			panic!(
				"failed to launch rustc - ensure rustc is in PATH (or set SKIP_COMPILE=1): {}",
				E
			)
		});

	std::fs::remove_file(&SrcPath).ok();

	assert!(
		Out.status.success(),
		"transformed code did not compile for {}:\n\nSOURCE:\n{}\n\nSTDERR:\n{}",
		Label,
		Code,
		String::from_utf8_lossy(&Out.stderr),
	);
}

/// Run the full assertion: (a) transform output matches `Expected`, and
/// (b) the transformed output compiles.
///
/// `Preamble` is prepended to the transformed source before compilation
/// (use it to define mock types/functions referenced by the snippet).
pub fn assert_eliminates_and_compiles(Input: &str, Expected: &str, Preamble: &str, Label: &str) {
	assert_eliminates(Input, Expected);

	let Transformed = transform(Input);

	let FullCode = format!(
		"#![allow(unused, dead_code, non_snake_case, non_camel_case_types)]\n{}\n{}",
		Preamble, Transformed
	);

	compile(&FullCode, Label);
}

/// Assert the input is unchanged AND that the (normalised) input compiles.
pub fn assert_unchanged_and_compiles(Input: &str, Preamble: &str, Label: &str) {
	assert_unchanged(Input);

	let FullCode = format!(
		"#![allow(unused, dead_code, non_snake_case, non_camel_case_types)]\n{}\n{}",
		Preamble,
		norm(Input)
	);

	compile(&FullCode, Label);
}
