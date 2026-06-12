<table>
	<tr>
		<td colspan="1">
			<h3 align="center">
				<picture>
					<source media="(prefers-color-scheme: dark)" srcset="https://editor.land/Dark/Image/GitHub/Land.svg">
					<source media="(prefers-color-scheme: light)" srcset="https://editor.land/Image/GitHub/Land.svg">
					<img width="28" alt="Land Logo" src="https://editor.land/Image/GitHub/Land.svg">
				</picture>
			</h3>
		</td>
		<td colspan="3" valign="top">
			<h3 align="center"> Maintain 🔧</h3>
		</td>
	</tr>
</table>

---

# **Maintain** 🔧 Architecture

`Maintain` is the Rust build system and CI/CD toolkit for `Land`. It
orchestrates builds across all Land elements, embeds a **Rhai** scripting engine
for flexible automation, and provides type-safe TOML/JSON5 configuration
editing.

---

## Overview

| Component | Description                       | Implementation          |
| --------- | --------------------------------- | ----------------------- |
| `CLI`     | Subcommand dispatcher             | `clap` derive macros    |
| `Build`   | Cross-element build orchestration | `cargo` integration     |
| `Script`  | Rhai scripting engine             | Embedded `rhai` runtime |
| `Config`  | Type-safe config editing          | `toml_edit` / `json5`   |

---

## Build Pipeline

```
CLI → Parse subcommand → Dispatch
  ├── build       → cargo build (all elements)
  ├── test        → cargo test (all elements)
  ├── lint        → clippy + rustfmt
  ├── release     → cargo build --release + packaging
  └── script      → Rhai interpreter
```

---

## Related Documentation 📚

- [DeepDive](./DeepDive.md) - In-depth Maintain documentation
- [BuildPipeline](https://github.com/CodeEditorLand/Land/tree/Current/Documentation/GitHub/BuildPipeline.md) -
  Build pipeline
- [Common](https://github.com/CodeEditorLand/Common/tree/Current/Documentation/GitHub/Architecture.md) -
  Abstract trait definitions

---

## Shim Compatibility

| 🟠 Low-Level Shim                              | 🔵 Coverage Shim                   |
| ---------------------------------------------- | ---------------------------------- |
| Tier: `TierShim=Own\|Preempt`                  | Tier: `TierShim=Proxy\|Replace`    |
| Engine prototype hooks                         | Service routing + audit            |
| Error, Emitter, Cancel, Dispose, Async, Timing | IPC SwallowMap, DI proxy, AuditLog |

> This Element supports the Land deep-shim interception system. The shim
> intercepts VS Code engine events at both the JavaScript prototype level (🟠
> orange) and the application service level (🔵 blue). Gated behind `TierShim`
> env var (default: `None` - zero overhead). See the
> [Shim documentation](/doc/low-level-shim).

**Shim Modules:** No shim-specific modules - events routed through
`Wind`/`Mountain`/`Cocoon`.

---

**Project Maintainers:** Source Open
([Source/Open@Editor.Land](mailto:Source/Open@Editor.Land)) |
[GitHub Repository](https://github.com/CodeEditorLand/Maintain) |
[Report an Issue](https://github.com/CodeEditorLand/Maintain/issues)
