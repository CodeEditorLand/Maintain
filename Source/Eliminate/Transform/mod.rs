//! # Transform - AST Transformation Pipeline
//!
//! Two entry points for inlining single-use `let` bindings:
//!
//! - **`Run::Fn(source, options)`**: Parse with `syn`, run the `VisitMut`
//!   eliminator, then attempt span-based text patching to preserve comments and
//!   whitespace. Falls back to `prettyplease::unparse` when span data is
//!   unavailable.
//!
//! - **`RunPreserve::Fn(source, options)`**: Preserve-layout behaviour (default
//!   when `Options.Reformat == false`). Identifies inlinable bindings via the
//!   same Collect/Safe/Count pipeline, then applies targeted text substitutions
//!   without touching anything outside the affected lines.
//!
//! Returns `Ok(None)` when no bindings were eliminated.
//!
//! ## Submodules
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`crate::Eliminate::Transform::Collect`] | Candidate let-binding discovery |
//! | [`crate::Eliminate::Transform::Count`] | Reference counting for candidates |
//! | [`crate::Eliminate::Transform::Inline`] | VisitMut transformer that eliminates single-use bindings |
//! | [`crate::Eliminate::Transform::Patch`] | Span-based source text patching |
//! | [`crate::Eliminate::Transform::Safe`] | Safety predicates for inlinable initialisers |

// Subdirectory modules
pub mod Collect;

pub mod Count;

pub mod Inline;

pub mod Patch;

pub mod Safe;

// Atom modules — pipeline stages
pub mod Run;

pub mod RunPreserve;

pub mod TryPatchSource;

pub mod CollectBlockPatches;

pub mod CollectInnerBlockPatches;

pub mod StmtTokensMatch;

// Atom modules — preserve-layout pipeline
pub mod PreservePass;

pub mod TryItemPreserve;

pub mod TryBlockPreserve;

// Atom modules — helpers
pub mod StmtToText;

pub mod StmtNestedBlock;
