//! Rust code generation for a lowered design.
//!
//! ```text
//! ExecIr ──emit──▶ ast::Module ──print──▶ src/lib.rs      (no_std semantic core)
//!        ──host──▶ ast::Module ──print──▶ src/bin/host.rs (std bridge, feature "host")
//!        ──manifest──▶ bdl-manifest.json
//!        ──────────▶ Cargo.toml
//! + AdapterPlan ──adapter──▶ src/adapter.rs     (sink glue, feature "adapter")
//!               ──targets──▶ src/bin/rp2040.rs  (Embassy firmware, feature "rp2040"),
//!                            memory.x, build.rs, .cargo/config.toml
//! ```
//!
//! Deterministic: the same `ExecIr` yields byte-identical files.  Nothing
//! here rediscovers dependencies, state cells, clocks or writer domains —
//! they arrive explicit in the plan.  The core is target-independent: no
//! HAL, no pin, no peripheral type; the platform adapter is generated
//! beside it from an [`adapter::AdapterPlan`] the compiler builds out of
//! the solved deployment (docs/architecture/embedded-adapter.md), behind
//! features the host build never enables.

#![forbid(unsafe_code)]

pub mod adapter;
pub mod ast;
pub mod emit;
pub mod host;
pub mod manifest;
pub mod names;
pub mod print;
pub mod targets;

use ast::Item;
use bdl_exec_ir::ExecIr;
use std::collections::BTreeMap;

pub use emit::EmitError;
pub use manifest::{Manifest, MANIFEST_VERSION};

/// The generator's identity, written into every artefact.
pub fn generator() -> String {
    format!("bdl-codegen-rust {}", env!("CARGO_PKG_VERSION"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CodegenOptions {
    /// Path to `bdl-runtime-core`, as written into the generated
    /// `Cargo.toml` (relative to the generated crate, or absolute).
    pub runtime_core_path: String,
    /// Path to `bdl-runtime-host`.
    pub runtime_host_path: String,
    /// Path to `bdl-runtime-embassy` (the adapter vocabulary).
    pub runtime_embassy_path: String,
    /// Path to `bdl-runtime-embassy-rp` (the RP2040 binding).
    pub runtime_embassy_rp_path: String,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        CodegenOptions {
            runtime_core_path: "../../runtime/bdl-runtime-core".into(),
            runtime_host_path: "../../runtime/bdl-runtime-host".into(),
            runtime_embassy_path: "../../runtime/bdl-runtime-embassy".into(),
            runtime_embassy_rp_path: "../../runtime/bdl-runtime-embassy-rp".into(),
        }
    }
}

/// A generated crate: relative path → contents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GeneratedCrate {
    pub package: String,
    pub files: BTreeMap<String, String>,
    pub manifest: Manifest,
}

impl GeneratedCrate {
    pub fn core_source(&self) -> &str {
        self.files
            .get("src/lib.rs")
            .map(String::as_str)
            .unwrap_or("")
    }
    /// Lines of generated core source (a baseline metric).
    pub fn core_lines(&self) -> usize {
        self.core_source().lines().count()
    }
    pub fn total_bytes(&self) -> usize {
        self.files.values().map(String::len).sum()
    }
}

pub fn generate(ir: &ExecIr, options: &CodegenOptions) -> Result<GeneratedCrate, EmitError> {
    generate_with_adapter(ir, options, None)
}

/// [`generate`] plus, with a plan, the platform adapter for one target:
/// the `adapter` module in the core crate (feature `adapter`), the target
/// firmware (feature `rp2040`), and the host bridge's recorded operations.
pub fn generate_with_adapter(
    ir: &ExecIr,
    options: &CodegenOptions,
    plan: Option<&adapter::AdapterPlan>,
) -> Result<GeneratedCrate, EmitError> {
    let package = names::package(&ir.name);
    let generator = generator();
    let mut core = emit::core_module(ir, &generator)?;
    let host = host::host_module(ir, &package, &generator, plan)?;
    let manifest = manifest::manifest(ir, &package, &generator, plan)?;
    let manifest_json =
        serde_json::to_string_pretty(&manifest).map_err(|e| EmitError(e.to_string()))?;
    let mut files = BTreeMap::new();
    if let Some(plan) = plan {
        if plan.family != targets::rp2040::FAMILY {
            return Err(EmitError(format!(
                "no target entry for board family `{}` (board `{}`)",
                plan.family, plan.board
            )));
        }
        core.items.push(Item::Comment(vec![
            "The platform adapter's glue, generated beside the core (docs/architecture/embedded-adapter.md).".into(),
        ]));
        core.items.push(Item::Mod {
            attrs: vec!["cfg(feature = \"adapter\")".into()],
            name: "adapter".into(),
        });
        let glue = adapter::adapter_module(ir, plan, &generator);
        files.insert("src/adapter.rs".to_string(), print::module(&glue));
        let firmware = targets::rp2040::firmware_module(ir, &package, plan, &generator)?;
        files.insert("src/bin/rp2040.rs".to_string(), print::module(&firmware));
        files.insert(
            "memory.x".to_string(),
            targets::rp2040::MEMORY_X.to_string(),
        );
        files.insert(
            "build.rs".to_string(),
            targets::rp2040::BUILD_RS.to_string(),
        );
        files.insert(
            ".cargo/config.toml".to_string(),
            targets::rp2040::CARGO_CONFIG.to_string(),
        );
    }
    files.insert(
        "Cargo.toml".to_string(),
        cargo_toml(&package, options, ir.uses_lists(), plan),
    );
    files.insert("src/lib.rs".to_string(), print::module(&core));
    files.insert("src/bin/host.rs".to_string(), print::module(&host));
    files.insert("bdl-manifest.json".to_string(), manifest_json + "\n");
    Ok(GeneratedCrate {
        package,
        files,
        manifest,
    })
}

fn cargo_toml(
    package: &str,
    o: &CodegenOptions,
    collections: bool,
    plan: Option<&adapter::AdapterPlan>,
) -> String {
    let features = if collections {
        r#", features = ["collections"]"#
    } else {
        ""
    };
    let Some(plan) = plan else {
        return cargo_toml_core(package, o, features);
    };
    let rp_features = if plan.arena_bytes.is_some() {
        r#", features = ["collections"]"#
    } else {
        ""
    };
    let static_cell_dep = if plan.arena_bytes.is_some() {
        "static-cell = { version = \"2.1\", optional = true }\n"
    } else {
        ""
    };
    let static_cell_feature = if plan.arena_bytes.is_some() {
        ", \"dep:static-cell\""
    } else {
        ""
    };
    format!(
        "# Generated by {gen} for target `{board}`. Do not edit.\n\
         [package]\n\
         name = \"{package}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\
         rust-version = \"1.89\"\n\
         publish = false\n\
         build = \"build.rs\"\n\
         \n\
         # A generated crate is its own workspace, wherever it is written.\n\
         [workspace]\n\
         \n\
         [lib]\n\
         path = \"src/lib.rs\"\n\
         \n\
         # Named after the package so several generated crates can share one\n\
         # target directory without their binaries overwriting each other.\n\
         [[bin]]\n\
         name = \"{package}-host\"\n\
         path = \"src/bin/host.rs\"\n\
         required-features = [\"host\"]\n\
         \n\
         # The firmware: `cargo build --release --target {triple} --features rp2040`.\n\
         [[bin]]\n\
         name = \"{package}-rp2040\"\n\
         path = \"src/bin/rp2040.rs\"\n\
         test = false\n\
         bench = false\n\
         required-features = [\"rp2040\"]\n\
         \n\
         [features]\n\
         default = []\n\
         # The std host bridge; the core itself is no_std.  It applies the adapter\n\
         # glue to recording sinks (`TickTrace.adapter`).\n\
         host = [\"dep:bdl-runtime-host\", \"adapter\"]\n\
         # The target-independent adapter glue (`src/adapter.rs`).\n\
         adapter = [\"dep:bdl-runtime-embassy\"]\n\
         # The RP2040 firmware over Embassy.\n\
         rp2040 = [\"adapter\", \"dep:bdl-runtime-embassy-rp\", \"dep:embassy-executor\", \"dep:embassy-rp\", \"dep:embassy-time\", \"dep:cortex-m-rt\", \"dep:panic-halt\"{static_cell_feature}]\n\
         \n\
         [dependencies]\n\
         bdl-runtime-core = {{ path = {core:?}{features} }}\n\
         bdl-runtime-host = {{ path = {host:?}, optional = true }}\n\
         bdl-runtime-embassy = {{ path = {embassy:?}, optional = true }}\n\
         bdl-runtime-embassy-rp = {{ path = {embassy_rp:?}{rp_features}, optional = true }}\n\
         embassy-rp = {{ version = \"0.10.0\", default-features = false, features = [\"rt\", \"rp2040\", \"time-driver\", \"critical-section-impl\", \"boot2-w25q080\"], optional = true }}\n\
         embassy-executor = {{ version = \"0.10.0\", features = [\"platform-cortex-m\", \"executor-thread\"], optional = true }}\n\
         embassy-time = {{ version = \"0.5.1\", optional = true }}\n\
         cortex-m-rt = {{ version = \"0.7.5\", optional = true }}\n\
         panic-halt = {{ version = \"1.0.0\", optional = true }}\n\
         {static_cell_dep}\
         \n\
         [profile.release]\n\
         debug = false\n\
         lto = true\n\
         opt-level = \"s\"\n",
        gen = generator(),
        board = plan.board,
        triple = targets::rp2040::TRIPLE,
        core = o.runtime_core_path,
        host = o.runtime_host_path,
        embassy = o.runtime_embassy_path,
        embassy_rp = o.runtime_embassy_rp_path,
    )
}

fn cargo_toml_core(package: &str, o: &CodegenOptions, features: &str) -> String {
    format!(
        "# Generated by {gen}. Do not edit.\n\
         [package]\n\
         name = \"{package}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\
         publish = false\n\
         \n\
         # A generated crate is its own workspace, wherever it is written.\n\
         [workspace]\n\
         \n\
         [lib]\n\
         path = \"src/lib.rs\"\n\
         \n\
         # Named after the package so several generated crates can share one\n\
         # target directory without their host binaries overwriting each other.\n\
         [[bin]]\n\
         name = \"{package}-host\"\n\
         path = \"src/bin/host.rs\"\n\
         required-features = [\"host\"]\n\
         \n\
         [features]\n\
         default = []\n\
         # The std host bridge; the core itself is no_std.\n\
         host = [\"dep:bdl-runtime-host\"]\n\
         \n\
         [dependencies]\n\
         bdl-runtime-core = {{ path = {core:?}{features} }}\n\
         bdl-runtime-host = {{ path = {host:?}, optional = true }}\n",
        gen = generator(),
        core = o.runtime_core_path,
        host = o.runtime_host_path,
    )
}
