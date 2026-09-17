# BDL monorepo task runner.  `just` lists recipes.

set shell := ["zsh", "-cu"]
# On Windows, recipes run under Git Bash (ships with Git for Windows).
set windows-shell := ["bash", "-cu"]

flutter := env_var_or_default("FLUTTER", "flutter")
protoc  := env_var_or_default("PROTOC", "protoc")
studio  := "apps/studio"
device  := if os() == "macos" { "macos" } else if os() == "windows" { "windows" } else { "linux" }
bdld    := if os() == "windows" { "bdld.exe" } else { "bdld" }

default:
    @just --list

# ---- Documentation --------------------------------------------------------

prettier      := "npx --yes prettier@3.9.7"
markdownlint  := "npx --yes markdownlint-cli2@0.23.2"

# Format every tracked Markdown file: Prettier for layout, bare fences
# labelled `text`, then markdownlint's own fixes.
docs-fmt:
    git ls-files -z '*.md' | xargs -0 {{prettier}} --log-level warn --write
    git ls-files -z '*.md' | xargs -0 python3 scripts/md_normalize.py
    git ls-files -z '*.md' | xargs -0 {{markdownlint}} --fix

# Report Markdown that `just docs-fmt` would change or that breaks a rule.
docs-lint:
    git ls-files -z '*.md' | xargs -0 {{prettier}} --log-level warn --check
    git ls-files -z '*.md' | xargs -0 {{markdownlint}}

docs-check: docs-lint
    python3 scripts/validate_docs.py
    python3 -m unittest scripts/test_validate_docs.py
    python3 scripts/check_screenshots.py

# Capture the user guide's screenshots from the real Studio against the real
# bdld on docs/fixtures/*, as docs/user-guide/screenshots/manifest.json says
# (writes the PNGs under docs/user-guide/assets and the capture ledger).
docs-shots: build
    cd {{studio}} && DOCS_SHOTS=1 {{flutter}} test test/docs_screenshots_test.dart
    python3 scripts/check_screenshots.py

# ---- Rust -----------------------------------------------------------------

build:
    cargo build --workspace

test:
    cargo test --workspace

lint:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings

fmt:
    cargo fmt --all
    cd {{studio}} && dart format --page-width 100 lib test

# ---- Protocol -------------------------------------------------------------

# Regenerate the checked-in Dart protobuf code (Rust regenerates at build time).
# Generated code is formatted so that `dart format` over lib/ is idempotent.
proto:
    {{protoc}} -I crates/bdl-protocol/proto \
        --dart_out={{studio}}/lib/protocol/gen \
        crates/bdl-protocol/proto/bdl/v1/bdl.proto
    cd {{studio}} && dart format --page-width 100 lib/protocol/gen

# Fail if the checked-in Dart protobuf code is out of date.
proto-check:
    #!/usr/bin/env zsh
    set -eu
    tmp=$(mktemp -d)
    {{protoc}} -I crates/bdl-protocol/proto --dart_out=$tmp crates/bdl-protocol/proto/bdl/v1/bdl.proto
    dart format --page-width 100 $tmp >/dev/null
    diff -r $tmp/bdl {{studio}}/lib/protocol/gen/bdl
    rm -rf $tmp

# ---- Studio ---------------------------------------------------------------

studio-deps:
    cd {{studio}} && {{flutter}} pub get

studio-analyze:
    cd {{studio}} && dart format --page-width 100 --output=none --set-exit-if-changed lib test
    cd {{studio}} && {{flutter}} analyze

# Flutter tests; the daemon integration test runs when target/debug/bdld exists.
studio-test: build
    cd {{studio}} && {{flutter}} test

# Render the Studio shell from hand-built state to PNGs (light/dark) for a
# quick look at the chrome.  Not a documentation source: the user guide's
# screenshots come from `just docs-shots`.
studio-snap out="/tmp/bdl-snap":
    mkdir -p {{out}}
    cd {{studio}} && SNAP_DIR={{out}} {{flutter}} test --update-goldens test/snapshot_preview_test.dart
    @echo "wrote {{out}}/shell_light.png and shell_dark.png"

# Build bdld, then run Studio against it (macOS, Windows or Linux desktop).
studio: build
    cd {{studio}} && {{flutter}} run -d {{device}} --dart-define=BDLD_PATH=$(pwd)/../../target/debug/{{bdld}}

# ---- Everything -----------------------------------------------------------

check: docs-check lint test studio-analyze studio-test proto-check

# Run bdld on stdio (for manual protocol experiments).
bdld:
    cargo run -p bdl-daemon -- serve
