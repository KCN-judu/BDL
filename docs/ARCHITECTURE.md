# Architecture

BDL is an engineering implementation of the Behavior Design Language whose
kernel was derived in the Lean 4 development [BDL_FV](https://github.com/KCN-judu/BDL_FV).
This repository builds what that development deliberately did not: the
elaborator, the tooling, and the execution path. It follows the formally
developed semantics; it is **not itself formally verified**.

## One property the whole system is built to keep obvious

```
BDL semantics flows downward.  Implementation mechanisms never flow upward
and redefine the language.
```

```
                 BDL Studio (Flutter)
                         │  typed, versioned protocol (protobuf over framed stdio)
                         ▼
                        bdld  ── one canonical revision stream per opened project
                         │
          ┌──────────────┼──────────────┐
       Compiler       Simulator      Hardware solver
          └──────────────┼──────────────┘
                    Reactive Core IR
                         │
                    Rust code generation
                         │
              generated no_std behaviour core
                   ┌─────┴─────┐
                Host        Embedded adapter (Embassy first)
             simulation            │
                             embedded-hal
                                   │
                             physical device
```

## Four trust layers

| Layer | Owns | Never does |
|---|---|---|
| **Flutter Studio** | presentation, interaction, layout, ephemeral render state | compute type validity, semantic identity, dimensions, causality, clocks, output ownership, hardware feasibility, simulation |
| **Rust compiler (`bdld` + crates)** | the canonical project model, every semantic judgment, diagnostics, simulation, allocation, code generation | render, decide layout |
| **Generated Rust core** | deterministic executable behaviour: domain step functions, state, output values | touch hardware, know about tasks or executors |
| **Platform adapter** | physical I/O, clock activation sources, telemetry transport | interpret BDL semantics |

Flutter may render an edit optimistically, but the truth comes back from the
compiler as a *projection*. Studio never holds a second copy of the language.

## Crates and their boundaries

```
crates/
  bdl-model        stable IDs · surface model · revisioned edits · persistence   (no deps on the rest)
  bdl-ir           Design IR · Reactive Core IR (the kernel's Ty/Expr/envs)      (→ bdl-model)
  bdl-diagnostics  Diagnostic · Span · stable codes · deterministic order        (→ bdl-model)
  bdl-syntax       Logos lexer · event parser (RD + Pratt) · Rowan CST · typed AST · lowering (→ diagnostics)
  bdl-elab         concepts → Θ · signatures → interfaces · formulas → Core      (→ ir, syntax, check)
  bdl-check        Core typing · Grant · realization vs interface · pretty       (→ ir, diagnostics)
  bdl-reactive     dependency graph · causality · Clocked · reference evaluator · simulation (→ ir, check)
  bdl-output       DriveWF · SingleDriver · CompleteOutputs · output_values       (→ ir, reactive)
  bdl-hardware     Capability/Resource/Hardware · device → requirements · boards · solve/diagnose (→ model)
  bdl-exec-ir      executable IR: slots, first-order expressions, evaluation plan; interpreter (→ ir, reactive)
  bdl-lower        reactive lowering: DesignIr → ExecIr (clock/state/input/output slots, inlining, order) (→ exec-ir, check)
  bdl-codegen-rust ExecIr → owned Rust AST → printed crate + host bridge + bdl-manifest.json (→ exec-ir)
  bdl-compiler     analyze(snapshot) → ProjectAnalysis; analyze_deployment(snapshot, target) → DeploymentAnalysis; compile(snapshot, options) → CompileArtifact (→ elab, check, reactive, output, hardware, lower, codegen)
  bdl-ide-db       IDE ground state: IdeHost · overlays · EntityRef/EntityRole · projections (text, visual) · index · immutable stamped AnalysisSnapshot · cancellation (→ compiler, syntax, elab)
  bdl-ide          semantic IDE queries over a snapshot: diagnostics · hover/explain · completion · references · rename · actions · edit plans · invalidation preview · symbols · tokens · draft verdict (→ ide-db)
  bdl-lsp          LSP adapter only: lsp-server transport · position encoding · lsp-types rendering (→ ide)
  bdl-protocol     protobuf schema · framing · conversions                       (→ model, compiler)
  bdl-daemon       bdld: session (owns the project's IdeHost), coordinator, transport, analysis push (→ protocol, compiler, ide)
planned:
  bdl-component  supplied Rust component contracts
  bdl-cli        headless front end sharing bdld's implementation
runtime/
  bdl-runtime-core   no_std vocabulary of every generated core: ActiveDomains, ClockSlot, RuntimeError, checked numerics (no deps)
  bdl-runtime-host   std harness: DynValue, JSON run request/trace over stdio, cargo driver (→ runtime-core)
planned:
  bdl-runtime-embassy  first platform adapter
```

Dependency direction is strict and acyclic: `model → ir → {syntax → elab,
check → reactive → output} → compiler → ide-db → ide → {lsp, daemon}`
(`protocol` sits between `compiler` and `daemon`); `hardware`
depends on `model` only (it never sees `Δ`) and `compiler` joins the two;
`lower → codegen` hang off `exec-ir` and are joined by `compiler`;
`runtime-core` depends on nothing and is what generated code links
against. A crate exists only where a real boundary exists; tiny crates are
merged rather than kept for the diagram.

## The generated core is an implementation of the reference evaluator

`bdl-reactive::eval` is the executable definition of BDL runtime
behaviour; generated Rust (`bdl-lower` + `bdl-codegen-rust`) is an
implementation of it that lowers representation — dense slots, static
structs, `f64` fields, inlined lambdas — and may not change meaning. The
two are kept independent and compared trace for trace (ADR-0016,
`docs/CODEGEN_RUST.md`). The core is target-independent and knows no
board: the platform adapter that binds `DeploymentAnalysis`'s assignment
to peripherals is a later, separate artefact.

## The compiler is a pipeline of explicit passes

Each pass has an explicit input and output type and is pure where practical:

```
load/parse → identity resolution → signature resolution → surface elaboration
→ type checking → semantic-construction (grant) checking → dimension checking
→ dependency analysis → causality → clock domains → physical outputs
→ hardware requirement generation → hardware allocation → reactive lowering
→ Rust code generation
```

See `COMPILER_PIPELINE.md`. Diagnostics are first-class outputs: an
incomplete project is the normal case, never a fail-fast.

## Functional core, imperative shell

* `bdl-model::apply_edit(&snapshot, &op) -> Result<Applied, EditError>` — the
  only way a project changes. Pure.
* `bdld`'s coordinator is the single imperative loop: it owns the `Session`,
  applies edits serially, and hands immutable snapshots to analyses.
* Studio's `reduce(state, action) -> (state, effects)` is pure; effects are
  data, executed by one effect executor that talks to the daemon.

Side effects live at the edges: process I/O in the daemon transport and the
Studio effect executor; file I/O in `bdl-model::persist`.

## Studio state is three things, kept apart

| Category | Examples | Owner |
|---|---|---|
| semantic projection | concepts, mappings, acceptance states, diagnostics, revision | the compiler (Studio only renders it) |
| editor state | selection, open inspector, pending requests, last error | Studio reducer |
| rendering state | drag position, hover, zoom, daemon log tail | Studio reducer, never persisted or sent |

Canvas positions are persisted in `ui/layout.json`, separately from the
design, and moving a node never creates a project revision.

## Revisions and stale results

Every semantic edit is sent against the revision Studio holds; `bdld`
refuses it if the project has moved on (`edit.stale_revision`). Every
response and event carries its revision. Studio discards anything older than
what it already holds. Revisions are strictly monotone for a session — undo
produces a *new* revision — so staleness is a `<` comparison.

## One language service for both authoring surfaces

Studio (visual) and text editors (textual) are two projections of one
semantic model and consume one service: `bdl-ide-db` holds the ground
state (committed snapshot + overlays for unsaved drafts and buffers) and
produces immutable, stamped `AnalysisSnapshot`s; `bdl-ide` answers
diagnostics, hover, completion, references, rename, actions and
invalidation previews in BDL-owned types keyed by `EntityRef`, never by
text position or node id; `bdl-lsp` and `bdld` are thin adapters at the
edge. LSP is an adapter, not the architecture (ADR-0017);
`docs/IDE_SERVICE_ARCHITECTURE.md` is the reference.

## Refinement vs edit is an engineering asset

`apply_edit` classifies every operation as a *refinement* (dependents'
established facts remain valid) or an *edit* (dependents must be
re-validated) and reports an explicit `Invalidation` set —
`Interface | Realization | Semantic | Reactive | Clock | Output | Deployment` —
with the originating declarations. Incremental analysis subscribes to these
categories; it is a model, not UI folklore.

## Hardware feasibility is not typing

A project can be typed, causal, clock-consistent and output-complete and
still not fit the chosen board. Board selection reruns deployment analysis
only, and the workspace reports the two in different places (paper §Target-
Specific Hardware Validation; ADR-0006). In code this is two functions:
`analyze(snapshot)` never takes a target; `analyze_deployment(snapshot,
target)` never touches `Ty`, `Grant`, causality, clocks or the evaluator
(ADR-0015). The same `OutputId` bound to a PWM channel on one board and a
digital output on another has identical semantic analysis and different
deployment analyses — and that is a test.

## Platform notes

* **Targets**: macOS and Windows are first-class (CI builds both); Linux
  desktop builds as a by-product. One design; details adapt
  (`apps/studio/lib/platform/desktop.dart`, STUDIO_UI.md §3).
* **Plugins**: macOS plugins are linked through Swift Package Manager
  (Flutter ≥ 3.24), so CocoaPods is not required. Windows plugins build
  with the CMake toolchain. Only first-party plugins (`file_selector`) are
  used so far.
* **macOS**: Studio is not App-Sandboxed (`macos/Runner/*.entitlements`). It
  spawns `bdld` and reads/writes project directories the user chooses; the
  sandbox would confine both. Revisit before any App Store distribution.
* **Daemon discovery**: `--dart-define=BDLD_PATH`, then `$BDLD_PATH`, then a
  `bdld` beside the Studio executable, then `target/debug/bdld` (dev).
