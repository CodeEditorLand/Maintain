//=============================================================================//
// File Path: Element/Maintain/Source/Run/Constant.rs
//=============================================================================//
// Module: Constant
//
// Brief Description: Run module constants and configuration values.
//
// RESPONSIBILITIES:
// ================
//
// Primary:
// - Provide file path constants for run operations
// - Provide delimiter constants
// - Provide environment variable name constants
// - Serve as single source of truth for run-related constant values
//
// Secondary:
// - Ensure consistent naming across the run module
//
// ARCHITECTURAL ROLE:
// ===================
//
// Position:
// - Infrastructure/Configuration layer
// - Constant definitions
//
// Dependencies (What this module requires):
// - External crates: None
// - Internal modules: None
// - Traits implemented: None
//
// Dependents (What depends on this module):
// - Run orchestration functions
// - Argument parsing module
// - Profile resolution logic
//
//=============================================================================//
// IMPLEMENTATION
//=============================================================================//

// File path constants
pub const ConfigFile: &str = ".vscode/land-config.json";
pub const LogFile: &str = "Target/run.log";

// Default values
pub const DirectoryDefault: &str = ".";
pub const ProfileDefault: &str = "debug";

// Delimiter constants
pub const WorkbenchDelimiter: &str = "-";

// Environment variable constants
pub const DebugEnv: &str = "Debug";
pub const LevelEnv: &str = "Level";
pub const NodeEnv: &str = "NODE_ENV";
pub const NodeVersionEnv: &str = "NODE_VERSION";
pub const WorkbenchEnv: &str = "Workbench";
pub const BrowserEnv: &str = "Browser";
pub const WindEnv: &str = "Wind";
pub const MountainEnv: &str = "Mountain";
pub const ElectronEnv: &str = "Electron";
pub const BundleEnv: &str = "Bundle";
pub const CleanEnv: &str = "Clean";
pub const CompileEnv: &str = "Compile";
pub const DependencyEnv: &str = "Dependency";
pub const DirEnv: &str = "RUN_DIR";
pub const LogEnv: &str = "RUST_LOG";
pub const ProfileEnv: &str = "RUN_PROFILE";

// Run-specific constants
pub const HotReloadEnv: &str = "HOT_RELOAD";
pub const WatchEnv: &str = "WATCH";
pub const LiveReloadPortEnv: &str = "LIVE_RELOAD_PORT";

// Default ports
pub const DefaultLiveReloadPort: u16 = 3001;
pub const DefaultDevServerPort: u16 = 3000;
