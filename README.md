<table>
	<tr>
		<td align="left" valign="middle">
			<h3 align="left">
				Maintain&#x2001;💪🏻
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				+
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				<a href="https://editor.land" target="_blank">
					<picture>
						<source media="(prefers-color-scheme: dark)" srcset="https://editor.land/Dark/Image/GitHub/Land.svg" />
						<source media="(prefers-color-scheme: light)" srcset="https://editor.land/Image/GitHub/Land.svg" />
						<img width="28" alt="Land Logo" src="https://editor.land/Image/GitHub/Land.svg" />
					</picture>
				</a>
			</h3>
		</td>
		<td align="left" valign="middle">
			<h3 align="left">
				<a href="https://editor.land" target="_blank">
					Land&#x2001;🏞️
				</a>
			</h3>
		</td>
	</tr>
</table>

---

# **Maintain**&#x2001;💪🏻

The Build System & CI/CD Toolkit for Land &#x2001;🏞️

> **Build pipelines that change behavior based on environment variables,
> implicit tool versions, or undeclared dependencies make debugging production
> issues impossible. The same commit produces different output on different
> machines.**

_"Deterministic builds. Same commit, same output, guaranteed."_

[![License: CC0-1.0](https://img.shields.io/badge/License-CC0_1.0-lightgrey.svg)](https://github.com/CodeEditorLand/Maintain/tree/Current/LICENSE)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/)&#x2001;[![Crates.io](https://img.shields.io/crates/v/Maintain.svg)](https://crates.io/crates/Maintain)
[<img src="https://editor.land/Image/Rust.svg" width="14" alt="Rust" />](https://www.rust-lang.org/)&#x2001;[![Rust Version](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Rhai Version](https://img.shields.io/badge/Rhai-latest-blue.svg)](https://rhai.rs/)

&#x2001;📖
**[Rust API Documentation](https://Rust.Documentation.editor.land/Maintain/)**

Welcome to **Maintain**, the Rust-based build system and CI/CD toolkit for the
**Land Code Editor** ecosystem. Maintain provides comprehensive build
orchestration, Rhai scripting capabilities, and configuration management for
TOML and JSON5 files.

**Maintain** is engineered to:

1. **Orchestrate Builds:** Provide a central build system for the entire Land
   ecosystem with configurable build groups.
2. **Enable Scripting:** Embed the Rhai scripting language for flexible build
   logic and custom automation.
3. **Manage Configuration:** Offer type-safe TOML and JSON5 editing capabilities
   for Cargo.toml and other configuration files.
4. **Provide CLI Interface:** Deliver a command-line interface for build
   operations with environment variable resolution.

---

## Key Features&#x2001;🔐

- **Rhai Scripting Engine:** Embedded Rhai interpreter for flexible build
  configuration and custom automation logic.
- **Configuration Editing:** Type-safe TOML and JSON5 editing for Cargo.toml and
  other configuration files with validation.
- **Environment Resolution:** Dynamic environment variable handling with
  scriptable resolvers for build-time configuration.
- **CLI Interface:** Comprehensive command-line interface with subcommands for
  build, debug, release, and profile operations.
- **Build Orchestration:** Central coordination of multi-stage builds across the
  Land ecosystem.

---

## Core Architecture Principles&#x2001;🏗️

| Principle                 | Description                                                                               | Key Components Involved                  |
| :------------------------ | :---------------------------------------------------------------------------------------- | :--------------------------------------- |
| **Scriptability**         | Enable flexible build logic through embedded Rhai scripting with full environment access. | `Rhai/ConfigLoader`, `Rhai/ScriptRunner` |
| **Type Safety**           | Provide compile-time checked configuration access with validation for TOML/JSON5.         | `toml_edit`, `json5` crates              |
| **Modularity**            | Separate concerns between CLI, scripting, and configuration editing components.           | `CLI.rs`, `Rhai/`, `Build/*`             |
| **Environment Awareness** | Dynamic resolution of environment variables for flexible build configurations.            | `EnvironmentResolver.rs`                 |

---

## `Maintain` in the Land Ecosystem 💪🏻 + 🏞️

| Component                 | Role & Key Responsibilities                                  |
| :------------------------ | :----------------------------------------------------------- |
| **Build Orchestrator**    | Central coordination of builds across all Land elements.     |
| **Scripting Host**        | Rhai engine for custom build logic and automation.           |
| **Configuration Manager** | TOML/JSON5 editing for Cargo.toml and project configuration. |
| **CLI Provider**          | Command-line interface for developers and CI/CD pipelines.   |

---

## System Architecture Diagram&#x2001;🏗️

This diagram illustrates `Maintain`'s build orchestration architecture.

```mermaid
graph LR
    classDef maintain fill:#fce8ff,stroke:#9b59b6,stroke-width:2px,color:#2c0050;
    classDef script   fill:#cce8ff,stroke:#2980b9,stroke-width:1px,color:#003050;
    classDef config   fill:#fff3c0,stroke:#f39c12,stroke-width:1px,color:#5a3e00;
    classDef artifact fill:#d4f5d4,stroke:#27ae60,stroke-width:1px,color:#0a3a0a;

    subgraph MAINTAIN["Maintain 💪🏻 - Rust Build System + CI/CD Toolkit"]
        direction TB
        subgraph BUILD["Source/Build/ - Core Logic"]
            CLI["Build/CLI.rs\n(clap - subcommands:\nbuild · debug · release · profile)"]:::maintain
            Fn["Build/Fn.rs\n(build functions)"]:::maintain
            Consts["Build/Constant.rs + Definition.rs"]:::maintain
            JsonEdit["Build/JsonEdit.rs\n(JSON5 editing)"]:::maintain
            TomlEdit["Build/TomlEdit.rs\n(Cargo.toml editing)"]:::maintain
            PlistEdit["Build/PlistEdit.rs\n(Info.plist editing)"]:::maintain
            Pascalize["Build/Pascalize.rs\nWordsFromPascal.rs"]:::maintain
            GetTriple["Build/GetTauriTargetTriple.rs"]:::maintain
        end
        subgraph RHAI["Build/Rhai/ - Embedded Scripting"]
            RhaiEngine["Rhai script interpreter"]:::maintain
            ConfigLoader["Rhai/ConfigLoader.rs\nloads config files"]:::maintain
            EnvResolver["Rhai/EnvironmentResolver.rs\ndynamic env var resolution"]:::maintain
            ScriptRunner["Rhai/ScriptRunner.rs\nexecutes .rhai scripts"]:::maintain
            RhaiEngine --> ConfigLoader
            RhaiEngine --> EnvResolver
            RhaiEngine --> ScriptRunner
        end
        subgraph RUN["Source/Run/ - Run-mode Logic"]
            RunCLI["Run/CLI.rs + Process.rs\n(dev server, hot reload)"]:::maintain
            Profile["Run/Profile.rs\n(perf profiling)"]:::maintain
        end

        CLI --> Fn
        CLI --> RHAI
        CLI --> RUN
        Fn --> TomlEdit
        Fn --> JsonEdit
        Fn --> PlistEdit
    end

    subgraph SCRIPTS["Shell Scripts - Entrypoints"]
        DebugBuild["Debug/Build.sh\n(cargo build --profile debug-electron\n+ SignBundle.sh)"]:::script
        DebugRun["Debug/Run.sh\n(launch binary)"]:::script
        ReleaseBuild["Release/Build.sh\n(cargo build --release\n+ SignBundle.sh)"]:::script
        SignBundle["Script/SignBundle.sh\n(ad-hoc codesign + entitlements)"]:::script
        BrotliPrebake["Build/Brotli/PreBake.ts\n(post-bundle .br siblings)"]:::script
    end

    subgraph TARGETS["Build Artifacts"]
        MountainBin["Mountain binary\n(.app bundle)"]:::artifact
        CargoTOML["Cargo.toml\n(workspace config)"]:::config
        PlistFile["Entitlements.plist\n+ Info.plist"]:::config
    end

    CLI -.invokes.-> DebugBuild
    CLI -.invokes.-> ReleaseBuild
    DebugBuild --> SignBundle
    ReleaseBuild --> SignBundle
    SignBundle --> MountainBin
    TomlEdit --> CargoTOML
    PlistEdit --> PlistFile
```

---

## Project Structure&#x2001;🗺️

```
Element/Maintain/
├── Source/
│   ├── Library.rs       # Main entry point
│   └── Build/
│       ├── CLI.rs        # Command-line interface with clap
│       ├── Constant.rs   # Build constants
│       ├── Definition.rs # Build definitions
│       ├── Fn.rs         # Build functions
│       ├── Rhai/         # Rhai scripting engine
│       │   ├── ConfigLoader.rs
│       │   ├── EnvironmentResolver.rs
│       │   └── ScriptRunner.rs
│       └── ...
├── Debug/                # Debug scripts
│   ├── All.sh
│   ├── Build.sh
│   ├── Run.sh
│   └── Wind.sh
├── Debug.sh              # Debug mode execution
├── Dev-Mountain.sh       # Mountain development mode
├── Profile.sh            # Performance profiling
└── Release.sh            # Release build
```

---

## Deep Dive & Component Breakdown&#x2001;🔬

To understand how `Maintain`'s internal components interact to provide the build
orchestration functionality, see the following source files:

- **[`Source/Library.rs`](https://github.com/CodeEditorLand/Maintain/tree/Current/Source/Library.rs)** -
  Main entry point and module declarations
- **[`Source/Build/CLI.rs`](https://github.com/CodeEditorLand/Maintain/tree/Current/Source/Build/CLI.rs)** -
  Command-line interface with clap
- **[`Source/Build/Rhai/`](https://github.com/CodeEditorLand/Maintain/tree/Current/Source/Build/Rhai/)** -
  Rhai scripting engine integration
    - [`ConfigLoader.rs`](https://github.com/CodeEditorLand/Maintain/tree/Current/Source/Build/Rhai/ConfigLoader.rs) -
      Configuration file loading
    - [`EnvironmentResolver.rs`](https://github.com/CodeEditorLand/Maintain/tree/Current/Source/Build/Rhai/EnvironmentResolver.rs) -
      Environment variable resolution
    - [`ScriptRunner.rs`](https://github.com/CodeEditorLand/Maintain/tree/Current/Source/Build/Rhai/ScriptRunner.rs) -
      Script execution engine

The source files explain the Rhai scripting integration, TOML/JSON5 editing
capabilities, and the build orchestration patterns.

---

## Getting Started&#x2001;🚀

### Installation&#x2001;📥

To add `Maintain` as a dependency:

```toml
[dependencies]
Maintain = { git = "https://github.com/CodeEditorLand/Maintain.git", branch = "Current" }
```

Or install the CLI globally:

```sh
cargo install Maintain
```

**Key Dependencies:**

- `rhai`: Embedded scripting engine
- `clap`: CLI argument parsing
- `toml_edit`: TOML parsing and editing
- `json5`: JSON5 configuration support
- `chrono`: Date/time handling
- `colored`: Colored terminal output
- `log` / `env_logger`: Logging framework

### Usage Pattern&#x2001;🚀

`Maintain` is typically invoked through its included shell scripts:

```sh
# Debug build
./Maintain/Debug.sh

# Development mode for Mountain
./Maintain/Dev-Mountain.sh

# Release build
./Maintain/Release.sh
```

As a binary:

```sh
Maintain [OPTIONS] [COMMAND]
```

As a library:

```rust
use Maintain::Build;

let Build = Build::new();
Build.execute()?;
```

---

## See Also

- [Maintain Documentation](https://Editor.Land/Doc/maintain)
- [Architecture Overview](https://Editor.Land/Doc/architecture)
- [Why Rust](https://Editor.Land/Doc/why-rust)
- [Mountain](https://github.com/CodeEditorLand/Mountain)
- [Rest](https://github.com/CodeEditorLand/Rest)
- [Output](https://github.com/CodeEditorLand/Output)

---

## License&#x2001;⚖️

This project is released into the public domain under the **Creative Commons CC0
Universal** license. You are free to use, modify, distribute, and build upon
this work for any purpose, without any restrictions. For the full legal text,
see the [`LICENSE`](https://github.com/CodeEditorLand/Maintain/tree/Current/LICENSE)
file.

---

## Changelog&#x2001;📜

Stay updated with our progress! See
[`CHANGELOG.md`](https://github.com/CodeEditorLand/Maintain/tree/Current/CHANGELOG.md) for a
history of changes specific to **Maintain**.

---

## Funding & Acknowledgements&#x2001;🙏🏻

**Maintain** is a core element of the **Land** ecosystem. This project is funded
through [NGI0 Commons Fund](https://NLnet.NL/commonsfund), a fund established by
[NLnet](https://NLnet.NL) with financial support from the European Commission's
[Next Generation Internet](https://ngi.eu) program. Learn more at the
[NLnet project page](https://NLnet.NL/project/Land).

The project is operated by PlayForm, based in Sofia, Bulgaria.

PlayForm acts as the open-source steward for Code Editor Land under the NGI0
Commons Fund grant.

<table>
	<thead>
		<tr>
			<th align="left">
				<strong>
					Land
				</strong>
			</th>
			<th align="left">
				<strong>
					PlayForm
				</strong>
			</th>
			<th align="left">
				<strong>
					NLnet
				</strong>
			</th>
			<th align="left">
				<strong>
					NGI0 Commons Fund
				</strong>
			</th>
		</tr>
	</thead>
	<tbody>
		<tr>
			<td align="left" valign="middle">
				<a href="https://editor.land">
					<img width="60" src="https://raw.githubusercontent.com/CodeEditorLand/Asset/refs/heads/Current/Logo/Land.svg" alt="Land" />
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://editor.land">
					<img width="76" src="https://raw.githubusercontent.com/PlayForm/Asset/refs/heads/Current/Logo/PlayForm.svg" alt="PlayForm" />
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL">
					<img width="240" src="https://NLnet.NL/logo/banner.svg" alt="NLnet" />
				</a>
			</td>
			<td align="left" valign="middle">
				<a href="https://NLnet.NL/commonsfund">
					<img width="240" src="https://NLnet.NL/image/logos/NGI0CommonsFund_tag_black_mono.svg" alt="NGI0 Commons Fund" />
				</a>
			</td>
		</tr>
	</tbody>
</table>

---

**Project Maintainers**: Source Open
([Source/Open@editor.land](mailto:Source/Open@editor.land)) |
[GitHub Repository](https://github.com/CodeEditorLand/Maintain) |
[Report an Issue](https://github.com/CodeEditorLand/Maintain/issues) |
[Security Policy](https://github.com/CodeEditorLand/Maintain/security/policy)
