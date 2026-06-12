//! Extract a directly nested `Block` from a statement for recursion.

pub fn Fn(Stmt:&syn::Stmt) -> Option<&syn::Block> {
	if let syn::Stmt::Expr(Expr, _) = Stmt {
		match Expr {
			syn::Expr::Block(B) => return Some(&B.block),

			syn::Expr::If(I) => return Some(&I.then_branch),

			syn::Expr::Loop(L) => return Some(&L.body),

			syn::Expr::While(W) => return Some(&W.body),

			syn::Expr::ForLoop(F) => return Some(&F.body),

			syn::Expr::Unsafe(U) => return Some(&U.block),

			_ => {},
		}
	}

	None
}
