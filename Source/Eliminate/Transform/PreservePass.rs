//! One pass: parse `Working`, find the first inlinable binding, apply the
//! text edit, return `Some(new_text)`. Returns `None` when nothing changed.

use crate::Eliminate::{Definition, Error};
use super::TryItemPreserve;

pub fn Fn(Working:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	let Ast:syn::File = syn::parse_str(Working).map_err(|E| Error::Error::Parse { Path:String::new(), Source:E })?;

	for Item in &Ast.items {
		if let Some(Result) = TryItemPreserve::Fn(Item, Working, Options)? {
			return Ok(Some(Result));
		}
	}

	Ok(None)
}
