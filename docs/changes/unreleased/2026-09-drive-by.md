# Output driving is spelled `drive light by brightness`

- Date: 2026-09-20
- Area: textual, ide, daemon, docs
- Affected: designers, project files, developers
- Related: ADR-0014, ADR-0029 (the staged policy for a legacy spelling),
  [2026-09 preferred unit-domain spelling](2026-09-preferred-unit-domain-spelling.md)

## What changed

- **The preferred surface spelling of a drive is `drive <output> by <driver>`:**
  `drive light by brightness` says that `brightness` drives the logical output
  `light` — the output is driven by the relationship. It is the one spelling
  every generator, renderer, example, fixture and page writes.
- **Semantics are unchanged.** Both spellings parse to one `DriveDecl`, lower to
  one `DriveItem`, build one `MappingBlock.drives`; `DriveWF`, `SingleDriver`,
  the Design IR, lowering, simulation, realization and code generation are byte
  for byte what they were (tested:
  `the_two_drive_spellings_build_the_same_system_across_files`,
  `both_spellings_are_one_drive_edge_and_only_the_legacy_one_is_hinted`).
- **`by` is contextual**, not reserved: an identifier everywhere but between an
  output and its driver; a relationship or an output may be named `by`
  (`drive light by by` is well formed), and `by` is classified as a keyword only
  in that position (semantic tokens; the VS Code grammar lists it with the other
  contextual words).
- **The legacy `drive light = brightness` stays accepted** as compatibility
  syntax. The IDE reports it as a hint (`text.legacy_drive`: _Prefer `by` for
  output driving: `drive light by brightness` says that `brightness` drives
  `light`._) with the quick fix _Write `by`_ — the `=` token replaced, nothing
  else moved; `bdld migrate-drive-by <project> [--dry-run] [--json]` rewrites a
  whole project losslessly with the same guards as `migrate-unit-domain`. The
  formatter keeps the authored spelling (formatting is never a migration, as for
  the unit-domain shorthand); formatting either spelling is idempotent.
- **Malformed drives get surface-language findings:** `drive light by` →
  _expected the driving relationship's name after `by`_; `drive light level` →
  _expected `by` between the output and the relationship that drives it_ with
  the hint _`drive light by brightness` says that …_, and the stray name is kept
  as the driver so one fault is reported.
- **Completion** after `drive light` offers `by`; after `by`, the relationships
  that can drive; `=` is not offered.

## Compatibility and migration

- Designers: nothing to do; existing files open and mean what they meant. The
  Code view shows the hint on a legacy drive; take the fix, or run the migration
  once.
- Project files: `drive o = m` still loads; written-back and generated drives
  are `drive o by m`. Opening a file never rewrites it.
- Developers: `ast::DriveDecl::legacy_eq` (syntax provenance only),
  `bdl_syntax::migrate::{legacy_drive_decls, drive_by_edits, make_drives_by}`,
  `bdl_text::make_drives_by`, `bdld migrate-drive-by`, `text.legacy_drive`, the
  action id `text.drive_by:<doc>:<offset>`; the semantic token classifier now
  marks the contextual item words (`by`, `for`, `optional`, `pin`,
  `realization`) as keywords in their place.
- No removal of `=` is scheduled: the staged policy of ADR-0029's amendment
  applies (hint now; a warning later; removal only with a language edition).

## Evidence

`crates/bdl-syntax/test_data/valid/{system,drive_legacy}.bdl` and
`invalid/broken_drive.bdl` (parse and error recovery, `by` as a name),
`crates/bdl-syntax/src/migrate.rs` (the token-only rewrite, idempotent),
`crates/bdl-ide/tests/drive_spelling.rs` (one edge, same analysis and IR, the
hint and fix, tokens, completion, formatter idempotence, the printer),
`crates/bdl-text/tests/workspace.rs` (across files, the same system),
`crates/bdl-daemon/tests/cli.rs` (`migrate-drive-by`), the corpus, fixtures and
e2e tests rewritten to `by`, two compatibility fixtures kept on `=`.
