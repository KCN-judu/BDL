# ADR-0002: bdld is a separate process

**Status**: accepted (2026-09-15)

## Context
The compiler could be linked into Studio via FFI. That couples compiler
crashes to the IDE process, makes headless use awkward, and invites Dart
code to reach into Rust state.

## Decision
The semantic engine runs as `bdld`, a child process of Studio, speaking a
typed protocol. `bdld` owns the opened project state, serializes edits
through one coordinator, runs analyses on immutable snapshots, and will
orchestrate build/flash/telemetry.

## Consequences
* Compiler crashes cannot corrupt the IDE process; a crash is a
  `ConnectionFailed` state with a reconnect path.
* CLI/CI/other editors reuse the same binary.
* All state crossing the boundary is explicit protocol data.
