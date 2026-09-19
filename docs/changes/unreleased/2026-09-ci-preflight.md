# CI split by platform coverage; one preflight for local and CI

- Date: 2026-09-19
- Area: process
- Affected: developers
- Related: `docs/project/ci.md`

## What changed

- **One validation contract.** `scripts/preflight.py` holds every check CI
  runs as a named, timed check; each CI job runs one of its profiles
  (`rust-ci`, `flutter-ci`, `proto-ci`, `windows-rust-ci`,
  `windows-flutter-ci`, `windows-build-ci`), and a developer runs the same
  checks locally: `fast` before a commit, `full` before a push, `platform`
  for what the host can prove of the Windows jobs. Read-only; `fix` is the
  one mutating profile. Cross-platform: Markdown files are enumerated with
  `git ls-files`, the generated protobuf and the rendered localized pages are
  compared in Python in `target/preflight`, never in the tree.
- **Windows is three parallel jobs** — the Rust compatibility crates (paths,
  URIs, processes, files, the host toolchain; builds `bdld.exe`), the
  `daemon`/`filesystem`-tagged Studio tests against it, and the native
  build — instead of one serial job that repeated the whole Linux Rust and
  Flutter suites. Linux remains the authority for everything
  platform-independent; the coverage table is `docs/project/ci.md`.
- **Test tags.** `apps/studio/dart_test.yaml` declares `daemon`,
  `filesystem` and `e2e`; the 19 test files that spawn `bdld` or touch real
  files carry `@Tags`, so the reason a test runs on Windows is in the suite.
- **Caches and cancellation.** Flutter jobs cache the SDK and the pub cache;
  a newer commit cancels a branch's or PR's run in flight (never `main`'s).
- A new check CI did not have: the checked-in `flutter gen-l10n` classes are
  current (`studio-l10n-generated`).
- `scripts/docs_l10n.py --out DIR` renders the catalogs and pages elsewhere
  with the canonical link geometry, for the freshness check.

## Compatibility and migration

- Developers: `just check` now runs `python scripts/preflight.py full`;
  `just docs-check`, `just docs-lint`, `just proto-check` and
  `just l10n-check` call the same checks. `just preflight` is the fast
  profile. Python 3.11+, Node 22 (for `npx`), Flutter 3.47.4, Rust 1.89 and
  — for the protocol check — `protoc` with `protoc_plugin` on PATH; the
  tool says which is missing before running anything.
- Project files, protocol, designers: nothing.

## Evidence

`.github/workflows/ci.yml`, `scripts/preflight.py`,
`apps/studio/dart_test.yaml`, `docs/project/ci.md` (measured timings before
and after).
