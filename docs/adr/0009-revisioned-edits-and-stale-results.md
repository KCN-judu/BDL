# ADR-0009: Revisioned edits; stale analysis results are discarded

**Status**: accepted (2026-09-15)

## Context
IDE race conditions come from asynchronous analysis landing on a project
that has moved on, and from edits applied to a stale view.

## Decision
`apply_edit(snapshot @ N, op) -> snapshot @ N+1` is pure. Every
`ApplyEdit` carries the client's base revision and is refused if stale.
Every response/event carries its revision; clients discard older ones.
Revisions are strictly monotone within a session — undo/redo produce new
revisions with earlier designs — so staleness is `<`.

## Consequences
* One canonical revision stream per project; analyses run on immutable
  snapshots and may be parallel.
* Undo/redo are semantic operations on revisions, not widget reversals.
