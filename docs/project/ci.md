---
kind: project
area: process
status: current
---

# Continuous integration and preflight

What each CI job proves, why the Windows jobs run the subset they run, and the
local profiles that reproduce the jobs before a push. The single source of the
checks is `scripts/preflight.py`: every job in `.github/workflows/ci.yml` runs
one of its profiles, so the workflow holds no second list of commands and a
developer runs the same checks locally.

## The contract

| Job                                | Profile              | Proves                                                                                                                                                                                                                                                                                                                                                                                                                                                                    |
| ---------------------------------- | -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Rust + docs (Linux)**            | `rust-ci`            | The semantic and compiler authority: `cargo fmt`, Prettier and markdownlint over every tracked Markdown file, the engineering-record validator and its tests, the screenshot ledger, the localization catalogs, the Standard Library's generated presentation strings and the rendered user-guide pages (regenerated into `target/preflight` and compared), the CI helper scripts' tests, `cargo clippy -D warnings`, `cargo test --workspace`. Uploads the Linux `bdld`. |
| **Flutter (Linux)**                | `flutter-ci`         | The full Studio authority: `flutter pub get`, `dart format --set-exit-if-changed`, `flutter analyze`, the checked-in `gen-l10n` classes current, every Studio test — the e2e suites against the Linux `bdld`.                                                                                                                                                                                                                                                             |
| **Protocol**                       | `proto-ci`           | The checked-in Dart protobuf code is what `protoc` + `dart format` produce for the current `.proto` (Rust regenerates at build time). Compared byte for byte in Python — the same check runs on Windows.                                                                                                                                                                                                                                                                  |
| **Windows compatibility (Rust)**   | `windows-rust-ci`    | The crates whose tests meet what differs on Windows — paths, drive letters, `file://` URIs, process spawning, files, line endings, the host toolchain (table below). Builds `bdld.exe` and uploads it.                                                                                                                                                                                                                                                                    |
| **Windows compatibility (Studio)** | `windows-flutter-ci` | The Studio tests tagged `daemon` or `filesystem` (`apps/studio/dart_test.yaml`) against `bdld.exe`: the real daemon process, project files on disk, preferences, fonts, fixtures.                                                                                                                                                                                                                                                                                         |
| **Windows native build**           | `windows-build-ci`   | `flutter build windows --debug`: the native target compiles and links (`flutter analyze` does not prove that). Runs in parallel with the two compatibility jobs.                                                                                                                                                                                                                                                                                                          |

Linux is the authority for everything platform-independent; Windows never
repeats it. A check that Linux proves once — the parser, the elaborator, the
checker, the evaluator, codegen text, the reducer, widget logic — is not run
again on Windows "just in case". A Windows job grows only when a test meets
something the platform changes.

## Windows coverage

Rust crates on the Windows job (`scripts/preflight.py`, `WINDOWS_RUST_CRATES`),
each with the reason it is there:

| Crate              | Why Windows                                                                                                                                         | Recent Windows regressions it holds                                                                                            |
| ------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| `bdl-daemon`       | spawns `bdld.exe` in the CLI, stdio, text, system and deploy e2e tests; writes project directories                                                  | `tests/cli.rs`, `tests/text_e2e.rs` (saved text projects, `.bdl/identities.json`), `tests/stdio_e2e.rs`, `tests/deploy_e2e.rs` |
| `bdl-lsp`          | `file://` URIs with drive letters and backslashes                                                                                                   | `dc4a5b4` (`path_to_uri` / `uri_to_path`, `tests/e2e.rs`), `d7590f9` (`tests/text_workspace.rs`), `src/position.rs`            |
| `bdl-text`         | source discovery, relative paths joined with `/`, write-back, line endings                                                                          | `tests/workspace.rs`                                                                                                           |
| `bdl-model`        | project persistence: manifest, layout, identities on the filesystem                                                                                 | `persist` tests                                                                                                                |
| `bdl-ide-db`       | document URIs and text workspaces                                                                                                                   | `textual`, `workspace` tests                                                                                                   |
| `bdl-compiler`     | golden files compared byte for byte (the CRLF checkout of `de26950`, now pinned LF by `.gitattributes`); generated crates built by the host `cargo` | `tests/backend_golden.rs`, `tests/backend_differential.rs`, `tests/examples.rs`                                                |
| `bdl-runtime-host` | the harness that runs `cargo` and the generated host binary                                                                                         | `harness.rs`                                                                                                                   |

Not on Windows (Linux proves them once; nothing in them reads a path, spawns a
process or touches a file): `bdl-syntax`, `bdl-elab`, `bdl-check`, `bdl-ir`,
`bdl-reactive`, `bdl-equations`, `bdl-exec-ir`, `bdl-lower`, `bdl-codegen-rust`
(the text it emits is checked by the compiler goldens), `bdl-output`,
`bdl-hardware`, `bdl-layout`, `bdl-library` (its one persistence test goes
through `bdl-model`), `bdl-system`, `bdl-diagnostics`, `bdl-protocol`,
`bdl-ide`, `bdl-runtime-core`. Every one of them is still _compiled_ on Windows
as a dependency of the crates above.

Studio tests on the Windows job: the files tagged in
`apps/studio/dart_test.yaml`'s vocabulary — `daemon` (spawns the real `bdld`:
every `*_e2e_test.dart`, `daemon_client_test`, `deploy_test`, `simulation_test`,
`docs_screenshots_test`) and `filesystem` (real files: `l10n_test` — the
preferences store, `recent_test`, the three `snapshot_*` tests — fonts on disk).
The 19 tagged files hold 85 tests; the other 19 files are reducer and widget
logic Linux proves. Tag a new test file when it spawns `bdld`, touches real
files or reads the platform; the reason is then visible in the suite, not in the
workflow.

## Measured (before)

Two successful runs on `main`, 2026-09-18/19
(`gh api …/actions/runs/<id>/jobs`):

| Step                                 | Linux                                      | Windows (old job)     | Duplicated?     | Platform-specific? | Candidate                                                                                                                                       |
| ------------------------------------ | ------------------------------------------ | --------------------- | --------------- | ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------- |
| checkout                             | 1–2 s                                      | 6–10 s                | —               | —                  | —                                                                                                                                               |
| rust-toolchain                       | 8 s                                        | 11–13 s               | —               | —                  | —                                                                                                                                               |
| rust-cache restore (full hit)        | 6–8 s                                      | 20–22 s               | —               | —                  | —                                                                                                                                               |
| Markdown, records, screenshots, l10n | 10–13 s                                    | —                     | no              | no                 | —                                                                                                                                               |
| `cargo clippy`                       | 12–20 s                                    | —                     | no              | no                 | —                                                                                                                                               |
| `cargo test --workspace`             | 64–87 s                                    | 156–174 s             | **yes**         | partly             | Windows subset: 2 m 00 s of it was compiling every crate's test binary on the cached deps, ~40 s running (the compiler differential alone 20 s) |
| flutter-action (SDK, no cache)       | 55–63 s                                    | 96–108 s              | **yes** (setup) | —                  | cache the SDK and the pub cache                                                                                                                 |
| `flutter pub get`                    | 7–11 s                                     | 38–55 s               | **yes**         | —                  | pub cache                                                                                                                                       |
| `dart format`, `flutter analyze`     | 12 s                                       | —                     | no              | no                 | —                                                                                                                                               |
| `flutter test`                       | 58–62 s                                    | 85–92 s               | **yes**         | partly             | Windows subset (`daemon \|\| filesystem`)                                                                                                       |
| `flutter build windows --debug`      | —                                          | 55–85 s               | no              | yes                | its own parallel job                                                                                                                            |
| protoc + dart + diff                 | 30–32 s                                    | —                     | no              | no                 | cross-platform comparison                                                                                                                       |
| **Job**                              | Rust 111–149 s; Flutter 142–152 s after it | **506–542 s**, serial |                 |                    |                                                                                                                                                 |

Old critical path: the Windows job, 8.4–9.0 min, one serial chain of five
expensive steps; Linux's Rust → Flutter chain was 4.5 min. Windows was slower
because it did every Linux step again on a slower runner (2× the Rust compile,
2× the Flutter test run, 1.5–2× the SDK setup with no cache and 5× the
`pub get`) and then the native build on top, all in one job.

## Changed

- **Jobs.** Windows is three jobs: `windows-rust` (the compatibility crates,
  `bdld.exe` uploaded), `windows-studio` (needs `windows-rust`: the tagged
  Studio tests against that `bdld.exe`) and `windows-build` (independent: the
  native build). The Rust and Studio compatibility jobs no longer set up what
  they do not use (no Flutter in the Rust job, no Rust in the Studio and build
  jobs). `rust` → `flutter` on Linux and `proto` are unchanged in shape.
- **Caches.** `subosito/flutter-action` `cache: true` on every Flutter job (SDK
  and pub cache, keyed by version, OS and `pubspec.lock`). `Swatinem/rust-cache`
  unchanged — its key already carries the OS, the toolchain and the lockfile, it
  restores the dependency artifacts (the workspace crates are rebuilt by design)
  and it sets `CARGO_INCREMENTAL=0` itself; the Windows restore was a full hit
  at 174 MB.
- **Concurrency.** A newer commit cancels the superseded run of its branch or
  PR; runs on `main` are never cancelled.
- **One list.** Each job runs `python scripts/preflight.py <profile>`; the
  Markdown enumeration (`git ls-files`), the proto comparison and the
  rendered-page comparison are done in Python, in scratch directories under
  `target/preflight`, so the identical check runs from PowerShell.

Expected new Windows critical path: `windows-rust` (checkout, toolchain, cache,
the subset's compile and ~15 s of tests) then `windows-studio` (cached SDK,
cached pub, ~60 s of tests) — about 4–5 min against 8.4–9.0; the native build
runs beside them at about 3 min with a cached SDK. The measured numbers of the
first runs on `main` are recorded below when they exist.

## Measured (after)

First run on `main` after the split (`25f9bca`, run 35422425790), with every new
job's caches cold — the Rust cache key carries the job name, so `windows-rust`
compiled its 152 dependency crates once more, and the Flutter SDK cache was
written, not read:

| Job                            | Duration | Of which                                                                            |
| ------------------------------ | -------- | ----------------------------------------------------------------------------------- |
| Rust + docs (Linux)            | 152 s    | preflight `rust-ci` 122 s (cache hit)                                               |
| Flutter (Linux), after Rust    | 159 s    | SDK setup 56 s (cold), `flutter-ci` 83 s                                            |
| Protocol                       | 32 s     | —                                                                                   |
| Windows compatibility (Rust)   | 227 s    | `windows-rust-ci` 178 s = 2 m 11 s compiling on a cold cache + 44 s of tests        |
| Windows compatibility (Studio) | 251 s    | SDK setup 124 s (cold; the pub cache already hit, 42 MB), `windows-flutter-ci` 93 s |
| Windows native build           | 262 s    | SDK setup 100 s (cold), build 118 s                                                 |

Windows critical path this run: 04:51:56 → 04:59:57, **8 m 01 s** (the Rust job
then the Studio job), against 8 m 26 s – 9 m 02 s before — the gain is masked by
the cold caches; the three jobs are parallel where the old one was serial.
Windows runner-minutes: 227 + 251 + 262 = 740 s (three jobs, two of them setting
up Flutter from nothing) against 506–542 s. The honest reading: with cold caches
the split costs more minutes and saves little wall time; the numbers that matter
are the warm ones below. What the split did remove: the platform-independent
Rust test binaries and 130-odd Studio tests no longer run on Windows; the 44 s
of Windows Rust tests are the compatibility set (20 s of it the compiler
differential building generated crates with the host cargo).

Second run on `main` (`f97609b`, run 35423383574; the dependency and SDK caches
warm, the workspace-crate cache first written by this run, so read only from the
third run on):

| Job                            | Duration | Of which                                                                                                 |
| ------------------------------ | -------- | -------------------------------------------------------------------------------------------------------- |
| Rust + docs (Linux)            | 150 s    | preflight `rust-ci` 120 s                                                                                |
| Flutter (Linux), after Rust    | 99 s     | SDK setup 14 s (warm), `flutter-ci` 77 s                                                                 |
| Protocol                       | 27 s     | —                                                                                                        |
| Windows compatibility (Rust)   | 158 s    | cache restore 20 s, `windows-rust-ci` 111 s = 1 m 08 s compiling the 25 workspace crates + 35 s of tests |
| Windows compatibility (Studio) | 192 s    | SDK setup 100 s (a cache hit: 1.8 GB restored and untarred in 67 s), `windows-flutter-ci` 79 s           |
| Windows native build           | 165 s    | SDK setup 85 s (same), build 69 s                                                                        |

Windows critical path this run: 05:13:39 → 05:19:32, **5 m 53 s** (the Rust job
then the Studio job), against 8 m 26 s – 9 m 02 s before the split — a third
off, with restoring the Flutter SDK on Windows now the single largest step of
the path (100 s of the Studio job's 192 s, 67 s of it untarring the 1.8 GB
cache). Windows runner-minutes: 158 + 192 + 165 = 515 s, level with the 506–542
s of the serial job while running three times the parallelism. The next win is a
slimmer Flutter SDK cache on Windows, not more splitting.

Third run (`446db61`, run 35444647909; a commit changing `bdl-library`,
`bdl-daemon`, `bdl-ide` and `bdl-protocol`): Windows Rust 177 s
(`windows-rust-ci` 118 s — the changed crates and everything above them
recompiled, so the workspace-crate cache could not show; the run saved a 311 MB
cache with the workspace crates for the next one), Studio 176 s (SDK setup 92 s,
`windows-flutter-ci` 70 s), native build 183 s (SDK setup 90 s, build 82 s);
critical path **5 m 55 s**, runner-seconds 536. Two warm runs agree: the Flutter
SDK restore is the bottleneck, ~90–100 s of each Flutter job.

## Windows Flutter SDK

What `subosito/flutter-action@v2` with `cache: true` does on Windows, from the
run logs:

| step                             | cold (no SDK cache, run 1)                | stock cache hit (run 2)                      |
| -------------------------------- | ----------------------------------------- | -------------------------------------------- |
| download the SDK                 | 7 s — the 1.79 GB release zip at 277 MB/s | 11 s — the 1.84 GB cache archive at 166 MB/s |
| extract it                       | 91 s (`unzip`)                            | 56 s (`tar`)                                 |
| pub cache (42 MB, hit both runs) | 19 s restore                              | 24 s restore                                 |
| `flutter pub get` afterwards     | 2–3 s                                     | 2–3 s                                        |
| SDK step total                   | ~124 s                                    | ~100 s                                       |

The cache saves 30 s over a fresh download and both are dominated by extraction:
the stable Windows archive is 3.48 GB and 23 085 files unpacked, and NTFS plus
the runner's antivirus make file creation the cost, not bytes. What is in it
(from the archive's central directory):

| tree                                                         | size   | files | needed by BDL's jobs            |
| ------------------------------------------------------------ | ------ | ----- | ------------------------------- |
| `bin/cache/dart-sdk`                                         | 616 MB | 1 154 | yes                             |
| `bin/cache/artifacts/engine/windows-x64{,-profile,-release}` | 977 MB | 113   | yes (the tool checks all three) |
| `bin/cache/artifacts/engine/android-*`                       | 828 MB | 45    | no                              |
| `.git`                                                       | 313 MB | 47    | yes (the version comes from it) |
| `.pub-preload-cache`                                         | 226 MB | 187   | yes (seeds the first `pub get`) |
| `bin` (tool, snapshot)                                       | 206 MB | 359   | yes                             |
| `bin/cache/flutter_web_sdk`                                  | 122 MB | 628   | no                              |
| `packages`                                                   | 76 MB  | 4 660 | yes                             |
| `engine` (sources)                                           | 58 MB  | 6 915 | no                              |
| `dev`, `examples`, `docs`                                    | 25 MB  | 8 336 | no                              |

Strategies compared:

- **A. stock SDK cache** (the second and third runs): ~90–100 s per Flutter job,
  two jobs, every run.
- **B. no SDK cache**: ~124 s per job (98 s of it the download and unzip) —
  worse; the download is fast, the unzip is not.
- **C. pub cache only**: B plus a warm pub cache — the pub cache is the small
  part of the problem (24 s) and a cold `flutter pub get` on Windows was 38–55 s
  in the old job, so the pub cache stays in every strategy.
- **D. slim SDK cache** (chosen): on the run that installs the SDK (a cache miss
  on the key `flutter-slim-…`), `scripts/slim_flutter_sdk.py` removes the
  Android and web artifacts and the `engine`, `dev`, `examples` and `docs` trees
  — 1.03 GB and 15 900 of the 23 085 files — before the post-job save; every
  later run restores the smaller archive. Nothing BDL builds reads what goes:
  the tool downloads an artifact only when a command needs it, and
  `flutter test`, `flutter analyze` and `flutter build windows` need the Windows
  and universal sets, which stay whole.

The topology stays three parallel Windows jobs. Folding the native build into
the Studio job would save one SDK setup and one job's overhead (about 100 s of
runner time) but put the 70–80 s build on the critical path after the tests;
with the slim cache the two parallel Flutter jobs are the cheaper path. Sharing
one prepared SDK between the two jobs as an artifact would be the same 1–2 GB
upload and download as the cache, with nothing gained.

**Workspace-crate cache, tried and reverted.** `cache-workspace-crates: true` on
the Windows Rust job (`f97609b`, runs 2–4): the run after it restored the 297 MB
cache with the 25 workspace crates as a full match and recompiled all 25 anyway
(`fd06125`, a commit with no Rust change: `windows-rust-ci` 122 s, the same as
without). A checkout gives every source a fresh mtime, and cargo's fingerprints
compare source mtimes with the artifacts', so a cached workspace build is always
stale; making the mtimes older would hide real changes. The job is back to
dependencies only (165 MB, 20 s restore, no post-job save).

Fourth run (`fd06125`, run 35445084383) — the slim key's first run, a cache miss
by design: Studio SDK setup 151 s (download and unzip) + 9 s slimming + 11 s
saving the slim archive; native build 139 s + 7 s + 12 s. Windows critical path
7 m 53 s — the one-time cost of the switch.

_Measured after the change: recorded below from the first warm run on the slim
key._

## Local preflight

```bash
python scripts/preflight.py fast       # before a commit (seconds to a minute)
python scripts/preflight.py full       # before a push: the Linux contract
python scripts/preflight.py platform   # what this host can prove of the Windows jobs
python scripts/preflight.py ci         # rust-ci + flutter-ci + proto-ci, exactly
python scripts/preflight.py list       # every check and profile
python scripts/preflight.py fix        # the formatters (the one mutating profile)
```

`fast`: Rust formatting, the engineering records, the localization catalogs and
rendered pages, the generated Dart protobuf, `cargo check --all-targets`, Dart
formatting, `flutter analyze`, the `gen-l10n` classes — the failures that most
often reached CI first. `full`: everything `rust-ci`, `flutter-ci` and
`proto-ci` run. `platform`: builds `bdld`, runs the Windows Rust compatibility
set and the tagged Studio tests against it, and the native build on Windows
(skipped with a note elsewhere: a Mac cannot prove a Windows build). Individual
checks run by name (`preflight.py l10n proto-generated`). `just check`,
`just preflight`, `just docs-check`, `just proto-check` and `just l10n-check`
call the same profiles.

The tool checks its tools first (cargo, flutter, dart, npx, protoc, git) and
says what is missing before running anything; each check is timed and the
summary lists the slow ones; command output is shown for failures (always, under
`GITHUB_ACTIONS`); it never formats, regenerates into the tree or writes a file
— `fix` is the explicit exception.

Deliberately not built: a `--changed` mode and CI path filters. The dependency
rules that would make a skip safe (which Rust crate a Dart test depends on
through `bdld`, which doc a screenshot depends on) are not written down, and a
wrong skip is worse than the minute it saves. Revisit once the compatibility
jobs have measured stable.
