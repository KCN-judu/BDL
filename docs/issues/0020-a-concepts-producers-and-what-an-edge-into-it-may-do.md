---
id: ISS-0020
state: open
area: formal
opened: 2026-09-21
resolved-by: []
related: [ADR-0034, ADR-0003, ISS-0002]
---

# ISS-0020: Whether a concept has one producer or several is under formal audit — until it concludes, no edge into a concept gets a destructive action

## Problem

The canvas draws a relationship's produce edge (relationship → the concept its
signature outputs) and, for a concept produced by more than one relationship,
draws every one of them (`lit → Lit` beside `pressed → Pressed`; two values
producing `Brightness` are two edges into the same row). What the model says
about that picture is the subject of a BDL_FV audit: whether a concept's
producers are one or several, what the sequencing of producers means, and
therefore what _taking one produce edge away_ would mean. Until the audit
concludes, Studio must not offer, imply or reshape any of it.

## Why it matters

The canvas now treats an edge as an object (a selection, an inspector, a
contextual affordance with a × where the model has a disconnect): a × on a
produce edge would promise an edit the model may not have, and a redrawn
projection (`lit → Lit → lamp` for `lit → Lit, lit → lamp`; a producer marker; a
hidden branch) would prejudge the audit in pictures.

## Current evidence

Frozen while this issue is open (`docs/architecture/studio-ui.md` §2, _Edges as
objects_):

- the Concept ↔ Mapping topology and the Mapping → Output drive projection as
  drawn today (`buildScene`, ADR-0034);
- branching: every producer's edge is drawn, none is hidden or merged;
- no producer-selection visualisation, no concept sequencing, no producer count
  on a concept row;
- `LinkId.disconnectable` (`apps/studio/lib/app/state.dart`) is true for a
  relationship's read (`UnlinkMappingInput`) and a sink's driver
  (`SetMappingDrive` to none) only; a produce edge and a collapsed group's
  aggregate edge get no × and no _Disconnect_ item, and
  `DisconnectLinkRequested` refuses them
  (`apps/studio/test/canvas_layout_reducer_test.dart`).

## Dependencies

The BDL_FV audit of producer uniqueness; ISS-0002 (several candidate
definitions, one active) touches the same question from the definition's side.

## Resolution

Open. When the audit concludes: an ADR records what a produce edge is and what
may be done to it; the freeze in studio-ui §2 is lifted or made permanent by
that record.
