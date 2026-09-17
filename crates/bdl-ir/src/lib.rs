//! BDL intermediate representations.
//!
//! ```text
//! Surface Model (bdl-model)  ──elaboration──▶  Design IR  ──lowering──▶  Reactive Core IR
//! ```
//!
//! * [`ty`]     — the kernel's `Ty`: `bool | nat | arr | sem s | q d | opt`.
//! * [`expr`]   — the Reactive Core IR: the kernel's `Expr` and `Prim`.  This
//!   is the alignment point between the formal semantics, the simulator and
//!   the Rust code generator; surface constructs (`previous`, `hold`,
//!   contexts, priority…) do not exist here.
//! * [`design`] — the Design IR: declarations with interfaces and optional
//!   realizations, concept bindings, clock and output environments.
//!
//! The definitions transcribe `BDL_FV/BDL/Core/{Base,Interface,Decl,Clock,
//! Output}.lean`; see `docs/architecture/ir.md` for the deliberate deviations.

#![forbid(unsafe_code)]

pub mod design;
pub mod expr;
pub mod ty;

pub use design::{
    ClockEnv, ConceptBinding, Declaration, DesignIr, DriveEnv, Interface, OutputSpec, PropertyId,
};
pub use expr::{Expr, Prim, Scalar};
pub use ty::Ty;
