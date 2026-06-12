//! Identify inlinable bindings via the same AST pipeline as `Run`, but apply
//! the substitutions as targeted text edits so that every character outside
//! the affected `let` binding and its single use-site is preserved verbatim.
//!
//! Uses no `proc_macro2` span APIs; works on stable Rust with the dependency
//! set already declared in `Cargo.toml`.
//!
//! Returns `Ok(None)` when no bindings were eliminated.

use crate::Eliminate::{Constant, Definition, Error};
use super::PreservePass;

pub fn Fn(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let mut Working = Source.to_owned();

	let mut AnyChanged = false;

	for _ in 0..Constant::MaxIterations {
		match PreservePass::Fn(&Working, Options)? {
			Some(Next) => {
				Working = Next;

				AnyChanged = true;
			},

			None => break,
		}
	}

	if AnyChanged { Ok(Some(Working)) } else { Ok(None) }
}
