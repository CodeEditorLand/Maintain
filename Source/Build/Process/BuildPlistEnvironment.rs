//! Collects environment variables from `.env.Land` for injection into
//! Info.plist LSEnvironment so the bundled .app works standalone.
//!
//! Sources from the `.env.Land` file in the repo root (where Maintain
//! runs from). This ensures every runtime-relevant variable -- Product*,
//! Tier*, Network*, Trace, Record, Inspect, Disable, etc. -- is
//! available when the .app is launched via LaunchServices.
//!
//! Build-time-only flags (CargoFeatures, CocoonEsbuildDefine, NODE_ENV)
//! are excluded because they have no meaning at runtime inside the .app.

use std::{collections::BTreeMap, env, fs, path::PathBuf};

use log::info;

use crate::Build::Constant::{LandDisableEnv, LandInspectEnv, LandRecordEnv, LandTraceEnv};

/// Collects environment variable entries suitable for LSEnvironment injection.
///
/// Reads from `.env.Land` and `.env.Land.Sample`, supplements with process
/// environment for dev-control knobs (Trace, Record, Inspect, Disable) that
/// may have been overridden by the user.
pub fn Fn() -> BTreeMap<String, String> {
	let mut EnvVars = BTreeMap::new();

	// Build-time / Maintain-control keys that should NOT leak into the
	// bundled .app's LSEnvironment.
	let SkipKeys = ["CargoFeatures", "CocoonEsbuildDefine", "NODE_ENV"];

	// Primary source: the .env.Land file at the repo root (Maintain's
	// working directory).
	for Source in [".env.Land", ".env.Land.Sample"] {
		let Path = PathBuf::from(Source);

		if Path.exists() {
			if let Ok(Content) = fs::read_to_string(&Path) {
				info!(target: "Build::Plist", "Loading LSEnvironment vars from {}", Source);

				for Line in Content.lines() {
					let Trimmed = Line.trim();

					if Trimmed.is_empty() || Trimmed.starts_with('#') {
						continue;
					}

					if let Some((Key, Value)) = Trimmed.split_once('=') {
						let CleanKey = Key.trim();

						let CleanValue = Value.trim().trim_matches('"').trim_matches('\'');

						// Skip build-time-only keys.
						if SkipKeys.contains(&CleanKey) {
							continue;
						}

						EnvVars.insert(CleanKey.to_string(), CleanValue.to_string());
					}
				}
			}

			break;
		}
	}

	// Supplement from the current process environment for dev-control
	// knobs that live outside .env.Land (Trace, Record, Inspect, Disable).
	// These may have been overridden by the user before invoking Maintain.
	// Only add if not already populated from .env.Land.
	for Key in [LandTraceEnv, LandRecordEnv, LandInspectEnv, LandDisableEnv] {
		if !EnvVars.contains_key(Key) {
			if let Ok(Value) = env::var(Key) {
				EnvVars.insert(Key.to_string(), Value);
			}
		}
	}

	EnvVars
}
