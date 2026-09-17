---
id: ADR-0003
status: accepted
date: 2026-09-15
area: persistence
supersedes: []
superseded-by: []
related: []
fv: []
---
# ADR-0003: Project layout is stored apart from semantics

**Status**: accepted (2026-09-15)

## Context
Node editors habitually store x/y next to node semantics; moving a node
then dirties the design, and a headless tool must parse layout it does not
need.

## Decision
`design/project.bdl.json` holds semantics; `ui/layout.json` holds canvas
positions keyed by stable id. Layout updates go through `SetLayout` and
create no project revision. A design opens with no layout file at all.

## Consequences
* Moving nodes never triggers analysis.
* Semantic undo and layout undo are separate mechanisms (documented in
  docs/architecture/overview.md).
* Renaming an entity changes neither file's keys.
