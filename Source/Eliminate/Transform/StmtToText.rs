//! Render a single `syn::Stmt` to its canonical text representation by
//! wrapping it in a dummy function body and extracting the inner line(s).
//! The wrapper indentation (one tab or 4 spaces from prettyplease) is
//! stripped so the result is indentation-relative.

pub fn Fn(Stmt:&syn::Stmt) -> String {
	use quote::quote;

	let Wrapped:syn::File = syn::parse_quote! { fn __d() { #Stmt } };

	let Full = prettyplease::unparse(&Wrapped);

	// Full looks like "fn __d() {\n    <stmt>\n}\n".
	// Extract between first '{' and last '}'.
	if let (Some(Open), Some(Close)) = (Full.find('{'), Full.rfind('}')) {
		let Inner = Full[Open + 1..Close].trim_matches('\n');

		return Inner
			.lines()
			.map(|L| L.strip_prefix('\t').or_else(|| L.strip_prefix("    ")).unwrap_or(L))
			.collect::<Vec<_>>()
			.join("\n");
	}

	Full
}
