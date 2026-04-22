# Changelog

All notable changes to Maintain (Build System and CI/CD) are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/).

## [v2.1] - Q2 2026: Full Workbench Lift

### Changed

- Struct colon formatting standardized across 30 files (3,648 insertions)
- `Source/Architecture.rs` added (+86 lines design audit template)
- README condensed to benefit-focused format

## [v2.0] - Q1 2026: Editor Launch Sprint

### Added

- Dedicated `Source/main.rs` entry point (12 lines, February 27)
- Guard disarm mechanism for modified configs (+30 lines, March 3)

### Changed

- Monolithic `Source/Build.rs` (835 lines) split into modular files:
  - `Build/CLI.rs` (1,434 lines) - command-line parsing, subcommand dispatch
  - `Build/Definition.rs` (615 lines) - build group definitions, task
    sequencing
  - `Build/Process.rs` (475 lines) - process spawning, output capture
  - `Build/Error.rs` (199 lines) - error handling
- `Source/Run/` module created (91 lines): CLI, Profile, Environment, Logger,
  Process, Error
- Functions renamed: ProcessFunction.rs → Process.rs, LoggerFunction.rs →
  Logger.rs
- Total: 5,008 insertions, 4,544 deletions across 55 files

## [v1.2] - Q3 2025: Full Stack Integration

### Added

- Rhai script engine integration (February 2026 backported):
  - `Source/Build/Rhai/mod.rs` (455 lines) - API bindings
  - `Source/Build/Rhai/ConfigLoader.rs` (416 lines) - environment script loading
  - `Source/Build/Rhai/EnvironmentResolver.rs` (334 lines) - dynamic env vars
  - `Source/Build/Rhai/ScriptRunner.rs` (235 lines) - interpreter context
  - `examples/test_rhai_config.rs` (389 lines)
  - `tests/test_rhai_config.rs` (567 lines)

### Changed

- Build artifact management standardized

## [v1.1] - Q2 2025: Architecture Buildout

### Added

- Structured build system established
- Node.js sidecar bundling infrastructure
- Submodule management (.gitmodules)

## [v0.2] - Q4 2024: Architecture Solidification

### Changed

- Large-scale module cleanup: -3,074 lines, removed ~52 unused Fn/* modules
- GritQL transformation queries expanded

## [v0.1] - Q3 2024: Rapid Development

### Changed

- Crate dependency versioning stabilization
- CI action upgrades

## [v0.0] - Q2 2024: Project Inception

### Added

- GitHub Actions workflows (Rust.yml, GitHub.yml)
- Dependabot configuration
- GritQL code transformation queries
- Rust formatting rules (rustfmt.toml)

### Dependencies

- clap, json5, serde, serde_json, thiserror, toml, toml_edit, log, chrono,
  env_logger, colored, rhai
