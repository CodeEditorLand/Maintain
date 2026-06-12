//=============================================================================//
// File Path: Element/Maintain/Source/Build/CLI/GetAllProfiles.rs
//=============================================================================//

use crate::Build::Rhai::ConfigLoader::LandConfig;

/// List all available profiles (avoids Self)
pub fn get_all_profiles(config:&LandConfig) -> Vec<&str> {
	let mut profiles:Vec<&str> = config.profiles.keys().map(|s| s.as_str()).collect();

	profiles.sort();

	profiles
}
