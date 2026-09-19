# Continuous integration and preflight

What each CI job proves, why the Windows jobs run the subset they run, and
the local profiles that reproduce the jobs before a push. The single source
of the checks is `scripts/preflight.py`: every job in
`.github/workflows/ci.yml` runs one of its profiles, so the workflow holds
no second list of commands and a developer runs the same checks locally.

## The contract

| Job                                 | Profile              | Proves                                                                                                                                                                                                                                                                                                                                          |
| ----------------------------------- | -------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Rust + docs (Linux)**             | `rust-ci`            | The semantic and compiler authority: `cargo fmt`, Prettier and markdownlint over every tracked Markdown file, the engineering-record validator and its tests, the screenshot ledger, the localization catalogs and the rendered user-guide pages (regenerated into `target/preflight` and compared), `cargo clippy -D warnings`, `cargo test --workspace`. Uploads the Linux `bdld`. |
| **Flutter (Linux)**                 | `flutter-ci`         | The full Studio authority: `flutter pub get`, `dart format --set-exit-if-changed`, `flutter analyze`, the checked-in `gen-l10n` classes current, every Studio test — the e2e suites against the Linux `bdld`.                                                                                                                                    |
| **Protocol**                        | `proto-ci`           | The checked-in Dart protobuf code is what `protoc` + `dart format` produce for the current `.proto` (Rust regenerates at build time). Compared byte for byte in Python — the same check runs on Windows.                                                                                                                                       |
| **Windows compatibility (Rust)**    | `windows-rust-ci`    | The crates whose tests meet what differs on Windows — paths, drive letters, `file://` URIs, process spawning, files, line endings, the host toolchain (table below). Builds `bdld.exe` and uploads it.                                                                                                                                          |
| **Windows compatibility (Studio)**  | `windows-flutter-ci` | The Studio tests tagged `daemon` or `filesystem` (`apps/studio/dart_test.yaml`) against `bdld.exe`: the real daemon process, project files on disk, preferences, fonts, fixtures.                                                                                                                                                              |
| **Windows native build**            | `windows-build-ci`   | `flutter build windows --debug`: the native target compiles and links (`flutter analyze` does not prove that). Runs in parallel with the two compatibility jobs.                                                                                                                                                                               |

Linux is the authority for everything platform-independent; Windows never
repeats it. A check that Linux proves once — the parser, the elaborator, the
checker, the evaluator, codegen text, the reducer, widget logic — is not run
again on Windows "just in case". A Windows job grows only when a test meets
something the platform changes.

## Windows coverage

Rust crates on the Windows job (`scripts/preflight.py`,
`WINDOWS_RUST_CRATES`), each with the reason it is there:

| Crate              | Why Windows                                                                                                                          | Recent Windows regressions it holds                                                                                         |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------- |
| `bdl-daemon`       | spawns `bdld.exe` in the CLI, stdio, text, system and deploy e2e tests; writes project directories                                    | `tests/cli.rs`, `tests/text_e2e.rs` (saved text projects, `.bdl/identities.json`), `tests/stdio_e2e.rs`, `tests/deploy_e2e.rs` |
| `bdl-lsp`          | `file://` URIs with drive letters and backslashes                                                                                    | `dc4a5b4` (`path_to_uri` / `uri_to_path`, `tests/e2e.rs`), `d7590f9` (`tests/text_workspace.rs`), `src/position.rs`        |
| `bdl-text`         | source discovery, relative paths joined with `/`, write-back, line endings                                                           | `tests/workspace.rs`                                                                                                        |
| `bdl-model`        | project persistence: manifest, layout, identities on the filesystem                                                                  | `persist` tests                                                                                                             |
| `bdl-ide-db`       | document URIs and text workspaces                                                                                                    | `textual`, `workspace` tests                                                                                                |
| `bdl-compiler`     | golden files compared byte for byte (the CRLF checkout of `de26950`, now pinned LF by `.gitattributes`); generated crates built by the host `cargo` | `tests/backend_golden.rs`, `tests/backend_differential.rs`, `tests/examples.rs`                                             |
| `bdl-runtime-host` | the harness that runs `cargo` and the generated host binary                                                                          | `harness.rs`                                                                                                                |

Not on Windows (Linux proves them once; nothing in them reads a path,
spawns a process or touches a file): `bdl-syntax`, `bdl-elab`, `bdl-check`,
`bdl-ir`, `bdl-reactive`, `bdl-equations`, `bdl-exec-ir`, `bdl-lower`,
`bdl-codegen-rust` (the text it emits is checked by the compiler goldens),
`bdl-output`, `bdl-hardware`, `bdl-layout`, `bdl-library` (its one
persistence test goes through `bdl-model`), `bdl-system`, `bdl-diagnostics`,
`bdl-protocol`, `bdl-ide`, `bdl-runtime-core`. Every one of them is still
_compiled_ on Windows as a dependency of the crates above.

Studio tests on the Windows job: the files tagged in
`apps/studio/dart_test.yaml`'s vocabulary — `daemon` (spawns the real
`bdld`: every `*_e2e_test.dart`, `daemon_client_test`, `deploy_test`,
`simulation_test`, `docs_screenshots_test`) and `filesystem` (real files:
`l10n_test` — the preferences store, `recent_test`, the three `snapshot_*`
tests — fonts on disk). The 19 tagged files hold 85 tests; the other 19 files
are reducer and widget logic Linux proves. Tag a new test file when it spawns
`bdld`, touches real files or reads the platform; the reason is then visible
in the suite, not in the workflow.

## Measured (before)

Two successful runs on `main`, 2026-09-18/19 (`gh api …/actions/runs/<id>/jobs`):

| Step                                | Linux            | Windows (old job) | Duplicated?         | Platform-specific? | Candidate                                          |
| ----------------------------------- | ---------------- | ----------------- | ------------------- | ------------------ | -------------------------------------------------- |
| checkout                            | 1–2 s            | 6–10 s            | —                   | —                  | —                                                  |
| rust-toolchain                      | 8 s              | 11–13 s           | —                   | —                  | —                                                  |
| rust-cache restore (full hit)       | 6–8 s            | 20–22 s           | —                   | —                  | —                                                  |
| Markdown, records, screenshots, l10n| 10–13 s          | —                 | no                  | no                 | —                                                  |
| `cargo clippy`                      | 12–20 s          | —                 | no                  | no                 | —                                                  |
| `cargo test --workspace`            | 64–87 s          | 156–174 s         | **yes**             | partly             | Windows subset: 2 m 00 s of it was compiling every crate's test binary on the cached deps, ~40 s running (the compiler differential alone 20 s) |
| flutter-action (SDK, no cache)      | 55–63 s          | 96–108 s          | **yes** (setup)     | —                  | cache the SDK and the pub cache                     |
| `flutter pub get`                   | 7–11 s           | 38–55 s           | **yes**             | —                  | pub cache                                           |
| `dart format`, `flutter analyze`    | 12 s             | —                 | no                  | no                 | —                                                  |
| `flutter test`                      | 58–62 s          | 85–92 s           | **yes**             | partly             | Windows subset (`daemon \|\| filesystem`)           |
| `flutter build windows --debug`     | —                | 55–85 s           | no                  | yes                | its own parallel job                                |
| protoc + dart + diff                | 30–32 s          | —                 | no                  | no                 | cross-platform comparison                           |
| **Job**                             | Rust 111–149 s; Flutter 142–152 s after it | **506–542 s**, serial | | | |

Old critical path: the Windows job, 8.4–9.0 min, one serial chain of five
expensive steps; Linux's Rust → Flutter chain was 4.5 min. Windows was slower
because it did every Linux step again on a slower runner (2× the Rust
compile, 2× the Flutter test run, 1.5–2× the SDK setup with no cache and
5× the `pub get`) and then the native build on top, all in one job.

## Changed

- **Jobs.** Windows is three jobs: `windows-rust` (the compatibility crates,
  `bdld.exe` uploaded), `windows-studio` (needs `windows-rust`: the tagged
  Studio tests against that `bdld.exe`) and `windows-build` (independent:
  the native build). The Rust and Studio compatibility jobs no longer set
  up what they do not use (no Flutter in the Rust job, no Rust in the Studio
  and build jobs). `rust` → `flutter` on Linux and `proto` are unchanged in
  shape.
- **Caches.** `subosito/flutter-action` `cache: true` on every Flutter job
  (SDK and pub cache, keyed by version, OS and `pubspec.lock`).
  `Swatinem/rust-cache` unchanged — its key already carries the OS, the
  toolchain and the lockfile, it restores the dependency artifacts (the
  workspace crates are rebuilt by design) and it sets `CARGO_INCREMENTAL=0`
  itself; the Windows restore was a full hit at 174 MB.
- **Concurrency.** A newer commit cancels the superseded run of its branch
  or PR; runs on `main` are never cancelled.
- **One list.** Each job runs `python scripts/preflight.py <profile>`; the
  Markdown enumeration (`git ls-files`), the proto comparison and the
  rendered-page comparison are done in Python, in scratch directories under
  `target/preflight`, so the identical check runs from PowerShell.

Expected new Windows critical path: `windows-rust` (checkout, toolchain,
cache, the subset's compile and ~15 s of tests) then `windows-studio` (cached
SDK, cached pub, ~60 s of tests) — about 4–5 min against 8.4–9.0; the native
build runs beside them at about 3 min with a cached SDK. The measured numbers
of the first runs on `main` are recorded below when they exist.

## Measured (after)

_To be filled from the first `main` runs after this change: job durations,
the compatibility set's compile and test time, cache hits, the native build
alone._

## Local preflight

```bash
python scripts/preflight.py fast       # before a commit (seconds to a minute)
python scripts/preflight.py full       # before a push: the Linux contract
python scripts/preflight.py platform   # what this host can prove of the Windows jobs
python scripts/preflight.py ci         # rust-ci + flutter-ci + proto-ci, exactly
python scripts/preflight.py list       # every check and profile
python scripts/preflight.py fix        # the formatters (the one mutating profile)
```

`fast`: Rust formatting, the engineering records, the localization catalogs
and rendered pages, the generated Dart protobuf, `cargo check --all-targets`,
Dart formatting, `flutter analyze`, the `gen-l10n` classes — the failures
that most often reached CI first. `full`: everything `rust-ci`, `flutter-ci`
and `proto-ci` run. `platform`: builds `bdld`, runs the Windows Rust
compatibility set and the tagged Studio tests against it, and the native
build on Windows (skipped with a note elsewhere: a Mac cannot prove a Windows
build). Individual checks run by name (`preflight.py l10n proto-generated`).
`just check`, `just preflight`, `just docs-check`, `just proto-check` and
`just l10n-check` call the same profiles.

The tool checks its tools first (cargo, flutter, dart, npx, protoc, git) and
says what is missing before running anything; each check is timed and the
summary lists the slow ones; command output is shown for failures (always,
under `GITHUB_ACTIONS`); it never formats, regenerates into the tree or
writes a file — `fix` is the explicit exception.

Deliberately not built: a `--changed` mode and CI path filters. The
dependency rules that would make a skip safe (which Rust crate a Dart test
depends on through `bdld`, which doc a screenshot depends on) are not
written down, and a wrong skip is worse than the minute it saves. Revisit
once the compatibility jobs have measured stable.
