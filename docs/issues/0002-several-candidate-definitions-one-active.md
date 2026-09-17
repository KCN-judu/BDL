---
id: ISS-0002
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-4"]
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
  reopens realization (`docs/architecture/compiler-pipeline.md`); drafts are overlays
  (`docs/architecture/ide-service.md`).
* The original entry proposed: the surface stores candidates, the kernel sees
  the active one, switching is an ordinary edit. DI-4.

## Dependencies

* A persistence and protocol shape for candidates (`docs/spec/project-format.md`,
  `docs/spec/protocol.md`), and a rule for identity of a candidate under rename.

## Resolution

Open.
