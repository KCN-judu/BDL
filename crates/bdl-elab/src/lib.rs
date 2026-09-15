//! Elaboration: from what the designer authored to the kernel's objects.
//!
//! * [`design::elaborate_design`] — concepts → `ConceptBinding` (Θ), mapping
//!   signatures → `Interface` (`sem A₁ → … → sem B`), formulas → realizations.
//! * [`formula::elaborate_formula`] — one formula → one Core `Expr`:
//!   `λ x₁ … xₙ. mk B (…)` where every input is observed through `rep` and
//!   the result is constructed with `mk`, which the declaration's own
//!   signature grants.  Units become scaled dimensioned literals; the
//!   dimension algebra itself is the kernel's (`Prim::ty`).
//!
//! Pure: `(snapshot, mapping, source) → (Expr, span map, diagnostics)`.
//! Nothing here mutates the project.  Diagnostics are accumulated; an
//! erroneous sub-expression yields an error type that silences cascades.

#![forbid(unsafe_code)]

pub mod design;
pub mod formula;
pub mod names;
pub mod units;

pub use design::{
    elaborate_design, elaborate_interface, representation_ty, Elaboration, MappingElab,
    RealizationOutcome,
};
pub use formula::{elaborate_formula, Realized};
