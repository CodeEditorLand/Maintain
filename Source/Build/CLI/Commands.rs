//=============================================================================//
// File Path: Element/Maintain/Source/Build/CLI/Commands.rs
//=============================================================================//

use clap::Subcommand;

use super::OutputFormat::OutputFormat;

/// Available subcommands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
	/// Execute a build with the specified profile
	Build {
		/// Build profile to use
		#[clap(long, short = 'p', value_parser = super::parse_profile_name)]
		profile:String,

		/// Enable dry-run mode
		#[clap(long)]
		dry_run:bool,
	},

	/// List all available build profiles
	ListProfiles {
		/// Show detailed information for each profile
		#[clap(long, short = 'v')]
		verbose:bool,
	},

	/// Show details for a specific profile
	ShowProfile {
		/// Profile name to show
		profile:String,
	},

	/// Validate a build profile
	ValidateProfile {
		/// Profile name to validate
		profile:String,
	},

	/// Show current environment variable resolution
	Resolve {
		/// Profile name to resolve
		#[clap(long, short = 'p')]
		profile:String,

		/// Output format
		#[clap(long, short = 'f', default_value = "table")]
		format:OutputFormat,
	},
}
