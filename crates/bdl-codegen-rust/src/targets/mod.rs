//! Target entries: one per board family, each turning an
//! [`crate::adapter::AdapterPlan`] into the firmware for that family
//! (docs/architecture/embedded-adapter.md § Target entries).
//!
//! What every entry answers, and nothing else decides:
//!
//! * which board resource is which peripheral ([`Entry::peripheral`]) —
//!   derived from the board file's resource id, never chosen;
//! * the firmware's entry module and the files beside it;
//! * the dependencies and the build command of the `<target>` feature;
//! * whether the family can carry the core's collections at all.
//!
//! The plan, the glue (`src/adapter.rs`) and the host's recorded
//! operations are family-independent; a second family adds an entry here
//! and touches nothing upstream.

pub mod arduino;
pub mod rp2040;

use crate::adapter::{AdapterPlan, SinkBinding};
use crate::ast::{Expr, Module};
use crate::emit::EmitError;
use crate::CodegenOptions;
use bdl_exec_ir::ExecIr;

/// The peripheral a sink lands on: how the firmware constructs it over
/// the HAL's peripherals, and the manifest's words for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Peripheral {
    pub construct: Expr,
    pub describe: String,
}

/// A board's target entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    /// The RP2040 over Embassy (`rp2040_pico`).
    Rp2040,
    /// An Arduino board over `avr-hal` (`arduino_nano`, …).
    Arduino(&'static arduino::Board),
}

impl Entry {
    /// The entry for a board, by its family and id; `None` when no entry
    /// exists (the compiler refuses the target).
    pub fn for_board(family: &str, board: &str) -> Option<Entry> {
        match family {
            rp2040::FAMILY => Some(Entry::Rp2040),
            arduino::FAMILY => arduino::board(board).map(Entry::Arduino),
            _ => None,
        }
    }

    /// The Cargo feature and the binary suffix: `rp2040`, `arduino_nano`.
    pub fn feature(&self) -> String {
        match self {
            Entry::Rp2040 => "rp2040".into(),
            Entry::Arduino(b) => b.id.to_string(),
        }
    }

    /// The Rust target the firmware is built for.
    pub fn triple(&self) -> &'static str {
        match self {
            Entry::Rp2040 => rp2040::TRIPLE,
            Entry::Arduino(_) => arduino::TRIPLE,
        }
    }

    /// The toolchain the build needs beyond the pinned one, if any.
    pub fn toolchain(&self) -> Option<&'static str> {
        match self {
            Entry::Rp2040 => None,
            Entry::Arduino(_) => Some(arduino::TOOLCHAIN),
        }
    }

    /// Whether the family can host a collection arena (an allocator over a
    /// static arena).  An 8-bit AVR with 2 KiB of SRAM cannot; a design
    /// that carries lists is refused for it.
    pub fn supports_collections(&self) -> bool {
        match self {
            Entry::Rp2040 => true,
            Entry::Arduino(_) => false,
        }
    }

    /// The command that builds the firmware in the generated crate.
    pub fn build_command(&self, package: &str) -> String {
        match self {
            Entry::Rp2040 => format!(
                "cargo build --release --target {} --features rp2040 --bin {package}-rp2040",
                rp2040::TRIPLE
            ),
            Entry::Arduino(b) => format!(
                "cargo +{} build --release --target {} -Zbuild-std=core --features {} --bin {package}-{}",
                arduino::TOOLCHAIN,
                arduino::TRIPLE,
                b.id,
                b.id
            ),
        }
    }

    pub fn peripheral(&self, b: &SinkBinding) -> Result<Peripheral, EmitError> {
        match self {
            Entry::Rp2040 => rp2040::peripheral(b).map(|p| Peripheral {
                construct: p.construct(),
                describe: p.describe(),
            }),
            Entry::Arduino(board) => arduino::peripheral(board, b),
        }
    }

    /// `src/bin/<feature>.rs`.
    pub fn firmware_module(
        &self,
        ir: &ExecIr,
        package: &str,
        plan: &AdapterPlan,
        generator: &str,
    ) -> Result<Module, EmitError> {
        match self {
            Entry::Rp2040 => rp2040::firmware_module(ir, package, plan, generator),
            Entry::Arduino(board) => arduino::firmware_module(board, ir, package, plan, generator),
        }
    }

    /// The files beside the sources: linker inputs, the cargo config.
    pub fn files(&self) -> Vec<(String, String)> {
        match self {
            Entry::Rp2040 => vec![
                ("memory.x".into(), rp2040::MEMORY_X.into()),
                ("build.rs".into(), rp2040::BUILD_RS.into()),
                (".cargo/config.toml".into(), rp2040::CARGO_CONFIG.into()),
            ],
            Entry::Arduino(_) => vec![(".cargo/config.toml".into(), arduino::CARGO_CONFIG.into())],
        }
    }

    /// Whether the crate needs a `build.rs` (`build = "build.rs"` in the
    /// manifest).
    pub fn has_build_script(&self) -> bool {
        matches!(self, Entry::Rp2040)
    }

    /// The `rust-version` the generated crate declares: the resolver's
    /// fallback keeps the HAL's dependencies within it.
    pub fn rust_version(&self) -> &'static str {
        match self {
            Entry::Rp2040 => "1.89",
            // the pinned AVR nightly is a 1.88 pre-release: its
            // dependencies must resolve as for 1.87
            Entry::Arduino(_) => arduino::RUST_VERSION,
        }
    }

    /// The `[features]` line and the `[dependencies]` lines of the target
    /// feature.
    pub fn cargo_sections(&self, o: &CodegenOptions, plan: &AdapterPlan) -> (String, String) {
        match self {
            Entry::Rp2040 => rp2040::cargo_sections(o, plan),
            Entry::Arduino(b) => arduino::cargo_sections(b, o),
        }
    }
}
