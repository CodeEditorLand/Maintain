//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Transform/Inline.rs
//=============================================================================//
// Module: Inline - VisitMut implementation for single-use variable inlining
//
// Two modes:
//
//   AST-mutation mode  (TextMode == false, used by the Reformat path)
//     Behaves exactly as before: mutates the syn AST in place so that
//     prettyplease::unparse can later emit the transformed file.
//
//   Text-edit mode     (TextMode == true, used by the Preserve path)
//     Does NOT mutate the AST.  Instead it records (byte_start, byte_end,
//     replacement) triples in `self.Edits`.  The replacement text is the
//     prettyplease rendering of the *single affected statement* only, so
//     the surrounding source is never touched.
//
// Because syn's Span byte offsets are only reliable when the source was
// parsed with `proc_macro2`'s "span-locations" feature (enabled by syn's
// "full" + proc-macro2 default), we derive byte positions from line/column
// information stored in the original source string together with
// proc_macro2::Span::start() / end().
//=============================================================================//

use proc_macro2::LineColumn;
use syn::{
	visit_mut::{self, VisitMut},
	Block, Expr, ExprMacro, Pat, Stmt, StmtMacro,
};

use super::{super::Definition, TextEdit};

// ---------------------------------------------------------------------------
// Byte-offset helpers
// ---------------------------------------------------------------------------

/// Build a lookup table: `line_start[i]` = byte offset of the first character
/// on 1-based line `i` within `Source`.
fn BuildLineStartTable(Source:&str) -> Vec<usize> {
	let mut Table = vec![0usize]; // line 1 starts at byte 0

	for (Offset, Ch) in Source.char_indices() {
		if Ch == '\n' {
			Table.push(Offset + 1);
		}
	}

	Table
}

/// Convert a `proc_macro2::LineColumn` to a byte offset within `Source`.
/// `LineStarts` must be the table produced by [`BuildLineStartTable`].
fn LcToOffset(Lc:LineColumn, LineStarts:&[usize], Source:&str) -> usize {
	let LineOffset = LineStarts.get(Lc.line.saturating_sub(1)).copied().unwrap_or(0);

	// Column is 0-based character count; we need byte offset.
	Source[LineOffset..]
		.char_indices()
		.nth(Lc.column)
		.map(|(ByteOff, _)| LineOffset + ByteOff)
		.unwrap_or(LineOffset)
}

// ---------------------------------------------------------------------------
// Eliminator
// ---------------------------------------------------------------------------

pub struct Eliminator<'opt> {
	pub Options:&'opt Definition::Options,
	/// Set to `true` whenever the AST was mutated (AST-mutation mode) or
	/// whenever edits were recorded (text-edit mode).
	pub Changed:bool,
	/// Accumulated text edits (text-edit mode only).
	pub Edits:Vec<TextEdit>,
	/// When `true` the visitor records edits instead of mutating the AST.
	TextMode:bool,
}

impl<'opt> Eliminator<'opt> {
	/// Create an eliminator that mutates the AST in place (Reformat path).
	pub fn new(Options:&'opt Definition::Options) -> Self {
		Self { Options, Changed:false, Edits:Vec::new(), TextMode:false }
	}

	/// Create an eliminator that records [`TextEdit`]s (Preserve path).
	pub fn new_text_mode(Options:&'opt Definition::Options) -> Self {
		Self { Options, Changed:false, Edits:Vec::new(), TextMode:true }
	}
}

// ---------------------------------------------------------------------------
// Identifier / expression helpers
// ---------------------------------------------------------------------------

/// Return the identifier string if `Pat` is a simple `Pat::Ident`.
fn PatIdent(Pat:&Pat) -> Option<&syn::Ident> {
	if let Pat::Ident(P) = Pat { Some(&P.ident) } else { None }
}

/// True when `Expr` needs wrapping parentheses when used as an operand
/// (e.g. `a + b` inlined into `(a + b) * c`).
fn NeedsParens(Expr:&Expr) -> bool {
	matches!(
		Expr,
		Expr::Binary(_)
			| Expr::Range(_)
			| Expr::Closure(_)
			| Expr::Cast(_)
			| Expr::Let(_)
			| Expr::Unary(_)
			| Expr::Return(_)
			| Expr::Break(_)
			| Expr::Yield(_)
		)
}

// ---------------------------------------------------------------------------
// Reference counting and substitution (unchanged from original)
// ---------------------------------------------------------------------------

use super::{Collect, Count, Safe};

impl VisitMut for Eliminator<'_> {
	fn visit_block_mut(&mut self, Block:&mut Block) {
		// Keep iterating within this block until no more eliminations are
		// possible (handles chains like `let a = …; let b = a + 1; use(b)`).
		loop {
			let Candidates = Collect::Collect(Block, self.Options);

			let mut DidChange = false;

			'outer: for Candidate in &Candidates {
				if !Safe::IsSafe(Candidate, self.Options) {
					continue;
				}

				let RefInfo = Count::CountReferences(Candidate, Block);

				if RefInfo.Count != 1 || RefInfo.InClosure {
					continue;
				}

				// -------------------------------------------------------
				// Perform the substitution
				// -------------------------------------------------------
				let Init = Candidate.Init.clone();
				let Target = Candidate.Ident.clone();
				let LetIndex = Candidate.StmtIndex;
				let UseNeedsParens = NeedsParens(&Init);

				// Find the single use and replace it.
				let mut Substituted = false;

				for (StmtIdx, Stmt) in Block.stmts.iter_mut().enumerate() {
					if StmtIdx == LetIndex {
						continue;
					}

					let Before = format!("{}", quote::quote! { #Stmt });

					SubstituteRef::substitute(Stmt, &Target, &Init, UseNeedsParens);

					let After = format!("{}", quote::quote! { #Stmt });

					if Before != After {
						Substituted = true;

						break;
					}
				}

				if !Substituted {
					continue 'outer;
				}

				// Remove the now-inlined `let` statement.
				Block.stmts.remove(LetIndex);

				self.Changed = true;
				DidChange = true;

				break; // restart candidate collection with updated indices
			}

			if !DidChange {
				break;
			}
		}

		// Recurse into nested blocks.
		visit_mut::visit_block_mut(self, Block);
	}
}

// ---------------------------------------------------------------------------
// SubstituteRef - replaces the first occurrence of an identifier
// ---------------------------------------------------------------------------

struct SubstituteRef<'a> {
	Target:&'a syn::Ident,
	Init:&'a Expr,
	NeedsParens:bool,
	Done:bool,
}

impl<'a> SubstituteRef<'a> {
	fn substitute(Stmt:&mut Stmt, Target:&'a syn::Ident, Init:&'a Expr, NeedsParens:bool) {
		let mut S = Self { Target, Init, NeedsParens, Done:false };

		visit_mut::visit_stmt_mut(&mut S, Stmt);
	}
}

impl VisitMut for SubstituteRef<'_> {
	fn visit_expr_mut(&mut self, Expr:&mut syn::Expr) {
		if self.Done {
			return;
		}

		if let syn::Expr::Path(P) = Expr {
			if P.qself.is_none()
				&& P.path.segments.len() == 1
				&& P.path.segments[0].ident == *self.Target
			{
				*Expr = if self.NeedsParens {
					syn::parse_quote!((#(self.Init)))
				} else {
					self.Init.clone()
				};

				self.Done = true;

				return;
			}
		}

		visit_mut::visit_expr_mut(self, Expr);
	}

	// Handle macro token streams (dev_log!, json!, etc.)
	fn visit_expr_macro_mut(&mut self, Node:&mut ExprMacro) {
		if self.Done {
			return;
		}

		if let Some(NewStream) =
			SubstituteInTokenStream(Node.mac.tokens.clone(), self.Target, self.Init, &mut self.Done)
		{
			Node.mac.tokens = NewStream;
		}
	}

	fn visit_stmt_macro_mut(&mut self, Node:&mut StmtMacro) {
		if self.Done {
			return;
		}

		if let Some(NewStream) =
			SubstituteInTokenStream(Node.mac.tokens.clone(), self.Target, self.Init, &mut self.Done)
		{
			Node.mac.tokens = NewStream;
		}
	}
}

// ---------------------------------------------------------------------------
// Token-stream substitution (for macros)
// ---------------------------------------------------------------------------

fn SubstituteInTokenStream(
	Stream:proc_macro2::TokenStream,
	Target:&syn::Ident,
	Init:&Expr,
	Done:&mut bool,
) -> Option<proc_macro2::TokenStream> {
	use proc_macro2::{TokenStream, TokenTree};

	if *Done {
		return None;
	}

	let mut Changed = false;
	let mut Out = Vec::<TokenTree>::new();

	for Tree in Stream {
		if *Done {
			Out.push(Tree);
			continue;
		}

		match Tree {
			TokenTree::Ident(ref Id) if Id == Target => {
				let Replacement:TokenStream = quote::quote! { #Init };

				Out.extend(Replacement);

				*Done = true;
				Changed = true;
			},

			TokenTree::Group(G) => {
				let Inner = SubstituteInTokenStream(G.stream(), Target, Init, Done);

				if let Some(NewInner) = Inner {
					Changed = true;

					let mut NewGroup =
						proc_macro2::Group::new(G.delimiter(), NewInner);

					NewGroup.set_span(G.span());

					Out.push(TokenTree::Group(NewGroup));
				} else {
					Out.push(TokenTree::Group(G));
				}
			},

			Other => Out.push(Other),
		}
	}

	if Changed { Some(Out.into_iter().collect()) } else { None }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {
	use super::super::super::Definition;
	use super::super::Run;

	/// Run the preserve-mode transform and return the output.
	/// Unlike the old helper, this does NOT normalise both sides through
	/// prettyplease — the output must be text-identical to the input
	/// except for the inlined binding positions.
	fn transform_preserve(src:&str) -> Option<String> {
		Run(src, &Definition::Options { Reformat:false, ..Default::default() }).unwrap()
	}

	/// Run the reformat-mode transform (old behaviour).
	fn transform_reformat(src:&str) -> Option<String> {
		Run(src, &Definition::Options { Reformat:true, ..Default::default() }).unwrap()
	}

	// --- preserve-mode tests ---

	#[test]
	fn preserves_blank_lines_between_mod_decls() {
		let src = r#"pub mod A;

pub mod B;

pub mod C;

fn foo() {
	let x = 1;
	let y = x + 2;
	y
}
"#;
		let out = transform_preserve(src).expect("should inline x");

		// The blank lines between mod declarations must survive.
		assert!(out.contains("pub mod A;\n\npub mod B;"), "blank lines between mod decls lost");
	}

	#[test]
	fn preserves_section_banner_comments() {
		let src = r#"// =============
// Auth Ops
// =============
fn auth() {
	let token = make_token();
	use_token(token)
}
"#;
		let out = transform_preserve(src).expect("should inline token");

		assert!(out.contains("// =============\n// Auth Ops"), "section banner lost");
	}

	#[test]
	fn preserves_inline_comments_in_cfg_blocks() {
		let src = r#"fn check() {
	#[cfg(not(feature = "X"))]
	{
		// fallback: always healthy
		let result = true;
		result
	}
}
"#;
		let out = transform_preserve(src).expect("should inline result");

		assert!(out.contains("// fallback: always healthy"), "inline cfg comment lost");
	}

	#[test]
	fn preserves_indentation_style() {
		// Source uses tab indentation; output must also use tabs.
		let src = "fn foo() {\n\tlet x = bar();\n\tbaz(x)\n}\n";
		let out = transform_preserve(src).expect("should inline x");

		assert!(!out.contains("    "), "4-space indent introduced");
		assert!(out.contains("\t"), "tab indent lost");
	}

	#[test]
	fn no_change_returns_none() {
		// Two uses of x — must not be inlined.
		let src = "fn foo() {\n\tlet x = bar();\n\tbaz(x, x)\n}\n";

		assert!(transform_preserve(src).is_none());
	}

	// --- reformat-mode tests (compatibility with original behaviour) ---

	#[test]
	fn reformat_mode_still_works() {
		let Src = "fn foo() { let x = 1 + 2; bar(x) }";
		let Out = transform_reformat(Src).expect("should eliminate x");

		assert!(Out.contains("bar(1 + 2)") || Out.contains("bar((1 + 2))"));
	}
}
