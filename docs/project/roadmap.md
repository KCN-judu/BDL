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

| #   | Outcome                                                                                                                                                                                                                     | Waits on                                                                                                                              | Records                      |
| --- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------- |
| 1   | First embedded platform adapter (Embassy): target I/O, clock activation, the numeric policy on device, a global allocator over an arena sized from the manifest's `collections` bounds, the most it supplies per list input | the generated `no_std` core (exists); the collections report (exists, `docs/spec/deployment-capacity.md`); ISS-0006 (`f32` on device) | ADR-0004, ADR-0016, ADR-0027 |
| 2   | Build orchestration in `bdld`: `cargo check` / build of the generated crate with structured progress and failures                                                                                                           | priority 1                                                                                                                            | —                            |
| 3   | Flash through `probe-rs`                                                                                                                                                                                                    | priority 2; board files (`hardware/boards/`), an RP2040 board file, runtime loading of `hardware/boards/`                             | ADR-0015                     |
| 4   | Telemetry back into Studio and the Monitor page (today a placeholder)                                                                                                                                                       | priority 3; a stable identity manifest (`bdl-manifest.json`)                                                                          | —                            |
| 5   | Second embedded target (ESP32-S3) to prove HAL independence                                                                                                                                                                 | priority 1                                                                                                                            | —                            |

## Language and toolchain

Not ordered against the priorities above; each moves when a concrete need or a
formal result arrives.

- Cross-domain occurrence windows: a surface form for the window the design can
  already write, bounded (ISS-0001) — with it, a compiler-recognised bounded
  transport becomes possible (ADR-0027's rejected ring buffer).
- The unit-domain spelling, stage 2 and 3 (ADR-0029 amendment): the hint on
  `mapping f : A` becomes a warning, the Code view shows it too, and a language
  edition may remove the shorthand with `bdld migrate-unit-domain` applied
  automatically — after a versioning policy for the language exists.
- The equation language's remaining edges: `zip`'s cost in the generated core
  (ISS-0013); record syntax lowering to nested grouped values — only if a case
  asks (FV Phase 11 removes the general quantifier and comprehension, FVD-0113).
- The Formula Composer's next slices: `match`, blocks (`let`), rules (`x => …`),
  collection and grouped literals and `delay` / `sync` as structured components
  (the Composer shows them as text, `NodeKind::Opaque`); a preferred display
  unit per concept and per simulation input (presentation only, FV Phase 10 §9);
  hover cards on components; affine units (°C, °F) once the formal
  point/difference follow-up lands (ISS-0004); drag-and-drop from the palette.
- Studio: the collections report (readiness, byte bounds, window requirements)
  on the Deploy page, and the schedule as deployment data there — today
  `bdld compile --period` only.
- User-defined enums — ISS-0005; affine units — ISS-0004; temporal modifiers and
  contexts — ISS-0010: each starts as a proposal.
- Studio gaps listed in `docs/architecture/studio-compiler-integration.md` §3
  (_What remains_): the Deploy page on the 0.5 read model (`rows` / `missing` /
  `blocker`), an Explain request over the protocol (hover cards exist; Explain
  is the compiler's formal detail and is a different answer), domain regions and
  cycle emphasis on the canvas, entity hover and fixes inside a component's
  source (a body-scoped `EntityRef` for `HoverEntity` / `ListSemanticActions`).

- Cross-surface actions applying model operations from text editors; workspace
  symbols in the LSP.
- The Code view's remaining slice (ADR-0023): inline fault ranges (the list and
  the caret jump stand in); rename from the editor; a second source file created
  from Studio; the unit-domain hint shown in the pane; whole-graph relayout on
  request; a persisted edit history that makes text-only changes undoable
  (ISS-0009).

- Projection deltas and a persisted edit history — ISS-0009.
- Team / project / package libraries of items (LIB-2); a device library that
  provides values and generates requirements (LIB-3, with ISS-0016 and PRP-0001)
  — `docs/spec/concept-library.md`. Output-side realization (a device encoder
  from a concept's representation to a raw command, chosen at deployment) is
  formally investigated (FV Phase 14) and has no production record: it starts as
  a proposal, if at all.
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
