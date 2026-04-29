# Changelog - Maintain

Maintain is our build / CI / GritQL surface - the Rust-driven scripts
that compile Mountain, the sidecars, and the bundled VS Code tree
across the profile matrix. This file records what we built in our
voice, version by version. Format adapted from
[Keep a Changelog](https://keepachangelog.com/).

## [v2.1] - Full Workbench Lift (April 2026)

We taught Maintain how to build the full bundled-electron profile
matrix and tightened the safety it provides around in-flight config
files.

### Added

- **Multi-profile build support** for Mountain, Electron, and
  compiler variants (`214ce63`, 2026-04-20). One entry point now
  handles every profile we ship the binary against.
- **Comprehensive README expansion** (`f474c38`, 2026-04-05) with a
  benefit-focused rewrite (`3a24011`, `1cebdaa`, 2026-04-04).
- **Crate-level rustdoc** rewritten benefit-first in `Library.rs`
  (`da7cd07`, 2026-04-04).
- **CHANGELOG history rebuild** (`b582776`, 2026-04-16; `648a928`
  2026-04-17) with quarter-based versioning.

### Changed

- **Wildcard re-exports removed**, explicit module paths enforced
  (`ca7ac56`, 2026-04-18). Brought Maintain in line with our "no
  `pub use` flattening" rule - call sites now spell the full
  reverse-hierarchical path.
- **Consistent Rust formatting** applied to Rhai config tests
  (`9b19fb5`, 2026-04-11).
- **Spaces after colons removed** in struct definitions and
  parameters (`3d0702d`, 2026-04-07) to match `rustfmt.toml`'s
  `space_after_colon = false`.
- **`.gitmodules` formatting fixed**, ignore directive added
  (`053e6d1`, 2026-04-06).

## [v2.0] - Editor Launch (Q1 2026)

We hardened the build-system entry point and the configuration
guard machinery during the editor-launch cycle.

### Added

- **Guard disarm mechanism** in `Maintain/Build` to preserve modified
  configs (`a12cf4f`, 2026-03-03). Before this, an interrupted build
  could leave a half-rewritten config on disk; the disarm pass makes
  the rewrite transactional.

### Changed

- **Binary entry point restructured** with a dedicated `main.rs`
  (`394b1e0`, 2026-02-27). Cleared the way for the multi-profile
  support that landed in v2.1.
- **Refactor TODOs reclassified as DEPENDENCY** (`eac0bcf`,
  2026-03-04) so the remaining backlog tracks against external work
  instead of stalling on internal tags.
- **Workbench fix** (`dd14174`, 2026-02-13) - the consumer-side build
  glue learned to drive the new bundled-electron boot path.

## [v1.3] - Dependency Maintenance (Q4 2025)

Continuous Dependabot rolls and submodule pin updates; no
substantive source changes through October-December 2025.

## [v1.2] - Full-Stack Integration (Q3 2025)

### Added

- **Compiled executables** for debug and release configurations
  refreshed across the build-tools surface (`9d3e441`, 2025-07-10;
  `b2b096d`, 2025-06-26; `3eb3bcc`, 2025-06-20).
- **Node.js sidecar version bundling support** (`a1fe1f2`,
  2025-06-23) - Maintain now stamps the bundled Node version into
  the build manifest so Cocoon and the workbench agree on which
  runtime they're loading.

## [v1.1] - Architecture Buildout (Build System Rewrite, May 2025)

The pivot cycle. We replaced the early scripted binary module with
a real structured build system - the foundation everything in v1.2
and v2.x sits on.

### Added

- **Structured build system** replacing the binary module
  (`3d73508`, 2025-05-29). End of the "ad-hoc shell scripts" era.
- **Build-flag restructure + compile environment support**
  (`1776397`, 2025-05-31).
- **Binary build configurability** for Mountain (`15a5576`,
  2025-05-29).
- **Initial `.gitignore`** (`a0b8253`, 2025-05-29). The build now
  has a clear story for what's tracked vs generated.
- **`.gitignore` updated** to exclude build artefacts while
  preserving the executables themselves (`9e19fb4`, 2025-05-29).

### Changed

- **Re-licensed to CC0 1.0 Universal** (`bb8ffc4`, 2025-05-22), then
  **migrated to Land Public License v1.0** (`935706d`, 2025-05-10) -
  the licence we converged on across the fleet.
- **Documentation URL protocols normalised to lowercase**
  (`18ea720`, 2025-05-20).
- **Queue submodule** kept current with rolling updates (`32b7c99`
  2025-05-09, `330f673` 2025-05-10, `6786c3a` 2025-05-11, `62cddee`
  2025-05-20, `f5254f4` 2025-05-22, `8e1f588` 2025-05-01).

## [v1.0] - Integration Phase (Q1-Q2 2025)

Maintenance-heavy cycle. Continuous Dependabot rolls and Queue
submodule updates with one substantive commit (`02fc9db`,
2025-03-21) tracking the latest Queue commit. The structured build
system arrived in v1.1 above.

## [v0.x] - Project Inception (2024)

Maintain existed as the catch-all build/CI repository through 2024
- nearly all 200+ commits in this window were `squash!` consolidation
or Dependabot rolls. The first substantive code didn't land until
the build-system rewrite in **May 2025** (v1.1 above).

### Added (2024-01-30)

- **Initial commit** (`defc832`) - empty repository scaffold with
  the standard CI metadata.
