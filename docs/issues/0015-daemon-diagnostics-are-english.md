---
id: ISS-0015
state: open
area: daemon
opened: 2026-09-19
resolved-by: []
related: [ADR-0031, ISS-0008]
---

# ISS-0015: The compiler's diagnostic sentences are English in every locale

## Problem

A diagnostic reaches Studio as `code`, `message` (English prose composed by
the daemon, usually naming entities: _Output light has no driver_), optional
`technical` text and a span. Studio can localize the sentence only for codes
whose meaning the code alone determines (`studio.*`, `project.changed_on_disk`,
`edit.stale_revision`, `simulation.not_started`, …); for the rest the English
sentence is shown with the code beside it (ADR-0031 §2). In zh-Hans and ja the
Inspector's findings, the Code view's reasons and the simulator's failures are
therefore mostly English.

## Why it matters

Findings are the product's main explanation surface; a designer who chose
Japanese reads them in English. The identity (the code) is right; the
presentation is not.

## Current evidence

`apps/studio/lib/l10n/diagnostics.dart` — `localizedMessage` and its switch;
`crates/bdl-daemon/src/server.rs` — `error(code, text)` composes the text;
`crates/bdl-model` — `Diagnostic { code, message, technical, span, fixes }`.
`apps/studio/test/l10n_test.dart` (`a diagnostic code is identity; only its
sentence changes`) pins the fallback. The simulator already renders four
`simulation.*` codes from structured fields (`mapping_id`), which is the shape
the rest needs.

## Dependencies

A diagnostic with **structured arguments** (entity ids, names, types, units)
beside its code — the same need ISS-0008 records for outputs — so a client can
word every sentence itself from the code and the arguments. That is a protocol
change (`Diagnostic` message), then a Studio catalog entry per code.

## Resolution

Open.
