# ADR-0018: Semantics are shown as structure, explained in prose, named formally only on demand

**Status**: accepted (2026-09-15)

## Context
The compiler now returns analyses (status, diagnostics, elaborated core
terms) and the temptation is to print them: status pills on every row,
kernel notation in the inspector, ids beside names. That turns Studio into a
control panel for the compiler. The paper's premise is the opposite — a
designer works in product meaning, and incompleteness is a normal state.

## Decision
Every semantic fact is designed for exactly one primary level
(`docs/STUDIO_UI.md` §7):

1. **Canvas** — structure and object state: silhouette, socket shape, socket
   hue, links, containment, dashed/solid, a red mark where something is wrong.
2. **Inspector** — designer vocabulary about the selection (*Meaning, Value,
   Measures, Reads, Produces, Relationship, Used by, Produced by*) and the
   consequence of the next change; diagnostics attached to the field they
   concern, in product language.
3. **Explain** — one collapsed disclosure at the end of the inspector, plus
   the technical part of a diagnostic: `SemanticId`, `DeclId`, `Ty`, `Grant`,
   `Interface`, invalidation categories, Core IR, codes, revision.

Formal vocabulary never appears at levels 1–2. A state becomes a badge only
when no property of the object can carry it. Incompleteness (*declared*,
*Decide later*) is never red.

## Consequences
* `MappingStatus` and `EditOutcome.kind` are rendered as object state and
  designer sentences; the enum words live in Explain.
* The canvas draws representation as socket shape and drops the type
  subtitles; the concept node becomes a single-row object.
* Where the UI needs a fact the read model does not carry (dependents of an
  edit by name), the requirement is documented for the protocol; Studio
  computes nothing semantic (ADR-0001).
