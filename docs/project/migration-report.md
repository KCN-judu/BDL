---
kind: project
area: process
status: current
---

# Engineering-record migration report

The records model in [governance.md](GOVERNANCE.md) was applied to the
repository at `d606405` (main, 2026-09-17). This report lets a reviewer verify
that nothing disappeared: every engineering document that existed is listed with
its old role, what was done to it and where its content now lives. User-guide
pages (`docs/user-guide/`, 40 files) were not part of the migration; only their
references to renumbered decisions were repaired.

## Second pass, 2026-09-17: one folder per kind

After the records model was in place the flat `docs/` folder (29 pages of seven
kinds, names in three conventions) was consolidated: every page moved into the
folder of its kind, took a lowercase kebab-case name and a
`kind / area / status` header, and every reference in code comments, tests, the
proto schema, the user guide and the entitlements files was rewritten (164
files). One page was removed. `docs/adr/` became `docs/decisions/`;
`docs/project-records/` became `docs/project/`.

| Was                                                                                      | Now                                                                                  |
| ---------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| `02-kernel-spec.md`                                                                      | `spec/kernel.md`                                                                     |
| `TEXTUAL_SYNTAX.md`                                                                      | `spec/textual-syntax.md`                                                             |
| `RUNTIME_SEMANTICS.md`                                                                   | `spec/runtime-semantics.md`                                                          |
| `PROJECT_FORMAT.md`                                                                      | `spec/project-format.md`                                                             |
| `PROTOCOL.md`                                                                            | `spec/protocol.md`                                                                   |
| `HARDWARE_MODEL.md`                                                                      | `spec/hardware-model.md`                                                             |
| `STANDARD_CONCEPT_LIBRARY.md`                                                            | `spec/concept-library.md`                                                            |
| `ARCHITECTURE.md`                                                                        | `architecture/overview.md`                                                           |
| `COMPILER_PIPELINE.md`, `IR.md`, `EXECUTABLE_IR.md`, `CODEGEN_RUST.md`                   | `architecture/compiler-pipeline.md`, `ir.md`, `executable-ir.md`, `codegen-rust.md`  |
| `BEHAVIOR_SYSTEM_ARCHITECTURE.md`                                                        | `architecture/behavior-systems.md`                                                   |
| `DEPLOYMENT_READ_MODEL.md`                                                               | `architecture/deployment-read-model.md`                                              |
| `IDE_SERVICE_ARCHITECTURE.md`                                                            | `architecture/ide-service.md`                                                        |
| `STUDIO_UI.md`, `STUDIO_COMPILER_INTEGRATION.md`                                         | `architecture/studio-ui.md`, `studio-compiler-integration.md`                        |
| `COMPONENT_BOUNDARY.md`                                                                  | `architecture/component-boundary.md`                                                 |
| `BEHAVIOR_SYSTEMS.md` (FV correspondence table)                                          | `evidence/behavior-systems-correspondence.md`                                        |
| `TESTING.md`                                                                             | `evidence/testing.md`                                                                |
| `GETTING_STARTED.md`, `EXECUTION_WALKTHROUGH.md`, `DEPLOYMENT_WALKTHROUGH.md`            | `guides/getting-started.md`, `execution-walkthrough.md`, `deployment-walkthrough.md` |
| `01-paper-digest.md`, `IDE_SERVICE_RESEARCH_CONTEXT.md`                                  | `background/paper-digest.md`, `ide-service-research.md`                              |
| `DESIGN_ISSUES.md`                                                                       | `archive/design-issues-ledger.md`                                                    |
| `ROADMAP.md`                                                                             | `project/roadmap.md`                                                                 |
| `adr/`                                                                                   | `decisions/` (record IDs unchanged)                                                  |
| `project-records/{STATUS,GOVERNANCE,RESEARCH,FORMAL_CORRESPONDENCE,MIGRATION_REPORT}.md` | `project/{status,governance,research,formal-correspondence,migration-report}.md`     |
| `03-open-questions.md`                                                                   | **removed** — see below                                                              |

### The pre-implementation checklist

`03-open-questions.md` (2026-09-15, Chinese) listed the questions to decide
before implementation started. Every one has since been decided, implemented, or
re-filed as an issue, so the page was removed; this table is where each section
went. The original text is in Git history
(`git show d606405:docs/03-open-questions.md`).

| Section                                                                                                                  | Answered by                                                                                                                                                                                                                                   |
| ------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A — language, numerics, ids, delivery                                                                                    | Rust workspace (`architecture/overview.md`); ADR-0011 (floats); ADR-0008 (ids); Studio + `bdld` + LSP + CLI (`project/status.md`)                                                                                                             |
| B — `PropertyId` / evidence, dimensions, lists, `Causal`                                                                 | ISS-0003 (open); DI-2 (7 SI + angle); ISS-0001 (lists — FV Phase 9a, production open); DI-8 (conservative `Causal` accepted)                                                                                                                  |
| C — surface: syntax, definition forms, temporal modifiers, contexts, candidates, affine units, device kinds, board files | ADR-0013, ADR-0014, ADR-0020 (syntax); formulas only (curve / example-fit / component forms: not built); ISS-0010 (modifiers, contexts); ISS-0002 (candidates); ISS-0004 (affine units); `spec/hardware-model.md` (device kinds, board files) |
| D — diagnostics format, workspace states, explanation view, declared vs computed evidence                                | `architecture/compiler-pipeline.md` (diagnostics), `architecture/studio-ui.md` (states, Explain), ADR-0018; evidence kinds: ISS-0003                                                                                                          |
| E — backend target, scheduler, numeric deviations, supplied blocks                                                       | ADR-0016 (Rust), `spec/runtime-semantics.md` (scheduler), DI-15 / ADR-0011 (numerics), `architecture/component-boundary.md` + ADR-0005 (designed, not built)                                                                                  |
| F — tests                                                                                                                | `evidence/testing.md`, `project/formal-correspondence.md`                                                                                                                                                                                     |

## Principles applied

- **First pass: classify without moving; second pass: move once, with every
  reference rewritten.** The first pass kept paths so nothing broke while the
  model was unproven; the second pass (above) moved each page into its kind's
  folder in one commit with all references repaired and the validator checking
  every link.
- **History moved out of current pages, never deleted.** Three documents carried
  chronology beside current truth; the chronology now lives under
  `docs/changes/history/` with a pointer left behind.
- **IDs are stable.** ADR numbers were not reassigned, with one exception forced
  by two collisions (below).
- **Nothing decided was re-decided.** Open items were re-filed as issues with
  their original text as evidence; closed items keep their text in the archived
  ledger.

## Structural changes

| Before                                                                                                                          | After                                                                                                                                                                                       | Action     | Why                                                                        |
| ------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------- | -------------------------------------------------------------------------- |
| no engineering front door (`README.md` reading list only)                                                                       | `docs/README.md`                                                                                                                                                                            | created    | one entry point routed by question; current snapshot                       |
| `docs/decisions/README.md`: a title list with two duplicate numbers                                                             | decision index with status, date, area, supersession; frontmatter on every record; `TEMPLATE.md`                                                                                            | rebuilt    | machine-checkable lifecycle                                                |
| ADR `0017-behaviour-systems-flatten-into-the-flat-design.md` (collided with `0017-lsp-is-an-adapter.md`, created a day earlier) | `0021-behaviour-systems-flatten-into-the-flat-design.md`, `renumbered-from: ADR-0017`                                                                                                       | renumbered | first claim keeps the number; the later record moves                       |
| ADR `0018-a-component-interface-is-a-stored-promise.md` (collided with `0018-three-information-levels.md`)                      | `0022-a-component-interface-is-a-stored-promise.md`, `renumbered-from: ADR-0018`                                                                                                            | renumbered | same rule                                                                  |
| no proposal record                                                                                                              | `docs/proposals/` (index, template)                                                                                                                                                         | created    | undecided designs stop masquerading as decisions                           |
| `docs/archive/design-issues-ledger.md`: 46 rows, open and closed mixed, DI-29 and DI-30 used twice                              | `docs/issues/` — ten records ISS-0001…0010 with frontmatter, an index; the ledger archived in place with a header and a DI → ISS map                                                        | split      | unique IDs and states for what is open; closed history untouched           |
| `docs/03-open-questions.md`: the pre-implementation checklist (Chinese)                                                         | archived in place with a table saying where each section was answered                                                                                                                       | archived   | most of it was decided by ADRs or implementation; the mapping is the value |
| `docs/project/roadmap.md`: ✅/⏳ ledger of steps A–O, BS, ST-1…9, IDE-1…4, LIB-1…3 plus priorities                              | `docs/project/roadmap.md` — priorities and gated items only; `docs/changes/history/milestones.md` — the completed ledger with its evidence; `docs/project/status.md` — what exists per area | split      | future, present and past have different owners and mutation rules          |
| `docs/spec/protocol.md` header: every minor version's additions in one paragraph                                                | `docs/changes/history/protocol-versions.md` (a table); `PROTOCOL.md` states the current version and links it                                                                                | split      | the spec states current truth; compatibility history is a change record    |
| `docs/architecture/studio-compiler-integration.md` §2 _Bugs found and fixed_, §4 _Non-goals of this milestone_                  | `docs/changes/history/formula-editing-milestone.md`; §2 left as a pointer; §3 (the evidence map) kept                                                                                       | split      | the page describes the boundary; the milestone record is history           |
| no status matrix                                                                                                                | `docs/project/status.md`                                                                                                                                                                    | created    | one place for _implemented / partial / planned_ with evidence              |
| no change mechanism                                                                                                             | `docs/changes/` — `unreleased/` fragments, `history/`, template                                                                                                                             | created    | observable changes recorded without a release process                      |
| no formal-correspondence summary (spread over `02-kernel-spec.md`, `BEHAVIOR_SYSTEMS.md`, ADR prose)                            | `docs/project/formal-correspondence.md` with strength labels                                                                                                                                | created    | one table of what is proved, informed, tested or chosen                    |
| no governance text                                                                                                              | `docs/project/governance.md`, `RESEARCH.md`                                                                                                                                                 | created    | the model and the precedents behind it                                     |
| no validation                                                                                                                   | `scripts/validate_docs.py` + unit tests, `just docs-check`, CI step                                                                                                                         | created    | IDs, statuses, supersession, index coverage, links                         |
| no agent guidance                                                                                                               | `.claude/skills/maintaining-bdl-engineering-records/` (local, untracked by project policy)                                                                                                  | created    | record selection, supersession, status hygiene, claim strength             |

## Inventory of every engineering document

Role vocabulary: **spec** (normative, current) · **arch** (current architecture
reference) · **decision** · **issues** · **roadmap** · **status** · **evidence**
· **walkthrough** (developer tutorial) · **background** (research context, not
truth). _Reader_: contributor unless stated. _Mutable_: current pages are edited
in place; decisions are append-only.

| Path                                                                            | Role before                                    | Reader                     | Normative?                    | Current or historical         | Findings                                                                              | Action                       | Now                                                            |
| ------------------------------------------------------------------------------- | ---------------------------------------------- | -------------------------- | ----------------------------- | ----------------------------- | ------------------------------------------------------------------------------------- | ---------------------------- | -------------------------------------------------------------- |
| `README.md`                                                                     | entry                                          | anyone                     | no                            | current                       | _Status_ paragraph restated the roadmap                                               | updated                      | points at the front door, status, roadmap, changes             |
| `docs/guides/getting-started.md`                                                | walkthrough                                    | new contributor            | no                            | current                       | reading order named `DESIGN_ISSUES.md` for new decisions                              | updated                      | routes to issues/proposals/adr, status, changes                |
| `docs/background/paper-digest.md`                                               | background (Chinese)                           | language reader            | no                            | historical context            | —                                                                                     | keep                         | registered as background                                       |
| `docs/spec/kernel.md`                                                           | spec (kernel, from Lean; Chinese)              | contributor, researcher    | yes                           | current (pre-Phase-9a kernel) | does not yet transcribe Phase 9a lists/buffers                                        | keep                         | spec; gap recorded in ISS-0001 and FORMAL_CORRESPONDENCE       |
| `docs/03-open-questions.md`                                                     | issues (pre-implementation, Chinese)           | contributor                | no                            | historical                    | overlapped `DESIGN_ISSUES.md`; most items decided since                               | archived in place            | header maps every section to its answer                        |
| `docs/architecture/overview.md`                                                 | arch                                           | contributor                | no                            | current                       | cited ADR-0017 for two different decisions                                            | keep; reference fixed        | arch (overview)                                                |
| `docs/evidence/behavior-systems-correspondence.md`                              | evidence (FV correspondence, object by object) | contributor, researcher    | no                            | current                       | —                                                                                     | keep                         | evidence; summarised in FORMAL_CORRESPONDENCE                  |
| `docs/architecture/behavior-systems.md`                                         | arch                                           | contributor                | no                            | current                       | §13 records a deferred design (nested packaging)                                      | keep                         | arch; the deferral is ISS-0007                                 |
| `docs/architecture/codegen-rust.md`                                             | arch + evidence                                | contributor                | no                            | current                       | —                                                                                     | keep                         | arch                                                           |
| `docs/architecture/compiler-pipeline.md`                                        | arch + diagnostic catalogue                    | contributor                | partly (diagnostic codes)     | current                       | two "planned" remarks                                                                 | keep                         | arch                                                           |
| `docs/architecture/component-boundary.md`                                       | design of an unbuilt feature                   | contributor                | no                            | designed, not implemented     | read as if it existed                                                                 | header added                 | arch (design), status row _planned_                            |
| `docs/architecture/deployment-read-model.md`                                    | spec/arch (read model)                         | contributor                | yes (wire shape)              | current                       | —                                                                                     | keep                         | arch                                                           |
| `docs/guides/deployment-walkthrough.md`                                         | walkthrough                                    | contributor                | no                            | current                       | —                                                                                     | keep                         | walkthrough                                                    |
| `docs/archive/design-issues-ledger.md`                                          | issues + decisions ledger                      | contributor                | no                            | mixed                         | duplicates DI-29/DI-30; open and closed mixed; DI-30 (parameters) closed by ADR-0020  | archived; open rows → issues | archived ledger                                                |
| `docs/architecture/executable-ir.md`                                            | arch                                           | contributor                | no                            | current                       | —                                                                                     | keep                         | arch                                                           |
| `docs/guides/execution-walkthrough.md`                                          | walkthrough                                    | contributor                | no                            | current                       | —                                                                                     | keep                         | walkthrough                                                    |
| `docs/spec/hardware-model.md`                                                   | spec (board files) + arch                      | contributor                | yes (file format)             | current                       | one "planned" remark                                                                  | keep                         | spec                                                           |
| `docs/architecture/ide-service.md`                                              | arch                                           | contributor                | no                            | current                       | "Textual surface today" and "Virtual documents" described future work that has landed | updated                      | arch                                                           |
| `docs/background/ide-service-research.md`                                       | background                                     | contributor                | no                            | historical context            | —                                                                                     | keep                         | background                                                     |
| `docs/architecture/ir.md`                                                       | arch                                           | contributor                | no                            | current                       | —                                                                                     | keep                         | arch                                                           |
| `docs/spec/project-format.md`                                                   | spec                                           | contributor, tool author   | yes                           | current                       | — (text kind documented)                                                              | keep                         | spec                                                           |
| `docs/spec/protocol.md`                                                         | spec + version history                         | contributor, client author | yes                           | current + historical          | history paragraph in the header                                                       | split                        | spec; history in `changes/history/protocol-versions.md`        |
| `docs/project/roadmap.md`                                                       | roadmap + status + history                     | contributor                | no                            | mixed                         | 29 status marks; the only record that several milestones happened                     | split                        | roadmap (future); `changes/history/milestones.md`; `STATUS.md` |
| `docs/spec/runtime-semantics.md`                                                | spec                                           | contributor                | yes                           | current                       | "since the backend milestone" phrasing                                                | keep                         | spec                                                           |
| `docs/spec/concept-library.md`                                                  | spec (library files) + arch                    | contributor                | yes (file format)             | current                       | LIB-2/3 plans inline                                                                  | keep                         | spec; plans on the roadmap                                     |
| `docs/architecture/studio-compiler-integration.md`                              | arch + evidence + milestone record             | contributor                | no                            | mixed                         | §2 and §4 were history                                                                | split                        | arch + evidence (§3); history moved                            |
| `docs/architecture/studio-ui.md`                                                | arch/design (Studio)                           | contributor, designer      | partly (interaction standard) | current                       | _Not built_ sections are status statements                                            | keep                         | arch; `STATUS.md` cites them                                   |
| `docs/evidence/testing.md`                                                      | evidence                                       | contributor                | no                            | current                       | —                                                                                     | keep                         | evidence                                                       |
| `docs/spec/textual-syntax.md`                                                   | spec                                           | contributor, tool author   | yes                           | current                       | —                                                                                     | keep                         | spec                                                           |
| `docs/decisions/0001` … `0016`, `0017-lsp`, `0018-three-levels`, `0019`, `0020` | decisions                                      | contributor, researcher    | rationale                     | historical (immutable)        | no frontmatter; no status on 0015–0020                                                | frontmatter added            | decisions                                                      |
| `docs/decisions/0017-behaviour-systems-flatten…`                                | decision                                       | —                          | —                             | —                             | duplicate number                                                                      | renumbered → ADR-0021        | decision                                                       |
| `docs/decisions/0018-a-component-interface…`                                    | decision                                       | —                          | —                             | —                             | duplicate number                                                                      | renumbered → ADR-0022        | decision                                                       |
| `docs/decisions/README.md`                                                      | index                                          | contributor                | no                            | current                       | no status, dates or areas                                                             | rebuilt                      | decision index                                                 |
| `docs/user-guide/**` (40 files)                                                 | user documentation                             | designer                   | no                            | current                       | referenced ADR-0017/0018 by the old numbers                                           | references repaired only     | separate product                                               |

## References repaired

- `ADR-0017` meaning _behaviour systems flatten_ → `ADR-0021` in
  `docs/architecture/overview.md`,
  `docs/user-guide/concepts/behavior-systems.md`,
  `docs/changes/history/milestones.md`, `docs/decisions/0022-…` (its context).
- `ADR-0018` meaning _component interface_ → `ADR-0022` in
  `docs/user-guide/concepts/components.md`, `docs/user-guide/VERIFICATION.md`.
- `ADR-0017` meaning _LSP is an adapter_ and `ADR-0018` meaning _three
  information levels_ are unchanged.
- Commit messages are history and were not rewritten; the decision index notes
  the two old numbers.

## What was verified afterwards

`scripts/validate_docs.py` (all decisions, issues and fragments valid; every
top-level page registered; no broken relative link under `docs/` or in
`README.md`); `python3 -m unittest scripts/test_validate_docs.py`; a search for
the old ADR file names across the repository; the user-guide link check;
`just check`.
