---
id: ISS-0005
state: open
area: language
opened: 2026-09-15
resolved-by: []
related: ["docs/archive/design-issues-ledger.md#di-19"]
---
# ISS-0005: User-defined enums

## Problem

`enum` items and constructor patterns parse (`docs/spec/textual-syntax.md`) and
every form except a user enum desugars into the existing Core; a user enum
has no surface-model type and no kernel sum type.

## Why it matters

A lamp *mode* (off / automatic / manual) is the first thing a designer
reaches for; today it is a `Count` or several booleans.

## Current evidence

* Production: `bdl-elab` reports `formula.constructor.unknown` for
  constructors other than `Some`/`None`; the text loader reports `enum`
  items as `text.unsupported_item` (open); Studio has no enum object.
  DI-19 (narrowed 2026-09-15).
* Formal: no sum type in the kernel; a kernel extension is Lean-first
  (ADR-0010).

## Dependencies

* A kernel sum type in BDL_FV, then a production mirror; or a decision that
  enums stay outside executable BDL (a surface convenience over `Count`).

## Resolution

Open.
