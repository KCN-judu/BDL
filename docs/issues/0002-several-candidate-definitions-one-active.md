---
id: ISS-0002
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/DESIGN_ISSUES.md#di-4"]
---
# ISS-0002: Several candidate definitions, one active

## Problem

The paper leaves open whether a relationship may hold several candidate
definitions of which one is active. The kernel realizes a declaration once;
the surface model has one `Definition` per mapping.

## Why it matters

Designers iterate on a formula; without candidates every experiment is a
replace-and-undo. Studio's *drafts* cover the unsaved case only.

## Current evidence

* Production: `MappingBlock.definition: Option<Definition>`
  (`crates/bdl-model/src/surface.rs`); `ReplaceDefinition` is an edit that
  reopens realization (`docs/COMPILER_PIPELINE.md`); drafts are overlays
  (`docs/IDE_SERVICE_ARCHITECTURE.md`).
* The original entry proposed: the surface stores candidates, the kernel sees
  the active one, switching is an ordinary edit. DI-4.

## Dependencies

* A persistence and protocol shape for candidates (`docs/PROJECT_FORMAT.md`,
  `docs/PROTOCOL.md`), and a rule for identity of a candidate under rename.

## Resolution

Open.
