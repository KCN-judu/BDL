# BDL engineering documentation

The front door for everything about *what BDL is, why it is built this
way, what is implemented, what changed and what is unresolved*. People
who want to *use* Behavior Designer read the separate
[user guide](user-guide/README.md); the formal development lives in the
sibling repository `../BDL_FV`.

## Where to look

| I want… | Read |
|---|---|
| **current language truth** — what a design means | [02-kernel-spec.md](02-kernel-spec.md) (the kernel, transcribed from Lean) · [TEXTUAL_SYNTAX.md](TEXTUAL_SYNTAX.md) · [RUNTIME_SEMANTICS.md](RUNTIME_SEMANTICS.md) |
| **current file and wire formats** | [PROJECT_FORMAT.md](PROJECT_FORMAT.md) · [PROTOCOL.md](PROTOCOL.md) · [HARDWARE_MODEL.md](HARDWARE_MODEL.md) (board files) · [STANDARD_CONCEPT_LIBRARY.md](STANDARD_CONCEPT_LIBRARY.md) (library files) |
| **current architecture** — how the implementation is built | [ARCHITECTURE.md](ARCHITECTURE.md), then the subsystem pages below |
| **why a decision exists** | [adr/README.md](adr/README.md) — the decision index; each ADR is immutable rationale |
| **whether a change is proposed or decided** | [proposals/README.md](proposals/README.md) — open proposals; anything not there and not an ADR is not decided |
| **unresolved design questions** | [issues/README.md](issues/README.md) — the issue registry |
| **what is implemented today** | [project-records/STATUS.md](project-records/STATUS.md) |
| **what is planned next** | [ROADMAP.md](ROADMAP.md) — priorities only, never evidence |
| **what changed for users, project files, clients or developers** | [changes/README.md](changes/README.md) — unreleased fragments and the moved-out history |
| **which claims are formally proved** | [project-records/FORMAL_CORRESPONDENCE.md](project-records/FORMAL_CORRESPONDENCE.md) |
| **which test backs a claim** | [TESTING.md](TESTING.md) · [STUDIO_COMPILER_INTEGRATION.md §3](STUDIO_COMPILER_INTEGRATION.md) · [user-guide/VERIFICATION.md](user-guide/VERIFICATION.md) |
| **how to enter the codebase** | [GETTING_STARTED.md](GETTING_STARTED.md), then the walkthroughs below |
| **how these records work** | [project-records/GOVERNANCE.md](project-records/GOVERNANCE.md) · [RESEARCH.md](project-records/RESEARCH.md) (the precedents) · [MIGRATION_REPORT.md](project-records/MIGRATION_REPORT.md) (where the old documents went) |

## Architecture pages by area

| Area | Pages |
|---|---|
| compiler and semantics | [COMPILER_PIPELINE.md](COMPILER_PIPELINE.md) · [IR.md](IR.md) · [EXECUTABLE_IR.md](EXECUTABLE_IR.md) |
| code generation and runtime | [CODEGEN_RUST.md](CODEGEN_RUST.md) · [RUNTIME_SEMANTICS.md](RUNTIME_SEMANTICS.md) · [COMPONENT_BOUNDARY.md](COMPONENT_BOUNDARY.md) (supplied Rust — designed, not built) |
| behaviour systems | [BEHAVIOR_SYSTEM_ARCHITECTURE.md](BEHAVIOR_SYSTEM_ARCHITECTURE.md) (implementation) · [BEHAVIOR_SYSTEMS.md](BEHAVIOR_SYSTEMS.md) (formal correspondence, object by object) |
| deployment | [HARDWARE_MODEL.md](HARDWARE_MODEL.md) · [DEPLOYMENT_READ_MODEL.md](DEPLOYMENT_READ_MODEL.md) |
| IDE service, LSP, textual | [IDE_SERVICE_ARCHITECTURE.md](IDE_SERVICE_ARCHITECTURE.md) · [TEXTUAL_SYNTAX.md](TEXTUAL_SYNTAX.md) §14 (project items) · `editors/vscode/README.md` |
| Studio | [STUDIO_UI.md](STUDIO_UI.md) (design) · [STUDIO_COMPILER_INTEGRATION.md](STUDIO_COMPILER_INTEGRATION.md) (the boundary and its evidence map) |
| daemon and protocol | [PROTOCOL.md](PROTOCOL.md) · [ARCHITECTURE.md](ARCHITECTURE.md) (`bdl-daemon`) |

**Developer walkthroughs** (one design through the whole pipeline; not
user documentation): [EXECUTION_WALKTHROUGH.md](EXECUTION_WALKTHROUGH.md) ·
[DEPLOYMENT_WALKTHROUGH.md](DEPLOYMENT_WALKTHROUGH.md).

**Background** (research context, not truth): [01-paper-digest.md](01-paper-digest.md)
(the paper) · [IDE_SERVICE_RESEARCH_CONTEXT.md](IDE_SERVICE_RESEARCH_CONTEXT.md).

**Archived** (history, not to be extended): [DESIGN_ISSUES.md](DESIGN_ISSUES.md)
(the DI ledger; open items became issues) · [03-open-questions.md](03-open-questions.md)
(the pre-implementation checklist, with where each item was decided).

## Current snapshot — 2026-09-17

* **Architecture:** a Rust semantic core (`bdl-model` → `bdl-ir` →
  elaboration, checking, reactive evaluation, outputs, hardware) with a
  compiler façade, an executable IR lowered to a `no_std` Rust core, a
  behaviour-system layer that flattens into the one flat design, a shared
  IDE service under both Studio and the LSP, and `bdld` as the process
  boundary that Studio (Flutter) talks protobuf to.
  [ARCHITECTURE.md](ARCHITECTURE.md).
* **Governing decisions:** ADR-0001/0002 (Rust owns semantics; bdld is a
  process), 0008/0009 (stable ids; revisioned edits), 0010/0011 (Lean is
  the specification; floats are a recorded deviation), 0013/0014/0020
  (formula language; lossless syntax; text projects), 0015/0016
  (target-relative deployment; generated Rust implements the evaluator),
  0021/0022/0019 (flattening; stored contracts; groups as metadata),
  0017/0018 (LSP is an adapter; three information levels).
* **Unresolved:** ten design issues — occurrence windows, candidate
  definitions, the evidence model, affine units, user enums, `f32` on
  device, nested packaging, a structural output entity, projection
  deltas, temporal modifiers. [issues/README.md](issues/README.md).
* **Active work:** the first embedded platform adapter is priority 1;
  nothing is in progress in this repository beyond that ordering.
  [ROADMAP.md](ROADMAP.md).
* **Recently changed:** text projects and protocol 0.9 (2026-09-17); the
  engineering records themselves (this structure).
  [changes/unreleased/](changes/unreleased/).

## Rules in one paragraph

Current truth lives in a specification or architecture page and is
edited in place. A consequential choice gets an ADR that is never
rewritten — supersede it. Something not yet decided is a proposal or an
issue, not an ADR. What exists is in the status matrix, not in the ADR
and not in the roadmap. What changed for someone is a change fragment.
Run `just docs-check` before committing. The full rules:
[GOVERNANCE.md](project-records/GOVERNANCE.md).
