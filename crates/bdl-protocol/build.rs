//! Compiles `proto/bdl/v1/bdl.proto` with the pure-Rust `protox` compiler so
//! no external `protoc` is needed to build the workspace.  (The Dart side is
//! generated offline with `just proto` and checked in.)

fn main() {
    println!("cargo:rerun-if-changed=proto");
    let fds = protox::compile(["proto/bdl/v1/bdl.proto"], ["proto"])
        .expect("protox: bdl.proto must compile");
    prost_build::Config::new()
        .compile_fds(fds)
        .expect("prost-build: code generation failed");
}
