//=============================================================================//
// File Path: Element/Maintain/Source/Run/CLI/OutputFormat.rs
//=============================================================================//

use clap::ValueEnum;

/// Output format options
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
	Table,

	Json,

	Env,
}

impl std::fmt::Display for OutputFormat {
	fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			OutputFormat::Table => write!(f, "table"),

			OutputFormat::Json => write!(f, "json"),

			OutputFormat::Env => write!(f, "env"),
		}
	}
}
