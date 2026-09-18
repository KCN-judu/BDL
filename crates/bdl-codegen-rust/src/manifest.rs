//! `bdl-manifest.json`: the versioned map from generated entities back to
//! BDL identities.  Everything a later tool needs to relate a Rust symbol,
//! a state slot, an input slot or an output slot to the design that
//! produced it — and, through `path`, to the expression inside a
//! realization.

use crate::names;
use bdl_check::pretty;
use bdl_exec_ir::{Activation, DeclKind, ExecIr};
use serde::{Deserialize, Serialize};

pub const MANIFEST_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub manifest_version: u32,
    pub generator: String,
    pub design: String,
    pub package: String,
    pub exec_ir_version: u32,
    pub has_domains: bool,
    /// The program carries list values: the core is built with the
    /// runtime's `collections` feature and its target needs an allocator.
    #[serde(default)]
    pub requires_allocator: bool,
    pub clocks: Vec<ClockEntry>,
    pub concepts: Vec<ConceptEntry>,
    pub inputs: Vec<InputEntry>,
    pub decls: Vec<DeclEntry>,
    pub cells: Vec<CellEntry>,
    pub outputs: Vec<OutputEntry>,
    /// Declarations inlined away (relationships with inputs).
    pub functions: Vec<FunctionEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockEntry {
    pub slot: u16,
    pub clock_id: u64,
    pub name: String,
    pub symbol: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptEntry {
    pub semantic_id: u64,
    pub name: String,
    pub symbol: String,
    pub representation: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputEntry {
    pub slot: u32,
    pub decl_id: u64,
    pub symbol: String,
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ActivationEntry {
    Domain { clock_slot: u16 },
    Agnostic,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclEntry {
    /// Position in the evaluation plan (and in `values` traces).
    pub index: u32,
    pub decl_id: u64,
    pub symbol: String,
    pub name: String,
    pub ty: String,
    pub activation: ActivationEntry,
    /// The input slot when the declaration is unresolved.
    pub input_slot: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellEntry {
    pub slot: u32,
    pub symbol: String,
    /// `StateCellId`: the owning declaration and the expression path of
    /// the `delay`/`sync` inside its realization.
    pub decl_id: u64,
    pub path: Vec<u8>,
    pub writer_clock_slot: u16,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputEntry {
    pub slot: u32,
    pub output_id: u64,
    pub symbol: String,
    pub name: String,
    pub driver_decl_id: u64,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionEntry {
    pub decl_id: u64,
    pub name: String,
    pub ty: String,
}

pub fn manifest(ir: &ExecIr, package: &str, generator: &str) -> Manifest {
    Manifest {
        manifest_version: MANIFEST_VERSION,
        generator: generator.into(),
        design: ir.name.clone(),
        package: package.into(),
        exec_ir_version: ir.version,
        has_domains: ir.has_domains,
        requires_allocator: ir.uses_lists(),
        clocks: ir
            .clocks
            .iter()
            .map(|c| ClockEntry {
                slot: c.slot.0,
                clock_id: c.id.raw(),
                name: c.name.clone(),
                symbol: names::clock(c.slot),
            })
            .collect(),
        concepts: ir
            .concepts
            .iter()
            .map(|c| ConceptEntry {
                semantic_id: c.id.raw(),
                name: c.name.clone(),
                symbol: names::concept(c.id),
                representation: pretty::kernel(&c.representation),
            })
            .collect(),
        inputs: ir
            .inputs
            .iter()
            .filter_map(|i| {
                let d = ir.decl(i.decl)?;
                Some(InputEntry {
                    slot: i.slot.0,
                    decl_id: d.id.raw(),
                    symbol: names::decl(d.id),
                    name: d.name.clone(),
                    ty: pretty::kernel(&d.ty),
                })
            })
            .collect(),
        decls: ir
            .decls
            .iter()
            .map(|d| DeclEntry {
                index: d.index.0,
                decl_id: d.id.raw(),
                symbol: names::decl(d.id),
                name: d.name.clone(),
                ty: pretty::kernel(&d.ty),
                activation: match d.activation {
                    Activation::Domain { clock } => ActivationEntry::Domain {
                        clock_slot: clock.0,
                    },
                    Activation::Agnostic => ActivationEntry::Agnostic,
                },
                input_slot: match d.kind {
                    DeclKind::Input { slot } => Some(slot.0),
                    DeclKind::Computed { .. } => None,
                },
            })
            .collect(),
        cells: ir
            .cells
            .iter()
            .map(|c| CellEntry {
                slot: c.slot.0,
                symbol: names::cell(c.slot),
                decl_id: c.cell.decl.raw(),
                path: c.cell.path.clone(),
                writer_clock_slot: c.writer.0,
                ty: pretty::kernel(&c.ty),
            })
            .collect(),
        outputs: ir
            .outputs
            .iter()
            .map(|o| OutputEntry {
                slot: o.slot.0,
                output_id: o.id.raw(),
                symbol: names::output(o.id),
                name: o.name.clone(),
                driver_decl_id: ir.decl(o.driver).map(|d| d.id.raw()).unwrap_or(u64::MAX),
                ty: pretty::kernel(&o.ty),
            })
            .collect(),
        functions: ir
            .functions
            .iter()
            .map(|f| FunctionEntry {
                decl_id: f.id.raw(),
                name: f.name.clone(),
                ty: pretty::kernel(&f.ty),
            })
            .collect(),
    }
}
