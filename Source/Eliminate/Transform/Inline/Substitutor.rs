//! Internal Substitutor visitor for replacing identifier references.

use proc_macro2::{Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{
	Expr,
	Stmt,
	visit_mut::{VisitMut, visit_expr_mut},
};

use super::{
	BlockShadowsTarget,
	ClosureParamShadows,
	ExprToTokenStream,
	IsTargetIdent,
	NeedsParen,
	SubstituteInTokenStream,
};

// ---------------------------------------------------------------------------
// Internal: Substitutor
// ---------------------------------------------------------------------------

pub struct Substitutor<'a> {
	pub Target:&'a str,

	pub Replacement:&'a Expr,

	pub Substituted:bool,

	/// True when the current AST position is a direct operand of a binary or
	/// unary expression - used to decide whether to wrap Replacement.
	pub InBinaryOperandPosition:bool,
}

impl<'a> VisitMut for Substitutor<'a> {
	fn visit_expr_mut(&mut self, Node:&mut Expr) {
		if self.Substituted {
			return;
		}

		if IsTargetIdent(Node, self.Target) {
			let NeedsWrapping = self.InBinaryOperandPosition && NeedsParen(self.Replacement);

			*Node = if NeedsWrapping {
				Expr::Paren(syn::ExprParen {
					attrs:vec![],
					paren_token:Default::default(),
					expr:Box::new(self.Replacement.clone()),
				})
			} else {
				self.Replacement.clone()
			};

			self.Substituted = true;

			return;
		}

		// Propagate binary-operand context for children.
		match Node {
			Expr::Binary(B) => {
				let Saved = self.InBinaryOperandPosition;

				self.InBinaryOperandPosition = true;

				self.visit_expr_mut(&mut B.left);

				if !self.Substituted {
					self.visit_expr_mut(&mut B.right);
				}

				self.InBinaryOperandPosition = Saved;
			},

			Expr::Unary(U) => {
				let Saved = self.InBinaryOperandPosition;

				self.InBinaryOperandPosition = true;

				self.visit_expr_mut(&mut U.expr);

				self.InBinaryOperandPosition = Saved;
			},

			_ => {
				let Saved = self.InBinaryOperandPosition;

				self.InBinaryOperandPosition = false;

				visit_expr_mut(self, Node);

				self.InBinaryOperandPosition = Saved;
			},
		}
	}

	/// Substitute inside macro token streams (e.g. json!(), dev_log!()).
	/// The default VisitMut does NOT recurse into Macro::tokens.
	fn visit_expr_macro_mut(&mut self, Node:&mut syn::ExprMacro) {
		if self.Substituted {
			return;
		}

		let ReplacementTokens = ExprToTokenStream(self.Replacement);

		let (NewTokens, Found) = SubstituteInTokenStream(Node.mac.tokens.clone(), self.Target, &ReplacementTokens);

		if Found {
			Node.mac.tokens = NewTokens;

			self.Substituted = true;
		}
	}

	/// syn v2 separates top-level macro statements (dev_log!("{}", X);) into
	/// Stmt::Macro(StmtMacro), which is never routed through
	/// visit_expr_macro_mut. Mirror the same substitution here.
	fn visit_stmt_macro_mut(&mut self, Node:&mut syn::StmtMacro) {
		if self.Substituted {
			return;
		}

		let ReplacementTokens = ExprToTokenStream(self.Replacement);

		let (NewTokens, Found) = SubstituteInTokenStream(Node.mac.tokens.clone(), self.Target, &ReplacementTokens);

		if Found {
			Node.mac.tokens = NewTokens;

			self.Substituted = true;
		}
	}

	// Skip inner blocks that shadow Target - mirrors Count logic.
	fn visit_block_mut(&mut self, Block:&mut syn::Block) {
		if BlockShadowsTarget(&Block.stmts, self.Target) {
			return;
		}

		syn::visit_mut::visit_block_mut(self, Block);
	}

	// Skip closures whose parameter shadows Target.
	fn visit_expr_closure_mut(&mut self, Node:&mut syn::ExprClosure) {
		if ClosureParamShadows(Node, self.Target) {
			return;
		}

		syn::visit_mut::visit_expr_closure_mut(self, Node);
	}
}
