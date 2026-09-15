# BDL monorepo task runner.  `just` lists recipes.

set shell := ["zsh", "-cu"]

flutter := env_var_or_default("FLUTTER", "flutter")
protoc  := env_var_or_default("PROTOC", "protoc")
studio  := "apps/studio"

default:
    @just --list

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

# Build bdld, then run Studio against it.
studio: build
    cd {{studio}} && {{flutter}} run -d macos --dart-define=BDLD_PATH=$(pwd)/../../target/debug/bdld

# ---- Everything -----------------------------------------------------------

check: lint test studio-analyze studio-test proto-check

# Run bdld on stdio (for manual protocol experiments).
bdld:
    cargo run -p bdl-daemon -- serve
