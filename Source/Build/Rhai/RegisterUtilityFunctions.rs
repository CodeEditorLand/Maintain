//! Registers utility functions that can be called from Rhai scripts.
//!
//! Functions registered:
//! - `get_os_type`, `get_arch`, `get_family` — system information
//! - `get_env` — read-only environment access
//! - `path_exists` — filesystem checking
//! - `timestamp` — UNIX epoch seconds
//! - `print` — logging output

use rhai::Engine;

/// Registers helper functions into a Rhai `Engine` for use by build scripts.
///
/// # Registered Functions
///
/// | Function | Returns | Description |
/// |----------|---------|-------------|
/// | `get_os_type()` | String | OS name (e.g. "macos") |
/// | `get_arch()` | String | Architecture (e.g. "aarch64") |
/// | `get_family()` | String | OS family (e.g. "unix") |
/// | `get_env(name)` | String | Environment variable or "" |
/// | `path_exists(path)` | bool | Whether the path exists |
/// | `timestamp()` | i64 | Current UNIX timestamp |
/// | `print(s)` | — | Prints to stdout |
pub fn Fn(engine:&mut Engine) {
	// System information
	engine.register_fn("get_os_type", || std::env::consts::OS.to_string());

	engine.register_fn("get_arch", || std::env::consts::ARCH.to_string());

	engine.register_fn("get_family", || std::env::consts::FAMILY.to_string());

	// Environment access (read-only for safety)
	engine.register_fn("get_env", |name:&str| -> String { std::env::var(name).unwrap_or_default() });

	// File system utilities
	engine.register_fn("path_exists", |path:&str| -> bool { std::path::Path::new(path).exists() });

	// Time utilities
	engine.register_fn("timestamp", || -> i64 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs() as i64
	});

	// Logging functions
	engine.register_fn("print", |s:&str| {
		println!("[Rhai] {}", s);
	});
}
