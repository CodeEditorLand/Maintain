//! Try to preserve a single item by walking its block(s).

use crate::Eliminate::{Definition, Error};
use super::TryBlockPreserve;

pub fn Fn(Item:&syn::Item, Working:&str, Options:&Definition::Options) -> Error::Result<Option<String>> {
	match Item {
		syn::Item::Fn(F) => TryBlockPreserve::Fn(&F.block, Working, Options),

		syn::Item::Impl(I) => {
			for ImplItem in &I.items {
				if let syn::ImplItem::Fn(M) = ImplItem {
					if let Some(R) = TryBlockPreserve::Fn(&M.block, Working, Options)? {
						return Ok(Some(R));
					}
				}
			}

			Ok(None)
		},

		_ => Ok(None),
	}
}
