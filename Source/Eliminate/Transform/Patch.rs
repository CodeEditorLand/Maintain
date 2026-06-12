//! Span-based source text patching.
//!
//! After the AST eliminator runs, locates the byte ranges of the removed
//! let-statements and their substitution sites in the original source text,
//! splicing only those ranges while preserving every other byte.

use proc_macro2::Span;
use quote::ToTokens;
use syn::{Expr, Stmt, visit::Visit};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A single byte-range replacement in the original source text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
	/// Start byte offset in the original source (inclusive).
	pub Start:usize,

	/// End byte offset in the original source (exclusive).
	pub End:usize,

	/// Replacement text. Empty string deletes the range.
	pub Replacement:String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Apply a sorted list of non-overlapping patches to Source.
///
/// Returns None when:
///   - Any patch has Start > End.
///   - Any two patches overlap.
///   - Any offset is out of bounds for Source.
pub fn ApplyPatches(Source:&str, Patches:&[Patch]) -> Option<String> {
	if Patches.is_empty() {
		return Some(Source.to_string());
	}

	let Bytes = Source.as_bytes();

	let mut Result = String::with_capacity(Source.len());

	let mut Cursor = 0usize;

	for P in Patches {
		if P.Start > P.End || P.End > Bytes.len() {
			return None;
		}

		if P.Start < Cursor {
			// Overlap with previous patch.
			return None;
		}

		// Copy unchanged bytes up to this patch.
		Result.push_str(&Source[Cursor..P.Start]);

		// Apply replacement (may be empty = delete).
		Result.push_str(&P.Replacement);

		Cursor = P.End;
	}

	// Copy tail after last patch.
	Result.push_str(&Source[Cursor..]);

	Some(Result)
}

/// Extract a byte range from a proc_macro2 Span relative to Source.
///
/// Returns None when span information is unavailable (both offsets are 0
/// for a token that is clearly not at the very start of the file, which
/// indicates that span-locations are not compiled in).
pub fn SpanBytes(Span:Span, Source:&str) -> Option<(usize, usize)> {
	let Start = Span.start();

	let End = Span.end();

	// proc_macro2 LineColumn is 1-based. Convert to byte offsets.
	let StartByte = LineColToByte(Source, Start.line, Start.column)?;

	let EndByte = LineColToByte(Source, End.line, End.column)?;

	if StartByte == 0 && EndByte == 0 {
		// Likely a call_site span with no location info.
		return None;
	}

	Some((StartByte, EndByte))
}

/// Convert a 1-based (line, col) pair to a byte offset in Source.
/// col is 0-based character offset within the line (proc_macro2 convention).
pub fn LineColToByte(Source:&str, Line:usize, Col:usize) -> Option<usize> {
	if Line == 0 {
		return None;
	}

	let mut CurrentLine = 1usize;

	let mut LineStart = 0usize;

	for (ByteIdx, Ch) in Source.char_indices() {
		if CurrentLine == Line {
			LineStart = ByteIdx;

			break;
		}

		if Ch == '\n' {
			CurrentLine += 1;
		}
	}

	if CurrentLine != Line {
		return None;
	}

	// Walk Col characters forward from LineStart.
	let mut ByteOffset = LineStart;

	for _ in 0..Col {
		if ByteOffset >= Source.len() {
			return None;
		}

		let Ch = Source[ByteOffset..].chars().next()?;

		ByteOffset += Ch.len_utf8();
	}

	Some(ByteOffset)
}

/// Given a Stmt::Local, return the byte range of the entire statement line
/// in Source, including the trailing newline if present.
pub fn StmtLineRange(Stmt:&Stmt, Source:&str) -> Option<(usize, usize)> {
	let LocalSpan = StmtSpan(Stmt)?;

	let (Start, End) = SpanBytes(LocalSpan, Source)?;

	// Extend End forward to consume the trailing newline (and any\r).
	let Bytes = Source.as_bytes();

	let mut LineEnd = End;

	while LineEnd < Bytes.len() && Bytes[LineEnd] != b'\n' {
		LineEnd += 1;
	}

	if LineEnd < Bytes.len() && Bytes[LineEnd] == b'\n' {
		LineEnd += 1;
	}

	// Also consume leading whitespace before Start on the same line
	// so the blank indentation is not left behind.
	let mut LineStart = Start;

	while LineStart > 0 && Bytes[LineStart - 1] != b'\n' {
		LineStart -= 1;
	}

	Some((LineStart, LineEnd))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn StmtSpan(Stmt:&Stmt) -> Option<Span> {
	let mut Tokens = proc_macro2::TokenStream::new();

	Stmt.to_tokens(&mut Tokens);

	let Trees:Vec<_> = Tokens.into_iter().collect();

	if Trees.is_empty() {
		return None;
	}

	let First = Trees.first()?.span();

	let Last = Trees.last()?.span();

	First.join(Last)
}

/// Collect all Spans of a plain-identifier Expr::Path matching Target
/// within an expression tree.
pub struct SpanCollector<'a> {
	pub Target:&'a str,

	pub Spans:Vec<Span>,
}

impl<'a, 'ast> Visit<'ast> for SpanCollector<'a> {
	fn visit_expr_path(&mut self, Node:&'ast syn::ExprPath) {
		if Node.qself.is_none() {
			if let Some(I) = Node.path.get_ident() {
				if I == self.Target {
					self.Spans.push(I.span());
				}
			}
		}
	}
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod Tests {

	use super::*;

	#[test]
	fn ApplyNoPatches() {
		let Src = "hello world";

		assert_eq!(ApplyPatches(Src, &[]).unwrap(), Src);
	}

	#[test]
	fn ApplySingleDelete() {
		let Src = "abc def ghi";

		let P = Patch { Start:4, End:8, Replacement:String::new() };

		assert_eq!(ApplyPatches(Src, &[P]).unwrap(), "abc ghi");
	}

	#[test]
	fn ApplySingleReplace() {
		let Src = "let X = 5;\nuse_x(X);";

		// Replace the second `X` (offset 16) with `5`.
		let P = Patch { Start:16, End:17, Replacement:"5".to_string() };

		assert_eq!(ApplyPatches(Src, &[P]).unwrap(), "let X = 5;\nuse_x(5);");
	}

	#[test]
	fn ApplyTwoPatches() {
		let Src = "AABBCC";

		let Patches = vec![
			Patch { Start:0, End:2, Replacement:"XX".to_string() },
			Patch { Start:4, End:6, Replacement:"YY".to_string() },
		];

		assert_eq!(ApplyPatches(Src, &Patches).unwrap(), "XXBBYY");
	}

	#[test]
	fn OverlappingPatchesReturnNone() {
		let Src = "ABCDE";

		let Patches = vec![
			Patch { Start:1, End:4, Replacement:"X".to_string() },
			Patch { Start:3, End:5, Replacement:"Y".to_string() },
		];

		assert!(ApplyPatches(Src, &Patches).is_none());
	}

	#[test]
	fn OutOfBoundsPatchReturnsNone() {
		let Src = "ABC";

		let P = Patch { Start:2, End:10, Replacement:String::new() };

		assert!(ApplyPatches(Src, &[P]).is_none());
	}

	#[test]
	fn LineColToByteFirstLine() {
		let Src = "hello\nworld";

		assert_eq!(LineColToByte(Src, 1, 0), Some(0));

		assert_eq!(LineColToByte(Src, 1, 5), Some(5));
	}

	#[test]
	fn LineColToByteSecondLine() {
		let Src = "hello\nworld";

		assert_eq!(LineColToByte(Src, 2, 0), Some(6));

		assert_eq!(LineColToByte(Src, 2, 5), Some(11));
	}

	#[test]
	fn LineColToByteOutOfRange() {
		let Src = "hi";

		assert_eq!(LineColToByte(Src, 5, 0), None);
	}
}
