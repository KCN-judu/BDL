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
    /// Path to `bdl-runtime-adapter` (the adapter vocabulary).
    pub runtime_adapter_path: String,
    /// Path to `bdl-runtime-embassy-rp` (the RP2040 binding).
    pub runtime_embassy_rp_path: String,
    /// Path to `bdl-runtime-arduino` (the Arduino binding).
    pub runtime_arduino_path: String,
}

impl Default for CodegenOptions {
    fn default() -> Self {
        CodegenOptions {
            runtime_core_path: "../../runtime/bdl-runtime-core".into(),
            runtime_host_path: "../../runtime/bdl-runtime-host".into(),
            runtime_adapter_path: "../../runtime/bdl-runtime-adapter".into(),
            runtime_embassy_rp_path: "../../runtime/bdl-runtime-embassy-rp".into(),
            runtime_arduino_path: "../../runtime/bdl-runtime-arduino".into(),
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
/// firmware (feature `<target>`), and the host bridge's recorded operations.
pub fn generate_with_adapter(
    ir: &ExecIr,
    options: &CodegenOptions,
    plan: Option<&adapter::AdapterPlan>,
) -> Result<GeneratedCrate, EmitError> {
    let package = names::package(&ir.name);
    let generator = generator();
    let mut core = emit::core_module(ir, &generator)?;
    let host = host::host_module(ir, &package, &generator, plan)?;
    let entry = match plan {
        None => None,
        Some(plan) => Some(
            targets::Entry::for_board(&plan.family, &plan.board).ok_or_else(|| {
                EmitError(format!(
                    "no target entry for board `{}` of family `{}`",
                    plan.board, plan.family
                ))
            })?,
        ),
    };
    let manifest = manifest::manifest(ir, &package, &generator, plan, entry.as_ref())?;
    let manifest_json =
        serde_json::to_string_pretty(&manifest).map_err(|e| EmitError(e.to_string()))?;
    let mut files = BTreeMap::new();
    if let (Some(plan), Some(entry)) = (plan, &entry) {
        core.items.push(Item::Comment(vec![
            "The platform adapter's glue, generated beside the core (docs/architecture/embedded-adapter.md).".into(),
        ]));
        core.items.push(Item::Mod {
            attrs: vec!["cfg(feature = \"adapter\")".into()],
            name: "adapter".into(),
        });
        let glue = adapter::adapter_module(ir, plan, &generator)?;
        files.insert("src/adapter.rs".to_string(), print::module(&glue));
        let firmware = entry.firmware_module(ir, &package, plan, &generator)?;
        files.insert(
            format!("src/bin/{}.rs", entry.feature()),
            print::module(&firmware),
        );
        for (path, text) in entry.files() {
            files.insert(path, text);
        }
    }
    files.insert(
        "Cargo.toml".to_string(),
        match (plan, &entry) {
            (Some(plan), Some(entry)) => {
                cargo_toml_target(&package, options, ir.uses_lists(), plan, entry)
            }
            _ => cargo_toml_core(&package, options, ir.uses_lists()),
        },
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

/// The manifest of a crate with a target: the core's sections plus the
/// target feature's binary, features and dependencies.
fn cargo_toml_target(
    package: &str,
    o: &CodegenOptions,
    collections: bool,
    plan: &adapter::AdapterPlan,
    entry: &targets::Entry,
) -> String {
    let features = if collections {
        r#", features = ["collections"]"#
    } else {
        ""
    };
    let feature = entry.feature();
    let build = if entry.has_build_script() {
        "build = \"build.rs\"\n"
    } else {
        ""
    };
    let (feature_line, dep_lines) = entry.cargo_sections(o, plan);
    format!(
        "# Generated by {gen} for target `{board}`. Do not edit.\n\
         [package]\n\
         name = \"{package}\"\n\
         version = \"0.1.0\"\n\
         edition = \"2021\"\n\
         rust-version = \"{rust_version}\"\n\
         publish = false\n\
         {build}\
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
         # The firmware: `{build_command}`.\n\
         [[bin]]\n\
         name = \"{package}-{feature}\"\n\
         path = \"src/bin/{feature}.rs\"\n\
         test = false\n\
         bench = false\n\
         required-features = [\"{feature}\"]\n\
         \n\
         [features]\n\
         default = []\n\
         # The std host bridge; the core itself is no_std.  It applies the adapter\n\
         # glue to recording sinks (`TickTrace.adapter`).\n\
         host = [\"dep:bdl-runtime-host\", \"adapter\"]\n\
         # The target-independent adapter glue (`src/adapter.rs`).\n\
         adapter = [\"dep:bdl-runtime-adapter\"]\n\
         {feature_line}\
         \n\
         [dependencies]\n\
         bdl-runtime-core = {{ path = {core:?}{features} }}\n\
         bdl-runtime-host = {{ path = {host:?}, optional = true }}\n\
         bdl-runtime-adapter = {{ path = {adapter:?}, optional = true }}\n\
         {dep_lines}\
         \n\
         [profile.dev]\n\
         panic = \"abort\"\n\
         \n\
         [profile.release]\n\
         panic = \"abort\"\n\
         debug = false\n\
         lto = true\n\
         opt-level = \"s\"\n",
        gen = generator(),
        board = plan.board,
        rust_version = entry.rust_version(),
        build_command = entry.build_command(package),
        core = o.runtime_core_path,
        host = o.runtime_host_path,
        adapter = o.runtime_adapter_path,
    )
}

fn cargo_toml_core(package: &str, o: &CodegenOptions, collections: bool) -> String {
    let features = if collections {
        r#", features = ["collections"]"#
    } else {
        ""
    };
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
