//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/mod.rs
//=============================================================================//
// Module: Transform - AST transformation pipeline
//
// Entry point: `Run(source, options)` parses the Rust source with `syn`,
// applies iterative single-use-variable elimination, and re-formats the result
// with `prettyplease`. Returns `Ok(None)` when the source is already minimal.
//=============================================================================//

pub mod Collect;
pub mod Count;
pub mod Inline;
pub mod Safe;

use super::{Definition, Error};

/// Parse `Source`, run up to [`super::Constant::MaxIterations`] elimination
/// passes, then format and return the result.
///
/// Returns `Ok(None)` when no bindings were eliminated (caller can skip the
/// write-back).
pub fn Run(Source: &str, Options: &Definition::Options) -> Error::Result<Option<String>> {
	let mut Ast: syn::File = syn::parse_str(Source).map_err(|E| Error::Error::Parse {
		Path: String::new(),
		Source: E,
	})?;

	let mut AnyChanged = false;

	for _ in 0..super::Constant::MaxIterations {
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

	Ok(Some(prettyplease::unparse(&Ast)))
}
