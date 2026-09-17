# Engineering records: one front door, typed records, validation

- Date: 2026-09-17
- Area: process
- Affected: developers, coding agents
- Related: none (a process change; `docs/project/governance.md` is the rule)

## What changed

* `docs/README.md` is the front door for engineering documentation; the
  user guide stays a separate product.
* Record kinds with lifecycles and IDs: decisions (`docs/decisions/`, ADR-NNNN,
  now with frontmatter), proposals (`docs/proposals/`, PRP-NNNN), design
  issues (`docs/issues/`, ISS-NNNN), implementation status
  (`docs/project/status.md`), change fragments (`docs/changes/`),
  formal correspondence (`docs/project/formal-correspondence.md`).
* Two ADRs that shared a number with an earlier record were renumbered:
  *behaviour systems flatten into the flat design* is ADR-0021 (was
  0017), *a component interface is a stored promise* is ADR-0022 (was
  0018). Every other ID is unchanged.
* History moved out of current pages: the completed-milestone ledger
  (from `ROADMAP.md`), the protocol version history (from `PROTOCOL.md`)
  and the formula-editing bug record (from
  `STUDIO_COMPILER_INTEGRATION.md`) are under `docs/changes/history/`.
  `DESIGN_ISSUES.md` and `03-open-questions.md` are archived in place.
* Every engineering page now lives in the folder of its kind (`spec/`,
  `architecture/`, `decisions/`, `proposals/`, `issues/`, `project/`,
  `changes/`, `evidence/`, `guides/`, `background/`, `archive/`) with a
  lowercase kebab-case name and a `kind / area / status` header; the
  path map is in `docs/project/migration-report.md`. The pre-implementation
  checklist (`03-open-questions.md`) was removed.
* `just docs-check` (`scripts/validate_docs.py`) runs in `just check` and
  CI.

## Compatibility and migration

* Links to `docs/decisions/0017-behaviour-systems-…` and
  `docs/decisions/0018-a-component-interface-…` must point at the `0021-` and
  `0022-` files; prose saying "ADR-0017" about behaviour systems or
  "ADR-0018" about component interfaces means ADR-0021 / ADR-0022. Commit
  messages are not rewritten.
* Every `docs/<NAME>.md` link is now `docs/<kind>/<name>.md`; the map is
  in the migration report. Do not add rows to
  `docs/archive/design-issues-ledger.md`; open a file in `docs/issues/`.

## Evidence

`python3 scripts/validate_docs.py`; `python3 -m unittest
scripts/test_validate_docs.py`; `docs/project/migration-report.md`.
