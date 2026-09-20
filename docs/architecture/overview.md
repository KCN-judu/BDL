---
kind: architecture
area: process
status: current
---

# Architecture

BDL is an engineering implementation of the Behavior Design Language whose
kernel was derived in the Lean 4 development
[BDL_FV](https://github.com/KCN-judu/BDL_FV). This repository builds what that
development deliberately did not: the elaborator, the tooling, and the execution
path. It follows the formally developed semantics; it is **not itself formally
verified**.

## One property the whole system is built to keep obvious

```text
BDL semantics flows downward.  Implementation mechanisms never flow upward
and redefine the language.
```

```text
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

| Layer                               | Owns                                                                                                                                                            | Never does                                                                                                                  |
| ----------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **Flutter Studio**                  | presentation, interaction, layout, ephemeral render state                                                                                                       | compute type validity, semantic identity, dimensions, causality, clocks, output ownership, hardware feasibility, simulation |
| **Rust compiler (`bdld` + crates)** | the canonical project model, every semantic judgment, diagnostics, simulation, allocation, code generation, the placement of entities that have no position yet | render, decide where a placed node goes                                                                                     |
| **Generated Rust core**             | deterministic executable behaviour: domain step functions, state, output values                                                                                 | touch hardware, know about tasks or executors                                                                               |
| **Platform adapter**                | physical I/O, clock activation sources, telemetry transport                                                                                                     | interpret BDL semantics                                                                                                     |

Flutter may render an edit optimistically, but the truth comes back from the
compiler as a _projection_. Studio never holds a second copy of the language.

## Crates and their boundaries

```text
crates/
  bdl-model        stable IDs · surface model · revisioned edits · persistence · quantity vocabulary (no deps on the rest)
  bdl-ir           Design IR · Reactive Core IR (the kernel's Ty/Expr/envs)      (→ bdl-model)
  bdl-diagnostics  Diagnostic · Span · stable codes · deterministic order        (→ bdl-model)
  bdl-syntax       Logos lexer · event parser (RD + Pratt) · Rowan CST · typed AST · lowering (→ diagnostics)
  bdl-equations    the equation library: rank-1 schemes (type and dimension variables, {Data, Eq, Ord}), first-order matching, one closed Core builder per equation (→ ir, check)
  bdl-elab         concepts → Θ · signatures → interfaces · formulas → Core (equations inlined at their instance) (→ ir, syntax, check, equations)
  bdl-check        Core typing · Grant · realization vs interface · pretty       (→ ir, diagnostics)
  bdl-reactive     dependency graph · causality · Clocked · reference evaluator · simulation (→ ir, check)
  bdl-output       DriveWF · SingleDriver · CompleteOutputs · output_values       (→ ir, reactive)
  bdl-hardware     Capability/Resource/Hardware · device → requirements · boards · solve/diagnose (→ model)
  bdl-exec-ir      executable IR: slots, first-order expressions, evaluation plan; interpreter (→ ir, reactive)
  bdl-lower        reactive lowering: DesignIr → ExecIr (clock/state/input/output slots, inlining, order) (→ exec-ir, check)
  bdl-codegen-rust ExecIr → owned Rust AST → printed crate + host bridge + bdl-manifest.json; with an AdapterPlan, the adapter glue and the target firmware (→ exec-ir)
  bdl-compiler     analyze(snapshot) → ProjectAnalysis; analyze_deployment(snapshot, target) → DeploymentAnalysis; compile(snapshot, options) → CompileArtifact; compile_for_target(snapshot, target, …) adds the adapter plan (→ elab, check, reactive, output, hardware, lower, codegen)
  bdl-system       behaviour systems: components · instances · bindings · freshening · flatten → ProjectSnapshot + origins · analyze_system = flatten + analyze · packaging · legacy system JSON reader (→ model, compiler)
  bdl-text         the project's persistence: source discovery · identity sidecar and reconciliation · load_workspace → BehaviorSystem · item-level write-back · legacy JSON migration · names are identifiers (→ model, system, syntax)
  bdl-layout       the layout service: deterministic, incremental placement of entities without a position; never semantics (→ model, system)
  bdl-library      the Standard Library: items (Concept items, Source presets) from library/std/concepts.toml · plan (a fragment as ordered edits) · rank (candidate concepts for a preset) · search · multi-library set; schema 1–2 (→ model, elab)
  bdl-ide-db       IDE ground state: IdeHost · overlays · EntityRef/EntityRole · projections (text, visual) · index · immutable stamped AnalysisSnapshot · cancellation (→ compiler, syntax, elab)
  bdl-ide          semantic IDE queries over a snapshot: diagnostics · navigation (one answer to what is at a position: hover, definition, references, an equation's words) · explain · completion (incl. library items) · rename · actions · edit plans · invalidation preview · symbols · semantic tokens (one classifier) · formula projection/slot/compose · draft verdict · format (→ ide-db, library)
  bdl-lsp          LSP adapter only: lsp-server transport · position encoding · lsp-types rendering (→ ide, text)
  bdl-protocol     protobuf schema · framing · conversions                       (→ model, compiler, library)
  bdl-daemon       bdld: session (owns the project's IdeHost, its sources and text drafts), coordinator, transport, analysis push, the layout service on open and commit; `bdld check|compile|simulate` as the headless front end over the same session (→ protocol, compiler, ide, text, layout)
planned:
  bdl-component  supplied Rust component contracts (docs/architecture/component-boundary.md)
runtime/
  bdl-runtime-core   no_std vocabulary of every generated core: ActiveDomains, ClockSlot, RuntimeError, checked numerics; feature `collections`: the list operators and the recursor over alloc::Vec (no deps)
  bdl-runtime-host   std harness: DynValue, JSON run request/trace over stdio, cargo driver, recording sinks (→ runtime-core, runtime-embassy)
  bdl-runtime-adapter     no_std adapter vocabulary: the numeric policy at the raw command boundary, sink traits, the compiled schedule (→ runtime-core)
  bdl-runtime-embassy-rp  the RP2040 binding over embassy-rp — outside the workspace, built only into generated firmware (docs/architecture/embedded-adapter.md)
  bdl-runtime-arduino     the Arduino binding over avr-hal — outside the workspace, nightly-only, built only into generated firmware
```

Editor integration outside the workspace: `editors/vscode` (a thin client of
`bdl-lsp`).

Dependency direction is strict and acyclic:
`model → ir → {syntax → elab, check → equations → elab, check → reactive → output} → compiler → ide-db → ide → {lsp, daemon}`
(`protocol` sits between `compiler` and `daemon`; `text` and `layout` hang off
`system` and are joined by `daemon`, `lsp` and `cli`); `hardware` depends on
`model` only (it never sees `Δ`) and `compiler` joins the two; `lower → codegen`
hang off `exec-ir` and are joined by `compiler`; `runtime-core` depends on
nothing and is what generated code links against. A crate exists only where a
real boundary exists; tiny crates are merged rather than kept for the diagram.

## Behaviour systems flatten into the one flat design

A reusable behaviour is a component: an ordinary flat design as body, a public
interface of ports by identity, and a partition of its concepts and sinks into
private and shared. Instantiating it freshens every private identity (from the
same allocator the system's own ids come from, so nothing collides and nothing
is renumbered); a binding realises the destination port with a reference to the
source — the kernel's `declRef`, or `sync` across domains; the result is a flat
`ProjectSnapshot` that every existing pass consumes unchanged, with an origin
map back to instances and ports. There is one BDL: no system type checker,
evaluator, clock judgment or code generator exists, and the kernel gained no
construct (FV Phase 8a, ADR-0021,
`docs/evidence/behavior-systems-correspondence.md`). Every project is a
behaviour system (a design with no components is the degenerate one, `is_flat`);
the flat design is derived and never persisted.

## One project, persisted as text, shown as graph or text

A project is one directory (ADR-0023, `docs/spec/project-format.md`): the
sources under `src/**/*.bdl` are the semantic source of every project,
`.bdl/identities.json` binds every declaration to its stable id,
`ui/layout.json` holds presentation. `bdl-text::load_project` is the one loader
(bdld, the CLI, the language server); a legacy JSON project is migrated in place
the first time it is opened. Design, Code and Split are Studio views of the same
open project:

- graph → text: a canvas operation is a semantic edit on the model; the textual
  projection is the item-level splice of the sources (`bdl-text::write_back`),
  computed for the Code view on request and written on save — comments and
  formatting outside the touched item stay;
- text → graph: a Code-view edit is the whole text of one file against a
  revision (`ApplySourceEdit`); if it builds, declarations are bound to their
  identities by reconciliation and the project moves to a new revision; if not,
  the committed project stays and the draft is held with the loader's faults
  (ADR-0023 §5) — malformed text never erases the graph;
- layout never enters the model: moving a node changes the layout file and no
  revision; the layout service (`bdl-layout`) places exactly the entities that
  have no position, on open (persisted) and on every commit, deterministically
  and without moving anything placed. Studio arranges nothing at render time.

## The generated core is an implementation of the reference evaluator

`bdl-reactive::eval` is the executable definition of BDL runtime behaviour;
generated Rust (`bdl-lower` + `bdl-codegen-rust`) is an implementation of it
that lowers representation — dense slots, static structs, `f64` fields, inlined
lambdas — and may not change meaning. The two are kept independent and compared
trace for trace (ADR-0016, `docs/architecture/codegen-rust.md`). The core is
target-independent and knows no board: the platform adapter that binds
`DeploymentAnalysis`'s assignment to peripherals is a separate artefact
generated beside it behind features the host never enables — the RP2040 over
Embassy first, the Arduino Nano over `avr-hal` second, one target entry per
family (ADR-0037, `docs/architecture/embedded-adapter.md`).

## The compiler is a pipeline of explicit passes

Each pass has an explicit input and output type and is pure where practical:

```text
load/parse → identity resolution → signature resolution → surface elaboration
→ type checking → semantic-construction (grant) checking → dimension checking
→ dependency analysis → causality → clock domains → physical outputs
→ hardware requirement generation → hardware allocation → reactive lowering
→ Rust code generation
```

See `docs/architecture/compiler-pipeline.md`. Diagnostics are first-class
outputs: an incomplete project is the normal case, never a fail-fast.

## Functional core, imperative shell

- `bdl-model::apply_edit(&snapshot, &op) -> Result<Applied, EditError>` — the
  only way a project changes. Pure.
- `bdld`'s coordinator is the single imperative loop: it owns the `Session`,
  applies edits serially, and hands immutable snapshots to analyses.
- Studio's `reduce(state, action) -> (state, effects)` is pure; effects are
  data, executed by one effect executor that talks to the daemon.

Side effects live at the edges: process I/O in the daemon transport and the
Studio effect executor; file I/O in `bdl-model::persist`.

## Studio state is three things, kept apart

| Category            | Examples                                                     | Owner                                   |
| ------------------- | ------------------------------------------------------------ | --------------------------------------- |
| semantic projection | concepts, mappings, acceptance states, diagnostics, revision | the compiler (Studio only renders it)   |
| editor state        | selection, open inspector, pending requests, last error      | Studio reducer                          |
| rendering state     | drag position, hover, zoom, daemon log tail                  | Studio reducer, never persisted or sent |

Canvas positions are persisted in `ui/layout.json`, separately from the design,
and moving a node never creates a project revision.

## Revisions and stale results

Every semantic edit is sent against the revision Studio holds; `bdld` refuses it
if the project has moved on (`edit.stale_revision`). Every response and event
carries its revision. Studio discards anything older than what it already holds.
Revisions are strictly monotone for a session — undo produces a _new_ revision —
so staleness is a `<` comparison.

## One language service for both authoring surfaces

Studio (visual) and text editors (textual) are two projections of one semantic
model and consume one service: `bdl-ide-db` holds the ground state (committed
snapshot + overlays for unsaved drafts and buffers) and produces immutable,
stamped `AnalysisSnapshot`s; `bdl-ide` answers diagnostics, hover, completion,
references, rename, actions and invalidation previews in BDL-owned types keyed
by `EntityRef`, never by text position or node id; `bdl-lsp` and `bdld` are thin
adapters at the edge. LSP is an adapter, not the architecture (ADR-0017);
`docs/architecture/ide-service.md` is the reference.

## Refinement vs edit is an engineering asset

`apply_edit` classifies every operation as a _refinement_ (dependents'
established facts remain valid) or an _edit_ (dependents must be re-validated)
and reports an explicit `Invalidation` set —
`Interface | Realization | Semantic | Reactive | Clock | Output | Deployment` —
with the originating declarations. Incremental analysis subscribes to these
categories; it is a model, not UI folklore.

## Hardware feasibility is not typing

A project can be typed, causal, clock-consistent and output-complete and still
not fit the chosen board. Board selection reruns deployment analysis only, and
the workspace reports the two in different places (paper §Target- Specific
Hardware Validation; ADR-0006). In code this is two functions:
`analyze(snapshot)` never takes a target; `analyze_deployment(snapshot, target)`
never touches `Ty`, `Grant`, causality, clocks or the evaluator (ADR-0015). The
same `OutputId` bound to a PWM channel on one board and a digital output on
another has identical semantic analysis and different deployment analyses — and
that is a test.

## Behaviour semantics stop at the output; realization is deployment's

BDL is a programming language for expressing product behaviour while keeping the
behaviour model apart from the machine that will realize it. The behaviour layer
is concepts, rules, values, `delay`, `sync`, clock domains and outputs as
semantic obligations; it is reactive and stateful, not purely functional, and
the goal is that its meaning stays compositional and explicit
(`docs/spec/kernel.md`). Everything about the machine — which peripheral, which
protocol, which pin, which HAL call, which side effect — belongs to realization,
deployment and the backend, and is deferred to them.

What that means in the code today:

- A physical output is `PhysicalOutput { accepts: SemanticId, clock, required }`
  (`bdl-model`), a _logical_ output at a concept. Its driver is one relationship
  `d` with `β(d) = o`, and `DriveWF` (`bdl-output`) checks **exact**
  compatibility — the driver's type equals the accepted concept, the driver's
  clock equals the output's — not a conversion, not a monomorphization. The
  value that reaches the adapter is the concept's representation.
- Which device kind (PWM channel, digital output, H-bridge, …) realizes an
  output is a `DeviceBinding` — project data that only deployment analysis reads
  (`bdl-hardware` turns the kind into requirements, ADR-0015); no concept
  implies a protocol, and `OutputSpec` names none. The generated core produces
  the driver's value at the output slot; the platform adapter, not the design,
  turns it into an effect (`docs/spec/runtime-semantics.md`).
- How that value becomes a raw command is a **realization profile** the binding
  may name (`DeviceBinding.realization`, ADR-0036): a pure, typed encoder
  `Rep(C) → raw` from a registry, judged apart from the board (well formed ∧
  fits ∧ placed), lowered as a machine sink _below_ the behavior plan so the
  generated core also carries `Commands`; the platform adapter applies those
  commands to the pads the placement assigned through one explicit numeric
  policy (`docs/architecture/embedded-adapter.md`), and the physical effect
  begins there; choosing a profile changes no analysis and no sample
  (`docs/architecture/output-realization.md`).
- A Source is `s : () -> C` with no realization, a value entering the model from
  its environment (ADR-0032); what provides it at deployment — a device, a host,
  a network — is a separate, later choice (ISS-0016, PRP-0001), and the Standard
  Library's _… Input_ presets are authoring intent, not a device catalogue.
- Not implemented, formally investigated: a device transducer that realizes a
  Source (FV Phase 13, PRP-0001, ISS-0016) — the other physical boundary, kept
  separate from output realization; and, on the output side, stateful adapters,
  a device clock and atomic frames (FVI-0022, ISS-0017) —
  `docs/project/formal-correspondence.md`.

This is an architectural position, not a usability result: whether the
separation makes behaviour easier to design is an open research question, not a
claim these pages make.

## Platform notes

- **Targets**: macOS and Windows are first-class (CI builds both); Linux desktop
  builds as a by-product. One design; details adapt
  (`apps/studio/lib/platform/desktop.dart`, docs/architecture/studio-ui.md §3).
- **Plugins**: macOS plugins are linked through Swift Package Manager (Flutter ≥
  3.24), so CocoaPods is not required. Windows plugins build with the CMake
  toolchain. Only first-party plugins (`file_selector`) are used so far.
- **macOS**: Studio is not App-Sandboxed (`macos/Runner/*.entitlements`). It
  spawns `bdld` and reads/writes project directories the user chooses; the
  sandbox would confine both. Revisit before any App Store distribution.
- **Daemon discovery**: `--dart-define=BDLD_PATH`, then `$BDLD_PATH`, then a
  `bdld` beside the Studio executable, then `target/debug/bdld` (dev).
