# BDL engineering documentation

The front door for everything about _what BDL is, why it is built this way, what
is implemented, what changed and what is unresolved_. People who want to _use_
Behavior Designer read the separate [user guide](user-guide/README.md); the
formal development lives in the sibling repository `../BDL_FV`.

## Layout

One folder per kind of record; every page carries a `kind / area / status`
header that `just docs-check` verifies against its folder.

| Folder                              | Holds                                                                 | Mutability                               |
| ----------------------------------- | --------------------------------------------------------------------- | ---------------------------------------- |
| [`spec/`](#specification)           | what BDL means now: language, runtime, files, wire, boards, libraries | current truth, edited in place           |
| [`architecture/`](#architecture)    | how the implementation is structured now                              | current truth, edited in place           |
| [`decisions/`](decisions/README.md) | why a consequential choice was made (ADR-NNNN)                        | append-only; superseded, never rewritten |
| [`proposals/`](proposals/README.md) | consequential changes not yet decided (PRP-NNNN)                      | open until an outcome                    |
| [`issues/`](issues/README.md)       | recognised problems with no answer yet (ISS-NNNN)                     | open · deferred · resolved               |
| [`project/`](#project)              | status, roadmap, governance, formal correspondence, research          | current                                  |
| [`changes/`](changes/README.md)     | what changed for someone, and the history moved out of current pages  | append-only                              |
| [`evidence/`](#evidence)            | which test or theorem backs a claim                                   | current index                            |
| [`guides/`](#guides)                | how a developer enters the codebase; one design through the pipeline  | current                                  |
| [`background/`](#background)        | research context that informed choices — not truth                    | historical                               |
| [`archive/`](#archive)              | superseded records kept because code and commits cite them            | frozen                                   |
| `user-guide/`                       | the designer-facing manual — a separate product                       | —                                        |

## Where to look

| I want…                                                          | Read                                                                                                                                                                                                                                                                                               |
| ---------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **current language truth**                                       | [spec/kernel.md](spec/kernel.md) · [spec/textual-syntax.md](spec/textual-syntax.md) · [spec/runtime-semantics.md](spec/runtime-semantics.md) · [spec/equation-library.md](spec/equation-library.md)                                                                                                |
| **current file and wire formats**                                | [spec/project-format.md](spec/project-format.md) · [spec/protocol.md](spec/protocol.md) (0.24) · [spec/hardware-model.md](spec/hardware-model.md) · [spec/deployment-capacity.md](spec/deployment-capacity.md) · [spec/concept-library.md](spec/concept-library.md) (the Standard Library)         |
| **current architecture**                                         | [architecture/overview.md](architecture/overview.md), then the page for the area                                                                                                                                                                                                                   |
| **why a decision exists**                                        | [decisions/README.md](decisions/README.md)                                                                                                                                                                                                                                                         |
| **whether a change is proposed or decided**                      | [proposals/README.md](proposals/README.md); anything not there and not an ADR is not decided                                                                                                                                                                                                       |
| **unresolved design questions**                                  | [issues/README.md](issues/README.md)                                                                                                                                                                                                                                                               |
| **what is implemented today**                                    | [project/status.md](project/status.md)                                                                                                                                                                                                                                                             |
| **what is planned next**                                         | [project/roadmap.md](project/roadmap.md) — priorities only, never evidence                                                                                                                                                                                                                         |
| **what changed for users, project files, clients or developers** | [changes/README.md](changes/README.md)                                                                                                                                                                                                                                                             |
| **which claims are formally proved**                             | [project/formal-correspondence.md](project/formal-correspondence.md)                                                                                                                                                                                                                               |
| **which test backs a claim**                                     | [evidence/testing.md](evidence/testing.md) · [evidence/behavior-systems-correspondence.md](evidence/behavior-systems-correspondence.md) · [architecture/studio-compiler-integration.md §3](architecture/studio-compiler-integration.md) · [user-guide/VERIFICATION.md](user-guide/VERIFICATION.md) |
| **what CI proves, and how to run it before a push**              | [project/ci.md](project/ci.md) — `python scripts/preflight.py fast \| full \| platform`                                                                                                                                                                                                            |
| **how to enter the codebase**                                    | [guides/getting-started.md](guides/getting-started.md)                                                                                                                                                                                                                                             |
| **the narrative design record — why every construct is there**   | the BDL Design and Formalization Monograph, `reference/paper/paper.md` (and the PDF) — a mirror of `KCN-judu/BDL_FV/paper/` at the commit in `reference/paper-mirror.toml`; edited there, never here (`reference/README.md`)                                                                       |
| **how these records work**                                       | [project/governance.md](project/governance.md) · [project/research.md](project/research.md) · [project/migration-report.md](project/migration-report.md)                                                                                                                                           |
| **how Studio and the guide are translated**                      | [project/localization-style.md](project/localization-style.md) · `locale/glossary.json`                                                                                                                                                                                                            |

## Pages

### Specification

| Page                                                  | Area        | What it fixes                                                                                                                      |
| ----------------------------------------------------- | ----------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| [kernel.md](spec/kernel.md)                           | language    | the kernel contract, transcribed from the Lean development                                                                         |
| [textual-syntax.md](spec/textual-syntax.md)           | textual     | the `.bdl` grammar: v0.1 core, v0.2 project items, the support matrix                                                              |
| [runtime-semantics.md](spec/runtime-semantics.md)     | runtime     | ticks, domains, `delay`/`sync`, numeric policy, what generated code must preserve                                                  |
| [project-format.md](spec/project-format.md)           | persistence | `bdl.toml`, flat / system / text projects, sidecars, migration rules                                                               |
| [protocol.md](spec/protocol.md)                       | protocol    | the Studio ↔ bdld messages, current version 0.24, compatibility rule                                                               |
| [hardware-model.md](spec/hardware-model.md)           | deployment  | capabilities, requirements, board description files                                                                                |
| [concept-library.md](spec/concept-library.md)         | language    | the Standard Library: Concept items, Source presets, the catalogue file (schema 2), Source creation, one-transaction instantiation |
| [equation-library.md](spec/equation-library.md)       | language    | the data core (collections, grouped and optional values), the equations, equality and order, diagnostics                           |
| [deployment-capacity.md](spec/deployment-capacity.md) | deployment  | static list bounds, window capacity under a schedule, the readiness matrix, allocator requirements                                 |

### Architecture

| Page                                                                          | Area             | Describes                                                                                                          |
| ----------------------------------------------------------------------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------ |
| [overview.md](architecture/overview.md)                                       | —                | the crates, the trust layers, the dependency direction                                                             |
| [compiler-pipeline.md](architecture/compiler-pipeline.md)                     | compiler         | every pass and its diagnostics                                                                                     |
| [ir.md](architecture/ir.md)                                                   | compiler         | the intermediate representations and recorded deviations                                                           |
| [executable-ir.md](architecture/executable-ir.md)                             | codegen          | slots, first-order expressions, the evaluation plan                                                                |
| [codegen-rust.md](architecture/codegen-rust.md)                               | codegen          | the owned Rust AST, printed crate, host bridge, differential tests                                                 |
| [behavior-systems.md](architecture/behavior-systems.md)                       | behavior-systems | components, contracts, instances, bindings, flattening, groups — implementation design                             |
| [deployment-read-model.md](architecture/deployment-read-model.md)             | deployment       | what a Deploy surface is handed                                                                                    |
| [output-realization.md](architecture/output-realization.md)                   | deployment       | a logical output, a deployment-chosen profile, a pure encoder, three judgments, sinks                              |
| [embedded-adapter.md](architecture/embedded-adapter.md)                       | runtime          | raw commands to pads on the RP2040 over Embassy: identity, numeric policy, clocks, arena                           |
| [firmware-build.md](architecture/firmware-build.md)                           | daemon           | from a project to the board: the build's stages, the UF2, the content identity, the flash, readiness composed once |
| [ide-service.md](architecture/ide-service.md)                                 | ide              | overlays, projections, text workspaces, the LSP adapter                                                            |
| [relationship-roles.md](architecture/relationship-roles.md)                   | compiler         | Source / Rule / Value: the derived role, its states, the boundary, the matrix                                      |
| [syntax-highlighting.md](architecture/syntax-highlighting.md)                 | ide              | one token classifier, the LSP vocabulary, two wire forms, Studio's theme                                           |
| [studio-ui.md](architecture/studio-ui.md)                                     | studio           | the design system and interaction standard of Studio                                                               |
| [studio-compiler-integration.md](architecture/studio-compiler-integration.md) | studio           | the Studio/compiler boundary and its evidence map                                                                  |
| [component-boundary.md](architecture/component-boundary.md)                   | runtime          | supplied Rust computation blocks — designed, not built                                                             |

### Project

[status.md](project/status.md) · [roadmap.md](project/roadmap.md) ·
[governance.md](project/governance.md) ·
[formal-correspondence.md](project/formal-correspondence.md) ·
[research.md](project/research.md) ·
[migration-report.md](project/migration-report.md) ·
[localization-style.md](project/localization-style.md) — the three locales, what
never changes with them, the glossary and the two pipelines

### Evidence

[testing.md](evidence/testing.md) — every test suite and what it holds ·
[behavior-systems-correspondence.md](evidence/behavior-systems-correspondence.md)
— the behaviour-system implementation object by object against the formal
theorems and the test that discharges each.

### Guides

[getting-started.md](guides/getting-started.md) — toolchain, map, run, reading
order · [execution-walkthrough.md](guides/execution-walkthrough.md) — one value
through a tick · [deployment-walkthrough.md](guides/deployment-walkthrough.md) —
one design out to a pin.

### Background

[paper-digest.md](background/paper-digest.md) — the engineering-binding content
of the conference-era manuscript (2026-09-15, archived in
`reference/paper/archive/`; superseded by the monograph) ·
[ide-service-research.md](background/ide-service-research.md) — the precedents
behind the IDE service.

### Archive

[design-issues-ledger.md](archive/design-issues-ledger.md) — the DI-1…DI-44
ledger; decided entries stay here because code comments cite them, open entries
became issues. The pre-implementation checklist was removed on 2026-09-17; where
each of its questions was answered is in the
[migration report](project/migration-report.md#the-pre-implementation-checklist).

## Current snapshot — 2026-09-20

- **Architecture:** a Rust semantic core (`bdl-model` → `bdl-ir` → elaboration,
  checking, reactive evaluation, outputs, hardware) with a compiler façade, an
  executable IR lowered to a `no_std` Rust core, a behaviour-system layer that
  flattens into the one flat design, a shared IDE service under both Studio and
  the LSP, and `bdld` as the process boundary that Studio (Flutter) talks
  protobuf to.
- **Governing decisions:** ADR-0001/0002 (Rust owns semantics; bdld is a
  process), 0008/0009 (stable ids; revisioned edits), 0010/0011 (Lean is the
  specification; floats are a recorded deviation), 0013/0014/0023 (formula
  language; lossless syntax; one project with Design, Code and Split views —
  ADR-0020's text mechanisms kept), 0015/0016 (target-relative deployment;
  generated Rust implements the evaluator), 0021/0022/0019 (flattening; stored
  contracts; groups as metadata), 0017/0018 (LSP is an adapter; three
  information levels), 0024/0025 (collections need an allocator; equations are
  definitional families and order is a declaration), 0026/0027 (a concept beside
  a plain value is observed; collections are bounded by the design and validated
  at deployment), 0032 (a Source is a derived presentation role, never a kernel
  type), 0033 (a value is rendered once, by the evaluator, in product words; fed
  inputs echoed in the trace), 0034 (canvas edges are signature edges and
  reference edges; _produces_ is the signature, _carried by_ a value per tick),
  0035 (highlighting is the IDE service's semantic tokens; Studio classifies
  nothing), 0031 (locale is presentation only), 0039 (the daemon owns the build
  and the flash; an artifact's identity is the content it was built from),
  0041/0042 (the Library offers value categories and a concept is named at
  creation; the Formula view is typed structure over the compiler's tree).
- **Unresolved:** fifteen design issues — occurrence windows, candidate
  definitions, the evidence model, affine units, user enums, `f32` on device,
  nested packaging, a structural output entity, projection deltas, temporal
  modifiers, `zip`'s cost in the core, the compiler's diagnostic sentences in
  one language, output realization beyond a pure encoder (ISS-0017), the input
  half beyond one line read once per tick (ISS-0018), a concept's display unit
  (ISS-0019); one proposal (PRP-0001, Source provision by device profile —
  consumed by ADR-0038). Realization on both sides — how a logical output
  reaches PWM, GPIO, I²C or an H-bridge, and how a Source is provided by a line,
  without the behaviour model knowing — is ADR-0036 / ADR-0038,
  [architecture/output-realization.md](architecture/output-realization.md) and
  [architecture/embedded-adapter.md](architecture/embedded-adapter.md) § The
  input half, consumed from FV Phases 13, 14 and 16
  ([formal-correspondence.md](project/formal-correspondence.md)).
- **Active work:** the first external UX study of the Pico demo, after the
  physical smoke test no change has yet run
  ([project/ux-study-pico.md](project/ux-study-pico.md),
  [evidence/pico-smoke-test.md](evidence/pico-smoke-test.md)), is priority 1;
  the platform adapter beyond one line and one duty (ISS-0018, ISS-0017) is
  second; nothing else is in progress in this repository.
- **Recently changed:** the Sem-block canvas (ADR-0044, protocol 0.30 — the
  canvas draws Sem blocks and, beside each, its mapping block, a concept is the
  template Sem blocks are created from and a rule the template mapping blocks
  apply; read edges from the analysis into a mapping block's sockets, a produce
  edge into its block, every gesture a text edit or the drive, no concept node;
  `MappingAnalysis.slots`, `Layout.definitions`, `ComposeAction.unreference` /
  `read`; parameters named where a concept repeats; ISS-0020 resolved —
  `docs/changes/unreleased/2026-09-sem-blocks.md`); the concept ladder's
  vocabulary (ADR-0043 — a concept is a type named by `ConceptId`, a Sem block
  its instance; `SemanticId` is gone, the `concept.*` diagnostic codes with an
  alias table; no semantics, no bytes changed —
  `docs/changes/unreleased/2026-09-concept-ladder.md`); edges as canvas objects
  with a contextual affordance, one disconnect path, the first-open arrangement
  and _Arrange Automatically_ (protocol 0.29 —
  `docs/changes/unreleased/2026-09-canvas-affordances.md`); the authoring UX
  slice (ADR-0041, ADR-0042, protocol 0.28 — the Standard Library as 22 value
  categories with the concept sheet naming a concept before it exists and the
  compiler's units shown as a fact; the Formula view as typed structure over the
  compiler's tree — a caret, keys as text edits or compiler actions, completion
  while typing, fractions and branch diagrams, every form structure; a saved
  formula unfolded on its node; `ComposeAction.apply`, positional completion
  ranking, `BDL_LIBRARIES`), composite units and the Formula view's structure
  (ADR-0040, protocol 0.27 — `180 deg per s`, `9.81 m per s^2`, `1 N * m` as a
  derived surface over the registered atoms with the dimension and scale the
  algebra gives; the compiler-owned value categories and unit candidates; roles,
  locals, structured forms, structural carets, keyboard insertion, completion at
  a caret, signature help and the saved-formula render in the IDE service), the
  first hardware demo end to end (ADR-0039, protocol 0.26 — the daemon builds
  the firmware stage by stage and flashes the Pico over its own bootloader; an
  image's identity is the content it was built from and a stale one is never
  flashed; the Deploy page ends in Build · Flash · Observe with one action per
  state; the Button → Lamp demo as two templates on the Welcome page;
  `bdld build` / `flash` / `init --template`), the Source half of the platform
  adapter (ADR-0038, protocol 0.25 — a device is for an output or a Source; a
  provider profile from `bdl-catalogue` turns the line's reading into the
  Source's value; the Pico firmware reads the line before each tick; an
  unprovided Source is an incomplete deployment with the reason), the first
  embedded platform adapter (ADR-0037 — `bdld compile --target rp2040_pico`
  generates the Embassy firmware beside the core; raw PWM and GPIO commands
  reach the solver-assigned pads through an explicit numeric policy;
  cross-compiled in CI), output realization (ADR-0036, protocol 0.24 — a device
  binding chooses a realization profile on the Deploy page; the profile's pure
  encoder lowers the output's value to a raw command below the behavior plan;
  admissibility is three judgments), the Source sheet (protocol 0.23 — a Source
  is created over a concept the designer chooses, existing or new in one
  transaction; the Standard Library's Source items are presets named _… Input_),
  the Code view as an IDE surface (protocol 0.22 — completion, hover,
  definition, references and _Format_ from `bdl-ide` over the text as typed),
  semantic highlighting (ADR-0035, protocol 0.21), one derived relationship role
  stated by the daemon (protocol 0.20, ADR-0032 amended), reference edges
  (ADR-0034, protocol 0.18). The full list, oldest last, is
  [changes/unreleased/](changes/unreleased/); the protocol's history is
  [changes/history/protocol-versions.md](changes/history/protocol-versions.md).

## Rules in one paragraph

Current truth lives in `spec/` or `architecture/` and is edited in place. A
consequential choice gets an ADR that is never rewritten — supersede it.
Something not yet decided is a proposal or an issue, not an ADR. What exists is
in `project/status.md`, not in the ADR and not in the roadmap. What changed for
someone is a change fragment. A new page goes into its kind's folder with a
`kind / area / status` header and a row on this page. Run `just docs-check`
before committing. The full rules:
[project/governance.md](project/governance.md).
