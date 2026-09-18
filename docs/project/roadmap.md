---
kind: project
area: process
status: current
---

# Roadmap

What is planned, in priority order, and what each item waits on. This page is
never evidence that something exists: what is implemented is in
[status.md](status.md); what landed and when is in
[changes/history/milestones.md](../changes/history/milestones.md) and
[changes/unreleased/](../changes/unreleased). A row is deleted when it lands;
nothing here has a status column.

## Priorities

| #   | Outcome                                                                                                           | Waits on                                                                                                  | Records            |
| --- | ----------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ------------------ |
| 1   | First embedded platform adapter (Embassy): target I/O, clock activation, the numeric policy on device             | the generated `no_std` core (exists); ISS-0006 (`f32` on device)                                          | ADR-0004, ADR-0016 |
| 2   | Build orchestration in `bdld`: `cargo check` / build of the generated crate with structured progress and failures | priority 1                                                                                                | —                  |
| 3   | Flash through `probe-rs`                                                                                          | priority 2; board files (`hardware/boards/`), an RP2040 board file, runtime loading of `hardware/boards/` | ADR-0015           |
| 4   | Telemetry back into Studio and the Monitor page (today a placeholder)                                             | priority 3; a stable identity manifest (`bdl-manifest.json`)                                              | —                  |
| 5   | Second embedded target (ESP32-S3) to prove HAL independence                                                       | priority 1                                                                                                | —                  |

## Language and toolchain

Not ordered against the priorities above; each moves when a concrete need or a
formal result arrives.

- Cross-domain occurrence windows: a surface form for the window the design can
  already write (ISS-0001); capacity validation for collections (ISS-0011).
- The equation language's remaining edges: the mixed concept-beside-value
  comparison (ISS-0012); `filter`'s cost in the generated core (ISS-0013); hover
  on an equation's name (completion carries its meaning today); record syntax
  lowering to nested grouped values, `forall`/`exists` sugar — only if a case
  asks.
- User-defined enums — ISS-0005; affine units — ISS-0004; temporal modifiers and
  contexts — ISS-0010: each starts as a proposal.
- Studio gaps listed in `docs/architecture/studio-compiler-integration.md` §3
  (_What remains_): the Deploy page on the 0.5 read model, an Explain request,
  domain regions and cycle emphasis on the canvas, entity hover and fixes inside
  a component's source.

- Cross-surface actions applying model operations from text editors; workspace
  symbols in the LSP.
- The Code view's remaining slice (ADR-0023): inline fault ranges, completion
  and hover in Studio's Code pane through the IDE service; a _Format_ command;
  whole-graph relayout on request; a persisted edit history that makes text-only
  changes undoable (ISS-0009).

- Projection deltas and a persisted edit history — ISS-0009.
- Team / project / package concept libraries (LIB-2); a separate device library
  that provides concepts and generates requirements (LIB-3) —
  `docs/spec/concept-library.md`.
- Native menu bar (`PlatformMenuBar` on macOS, in-window on Windows).
- An incremental query engine for the IDE service — only if profiling on real
  projects asks for it (baseline: ~2 ms per full analysis at 400 mappings).
- Supplied Rust computation blocks — designed in
  `docs/architecture/component-boundary.md` under ADR-0005; not started.

## Explicitly out of scope

From the engineering brief (§52): AI assistant · plugin marketplace · animation
polish · cloud sync · collaboration · full StateHandler editor · arbitrary
firmware backends · SMT · full numeric physical validation · code sandbox ·
component marketplace · custom debugger.
