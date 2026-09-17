# Engineering-record migration report

The records model in [GOVERNANCE.md](GOVERNANCE.md) was applied to the
repository at `d606405` (main, 2026-09-17). This report lets a reviewer
verify that nothing disappeared: every engineering document that existed
is listed with its old role, what was done to it and where its content
now lives. User-guide pages (`docs/user-guide/`, 40 files) were not part
of the migration; only their references to renumbered decisions were
repaired.

## Principles applied

* **Existing reference pages keep their paths.** They are linked from
  code comments, tests, the user guide and the formal repository; the
  front door classifies them instead of moving them. This is a
  classification, not a folder shuffle.
* **History moved out of current pages, never deleted.** Three documents
  carried chronology beside current truth; the chronology now lives under
  `docs/changes/history/` with a pointer left behind.
* **IDs are stable.** ADR numbers were not reassigned, with one
  exception forced by two collisions (below).
* **Nothing decided was re-decided.** Open items were re-filed as issues
  with their original text as evidence; closed items keep their text in
  the archived ledger.

## Structural changes

| Before | After | Action | Why |
|---|---|---|---|
| no engineering front door (`README.md` reading list only) | `docs/README.md` | created | one entry point routed by question; current snapshot |
| `docs/adr/README.md`: a title list with two duplicate numbers | decision index with status, date, area, supersession; frontmatter on every record; `TEMPLATE.md` | rebuilt | machine-checkable lifecycle |
| ADR `0017-behaviour-systems-flatten-into-the-flat-design.md` (collided with `0017-lsp-is-an-adapter.md`, created a day earlier) | `0021-behaviour-systems-flatten-into-the-flat-design.md`, `renumbered-from: ADR-0017` | renumbered | first claim keeps the number; the later record moves |
| ADR `0018-a-component-interface-is-a-stored-promise.md` (collided with `0018-three-information-levels.md`) | `0022-a-component-interface-is-a-stored-promise.md`, `renumbered-from: ADR-0018` | renumbered | same rule |
| no proposal record | `docs/proposals/` (index, template) | created | undecided designs stop masquerading as decisions |
| `docs/DESIGN_ISSUES.md`: 46 rows, open and closed mixed, DI-29 and DI-30 used twice | `docs/issues/` — ten records ISS-0001…0010 with frontmatter, an index; the ledger archived in place with a header and a DI → ISS map | split | unique IDs and states for what is open; closed history untouched |
| `docs/03-open-questions.md`: the pre-implementation checklist (Chinese) | archived in place with a table saying where each section was answered | archived | most of it was decided by ADRs or implementation; the mapping is the value |
| `docs/ROADMAP.md`: ✅/⏳ ledger of steps A–O, BS, ST-1…9, IDE-1…4, LIB-1…3 plus priorities | `docs/ROADMAP.md` — priorities and gated items only; `docs/changes/history/milestones.md` — the completed ledger with its evidence; `docs/project-records/STATUS.md` — what exists per area | split | future, present and past have different owners and mutation rules |
| `docs/PROTOCOL.md` header: every minor version's additions in one paragraph | `docs/changes/history/protocol-versions.md` (a table); `PROTOCOL.md` states the current version and links it | split | the spec states current truth; compatibility history is a change record |
| `docs/STUDIO_COMPILER_INTEGRATION.md` §2 *Bugs found and fixed*, §4 *Non-goals of this milestone* | `docs/changes/history/formula-editing-milestone.md`; §2 left as a pointer; §3 (the evidence map) kept | split | the page describes the boundary; the milestone record is history |
| no status matrix | `docs/project-records/STATUS.md` | created | one place for *implemented / partial / planned* with evidence |
| no change mechanism | `docs/changes/` — `unreleased/` fragments, `history/`, template | created | observable changes recorded without a release process |
| no formal-correspondence summary (spread over `02-kernel-spec.md`, `BEHAVIOR_SYSTEMS.md`, ADR prose) | `docs/project-records/FORMAL_CORRESPONDENCE.md` with strength labels | created | one table of what is proved, informed, tested or chosen |
| no governance text | `docs/project-records/GOVERNANCE.md`, `RESEARCH.md` | created | the model and the precedents behind it |
| no validation | `scripts/validate_docs.py` + unit tests, `just docs-check`, CI step | created | IDs, statuses, supersession, index coverage, links |
| no agent guidance | `.claude/skills/maintaining-bdl-engineering-records/` (local, untracked by project policy) | created | record selection, supersession, status hygiene, claim strength |

## Inventory of every engineering document

Role vocabulary: **spec** (normative, current) · **arch** (current
architecture reference) · **decision** · **issues** · **roadmap** ·
**status** · **evidence** · **walkthrough** (developer tutorial) ·
**background** (research context, not truth). *Reader*: contributor
unless stated. *Mutable*: current pages are edited in place; decisions
are append-only.

| Path | Role before | Reader | Normative? | Current or historical | Findings | Action | Now |
|---|---|---|---|---|---|---|---|
| `README.md` | entry | anyone | no | current | *Status* paragraph restated the roadmap | updated | points at the front door, status, roadmap, changes |
| `docs/GETTING_STARTED.md` | walkthrough | new contributor | no | current | reading order named `DESIGN_ISSUES.md` for new decisions | updated | routes to issues/proposals/adr, status, changes |
| `docs/01-paper-digest.md` | background (Chinese) | language reader | no | historical context | — | keep | registered as background |
| `docs/02-kernel-spec.md` | spec (kernel, from Lean; Chinese) | contributor, researcher | yes | current (pre-Phase-9a kernel) | does not yet transcribe Phase 9a lists/buffers | keep | spec; gap recorded in ISS-0001 and FORMAL_CORRESPONDENCE |
| `docs/03-open-questions.md` | issues (pre-implementation, Chinese) | contributor | no | historical | overlapped `DESIGN_ISSUES.md`; most items decided since | archived in place | header maps every section to its answer |
| `docs/ARCHITECTURE.md` | arch | contributor | no | current | cited ADR-0017 for two different decisions | keep; reference fixed | arch (overview) |
| `docs/BEHAVIOR_SYSTEMS.md` | evidence (FV correspondence, object by object) | contributor, researcher | no | current | — | keep | evidence; summarised in FORMAL_CORRESPONDENCE |
| `docs/BEHAVIOR_SYSTEM_ARCHITECTURE.md` | arch | contributor | no | current | §13 records a deferred design (nested packaging) | keep | arch; the deferral is ISS-0007 |
| `docs/CODEGEN_RUST.md` | arch + evidence | contributor | no | current | — | keep | arch |
| `docs/COMPILER_PIPELINE.md` | arch + diagnostic catalogue | contributor | partly (diagnostic codes) | current | two "planned" remarks | keep | arch |
| `docs/COMPONENT_BOUNDARY.md` | design of an unbuilt feature | contributor | no | designed, not implemented | read as if it existed | header added | arch (design), status row *planned* |
| `docs/DEPLOYMENT_READ_MODEL.md` | spec/arch (read model) | contributor | yes (wire shape) | current | — | keep | arch |
| `docs/DEPLOYMENT_WALKTHROUGH.md` | walkthrough | contributor | no | current | — | keep | walkthrough |
| `docs/DESIGN_ISSUES.md` | issues + decisions ledger | contributor | no | mixed | duplicates DI-29/DI-30; open and closed mixed; DI-30 (parameters) closed by ADR-0020 | archived; open rows → issues | archived ledger |
| `docs/EXECUTABLE_IR.md` | arch | contributor | no | current | — | keep | arch |
| `docs/EXECUTION_WALKTHROUGH.md` | walkthrough | contributor | no | current | — | keep | walkthrough |
| `docs/HARDWARE_MODEL.md` | spec (board files) + arch | contributor | yes (file format) | current | one "planned" remark | keep | spec |
| `docs/IDE_SERVICE_ARCHITECTURE.md` | arch | contributor | no | current | "Textual surface today" and "Virtual documents" described future work that has landed | updated | arch |
| `docs/IDE_SERVICE_RESEARCH_CONTEXT.md` | background | contributor | no | historical context | — | keep | background |
| `docs/IR.md` | arch | contributor | no | current | — | keep | arch |
| `docs/PROJECT_FORMAT.md` | spec | contributor, tool author | yes | current | — (text kind documented) | keep | spec |
| `docs/PROTOCOL.md` | spec + version history | contributor, client author | yes | current + historical | history paragraph in the header | split | spec; history in `changes/history/protocol-versions.md` |
| `docs/ROADMAP.md` | roadmap + status + history | contributor | no | mixed | 29 status marks; the only record that several milestones happened | split | roadmap (future); `changes/history/milestones.md`; `STATUS.md` |
| `docs/RUNTIME_SEMANTICS.md` | spec | contributor | yes | current | "since the backend milestone" phrasing | keep | spec |
| `docs/STANDARD_CONCEPT_LIBRARY.md` | spec (library files) + arch | contributor | yes (file format) | current | LIB-2/3 plans inline | keep | spec; plans on the roadmap |
| `docs/STUDIO_COMPILER_INTEGRATION.md` | arch + evidence + milestone record | contributor | no | mixed | §2 and §4 were history | split | arch + evidence (§3); history moved |
| `docs/STUDIO_UI.md` | arch/design (Studio) | contributor, designer | partly (interaction standard) | current | *Not built* sections are status statements | keep | arch; `STATUS.md` cites them |
| `docs/TESTING.md` | evidence | contributor | no | current | — | keep | evidence |
| `docs/TEXTUAL_SYNTAX.md` | spec | contributor, tool author | yes | current | — | keep | spec |
| `docs/adr/0001` … `0016`, `0017-lsp`, `0018-three-levels`, `0019`, `0020` | decisions | contributor, researcher | rationale | historical (immutable) | no frontmatter; no status on 0015–0020 | frontmatter added | decisions |
| `docs/adr/0017-behaviour-systems-flatten…` | decision | — | — | — | duplicate number | renumbered → ADR-0021 | decision |
| `docs/adr/0018-a-component-interface…` | decision | — | — | — | duplicate number | renumbered → ADR-0022 | decision |
| `docs/adr/README.md` | index | contributor | no | current | no status, dates or areas | rebuilt | decision index |
| `docs/user-guide/**` (40 files) | user documentation | designer | no | current | referenced ADR-0017/0018 by the old numbers | references repaired only | separate product |

## References repaired

* `ADR-0017` meaning *behaviour systems flatten* → `ADR-0021` in
  `docs/ARCHITECTURE.md`, `docs/user-guide/concepts/behavior-systems.md`,
  `docs/changes/history/milestones.md`, `docs/adr/0022-…` (its context).
* `ADR-0018` meaning *component interface* → `ADR-0022` in
  `docs/user-guide/concepts/components.md`, `docs/user-guide/VERIFICATION.md`.
* `ADR-0017` meaning *LSP is an adapter* and `ADR-0018` meaning *three
  information levels* are unchanged.
* Commit messages are history and were not rewritten; the decision index
  notes the two old numbers.

## What was verified afterwards

`scripts/validate_docs.py` (all decisions, issues and fragments valid;
every top-level page registered; no broken relative link under `docs/` or
in `README.md`); `python3 -m unittest scripts/test_validate_docs.py`; a
search for the old ADR file names across the repository; the user-guide
link check; `just check`.
