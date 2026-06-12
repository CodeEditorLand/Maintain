//! Parse `Source`, run up to `MaxIterations` elimination passes, then return
//! the patched source text.
//!
//! Preferred path: span-based text patching via `TryPatchSource` preserves
//! inline comments and blank lines. Falls back to `prettyplease::unparse`
//! when span-location data is unavailable.
//!
//! Returns `Ok(None)` when no bindings were eliminated.

use crate::Eliminate::{Constant, Definition, Error};
use super::{Inline, TryPatchSource};

pub fn Fn(Source:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let mut Ast:syn::File = syn::parse_str(Source).map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

	let mut AnyChanged = false;

	for _ in 0..Constant::MaxIterations {
		let mut Eliminator = Inline::Eliminator::new(Options);

		syn::visit_mut::visit_file_mut(&mut Eliminator, &mut Ast);

		if Eliminator.Changed {
			AnyChanged = true;
		} else {
			break;
		}
	}

	if !AnyChanged {
		return Ok(None);
	}

	if let Some(Patched) = TryPatchSource::Fn(Source, &Ast) {
		return Ok(Some(Patched));
	}

	Ok(Some(prettyplease::unparse(&Ast)))
}
