//! Compare two statements by their token stream text.

pub fn Fn(A:&syn::Stmt, B:&syn::Stmt) -> bool {
	use quote::ToTokens;

	let mut Ta = proc_macro2::TokenStream::new();

	let mut Tb = proc_macro2::TokenStream::new();

	A.to_tokens(&mut Ta);

	B.to_tokens(&mut Tb);

	Ta.to_string() == Tb.to_string()
}
