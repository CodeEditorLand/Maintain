//! Top-level entry point for the Eliminate module.

use clap::Parser;

use super::{CLI, Logger};

/// Initialise logging, parse CLI arguments, and run the elimination pipeline.
pub fn Fn() {
	Logger::Logger();

	match CLI::Cli::try_parse() {
		Ok(Cli) => {
			if let Err(E) = Cli.execute() {
				eprintln!("eliminate: error: {}", E);

				std::process::exit(1);
			}
		},

		Err(E) => {
			E.print().expect("Failed to print clap error");

			std::process::exit(E.exit_code());
		},
	}
}
