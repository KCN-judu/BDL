//! The executable IR: what is left of a Design IR once everything a backend
//! does not need has been lowered away.
//!
//! * No binders.  Formula lambdas and declaration-level functions are
//!   inlined at their (saturated) application sites as `Let`s; a design
//!   that needs a closure at runtime is refused by lowering, not
//!   half-supported here.
//! * Every temporal form is a **state slot** (`ReadCell` in expressions,
//!   a `CellPlan` with its writer domain and write operand in the plan).
//! * Every clock domain is a dense **clock slot**; every unresolved
//!   declaration an **input slot**; every physical output an **output
//!   slot** projected from its single driver.
//! * Declarations are listed in the order they are evaluated — the
//!   reference evaluator's traversal order, so that when several
//!   declarations fail at one tick both engines name the same one.
//!
//! [`interp`] executes this IR with the reference evaluator's `Value`s: it
//! is the bridge in the differential tests (reference ↔ exec IR ↔ generated
//! Rust) and a debugging aid, never a second semantics.

#![forbid(unsafe_code)]

pub mod interp;

use bdl_ir::{Scalar, Ty};
use bdl_model::{ClockId, DeclId, Dim, OutputId, SemanticId};
use bdl_reactive::StateCellId;
use serde::{Deserialize, Serialize};

/// Bumped on any change to this representation.
pub const EXEC_IR_VERSION: u32 = 1;

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
}

impl PrimOp {
    pub fn arity(&self) -> usize {
        match self {
            PrimOp::None { .. } => 0,
            PrimOp::Not | PrimOp::Some { .. } | PrimOp::IsSome { .. } => 1,
            PrimOp::Add { .. }
            | PrimOp::Sub { .. }
            | PrimOp::Mul { .. }
            | PrimOp::Div { .. }
            | PrimOp::Lt
            | PrimOp::Eq
            | PrimOp::And
            | PrimOp::Or
            | PrimOp::GetD { .. } => 2,
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
        }
    }
}
