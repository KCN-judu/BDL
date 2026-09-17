---
id: ISS-0009
state: open
area: protocol
opened: 2026-09-15
resolved-by: []
related: []
---
# ISS-0009: Projection deltas and a persisted edit history

## Problem

`ProjectChanged` sends the full projection after every commit, and the
revision/undo history lives only in the session: closing the project
forgets it.

## Why it matters

Large projects will pay for full projections on every keystroke-level
edit, and a persisted edit log is what an audit trail, a collaborative
session and a *reopen where I was* undo need.

## Current evidence

* Production: `docs/PROTOCOL.md` *Deltas* section (full projections),
  `crates/bdl-daemon/src/session.rs` (`undo`/`redo` vectors);
  ADR-0009 (revisioned edits). Recorded in the old roadmap as
  *known v0.1 simplifications*.

## Dependencies

* A delta encoding in the protocol (a minor bump) and a log file format in
  `docs/PROJECT_FORMAT.md`; both are additive.

## Resolution

Open.
