# Change records

What changed for someone outside the commit: designers, project authors,
protocol clients, developers building on the crates. A record says what
changed, who is affected, how to migrate, and where the evidence is. A
refactor with no observable effect has no record; Git has it.

* [`unreleased/`](unreleased/) — one file per observable change since the
  last snapshot, landing with the code. BDL has no release cadence; until
  it does, this folder *is* the change history. When a release happens it
  collects these into `<version>.md` and empties the folder.
* [`history/`](history/) — the history that used to live inside other
  documents and was moved here so those documents could state current
  truth only:
  * [milestones.md](history/milestones.md) — the completed milestone
    ledger that was the old roadmap (steps A–O, BS, ST-1…ST-9, IDE-1,
    LIB-1) with the evidence each cited;
  * [protocol-versions.md](history/protocol-versions.md) — what each
    protocol minor version added;
  * [formula-editing-milestone.md](history/formula-editing-milestone.md)
    — the bugs found and fixed while the definition editor was built, and
    the milestone's non-goals.

Start a new record from [TEMPLATE.md](TEMPLATE.md). Name it
`YYYY-MM-slug.md`. Cite the ADR or issue behind it; do not restate the
rationale.
