# Roadmap

Ordered as in the engineering brief; nothing beyond the first vertical
slice is started before the slice works end-to-end.

## Vertical slice (milestone 1)

| Step | | Status |
|---|---|---|
| A | architecture documents, ADRs | ✅ |
| B | Rust project model, stable ids, revisioned edits | ✅ `bdl-model` |
| C | protocol schema | ✅ `bdl-protocol` (0.1.0) |
| D | `bdld` process + Flutter connection, handshake, versions in the status bar | ✅ |
| E | crash-safe persistence, save/reopen of an unresolved mapping | ✅ |
| F | Project manager / welcome screen (hero, Start, Recent) and Concepts + Mapping editor: page shell, node canvas (drag, link, unlink, delete), inspector with full concept/mapping editing, macOS look | ✅ (docs/STUDIO_UI.md) |
| — | **architecture review gate** (brief §63) | ⏳ next |
| G | type / semantic checking (`bdl-elab`, `bdl-check`) | |
| H | incomplete-declaration support in the checker (declared vs defined vs type-valid) | |
| I | host simulator with explicit input traces (`bdl-reactive`) | |
| J | `delay` / hold state, initial values | |
| K | output binding + single-driver diagnostic | |
| L | hardware resource allocator (`bdl-hardware`), Nano golden cases | |
| M | first board description: RP2040 | |
| N | Rust code generation via a backend AST + `bdl-manifest.json` | |
| O | `no_std` generated core, host execution == interpreter (differential test) | |
| P | first Embassy runtime adapter | |
| Q | `cargo check` / build orchestration in `bdld` with structured events | |
| R | flash via `probe-rs` | |
| S | telemetry back into Studio | |

## Explicitly deferred (brief §52)

AI assistant · plugin marketplace · animation polish · cloud sync ·
collaboration · full StateHandler editor · arbitrary firmware backends ·
SMT · full numeric physical validation · code sandbox · component
marketplace · custom debugger.

## Known v0.1 simplifications to revisit

* `ProjectChanged` sends the full projection; deltas planned (PROTOCOL.md).
* Revision/undo history is session-only; a persisted edit log is planned.
* Second embedded target (ESP32-S3) after RP2040 to prove HAL independence.
* Native menu bar (`PlatformMenuBar` on macOS, in-window on Windows).
