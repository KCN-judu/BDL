---
kind: project
area: process
status: current
---
# Engineering-record governance

How BDL keeps four things apart: **what is true now**, **why it was chosen**,
**what is implemented**, and **what changed for someone**. The model is
sized for a few maintainers, coding agents and future readers of the
language's history — the smallest set of records that keeps those four
facts from collapsing into one another. Which precedent informed each
choice is in [research.md](research.md).

## The record kinds

| Kind | Answers | Lives in | Mutability | ID |
|---|---|---|---|---|
| **Specification** | what BDL means now (language, files, protocol, kernel contract) | `docs/spec/kernel.md`, `docs/spec/textual-syntax.md`, `docs/spec/runtime-semantics.md`, `docs/spec/project-format.md`, `docs/spec/protocol.md`, `docs/spec/hardware-model.md`, `docs/spec/concept-library.md` | mutable current truth | — |
| **Architecture reference** | how the implementation is structured now | `docs/architecture/overview.md` and the subsystem pages listed in [docs/README.md](../README.md) | mutable current truth | — |
| **Decision record** (ADR) | why an important choice was made, what else was considered | `docs/decisions/NNNN-slug.md` | append-only once accepted | `ADR-NNNN` |
| **Proposal** | a consequential change that is *not yet* decided | `docs/proposals/NNNN-slug.md` | mutable while open, frozen at its outcome | `PRP-NNNN` |
| **Design issue** | a recognised problem with no chosen answer | `docs/issues/NNNN-slug.md` | mutable while open, frozen at resolution | `ISS-NNNN` |
| **Implementation status** | what of the accepted design exists, per area | `docs/project/status.md` | mutable | — |
| **Change record** | an externally visible change: what, who is affected, how to migrate | `docs/changes/unreleased/*.md`, later `docs/changes/<release>.md`; `docs/changes/history/` for the past | append-only once landed | date + slug |
| **Roadmap** | what is planned next, in order | `docs/project/roadmap.md` | mutable; never evidence that something exists | — |
| **Evidence** | which test, benchmark or theorem backs a claim | `docs/evidence/testing.md`, `docs/project/formal-correspondence.md`, `docs/architecture/studio-compiler-integration.md` §3, `docs/user-guide/VERIFICATION.md`, the BDL_FV repository | mutable index of links | — |

Two things are deliberately **not** engineering records: the
[user guide](../user-guide/README.md) (a separate product: *how do I use
it*), and the formal development's own history (`../BDL_FV`'s reports and
design decisions — the authority for what was proved and why; this
repository keeps only the correspondence).

The front door for all of it is [docs/README.md](../README.md).

## Selecting the record: what kind of fact changed?

| The fact that changed | Record to touch | Not this |
|---|---|---|
| the meaning of a construct, a file, a message, a runtime rule | the specification page | an ADR (unless the *choice* is new and consequential), the roadmap |
| where a responsibility lives, a crate boundary, a data flow | the architecture page | a diary entry in the page ("we then moved…") |
| a consequential choice was **made** | a new ADR (or `supersedes` an old one) | editing an old ADR's rationale |
| a choice is **being weighed** with concrete alternatives | a proposal | an ADR with status *proposed* |
| a problem is known, no alternative is concrete yet | a design issue | a proposal, an ADR |
| something got implemented, became partial, or was removed | `docs/project/status.md`, the roadmap row it completes | the ADR that designed it |
| users, project files, protocol clients or developers must react | a change fragment | prose inside the specification only |
| priorities moved | `docs/project/roadmap.md` | `docs/project/status.md` |
| a formal result now (or no longer) backs a claim | `docs/project/formal-correspondence.md`, the affected spec/ADR's `fv` field | broadening the theorem in prose |

One change may touch several kinds — protocol 0.9 touched the specification
(`docs/spec/protocol.md`), status (daemon/textual rows), a change fragment
(clients must handle `project.changed_on_disk`) and an ADR (the decision to
make text canonical). It touched each **once**. A refactor with no
observable effect touches nothing here; Git has it.

## Lifecycles

### Decision record

* **Created** when a consequential choice is made: a new subsystem
  boundary, a semantic rule, a persistence or protocol commitment, a
  deviation from the formal kernel. Small, reversible choices go in the
  page they affect.
* **Fields**: frontmatter `id`, `status`, `date`, `area`, `supersedes`,
  `superseded-by`; optional `related`, `fv`, `renumbered-from`. Body:
  Status (prose), Context, Decision, Alternatives (when there were any),
  Consequences. An *Amendment* section may be appended when the choice is
  extended without being replaced (ADR-0019 has one); it is dated and does
  not rewrite the text above it.
* **Statuses**: `accepted` · `superseded` · `rejected` (a decision taken
  and recorded, then reversed before implementation) · `withdrawn`.
  There is no `proposed` ADR: undecided work is a proposal.
* **Immutable after acceptance** except: the frontmatter, a supersession
  note at the top, an appended amendment, and factual corrections
  (a wrong path, a typo). Never rewrite context, rationale or the
  alternatives.
* **Superseded** by a new ADR that lists it in `supersedes`; the old one
  gets `superseded-by` and status `superseded`. The new record says
  whether the old choice was *wrong on the same evidence* or *right until
  the circumstances changed* — the two are different lessons. Current
  specification and architecture pages cite only the replacement.
* **Never deleted or archived**; the index lists every ID ever assigned.
  IDs are never reused or renumbered (the one renumbering, of two
  duplicate IDs, is recorded in `renumbered-from` and the
  [migration report](migration-report.md)).

### Proposal

* **Created** when a consequential change has a concrete shape but no
  decision — the stage GHC calls *discussion* and Rust calls an open RFC.
* **Fields**: `id`, `status`, `date`, `area`, `related-issues`,
  `superseded-by`; body: Problem · Goals and non-goals · Proposed design ·
  Compatibility and migration · Alternatives · Implementation and evidence
  · Open questions.
* **Statuses**: `draft` → `discussion` → `accepted` | `rejected` |
  `withdrawn`; `superseded` when another proposal replaces it.
* **Accepted** ⇒ an ADR is written (the proposal's *Proposed design* is
  its raw material; the ADR is the durable record, the proposal freezes
  and links it). Accepted is not implemented: `docs/project/status.md` and the roadmap
  carry the work.
* **Rejected / withdrawn** proposals stay: a negative result is history.

### Design issue

* **Created** when implementation pressure, the formal development or a
  review exposes a problem without an answer.
* **Fields**: `id`, `state`, `area`, `opened`, `resolved-by` (an ADR, a
  proposal, or a spec section), `related`; body: Problem · Why it matters
  · Current evidence · Dependencies · Resolution (when closed).
* **States**: `open` · `deferred` (a decision *not to decide now*, with
  the reason) · `resolved`.
* **Resolved** ⇒ `resolved-by` names the record that answers it; the
  issue leaves the active table and stays in the registry.
* The pre-2026-09-17 ledger `docs/archive/design-issues-ledger.md` (DI-1…DI-44) is
  archived as history; its open items were migrated as issues, its closed
  items keep their original text there.

### Implementation status

* `docs/project/status.md` has one row per area with a state — `implemented` ·
  `partial` · `planned` · `blocked` — one line of *what exists*, one line
  of *what does not*, and evidence links. "Implemented" means present at
  HEAD with a test that exercises it, nothing more.
* Updated when work lands or is removed. When a roadmap item completes,
  its row moves out of the roadmap and into `docs/project/status.md` (capability) and,
  when observable, a change fragment (what changed).

### Change record

* **Created** for an observable change: syntax, semantics, project
  format, protocol, tooling behaviour, a removal, a new capability people
  will use, a migration someone must perform. A refactor is not a change
  record; a fix is one only if behaviour people relied on moved.
* **Fields**: `date`, `area`, `affected` (designers · project authors ·
  protocol clients · developers), `related` (ADR/issue/proposal), then
  *What changed*, *Compatibility and migration*, *Evidence*.
* Lands as one file in `docs/changes/unreleased/` beside the code. There
  is no release cadence yet; when one exists, a release collects the
  fragments into `docs/changes/<version>.md` and empties the folder. Until
  then `unreleased/` **is** the change history since the last snapshot.
* `docs/changes/history/` holds the pre-existing history that used to
  live inside other documents: completed milestones, protocol versions,
  the formula-editing bug record.

### Roadmap

* Ordered priorities and the dependency that gates each; no status
  columns, no checkmarks. A completed row is deleted (status and change
  records take over); a dropped row is deleted with a change fragment if
  anyone was promised it.

## Metadata

Decisions, proposals and issues carry YAML frontmatter with scalar values
and inline lists (`[a, b]`) — the subset `scripts/validate_docs.py` parses
without a YAML library. Every other page carries a three-line header —
`kind`, `area`, `status` (`current` | `archived`) — and lives in the
folder of its kind: `spec/`, `architecture/`, `evidence/`, `guides/`,
`background/`, `archive/`, `project/`. The validator checks that the kind
matches the folder, that no page sits loose at the top of `docs/`, and
that the front door registers every page. File names are lowercase
kebab-case.

A page is **removed** when everything it says is either false or fully
carried by another record (the pre-implementation checklist of 2026-09-15
was removed once the migration report mapped each of its questions to its
answer); it is **archived** when code or commits cite it (the DI ledger).
Git keeps both.

`area` is one of: `language`, `textual`, `compiler`, `runtime`, `codegen`,
`persistence`, `protocol`, `daemon`, `ide`, `studio`, `behavior-systems`,
`deployment`, `formal`, `process`. It names the subsystem, not a person.

### Claim strength

Records say how strong a claim is, with these words and no others:

| Word | Means |
|---|---|
| *intended* | a goal in the roadmap |
| *designed* | a proposal or an architecture page describes it; nothing runs |
| *accepted* | an ADR chose it |
| *implemented* | present at HEAD |
| *tested* | a named test exercises it |
| *formally proved* | a named BDL_FV theorem proves the stated property — of the *model*, never of Rust or Dart |
| *released* | not used yet: BDL has no releases |

Formal links use one of four labels in an ADR's `fv` field or a
`docs/project/formal-correspondence.md` row: `formally proved` (the theorem proves
exactly the property stated), `informed by FV` (a formal result shaped
the choice), `production-tested` (executable tests support the claim),
`engineering choice` (no formal backing is claimed). A commit does not
turn *accepted* into *proved*; a theorem about the kernel does not prove
Studio behaviour.

## What every change to the records must do

1. Route the fact (table above) and touch each authority once.
2. If a decision was replaced, supersede — never rewrite.
3. Sweep for stale claims in the authorities you did not touch:
   `grep -rn -E "planned|pending|not built|not yet|incomplete|TODO|deferred" docs/spec docs/architecture docs/project`
   plus the subsystem name, the protocol/schema version, and any
   milestone or record ID involved. Fix only what is stale; do not add
   the same sentence to five files.
4. Run `just docs-check` (`scripts/validate_docs.py`: IDs, statuses,
   frontmatter, index coverage, supersession in both directions, resolved
   issues without a resolution, relative links across `docs/` and the
   READMEs) and the tests that back any capability claim you changed.
5. Update [docs/README.md](../README.md)'s *Current snapshot* if what a
   newcomer should know first has changed.

## Ownership

Areas route to code, not to people: each area's authoritative pages are
listed in [docs/README.md](../README.md), and the crates that implement it
are named in `docs/architecture/overview.md`. If maintainers multiply, an
`area → maintainer` table belongs here (the Linux `MAINTAINERS` idea at
BDL's scale), not in every record.
