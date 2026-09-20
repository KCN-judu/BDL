//! `bdl-manifest.json`: the versioned map from generated entities back to
//! BDL identities.  Everything a later tool needs to relate a Rust symbol,
//! a state slot, an input slot or an output slot to the design that
//! produced it — and, through `path`, to the expression inside a
//! realization.

use crate::names;
use bdl_check::pretty;
use bdl_exec_ir::bounds::{self, Bound, Shape};
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
    /// What the program's collections need of a target's memory; absent
    /// when it carries no list (docs/spec/deployment-capacity.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collections: Option<CollectionsEntry>,
    pub clocks: Vec<ClockEntry>,
    pub concepts: Vec<ConceptEntry>,
    pub inputs: Vec<InputEntry>,
    pub decls: Vec<DeclEntry>,
    pub cells: Vec<CellEntry>,
    pub outputs: Vec<OutputEntry>,
    /// Declarations inlined away (relationships with inputs).
    pub functions: Vec<FunctionEntry>,
    /// Machine sinks: the raw command each realised output's device
    /// binding receives (docs/architecture/output-realization.md).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sinks: Vec<SinkEntry>,
    /// The platform adapter generated beside the core, when a target was
    /// given (docs/architecture/embedded-adapter.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter: Option<AdapterEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterEntry {
    /// The board id and its family (the target entry).
    pub board: String,
    pub family: String,
    /// The Rust target triple the firmware is built for, the Cargo feature
    /// and binary suffix, the toolchain it needs beyond the pinned one (if
    /// any), and the build command, verbatim.
    pub triple: String,
    pub feature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub toolchain: Option<String>,
    pub build: String,
    pub tick_micros: u64,
    /// Activation period in ticks per clock slot, in slot order.
    pub periods: Vec<u64>,
    /// The collection arena in bytes, when the core carries lists.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arena_bytes: Option<u64>,
    /// One per machine sink, in sink order.
    pub bindings: Vec<AdapterBindingEntry>,
    /// One per provider, in input-slot order (the input half).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<AdapterSourceEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterSourceEntry {
    pub slot: u32,
    pub device_id: u64,
    pub symbol: String,
    pub profile: String,
    /// The reader / source trait: `level`.
    pub kind: String,
    /// The board resource the placement assigned, as the board names it.
    pub resource: String,
    pub capability: String,
    /// The target's peripheral for it, in the target's words.
    pub peripheral: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterBindingEntry {
    pub slot: u32,
    pub device_id: u64,
    pub symbol: String,
    pub profile: String,
    /// The numeric policy / sink trait: `pwm_duty8` or `level`.
    pub kind: String,
    /// The board resource the placement assigned, as the board names it.
    pub resource: String,
    pub capability: String,
    /// The target's peripheral for it, in the target's words.
    pub peripheral: String,
}

/// The static bounds of the lists a program carries: per state cell the
/// bound of its outermost list, the list-typed inputs (the platform
/// bounds those), and byte estimates as the core stores values (`None`
/// when not bounded by the design).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionsEntry {
    pub cells: Vec<CellCapacityEntry>,
    /// Input slots of list type.
    pub input_slots: Vec<u32>,
    pub state_bytes_max: Option<u64>,
    pub tick_bytes_max: Option<u64>,
    /// Any remembered collection the design grows without bound.
    pub unbounded: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellCapacityEntry {
    pub slot: u32,
    pub bound: Bound,
    pub bytes: Option<u64>,
}

fn collections_entry(ir: &ExecIr) -> Option<CollectionsEntry> {
    if !ir.uses_lists() {
        return None;
    }
    let b = bounds::analyse(ir);
    let outer = |s: &Shape| match s {
        Shape::List { bound, .. } => *bound,
        s if s.is_unbounded() => Bound::Unbounded,
        s if s.depends_on_input() => Bound::Input,
        _ => Bound::Finite { elements: 0 },
    };
    fn sum(mut it: impl Iterator<Item = Option<u64>>) -> Option<u64> {
        it.try_fold(0u64, |a, x| Some(a.saturating_add(x?)))
    }
    Some(CollectionsEntry {
        cells: ir
            .cells
            .iter()
            .zip(&b.cells)
            .map(|(c, s)| CellCapacityEntry {
                slot: c.slot.0,
                bound: outer(s),
                bytes: s.bytes(&c.ty),
            })
            .collect(),
        input_slots: ir
            .inputs
            .iter()
            .filter(|i| {
                ir.decl(i.decl)
                    .is_some_and(|d| bdl_exec_ir::ty_uses_lists(&d.ty))
            })
            .map(|i| i.slot.0)
            .collect(),
        state_bytes_max: sum(ir.cells.iter().zip(&b.cells).map(|(c, s)| s.bytes(&c.ty))),
        tick_bytes_max: sum(ir.decls.iter().zip(&b.decls).map(|(d, s)| s.bytes(&d.ty))),
        unbounded: b.cells.iter().chain(&b.decls).any(Shape::is_unbounded),
    })
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
pub struct SinkEntry {
    pub slot: u32,
    pub device_id: u64,
    pub device_name: String,
    pub symbol: String,
    pub output_id: u64,
    pub driver_decl_id: u64,
    pub profile: String,
    /// The raw command type.
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionEntry {
    pub decl_id: u64,
    pub name: String,
    pub ty: String,
}

pub fn manifest(
    ir: &ExecIr,
    package: &str,
    generator: &str,
    plan: Option<&crate::adapter::AdapterPlan>,
    entry: Option<&crate::targets::Entry>,
) -> Result<Manifest, crate::emit::EmitError> {
    let adapter = match (plan, entry) {
        (Some(plan), Some(entry)) => {
            let mut bindings = Vec::new();
            for b in &plan.sinks {
                let peripheral = entry.peripheral(b)?;
                bindings.push(AdapterBindingEntry {
                    slot: b.slot.0,
                    device_id: b.device.raw(),
                    symbol: names::command(b.device),
                    profile: b.profile.0.clone(),
                    kind: match b.kind {
                        crate::adapter::SinkKind::PwmDuty8 => "pwm_duty8".into(),
                        crate::adapter::SinkKind::Level => "level".into(),
                    },
                    resource: b.resource.clone(),
                    capability: b.kind.capability().into(),
                    peripheral: peripheral.describe,
                });
            }
            let mut sources = Vec::new();
            for b in &plan.providers {
                let peripheral = entry.source_peripheral(b)?;
                sources.push(AdapterSourceEntry {
                    slot: b.slot.0,
                    device_id: b.device.raw(),
                    symbol: names::reading(b.device),
                    profile: b.profile.0.clone(),
                    kind: match b.kind {
                        crate::adapter::SourceKind::LevelPullDown
                        | crate::adapter::SourceKind::LevelPullUp => "level".into(),
                    },
                    resource: b.resource.clone(),
                    capability: b.kind.capability().into(),
                    peripheral: peripheral.describe,
                });
            }
            Some(AdapterEntry {
                board: plan.board.clone(),
                family: plan.family.clone(),
                triple: entry.triple().into(),
                feature: entry.feature(),
                toolchain: entry.toolchain().map(str::to_owned),
                build: entry.build_command(package),
                tick_micros: plan.tick_micros,
                periods: plan.periods.clone(),
                arena_bytes: plan.arena_bytes,
                bindings,
                sources,
            })
        }
        _ => None,
    };
    Ok(Manifest {
        manifest_version: MANIFEST_VERSION,
        generator: generator.into(),
        design: ir.name.clone(),
        package: package.into(),
        exec_ir_version: ir.version,
        has_domains: ir.has_domains,
        requires_allocator: ir.uses_lists(),
        collections: collections_entry(ir),
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
        sinks: ir
            .sinks
            .iter()
            .map(|s| SinkEntry {
                slot: s.slot.0,
                device_id: s.device.raw(),
                device_name: s.device_name.clone(),
                symbol: names::command(s.device),
                output_id: s.output.raw(),
                driver_decl_id: ir.decl(s.driver).map(|d| d.id.raw()).unwrap_or(u64::MAX),
                profile: s.profile.0.clone(),
                ty: pretty::kernel(&s.raw),
            })
            .collect(),
        adapter,
    })
}
