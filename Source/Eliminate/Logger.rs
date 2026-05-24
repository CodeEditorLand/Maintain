//=============================================================================//
// File Path: Element/Maintain/Source/Eliminate/Logger.rs
//=============================================================================//
// Module: Logger - Logging initialisation for the Eliminate module
//=============================================================================//

use colored::Colorize;

/// Initialise `env_logger` with a coloured, human-readable format.
/// Reads `RUST_LOG`; defaults to `info` when the variable is absent.
pub fn Logger() {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
		.format(|Buf, Record| {
			use std::io::Write;

			let Level = match Record.level() {
				log::Level::Error => "ERROR".red().bold().to_string(),
				log::Level::Warn => "WARN ".yellow().bold().to_string(),
				log::Level::Info => "INFO ".green().to_string(),
				log::Level::Debug => "DEBUG".cyan().to_string(),
				log::Level::Trace => "TRACE".dimmed().to_string(),
			};

			writeln!(Buf, "[{}] {}", Level, Record.args())
		})
		.init();
}
