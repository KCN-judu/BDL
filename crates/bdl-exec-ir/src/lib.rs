//! The executable IR: what is left of a Design IR once everything a backend
//! does not need has been lowered away.
//!
//! * No closures.  Formula lambdas and declaration-level functions are
//!   inlined at their (saturated) application sites as `Let`s, and a
//!   function passed to a library combinator is inlined where the
//!   combinator applies it; a design that needs a closure *value* at
//!   runtime is refused by lowering, not half-supported here.  The one
//!   binding form besides `Let` is [`ExecExpr::Fold`], whose step is a
//!   first-order expression over two locals — the kernel's list recursor
//!   with its function argument already inlined.
//! * Lists and pairs stay structured values (`Vec`, tuples in the generated
//!   core; `Value::List`/`Value::Pair` in the interpreter): lowering them
//!   further would be a second interpretation of collection behaviour.
//! * Every temporal form is a **state slot** (`ReadCell` in expressions,
//!   a `CellPlan` with its writer domain and write operand in the plan).
//! * Every clock domain is a dense **clock slot**; every unresolved
//!   declaration an **input slot**; every physical output an **output
//!   slot** projected from its single driver.
//! * Every realised output additionally has a **machine sink**
//!   ([`SinkPlan`]): the raw command a chosen realization profile's encoder
//!   makes of the driver's value, evaluated after the outputs in the
//!   driver's own domain.  Sinks are downstream of behavior — no
//!   declaration, cell or output reads one — so a design lowered with or
//!   without them has the same `values` and `outputs` at every tick.
//! * Declarations are listed in the order they are evaluated — the
//!   reference evaluator's traversal order, so that when several
//!   declarations fail at one tick both engines name the same one.
//!
//! [`interp`] executes this IR with the reference evaluator's `Value`s: it
//! is the bridge in the differential tests (reference ↔ exec IR ↔ generated
//! Rust) and a debugging aid, never a second semantics.

#![forbid(unsafe_code)]

pub mod bounds;
pub mod interp;

use bdl_ir::{Scalar, Ty};
use bdl_model::{
    ClockId, DeclId, DeviceId, Dim, InputProfileId, OutputId, OutputProfileId, SemanticId,
};
use bdl_reactive::StateCellId;
use serde::{Deserialize, Serialize};

/// Bumped on any change to this representation.
pub const EXEC_IR_VERSION: u32 = 4;

macro_rules! slot {
    ($(#[$m:meta])* $name:ident($t:ty)) => {
        $(#[$m])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub $t);
    };
}

slot!(
    /// Dense clock-domain index.
    ClockSlot(u16)
);
slot!(
    /// Dense index of an unresolved declaration among the inputs.
    InputSlot(u32)
);
slot!(
    /// Dense index of a temporal state cell.
    StateSlot(u32)
);
slot!(
    /// Dense index of a physical output.
    OutputSlot(u32)
);
slot!(
    /// Dense index of a machine sink (a realised output's raw command).
    SinkSlot(u32)
);
slot!(
    /// Position of a declaration in the evaluation plan.
    DeclIndex(u32)
);
slot!(
    /// A `Let`-bound local, unique within one expression tree.
    LocalId(u32)
);

/// When a declaration is evaluated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Activation {
    /// At every tick its domain is active.
    Domain { clock: ClockSlot },
    /// Domain-agnostic: whenever any domain is active, and at every tick of
    /// a design that has no domains at all (DI-16).
    Agnostic,
}

/// A concept whose values the program carries: the nominal boundary the
/// generated code keeps as a newtype over the representation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptPlan {
    pub id: SemanticId,
    pub name: String,
    pub representation: Ty,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockPlan {
    pub slot: ClockSlot,
    pub id: ClockId,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputPlan {
    pub slot: InputSlot,
    pub decl: DeclIndex,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DeclKind {
    /// Unresolved: its value is supplied per tick.
    Input {
        slot: InputSlot,
    },
    Computed {
        body: ExecExpr,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeclPlan {
    pub index: DeclIndex,
    pub id: DeclId,
    pub name: String,
    /// The declaration's type view; always first-order data here.
    pub ty: Ty,
    pub activation: Activation,
    pub kind: DeclKind,
}

/// One temporal state cell.  Read as `ReadCell { slot, init }` wherever the
/// `delay`/`sync` stood; written — into the *next* state — at every tick
/// its `writer` domain is active, with `operand` evaluated in read mode.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellPlan {
    pub slot: StateSlot,
    pub cell: StateCellId,
    pub owner: DeclIndex,
    pub ty: Ty,
    /// The owner's domain for `delay`, the source domain for `sync`.
    pub writer: ClockSlot,
    pub operand: ExecExpr,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputPlan {
    pub slot: OutputSlot,
    pub id: OutputId,
    pub name: String,
    pub driver: DeclIndex,
    pub ty: Ty,
}

/// A machine sink: the raw command one device binding makes of its
/// output's value (`lowerΩ`/`lowerβ`, docs/architecture/output-realization.md).
/// `command` reads the driver and nothing else; it is due exactly when the
/// driver is, and carries no state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SinkPlan {
    pub slot: SinkSlot,
    /// The device binding this command is for.
    pub device: DeviceId,
    pub device_name: String,
    /// The logical output it realises, and that output's driver.
    pub output: OutputId,
    pub driver: DeclIndex,
    pub profile: OutputProfileId,
    /// The raw command type — sem-free data.
    pub raw: Ty,
    pub command: ExecExpr,
}

/// A provider: the value one device binding makes of its raw reading for
/// a Source (`Provision.one`, docs/architecture/embedded-adapter.md § The
/// input half).  `provide` reads the raw local and nothing else — no
/// declaration, no memory, no other domain — and constructs exactly the
/// Source's concept; the interpreter evaluates it before the tick and
/// hands the result to the Source's input slot.  A raw reading the
/// adapter did not take leaves the slot empty, and the Source's domain
/// must not be due then.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderPlan {
    pub slot: InputSlot,
    /// The Source this provides.
    pub decl: DeclIndex,
    pub device: DeviceId,
    pub device_name: String,
    pub profile: InputProfileId,
    /// The raw reading type — sem-free data.
    pub raw: Ty,
    /// The local `provide` reads the raw reading from.
    pub local: LocalId,
    pub provide: ExecExpr,
}

/// A declaration lowered away entirely (a function inlined at its uses).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionPlan {
    pub id: DeclId,
    pub name: String,
    pub ty: Ty,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecIr {
    pub version: u32,
    pub name: String,
    /// Whether the design has any clock domain (decides when agnostic
    /// declarations run).
    pub has_domains: bool,
    /// Every concept mentioned by a type below, in `SemanticId` order.
    pub concepts: Vec<ConceptPlan>,
    /// In `ClockId` order.
    pub clocks: Vec<ClockPlan>,
    /// In `DeclId` order.
    pub inputs: Vec<InputPlan>,
    /// In evaluation order.
    pub decls: Vec<DeclPlan>,
    /// In `StateCellId` order, which is also the write order.
    pub cells: Vec<CellPlan>,
    /// In `OutputId` order.
    pub outputs: Vec<OutputPlan>,
    /// In `DeclId` order.
    pub functions: Vec<FunctionPlan>,
    /// In `DeviceId` order; empty when no output has a realization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sinks: Vec<SinkPlan>,
    /// In input-slot order; empty when no Source has a provider.  A slot
    /// without a provider is still supplied by the caller (a simulation
    /// input); one with a provider is supplied by [`interp::provide`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub providers: Vec<ProviderPlan>,
}

impl ExecIr {
    pub fn decl(&self, i: DeclIndex) -> Option<&DeclPlan> {
        self.decls.get(i.0 as usize)
    }
    pub fn clock_slot(&self, id: ClockId) -> Option<ClockSlot> {
        self.clocks.iter().find(|c| c.id == id).map(|c| c.slot)
    }
    pub fn decl_index(&self, id: DeclId) -> Option<DeclIndex> {
        self.decls.iter().find(|d| d.id == id).map(|d| d.index)
    }
    /// Whether a declaration with this activation is evaluated at a tick
    /// with these active slots.
    pub fn due(&self, activation: Activation, active: &[ClockSlot]) -> bool {
        match activation {
            Activation::Domain { clock } => active.contains(&clock),
            Activation::Agnostic => !active.is_empty() || !self.has_domains,
        }
    }
}

/// Saturated primitives.  Dimensions are carried for the interpreter and
/// the manifest; the generated core needs none of them at runtime.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum PrimOp {
    Add {
        dim: Dim,
    },
    Sub {
        dim: Dim,
    },
    Mul {
        d1: Dim,
        d2: Dim,
    },
    Div {
        d1: Dim,
        d2: Dim,
    },
    Lt,
    Eq,
    Not,
    And,
    Or,
    /// Strict: all three operands are evaluated first, as in the reference.
    Ite {
        ty: Ty,
    },
    None {
        ty: Ty,
    },
    Some {
        ty: Ty,
    },
    IsSome {
        ty: Ty,
    },
    GetD {
        ty: Ty,
    },
    /// The empty list of `ty`.
    Nil {
        ty: Ty,
    },
    Cons {
        ty: Ty,
    },
    /// The length as a dimensionless quantity.
    Length {
        ty: Ty,
    },
    /// `take k xs`: `k` a dimensionless quantity read as a count.
    Take {
        ty: Ty,
    },
    Drop {
        ty: Ty,
    },
    Reverse {
        ty: Ty,
    },
    Head {
        ty: Ty,
    },
    ToList {
        ty: Ty,
    },
    Pair {
        fst: Ty,
        snd: Ty,
    },
    Fst {
        fst: Ty,
        snd: Ty,
    },
    Snd {
        fst: Ty,
        snd: Ty,
    },
}

impl PrimOp {
    pub fn arity(&self) -> usize {
        match self {
            PrimOp::None { .. } | PrimOp::Nil { .. } => 0,
            PrimOp::Not
            | PrimOp::Some { .. }
            | PrimOp::IsSome { .. }
            | PrimOp::Length { .. }
            | PrimOp::Reverse { .. }
            | PrimOp::Head { .. }
            | PrimOp::ToList { .. }
            | PrimOp::Fst { .. }
            | PrimOp::Snd { .. } => 1,
            PrimOp::Add { .. }
            | PrimOp::Sub { .. }
            | PrimOp::Mul { .. }
            | PrimOp::Div { .. }
            | PrimOp::Lt
            | PrimOp::Eq
            | PrimOp::And
            | PrimOp::Or
            | PrimOp::GetD { .. }
            | PrimOp::Cons { .. }
            | PrimOp::Take { .. }
            | PrimOp::Drop { .. }
            | PrimOp::Pair { .. } => 2,
            PrimOp::Ite { .. } => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "expr", rename_all = "snake_case")]
pub enum ExecExpr {
    Bool {
        value: bool,
    },
    Nat {
        value: u64,
    },
    Quantity {
        dim: Dim,
        value: Scalar,
    },
    Local {
        id: LocalId,
    },
    Let {
        local: LocalId,
        value: Box<ExecExpr>,
        body: Box<ExecExpr>,
    },
    ReadDecl {
        decl: DeclIndex,
    },
    /// `mk s e`: the nominal boundary, kept.
    Wrap {
        sem: SemanticId,
        e: Box<ExecExpr>,
    },
    /// `rep e`.
    Unwrap {
        e: Box<ExecExpr>,
    },
    Prim {
        op: PrimOp,
        args: Vec<ExecExpr>,
    },
    /// The cell's committed value, else `init` evaluated now.
    ReadCell {
        slot: StateSlot,
        init: Box<ExecExpr>,
    },
    /// The list recursor with its step inlined: `fold f z [x₁, …, xₙ] =
    /// f x₁ (… (f xₙ z))`, computed as finite iteration from the last
    /// element with `elem` bound to the element and `acc` to the value so
    /// far.  Never general recursion: exactly one step per element.
    Fold {
        elem: LocalId,
        acc: LocalId,
        step: Box<ExecExpr>,
        init: Box<ExecExpr>,
        list: Box<ExecExpr>,
    },
}

impl ExecExpr {
    pub fn quantity(dim: Dim, value: f64) -> ExecExpr {
        ExecExpr::Quantity {
            dim,
            value: Scalar(value),
        }
    }
    pub fn prim(op: PrimOp, args: impl IntoIterator<Item = ExecExpr>) -> ExecExpr {
        ExecExpr::Prim {
            op,
            args: args.into_iter().collect(),
        }
    }
    pub fn read(decl: DeclIndex) -> ExecExpr {
        ExecExpr::ReadDecl { decl }
    }
    pub fn wrap(sem: SemanticId, e: ExecExpr) -> ExecExpr {
        ExecExpr::Wrap {
            sem,
            e: Box::new(e),
        }
    }
    pub fn unwrap(e: ExecExpr) -> ExecExpr {
        ExecExpr::Unwrap { e: Box::new(e) }
    }
    pub fn let_(local: LocalId, value: ExecExpr, body: ExecExpr) -> ExecExpr {
        ExecExpr::Let {
            local,
            value: Box::new(value),
            body: Box::new(body),
        }
    }
    pub fn read_cell(slot: StateSlot, init: ExecExpr) -> ExecExpr {
        ExecExpr::ReadCell {
            slot,
            init: Box::new(init),
        }
    }

    /// Every declaration read anywhere in the expression, in traversal order.
    pub fn reads(&self, out: &mut Vec<DeclIndex>) {
        match self {
            ExecExpr::Bool { .. }
            | ExecExpr::Nat { .. }
            | ExecExpr::Quantity { .. }
            | ExecExpr::Local { .. } => {}
            ExecExpr::Let { value, body, .. } => {
                value.reads(out);
                body.reads(out);
            }
            ExecExpr::ReadDecl { decl } => out.push(*decl),
            ExecExpr::Wrap { e, .. } | ExecExpr::Unwrap { e } => e.reads(out),
            ExecExpr::Prim { args, .. } => args.iter().for_each(|a| a.reads(out)),
            ExecExpr::ReadCell { init, .. } => init.reads(out),
            ExecExpr::Fold {
                step, init, list, ..
            } => {
                step.reads(out);
                init.reads(out);
                list.reads(out);
            }
        }
    }

    /// Whether the expression mentions a list type anywhere (a `Nil`,
    /// `Cons`, … operator or a fold) — what decides whether a generated
    /// core needs an allocator.
    pub fn uses_lists(&self) -> bool {
        match self {
            ExecExpr::Bool { .. }
            | ExecExpr::Nat { .. }
            | ExecExpr::Quantity { .. }
            | ExecExpr::Local { .. }
            | ExecExpr::ReadDecl { .. } => false,
            ExecExpr::Let { value, body, .. } => value.uses_lists() || body.uses_lists(),
            ExecExpr::Wrap { e, .. } | ExecExpr::Unwrap { e } => e.uses_lists(),
            ExecExpr::Prim { op, args } => {
                matches!(
                    op,
                    PrimOp::Nil { .. }
                        | PrimOp::Cons { .. }
                        | PrimOp::Length { .. }
                        | PrimOp::Take { .. }
                        | PrimOp::Drop { .. }
                        | PrimOp::Reverse { .. }
                        | PrimOp::Head { .. }
                        | PrimOp::ToList { .. }
                ) || args.iter().any(ExecExpr::uses_lists)
            }
            ExecExpr::ReadCell { init, .. } => init.uses_lists(),
            ExecExpr::Fold { .. } => true,
        }
    }
}

/// Whether a type mentions a list anywhere.
pub fn ty_uses_lists(t: &Ty) -> bool {
    match t {
        Ty::List { .. } => true,
        Ty::Opt { inner } => ty_uses_lists(inner),
        Ty::Prod { fst, snd } => ty_uses_lists(fst) || ty_uses_lists(snd),
        Ty::Arr { dom, cod } => ty_uses_lists(dom) || ty_uses_lists(cod),
        Ty::Bool | Ty::Nat | Ty::Unit | Ty::Q { .. } | Ty::Sem { .. } => false,
    }
}

impl ExecIr {
    /// Whether the program carries list values anywhere — in a
    /// declaration, cell, output, concept representation or expression —
    /// and so needs an allocator on its target.
    pub fn uses_lists(&self) -> bool {
        self.decls.iter().any(|d| {
            ty_uses_lists(&d.ty)
                || matches!(&d.kind, DeclKind::Computed { body } if body.uses_lists())
        }) || self
            .cells
            .iter()
            .any(|c| ty_uses_lists(&c.ty) || c.operand.uses_lists())
            || self.outputs.iter().any(|o| ty_uses_lists(&o.ty))
            || self
                .concepts
                .iter()
                .any(|c| ty_uses_lists(&c.representation))
    }
}
