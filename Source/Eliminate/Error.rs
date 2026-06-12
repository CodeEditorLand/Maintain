//! Error types for the Eliminate module.

/// All errors that can occur during elimination.
#[derive(thiserror::Error, Debug)]
pub enum Error {
	#[error("IO error: {0}")]
	Io(#[from] std::io::Error),

	#[error("Parse error in {Path}: {Source}")]
	Parse { Path:String, Source:syn::Error },

	#[error("Glob pattern error: {0}")]
	GlobPattern(#[from] globset::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
