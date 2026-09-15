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
(**bdld**, Rust), and — later — simulator, hardware allocator, Rust code
generator and embedded runtime.

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

## 5. Repository map

```
apps/studio/            Flutter BDL Studio
  lib/app/              state · actions · effects · reducer (pure) · store
  lib/effects/          effect executor (daemon process, pickers, recent list)  ← the only I/O
  lib/daemon/           framed stdio client for bdld
  lib/ui/               shell, welcome, design page, canvas, inspector, mac/ (tokens, theme, controls)
  lib/protocol/gen/     generated protobuf (do not edit; `just proto`)
crates/
  bdl-model/            stable ids · surface model · apply_edit · persistence      (foundation)
  bdl-ir/               kernel types Ty/Expr/Prim · Design IR (Θ Δ Κ Ω β)
  bdl-diagnostics/      Diagnostic, Span, codes (shared by every pass)
  bdl-syntax/           textual syntax: lexer, parser, lossless CST, typed AST, lowering (docs/TEXTUAL_SYNTAX.md)
  bdl-elab/             signature → Interface; formula → Core Expr (rep/mk insertion, units)
  bdl-check/            Core typing, Grant, declaration checking
  bdl-reactive/         dependency graph, causality, clocks, reference evaluator, simulation
  bdl-output/           physical outputs: DriveWF, SingleDriver, completeness (docs/DEPLOYMENT_WALKTHROUGH.md)
  bdl-hardware/         capability model, device → requirements, boards, finite solver + diagnose
  bdl-exec-ir/          executable IR (slots, first-order expressions, plan) + interpreter (docs/EXECUTABLE_IR.md)
  bdl-lower/            DesignIr → ExecIr: clock/state/input/output slots, lambda inlining, evaluation order
  bdl-codegen-rust/     ExecIr → owned Rust AST → no_std core crate + host bridge + manifest (docs/CODEGEN_RUST.md)
  bdl-compiler/         analyze(snapshot) → ProjectAnalysis; compile(snapshot, options) → CompileArtifact
  bdl-library/          concept libraries: templates → ordinary concepts (docs/STANDARD_CONCEPT_LIBRARY.md)
  bdl-ide-db/           IDE ground state: host, overlays, entity refs, projections, snapshots (docs/IDE_SERVICE_ARCHITECTURE.md)
  bdl-ide/              semantic IDE queries: diagnostics, hover, completion, references, rename, actions
  bdl-lsp/              the LSP adapter (bdl-lsp binary over stdio)
  bdl-protocol/         bdl.proto · framing · conversions
  bdl-daemon/           bdld: session (owns the IdeHost), coordinator, transport
assets/brand/           the compass-λ mark (generator, SVGs, icons)
docs/                   architecture, formats, pipeline, IR, protocol, ADRs, design issues
reference/paper/        the paper (PDF + markdown source)
hardware/boards/        board descriptions as data
runtime/bdl-runtime-core  no_std vocabulary every generated core links against
runtime/bdl-runtime-host  std harness: run generated programs, JSON traces, cargo driver
```

Dependency direction (acyclic, enforced by Cargo):
`model → ir → {syntax → elab, check} → compiler → protocol → daemon`.

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
| understand the system | `docs/ARCHITECTURE.md`, then `docs/adr/` (short, numbered) |
| understand the language | `docs/01-paper-digest.md` (Chinese), `docs/02-kernel-spec.md` (exact kernel contract), then the paper |
| add a compiler pass | `docs/COMPILER_PIPELINE.md`, `docs/IR.md`, `crates/bdl-ir`, then the pass crate it belongs to |
| touch the editor | `docs/STUDIO_UI.md` (design system + interaction standard), `apps/studio/lib/app/reducer.dart` |
| change the wire format | `docs/PROTOCOL.md`, `crates/bdl-protocol/proto/bdl/v1/bdl.proto`, then `just proto` |
| decide something the paper left open | `docs/DESIGN_ISSUES.md` — record it there, never silently in code |
| follow a design to a board | `docs/DEPLOYMENT_WALKTHROUGH.md`, `docs/HARDWARE_MODEL.md`, `crates/bdl-output`, `crates/bdl-hardware` |
| build a Deploy surface | `docs/DEPLOYMENT_READ_MODEL.md`, `crates/bdl-compiler/src/deploy_report.rs`, `crates/bdl-daemon/tests/deploy_e2e.rs` |
| generate and run Rust from a design | `docs/EXECUTABLE_IR.md`, `docs/CODEGEN_RUST.md`, `crates/bdl-compiler/tests/backend_differential.rs` |
| see what is next | `docs/ROADMAP.md` |

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

Contexts, outputs, clocks, hardware allocation, code generation, runtime,
telemetry — see `docs/ROADMAP.md` for the order and `docs/DESIGN_ISSUES.md`
for the open questions each will have to answer.
