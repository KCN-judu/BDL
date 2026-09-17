---
kind: guide
area: process
status: current
---
# Getting started — how to enter this project

One page for anyone (including future us) who opens this repository cold.
It says what the thing is, where the pieces live, how to run it, and in
what order to read.

## 1. What this is, in three sentences

BDL is a language in which a product's behaviour is a set of *typed
relationships between product concepts* (`dimByTilt : Tilt -> Brightness`),
each of which may exist before it is defined. The language's kernel was
derived and proved in Lean 4 in the sibling repo
[BDL_FV](https://github.com/KCN-judu/BDL_FV); that repo is the *semantic
authority* and is never a runtime dependency. This repo is the *production
implementation*: the editor (**BDL Studio**, Flutter), the compiler service
(**bdld**, Rust) with its simulator, hardware allocator and Rust code
generator, the host runtime the generated cores run on, a shared IDE
service with an LSP, and — later — the embedded platform adapter,
flashing and telemetry.

## 2. The one rule to internalise

```
BDL semantics flows downward.  Studio renders; Rust decides.
```

Studio never computes a semantic fact (type validity, identity, causality,
feasibility…). It sends edits to bdld and renders the projection and
diagnostics that come back. If you find yourself writing language logic in
Dart, stop (ADR-0001).

## 3. Toolchain

| Tool | Version | Notes |
|---|---|---|
| Rust | 1.89 (pinned in `rust-toolchain.toml`; rustup pulls rustfmt, clippy, rust-analyzer, rust-src) | |
| Flutter | 3.47 (`brew install --cask flutter`) | macOS and Windows are first-class targets |
| just | any | task runner; `just` lists recipes |
| protoc + protoc-gen-dart | only to regenerate Dart protocol code (`dart pub global activate protoc_plugin`) | Rust regenerates at build time |

No CocoaPods (macOS plugins link through Swift Package Manager).

## 4. Run it

```bash
just check      # everything CI runs: fmt, clippy, cargo test, dart format, flutter analyze/test, proto drift
just studio     # build bdld, run Studio against it
```

Run `just studio` from Terminal or a Finder-launched shell: macOS refuses
file dialogs to children of sandboxed hosts (an embedded terminal, say) —
Studio detects that and offers typed paths, but the native dialogs are the
intended path.

First walk-through in Studio: New Project… → in the Design page add
concepts `Tilt` (Quantity, angle) and `Brightness` (Quantity,
dimensionless) → add mapping `dimByTilt` reading Tilt, producing Brightness
→ select it, attach a formula in the inspector → Save.

Or open `examples/smart_lamp` (Open Project…): a finished design — tilt
and ambient light in, a required Light Output driven by
`adaptBrightness(dimByTilt(tilt), ambient)`, a PWM device — then walk
Design → Simulate (set the tilt, step) → Deploy (choose a board, see the
PWM line placed). `crates/bdl-compiler/tests/examples.rs` keeps it
faithful.

## 5. Repository map

```
apps/studio/            Flutter BDL Studio
  lib/app/              state · actions · effects · reducer (pure) · store
  lib/effects/          effect executor (daemon process, pickers, recent list)  ← the only I/O
  lib/daemon/           framed stdio client for bdld
  lib/ui/               shell, welcome, pages/ (design, simulate, deploy), canvas, inspector,
                        definition editor, library panels, mac/ (tokens, theme, controls)
  lib/protocol/gen/     generated protobuf (do not edit; `just proto`)
crates/
  bdl-model/            stable ids · surface model · apply_edit · persistence      (foundation)
  bdl-ir/               kernel types Ty/Expr/Prim · Design IR (Θ Δ Κ Ω β)
  bdl-diagnostics/      Diagnostic, Span, codes (shared by every pass)
  bdl-syntax/           textual syntax: lexer, parser, lossless CST, typed AST, lowering (docs/spec/textual-syntax.md)
  bdl-elab/             signature → Interface; formula → Core Expr (rep/mk insertion, units)
  bdl-check/            Core typing, Grant, declaration checking
  bdl-reactive/         dependency graph, causality, clocks, reference evaluator, simulation
  bdl-output/           physical outputs: DriveWF, SingleDriver, completeness (docs/guides/deployment-walkthrough.md)
  bdl-hardware/         capability model, device → requirements, boards, finite solver + diagnose
  bdl-exec-ir/          executable IR (slots, first-order expressions, plan) + interpreter (docs/architecture/executable-ir.md)
  bdl-lower/            DesignIr → ExecIr: clock/state/input/output slots, lambda inlining, evaluation order
  bdl-codegen-rust/     ExecIr → owned Rust AST → no_std core crate + host bridge + manifest (docs/architecture/codegen-rust.md)
  bdl-compiler/         analyze(snapshot) → ProjectAnalysis; compile(snapshot, options) → CompileArtifact
  bdl-system/           behaviour systems: components, instances, bindings → flatten → the flat design (docs/evidence/behavior-systems-correspondence.md)
  bdl-library/          concept libraries: templates → ordinary concepts (docs/spec/concept-library.md)
  bdl-ide-db/           IDE ground state: host, overlays, entity refs, projections, snapshots (docs/architecture/ide-service.md)
  bdl-ide/              semantic IDE queries: diagnostics, hover, completion, references, rename, actions
  bdl-lsp/              the LSP adapter (bdl-lsp binary over stdio)
  bdl-protocol/         bdl.proto · framing · conversions
  bdl-daemon/           bdld: session (owns the IdeHost), coordinator, transport
assets/brand/           the compass-λ mark (generator, SVGs, icons)
docs/                   architecture, formats, pipeline, IR, protocol, ADRs, design issues
reference/paper/        the paper (PDF + markdown source)
hardware/boards/        board descriptions as data
library/std/            the Standard Concept Library, concepts.toml
examples/smart_lamp/    the canonical example project (Design → Simulate → Deploy)
runtime/bdl-runtime-core  no_std vocabulary every generated core links against
runtime/bdl-runtime-host  std harness: run generated programs, JSON traces, cargo driver
```

Dependency direction (acyclic, enforced by Cargo):
`model → ir → {syntax → elab, check → reactive → output} → compiler → ide-db → ide → {lsp, daemon}`,
with `protocol` between `compiler` and `daemon`, `hardware` off `model`,
`lower → codegen` off `exec-ir` (docs/architecture/overview.md).

## 6. How a keystroke becomes a diagnostic

```
Studio widget ─UserAction─▶ reduce() ─Effect─▶ effect executor ─protobuf frame─▶ bdld
                                                                                   │
   Session.apply(base_revision, EditOp) → apply_edit(snapshot, op) → snapshot @ N+1
                                                                                   │
   bdl-compiler::analyze(snapshot@N+1)                                             │
     elab: Signature → Interface;  formula → parse → SurfaceExpr → Core Expr        │
     check: infer Core type · Grant · compare with Interface                        │
     → ProjectAnalysis { revision N+1, per-mapping status + diagnostics }           │
                                                                                   ▼
Studio ◀─ProjectChanged / AnalysisResult (revision-tagged; stale ones discarded)──┘
```

Everything above the daemon line is presentation; everything below is
pure functions over immutable snapshots, with I/O only at the transport and
persistence edges.

## 7. Reading order

| If you are here to… | Read |
|---|---|
| use Studio as a designer or prototyper | `docs/user-guide/README.md` — tutorials, concepts, the interface, troubleshooting |
| understand the system | `docs/README.md` (the front door), `docs/architecture/overview.md`, then `docs/decisions/` (short, numbered) |
| understand the language | `docs/background/paper-digest.md` (Chinese), `docs/spec/kernel.md` (exact kernel contract), then the paper |
| add a compiler pass | `docs/architecture/compiler-pipeline.md`, `docs/architecture/ir.md`, `crates/bdl-ir`, then the pass crate it belongs to |
| touch the editor | `docs/architecture/studio-ui.md` (design system + interaction standard), `apps/studio/lib/app/reducer.dart` |
| change the wire format | `docs/spec/protocol.md`, `crates/bdl-protocol/proto/bdl/v1/bdl.proto`, then `just proto` |
| decide something the paper left open | `docs/issues/` (the problem), `docs/proposals/` (a concrete alternative), `docs/decisions/` (the choice) — never silently in code; `docs/project/governance.md` says which |
| follow a design to a board | `docs/guides/deployment-walkthrough.md`, `docs/spec/hardware-model.md`, `crates/bdl-output`, `crates/bdl-hardware` |
| build a Deploy surface | `docs/architecture/deployment-read-model.md`, `crates/bdl-compiler/src/deploy_report.rs`, `crates/bdl-daemon/tests/deploy_e2e.rs` |
| generate and run Rust from a design | `docs/architecture/executable-ir.md`, `docs/architecture/codegen-rust.md`, `crates/bdl-compiler/tests/backend_differential.rs` |
| compose reusable behaviours | `docs/architecture/behavior-systems.md`, `docs/evidence/behavior-systems-correspondence.md`, `crates/bdl-system/tests/vertical_slice.rs` |
| see what exists and what is next | `docs/project/status.md`, then `docs/project/roadmap.md` |
| see what changed for users or clients | `docs/changes/` |

## 8. Conventions that are checked

* Rust: `#![forbid(unsafe_code)]`, no `unwrap` outside tests (clippy), typed
  errors (`thiserror`), `BTreeMap` for anything that is iterated into output.
* Dart: `dart format --page-width 100`, sealed actions, immutable state, no
  business logic in widgets.
* Diagnostics speak product language; kernel vocabulary goes in
  `technical_details` for the explanation view.
* Determinism: same project + same compiler version ⇒ byte-identical
  analysis, diagnostics order, generated code.
* Commits: no AI attribution trailers.

## 9. Where things are deliberately *not* yet

The embedded platform adapter (Embassy), `cargo` build orchestration and
flashing from `bdld`, telemetry and the Monitor page, contexts, supplied
Rust components, a persisted edit log — `docs/project/status.md`
says what exists, `docs/project/roadmap.md` the order, and `docs/issues/` the open
questions each will have to answer.
