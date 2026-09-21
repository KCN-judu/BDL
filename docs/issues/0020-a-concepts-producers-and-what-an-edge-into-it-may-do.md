---
id: ISS-0020
state: resolved
area: formal
opened: 2026-09-21
resolved-by: [ADR-0044]
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

Resolved by ADR-0044 (2026-09-21). The audit (BDL_FV Phase 21, FVD-0163)
withdrew the question rather than answering it either way: the value is the
declaration's, not the concept's — a Sem block has one producer by construction
(`producedBy_unique`), several Sem blocks of one concept are ordinary, and a
formula names Sem blocks by `declRef`. The canvas no longer draws a concept
node, so there is no edge into a concept to act on; the produce edge is the
attachment of a mapping block to its Sem block; a read edge is a name in a
formula and gets no destructive action; no diagnostic counts a concept's blocks
(Phase 20's `ProducerUnique` stays an optional judgment of the formal
development, unbuilt here).
