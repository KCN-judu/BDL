---
kind: project
area: process
status: current
---
# Engineering-record research

How four mature projects keep their engineering history, read from their
official sources on 2026-09-17, organised by the problem BDL has to solve
rather than by project. Each row says what the practice is, why it works
there, what BDL takes, and what BDL deliberately does not copy. The last
section compares the chosen BDL model with each precedent.

Projects studied: the **Linux kernel** (`Documentation/process`, `ABI`,
`deprecated`), **GHC** (proposals, user's guide, release notes), **Rust**
(RFCs, Reference, Edition Guide, release notes) and **Python** (PEP 1,
PEP 387) as the fourth language project — chosen over LLVM/Swift because
its status vocabulary and its explicit *Accepted ≠ Final* rule are the
closest fit for a language with a separate formal development.

## 1. Decision lifecycle: proposal versus accepted decision

| Source | Pattern | Why it works | BDL takes | BDL does not copy |
|---|---|---|---|---|
| [GHC proposals README](https://github.com/ghc-proposals/ghc-proposals/blob/master/README.rst) | Stages with labels: discussion → *Pending shepherd recommendation* → *Pending committee review* → *Needs revision* → **Accepted** → **Implemented** (label set "once it hits GHC's master branch"). Significant changes to an accepted proposal are an *Amendment* that goes through the same review. | "Accepted" and "Implemented" are visibly different states on the same record; an amendment is a first-class event, not an edit. | Acceptance and implementation are separate states in separate records (ADR vs `docs/project/status.md`); an *Amendment* section may be appended to an ADR and is dated. | Shepherds, a committee, review-deadline labels. |
| [Rust RFC README](https://github.com/rust-lang/rfcs/blob/master/README.md) | An RFC is required for "any semantic or syntactic change to the language that is not a bugfix" and for removing features; not for refactors or small additions. Lifecycle: PR → discussion → *final comment period* (disposition merge/close/postpone) → merged = *active*. "Active" explicitly "does not guarantee implementation"; "every accepted RFC has an associated issue tracking its implementation". Substantial later changes "warrant new RFCs". | The threshold for a formal record is stated, so ordinary work is not blocked; the tracking issue keeps the RFC text from becoming a task list. | The threshold rule (consequential choices only); a proposal stage before an ADR; a replacement decision instead of editing the old one. | The FCP ritual and the ten-day clock; RFC numbers taken from PR numbers (BDL uses sequential IDs — the repository, not a PR queue, is the ledger). |
| [PEP 1](https://peps.python.org/pep-0001/) | Statuses: *Draft, Active, Accepted, Provisional, Deferred, Rejected, Withdrawn, Final, Superseded*; headers *Replaces* / *Superseded-By* / *Requires* / *Resolution*. "The reference implementation must be completed before any PEP is given status Final." Rejected and withdrawn PEPs stay published. | A typed header makes the outcome machine-readable and negative results discoverable. | Frontmatter with `status`, `supersedes`, `superseded-by`; rejected/withdrawn proposals are kept; a *deferred* state for issues (a decision not to decide now). | *Provisional* and *Final* as ADR states — BDL's ADR records the choice; whether it is built is `docs/project/status.md`'s fact, not a second status on the decision. Delegates and a steering council. |

**Chosen for BDL:** one decision system. A problem is an *issue*; a
concrete alternative under discussion is a *proposal*; an accepted
proposal becomes an *ADR* (the proposal freezes and links it). Rejected
proposals remain as history. ADRs are never *proposed*: that is what
proposals are for.

## 2. Normative documentation: current truth

| Source | Pattern | Why it works | BDL takes | BDL does not copy |
|---|---|---|---|---|
| [Rust Reference](https://doc.rust-lang.org/reference/introduction.html) | "The primary reference for the Rust programming language", covering stable Rust only; unstable features are in a separate book; "the main text describes the latest stable edition. Differences to previous editions are separated in edition blocks". It says what it is not (not a tutorial, not the std docs, not the compiler's behaviour). | Readers do not reconstruct current semantics from RFC history; scope statements keep the document from absorbing everything. | Specification pages state current truth and only that; each says what it is not; historical differences live in change records, not inline narrative. | Edition blocks — BDL has no compatibility epochs yet. |
| [GHC user's guide extension pages](https://downloads.haskell.org/ghc/latest/docs/users_guide/exts/lambda_case.html) | Every extension carries a metadata block — *Since: 7.6.1*, *Status: Included in GHC2024* — and the prose cites the proposal (#302) that introduced a form. | Current behaviour, the version it arrived in and the rationale link sit together without the page becoming history. | Specification sections cite the ADR that chose a rule; the pre-existing "(v0.1 core, v0.2 project items)" section labels in `docs/spec/textual-syntax.md` are kept as the equivalent of *Since*. | Per-feature version fields everywhere — BDL is unreleased; the change fragments carry dates. |
| [Linux in-tree documentation](https://www.kernel.org/doc/html/latest/) | Documentation lives with the source and changes in the same patch as the code; `submitting-patches` asks that interface changes reach the man-pages maintainer. | Truth and code travel through one review and one history. | Engineering docs stay in this repository beside the crates; `just check` runs the record validator with the tests. | Mailing-list workflow. |

## 3. Superseding old decisions

GHC handles change to an accepted proposal as an *Amendment* that is
reviewed like a proposal; Python gives a PEP *Superseded* status and a
*Superseded-By* header while the old text stands; Rust leaves merged RFCs
untouched and expects "substantial changes" to arrive as new RFCs.

**BDL takes:** bidirectional `supersedes` / `superseded-by`, validated so
neither side can be missed; the old record keeps its context,
alternatives and rationale verbatim; the replacement states whether the
old choice was wrong on the same evidence or overtaken by new
circumstances; current pages cite only the replacement. Small extensions
that do not replace the choice are appended as a dated *Amendment*
(ADR-0019 already has one). **Not copied:** re-review of amendments by a
committee.

## 4. Implementation tracking

Rust's tracking issues and GHC's *Implemented* label both say the same
thing: the record of a decision is not the place to track the work. The
old BDL documents did the opposite — `docs/project/roadmap.md` carried ✅ marks and
ADR bodies carried phase checklists.

**BDL takes:** `docs/project/status.md` is the only place that says what exists, per
area, with evidence links; the roadmap holds only what is next; ADRs hold
no checklists. **Not copied:** one tracking issue per accepted design —
BDL's areas are few enough for one matrix.

## 5. Open questions

Rust and GHC start a significant change with discussion; Python lets a
PEP sit in *Draft* or *Deferred*. None of them has a record for "we know
this is a problem and have no answer", which is exactly what BDL's
`docs/archive/design-issues-ledger.md` had been used for — mixed with decided items and with
duplicate IDs.

**BDL takes:** a design-issue registry with unique IDs and three states
(`open`, `deferred`, `resolved`), a `resolved-by` link the validator
checks, and a rule that an issue is never a permission to implement its
tentative answer. Python's *Deferred* supplies the third state.

## 6. Release-visible changes and migration

| Source | Pattern | Why it works | BDL takes | BDL does not copy |
|---|---|---|---|---|
| [GHC 9.14.1 release notes](https://downloads.haskell.org/ghc/latest/docs/users_guide/9.14.1-notes.html) | Sections *Language / Compiler / GHCi / Runtime system / Libraries*; each language entry cites its proposal ("GHC Proposal 493"); deprecations are inline with the removal version ("scheduled to be removed in GHC 9.18"). | A user reads what changed, why, and what to do, without the proposal thread. | Change fragments cite their ADR/issue and say what to migrate; fragments group by area. | Per-release shells while there is no release. |
| [Rust release notes](https://doc.rust-lang.org/nightly/releases.html) | Observable changes grouped by release; internal work is not promoted. | The notes are not a copy of the commit log. | Only observable changes get a fragment; refactors do not. | — |
| [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/editions/index.html) | Incompatible changes are opt-in per crate; "crates in one edition must seamlessly interoperate with those compiled with other editions"; `cargo fix` migrates. | Compatibility is a promise with tooling behind it. | The *Compatibility and migration* section is mandatory in a fragment; project-format changes must state the migration (`persist::check_schema`). | Editions. |
| [PEP 387](https://peps.python.org/pep-0387/) | Backward-incompatible = "preexisting code ceases to comparatively function"; deprecation warning for at least two releases (two years) before removal; *soft deprecation* discourages without removing. | The compatibility promise is explicit and proportional. | The definition of "observable"; the idea that a removal is announced before it happens (in a fragment, then in the spec). | The two-year clock. |
| [Linux `Documentation/ABI`](https://www.kernel.org/doc/html/latest/admin-guide/abi.html) and [`process/deprecated`](https://www.kernel.org/doc/html/latest/process/deprecated.html) | Interfaces are filed as *stable / testing / obsolete / removed*; removed interfaces keep their documentation; deprecated practices are a living list of *what · why · replacement*. | Compatibility state is a property of the interface, recorded where it is defined, and history is never deleted. | `docs/spec/protocol.md` and `docs/spec/project-format.md` state compatibility rules in the spec; protocol version history moved to `changes/history/protocol-versions.md` and stays complete; a removed message or field is recorded, not erased. | Four stability classes — BDL's protocol has one: minor versions are additive within a major. |

**Chosen for BDL:** fragments in `docs/changes/unreleased/` (one file per
observable change, landing with the code), no `CHANGELOG.md` and no
release notes until a release cadence exists. Fragments are the model
Rust and GHC use internally (`relnotes` labels, `docs/users_guide/*-notes`)
without a release to assemble them into yet.

## 7. Evidence and claim strength

Linux `submitting-patches` asks a change to describe the problem, the
user-visible impact, the trade-offs, and to "include numbers that back
them up"; Rust and GHC keep normative text apart from tests and tracking.

**BDL takes:** every implemented claim in `docs/project/status.md` links a test; every
formal claim links a BDL_FV module or theorem and carries one of four
strength labels (`formally proved`, `informed by FV`, `production-tested`,
`engineering choice`); `docs/project/formal-correspondence.md` is the one table of what
the formal development does and does not cover. **Not copied:** pasting
evidence (traces, numbers) into long-lived records — links only.

## 8. Ownership

The Linux [maintainer handbooks](https://www.kernel.org/doc/html/latest/process/maintainer-handbooks.html)
and `MAINTAINERS` file map paths to people and let a subsystem add its own
expectations. **BDL takes** an `area` field on every record and a
per-area list of authoritative pages in the front door; **does not copy**
named maintainers or per-subsystem handbooks until there are enough
contributors to need them.

## 9. Historical retention and commit history

All four keep every proposal ever filed, including rejected ones; Linux
keeps removed ABI. Git already holds diffs, authors and dates, and the
kernel's *Fixes:* / *Link:* tags show the right use of commit references:
a pointer from a record to a commit, never the commit text copied into
the record.

**BDL takes:** decisions and issues are never deleted; superseded
records stay in the index; change records link commits where a reader
would otherwise search; architecture pages carry no chronology.

## 10. Indexing and discovery

Rust (`text/` + README), GHC (proposals index by status) and Python (PEP
index generated from headers) all generate or maintain a single index
keyed by ID and status. **BDL takes:** one front door (`docs/README.md`)
routed by question, one index per record kind, machine-readable
frontmatter, and a validator that fails when an ID is missing from its
index. **Not copied:** a generated website.

## The BDL model against each precedent

| Choice | Linux | GHC | Rust | Python |
|---|---|---|---|---|
| Records live in-tree beside code, checked by `just check` | ✔ the source of this practice | — | — | — |
| Threshold: only consequential choices get a record | patch description for every change | proposal for language changes | "any semantic or syntactic change that is not a bugfix" — adopted as the threshold | PEP for "new feature or implementation" |
| Issue → proposal → ADR, one decision system | — | discussion → accepted, amendments | RFC → merged, new RFC for substantial change | Draft → Accepted; *Deferred* adopted for issues |
| Accepted ≠ implemented: `docs/project/status.md`, not the ADR, says what exists | — | *Implemented* label | tracking issues — adopted as one matrix | Accepted vs Final — adopted as a rule, not as an ADR state |
| Supersession in both directions, old text intact | removed ABI stays documented | amendment | new RFC | *Superseded-By* — adopted |
| Change fragments, no release notes yet | — | release notes per version cite proposals — adopted as the fragment shape | release notes + Edition Guide — adopted: mandatory migration section | PEP 387 — adopted: definition of observable change |
| Evidence linked, strength labelled | "include numbers that back them up" | — | — | — |
| No committees, shepherds or FCP clocks | — | not copied | not copied | not copied |
