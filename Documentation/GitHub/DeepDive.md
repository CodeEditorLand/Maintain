# Maintain: Deep Dive

This document provides the technical foundation for the Maintain build system
and CI/CD toolkit within the Land ecosystem. **Maintain** orchestrates builds
across all Land elements, embeds a Rhai scripting engine for flexible
automation, and provides type-safe TOML/JSON5 configuration editing.

---

## Architecture

Maintain is a Rust binary and library. The CLI layer accepts subcommands and
delegates to the Rhai scripting engine or direct build functions. Configuration
editing is performed through `toml_edit` and `json5` for structured, non-lossy
modifications to project configuration files.

```mermaid
graph TB
    subgraph "Maintain Build System"
        LibraryRS["Source/Library.rs\nEntry point"]
        CLI["Source/Build/CLI.rs\nCommand-line interface (clap)"]
        Constant["Source/Build/Constant.rs\nBuild constants"]
        Definition["Source/Build/Definition.rs\nBuild group definitions"]
        FnRS["Source/Build/Fn.rs\nBuild function dispatch"]
        Error["Source/Build/Error.rs\nError types"]
        Logger["Source/Build/Logger.rs\nColored terminal output"]
        Process["Source/Build/Process.rs\nChild process management"]
        TomlEdit["Source/Build/TomlEdit.rs\nCargo.toml editor"]
        JsonEdit["Source/Build/JsonEdit.rs\nJSON5 editor"]
        Pascalize["Source/Build/Pascalize.rs\nPascalCase naming utilities"]
        GetTauri["Source/Build/GetTauriTargetTriple.rs\nPlatform detection"]
        EnvResolve["Source/Build/EnvironmentResolver.rs\nEnvironment variable resolution"]
    end

    subgraph "Rhai Scripting"
        ConfigLoader["Source/Build/Rhai/ConfigLoader.rs\nConfiguration file loading"]
        ScriptRunner["Source/Build/Rhai/ScriptRunner.rs\nScript execution engine"]
        RhaiEnvResolve["Source/Build/Rhai/EnvironmentResolver.rs\nEnvironment in scripts"]
    end

    subgraph "Shell Scripts"
        DebugSh["Debug.sh\nDebug build entry"]
        DevMountain["Dev-Mountain.sh\nMountain development mode"]
        ReleaseSh["Release.sh\nRelease build entry"]
        ProfileSh["Profile.sh\nPerformance profiling"]
        DebugAll["Debug/All.sh"]
        DebugBuild["Debug/Build.sh"]
        DebugRun["Debug/Run.sh"]
        DebugWind["Debug/Wind.sh"]
    end

    LibraryRS --> CLI
    CLI --> FnRS
    CLI --> ConfigLoader
    CLI --> ScriptRunner
    FnRS --> Process
    FnRS --> TomlEdit
    FnRS --> JsonEdit
    ScriptRunner --> RhaiEnvResolve
    ConfigLoader --> ScriptRunner
    DebugSh --> CLI
    DevMountain --> CLI
    ReleaseSh --> CLI
```

---

## Key Modules

| Path                                       | Description                                                                  |
| :----------------------------------------- | :--------------------------------------------------------------------------- |
| `Source/Library.rs`                        | Binary entry point; wires together CLI, logging, and error handling          |
| `Source/Build/CLI.rs`                      | `clap`-based CLI with subcommands: debug, release, profile, dev              |
| `Source/Build/Definition.rs`               | Build group definitions: which elements to build, in what order              |
| `Source/Build/Fn.rs`                       | Core build function dispatch: clean, compile, link, post-process             |
| `Source/Build/Process.rs`                  | Child process spawning with stdout/stderr capture and exit code handling     |
| `Source/Build/TomlEdit.rs`                 | Non-lossy TOML editing for `Cargo.toml` version bumps and dependency changes |
| `Source/Build/JsonEdit.rs`                 | JSON5-aware configuration editing for `package.json` and `.json5` files      |
| `Source/Build/Pascalize.rs`                | PascalCase ↔ words conversion utilities for naming convention enforcement    |
| `Source/Build/GetTauriTargetTriple.rs`     | Detects current Rust target triple for SideCar binary selection              |
| `Source/Build/EnvironmentResolver.rs`      | Resolves environment variables with fallbacks for build-time configuration   |
| `Source/Build/Logger.rs`                   | Colored terminal output using the `colored` crate                            |
| `Source/Build/Error.rs`                    | Unified error type for build failures                                        |
| `Source/Build/Rhai/ConfigLoader.rs`        | Loads and parses Rhai build configuration scripts                            |
| `Source/Build/Rhai/ScriptRunner.rs`        | Executes Rhai scripts within the build context with Land API bindings        |
| `Source/Build/Rhai/EnvironmentResolver.rs` | Exposes environment variable resolution to Rhai scripts                      |

---

## Data Flow

```mermaid
sequenceDiagram
    participant Developer as Developer / CI
    participant ShellScript as Shell Script
    participant CLI as Maintain CLI
    participant Rhai as Rhai Engine
    participant Process as Process Manager
    participant Config as Config Editor

    Developer->>ShellScript: ./Maintain/Release.sh
    ShellScript->>CLI: Maintain release --target aarch64-apple-darwin
    CLI->>Rhai: Load build configuration script
    Rhai->>CLI: Build group definition
    CLI->>Config: Read Cargo.toml / package.json
    Config->>CLI: Current version and dependencies
    loop For each element in build group
        CLI->>Process: spawn("cargo build --release -p Mountain")
        Process->>CLI: Exit code + output
    end
    CLI->>Config: Write updated Cargo.toml (if version bump)
    CLI->>Developer: Build summary with timing
```

---

## Integration Points

| Connecting Element | Direction     | Mechanism                            | Description                                                         |
| :----------------- | :------------ | :----------------------------------- | :------------------------------------------------------------------ |
| **Mountain**       | Build target  | `cargo build` subprocess             | Maintain compiles Mountain as part of the debug/release build group |
| **Air**            | Build target  | `cargo build` subprocess             | Air daemon compiled alongside Mountain                              |
| **Echo**           | Build target  | `cargo build` subprocess             | Echo scheduler compiled as a dependency of Mountain                 |
| **Rest**           | Build target  | `cargo build` subprocess             | Rest compiler binary built for use by Output                        |
| **SideCar**        | Build target  | `cargo build` subprocess             | SideCar Download tool compiled and run as part of setup             |
| **Output**         | Build trigger | `pnpm run prepublishOnly` subprocess | Maintain triggers TypeScript builds via pnpm                        |
| **Wind / Sky**     | Build trigger | `pnpm run prepublishOnly` subprocess | Frontend packages built through Turborepo via Maintain              |

---

## Configuration

| Option         | CLI Flag                       | Description                                              |
| :------------- | :----------------------------- | :------------------------------------------------------- |
| Build mode     | `debug` / `release` subcommand | Controls `--release` flag and optimization level         |
| Target triple  | `--target`                     | Rust target triple for cross-compilation                 |
| Element filter | `--element`                    | Build only a specific element rather than the full group |
| Script path    | `--script`                     | Override the default Rhai build configuration script     |
| Verbosity      | `--verbose`                    | Enable detailed subprocess output logging                |

**Shell script entry points:**

| Script            | Purpose                                      |
| :---------------- | :------------------------------------------- |
| `Debug.sh`        | Full debug build of all elements             |
| `Dev-Mountain.sh` | Hot-reload development mode for Mountain     |
| `Release.sh`      | Optimized release build with all elements    |
| `Profile.sh`      | Release build with profiling instrumentation |
| `Debug/All.sh`    | Debug all components including frontend      |
| `Debug/Wind.sh`   | Debug Wind TypeScript service layer only     |
