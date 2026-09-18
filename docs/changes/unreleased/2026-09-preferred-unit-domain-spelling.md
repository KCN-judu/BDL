# `() -> A` is the preferred spelling; the output-only shorthand is deprecated

- Date: 2026-09-18
- Area: language, textual, ide, daemon
- Affected: designers, developers
- Related: ADR-0029 (amendment)

## What changed

- **Preferred spelling.** A relationship without inputs is written
  `mapping f : () -> A`. The historical `mapping f : A` still opens and means
  the same declaration (the one canonical type `() -> A`), as compatibility
  syntax only.
- **Nothing generates the shorthand any more.** The Code view, source
  write-back and legacy-project migration (`bdl-text`), the IDE's rendered
  projection and hover, the Smart Lamp example and the documentation write
  `() -> A` from the start.
- **A hint, never an error.** The IDE reports the shorthand as
  `text.legacy_unit_domain` — _A relationship with no inputs is written
  explicitly as `() -> RoomTemp`. The output-only shorthand is deprecated._ —
  at the signature of a `mapping` only, with the quick fix **Make empty domain
  explicit** (one insertion of `() -> `). Hover and Explain show
  `declared spelling: RoomTemp` beside `type: () -> RoomTemp` and say that the
  omitted domain is the empty product.
- **The formatter is not a migration**: it keeps whichever spelling was
  authored. `bdld migrate-unit-domain <project> [--dry-run] [--json]` is the
  opt-in rewrite: lossless outside the inserted text, comments and trivia
  preserved, identities and the design verified unchanged before anything is
  written.
- **References stay `f`.** The signature exposes the empty domain; the
  observation of the value erases the unique argument, so `f` remains the
  ordinary spelling in formulas (`f()` and `f(())` stay accepted, not
  preferred).

## Compatibility and migration

- Designers: existing projects are not rewritten; a hint appears in the
  editor on each legacy signature, with a fix. Run `bdld migrate-unit-domain`
  on a project to rewrite it all at once, or leave it: stage 1 of the policy
  in ADR-0029's amendment changes nothing else.
- Project files: nothing. Protocol: nothing (0.14 already carries the unit
  kind; a spelling is source syntax, not protocol state).
- Developers: `SemanticSeverity::Hint` (LSP `Hint` + `Deprecated`; daemon
  `Info`); `bdl_syntax::migrate`; `bdl_text::make_unit_domains_explicit`;
  `bdl_text::print` writes `() -> A` for mappings (ports unchanged);
  `hover::declared_spelling`. Tests that asserted rendered shorthand were
  updated.
- Not yet: the Studio Code view (fed by `bdl-text` load faults) shows no hint;
  port types (`requires`/`param`/`provides`) keep the bare output.

## Evidence

`crates/bdl-syntax/src/migrate.rs`, `crates/bdl-ide/tests/acceptance.rs`
(`the_legacy_output_only_shorthand_is_a_hint_with_a_quick_fix_and_the_explicit_form_is_clean`),
`crates/bdl-daemon/tests/cli.rs`
(`migrate_unit_domain_rewrites_only_the_legacy_signatures_and_keeps_identities`),
`crates/bdl-daemon/tests/text_e2e.rs`, `crates/bdl-daemon/tests/stdio_e2e.rs`.
