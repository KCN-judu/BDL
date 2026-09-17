# runtime

Crates linked into, or driving, generated programs — never into the
compiler.

* `bdl-runtime-core` — `no_std`, allocation-free, `unsafe`-free: the
  vocabulary every generated semantic core is written against
  (`ActiveDomains`, `ClockSlot`, `RuntimeError`, checked numerics, strict
  primitive helpers). Knows no board, device kind, transport or editor.
* `bdl-runtime-host` — `std`: `DynValue`, JSON run requests and traces
  over stdio (`main_stdio`), and a cargo driver (`harness`) that writes,
  checks, builds and runs a generated crate. Used by the differential
  tests and by tooling.

Planned: `bdl-runtime-embassy`, the first platform adapter (docs/project/roadmap.md
P–S). See docs/spec/runtime-semantics.md and docs/architecture/codegen-rust.md.
