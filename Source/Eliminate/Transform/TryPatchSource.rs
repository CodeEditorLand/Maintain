//! Attempt to reconstruct the output by splicing only the changed ranges.
//! Returns None on any span resolution failure, triggering the fallback.

use super::{CollectBlockPatches, Patch};

pub fn Fn(Source:&str, MutatedAst:&syn::File) -> Option<String> {
	let OriginalAst:syn::File = syn::parse_str(Source).ok()?;

	let mut Patches:Vec<Patch::Patch> = Vec::new();

	for (OrigItem, MutItem) in OriginalAst.items.iter().zip(MutatedAst.items.iter()) {
		if let (syn::Item::Fn(OrigFn), syn::Item::Fn(MutFn)) = (OrigItem, MutItem) {
			CollectBlockPatches::Fn(&OrigFn.block, &MutFn.block, Source, &mut Patches)?;
		}
	}

	if Patches.is_empty() {
		return None;
	}

	Patches.sort_by_key(|P| P.Start);

	Patch::ApplyPatches(Source, &Patches)
}
