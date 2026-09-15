//! The Design IR: the kernel's environments (`Interface.lean`, `Decl.lean`,
//! `Clock.lean`, `Output.lean`).
//!
//! A design at this level is the tuple `(Θ, Δ, Κ, Ω, β)`: concept bindings,
//! declarations, clock assignment, sink specifications and drive edges.
//! Surface names are retained only as documentation for diagnostics.

use crate::expr::Expr;
use crate::ty::Ty;
use bdl_model::{ClockId, DeclId, OutputId, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Public commitments a declaration makes.  Atomic labels in the kernel; the
/// closed vocabulary is an enum here.  Discharge is a validation matter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyId {
    Monotone,
    Deterministic,
    Total,
    BoundedRange,
}

/// `DeclInterface`: what clients may depend on.  `expected_type` is frozen
/// under refinement; `commitments` only grow.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Interface {
    pub expected_type: Ty,
    #[serde(default)]
    pub commitments: Vec<PropertyId>,
}

impl Interface {
    /// `InterfaceRefines self new`: same type, commitments preserved.
    pub fn is_refined_by(&self, new: &Interface) -> bool {
        self.expected_type == new.expected_type
            && self.commitments.iter().all(|c| new.commitments.contains(c))
    }
}

/// `DesignDecl`: identity, interface, optional (write-once) realization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Declaration {
    pub id: DeclId,
    /// Display name, for diagnostics only.
    pub name: String,
    pub interface: Interface,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub realization: Option<Expr>,
}

impl Declaration {
    pub fn is_unresolved(&self) -> bool {
        self.realization.is_none()
    }
}

/// `Θ s = some R` together with the concept's display name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptBinding {
    pub id: SemanticId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representation: Option<Ty>,
}

/// `OutputSpec`: what a sink accepts and in which domain.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutputSpec {
    pub id: OutputId,
    pub name: String,
    pub accepts: Ty,
    pub clock: ClockId,
}

/// `Κ : DeclId → Option ClockId`.  Absent means domain-agnostic.
pub type ClockEnv = BTreeMap<DeclId, ClockId>;
/// `β : DeclId → Option OutputId`, the drive edges.
pub type DriveEnv = BTreeMap<DeclId, OutputId>;

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DesignIr {
    pub concepts: BTreeMap<SemanticId, ConceptBinding>,
    pub decls: BTreeMap<DeclId, Declaration>,
    pub clocks: ClockEnv,
    /// Display names of domains, for diagnostics only.
    #[serde(default)]
    pub clock_names: BTreeMap<ClockId, String>,
    pub outputs: BTreeMap<OutputId, OutputSpec>,
    /// Display names of every surface output, including the open ones that
    /// have no `OutputSpec` yet; for diagnostics only.
    #[serde(default)]
    pub output_names: BTreeMap<OutputId, String>,
    pub drives: DriveEnv,
}

impl DesignIr {
    /// `Δ.tyView`: the only projection typing may consult.
    pub fn ty_view(&self, d: DeclId) -> Option<&Ty> {
        self.decls.get(&d).map(|h| &h.interface.expected_type)
    }

    /// `Δ.realizationOf`.
    pub fn realization_of(&self, d: DeclId) -> Option<&Expr> {
        self.decls.get(&d).and_then(|h| h.realization.as_ref())
    }

    /// `Θ s`.
    pub fn representation_of(&self, s: SemanticId) -> Option<&Ty> {
        self.concepts
            .get(&s)
            .and_then(|c| c.representation.as_ref())
    }

    /// `ConceptEnv.WF`: every bound representation is sem-free data.
    pub fn concepts_well_formed(&self) -> bool {
        self.concepts.values().all(|c| {
            c.representation
                .as_ref()
                .is_none_or(|r| r.is_sem_free() && r.is_data())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_refinement_freezes_type_and_grows_commitments() {
        let base = Interface {
            expected_type: Ty::Bool,
            commitments: vec![PropertyId::Monotone],
        };
        let more = Interface {
            expected_type: Ty::Bool,
            commitments: vec![PropertyId::Total, PropertyId::Monotone],
        };
        let retyped = Interface {
            expected_type: Ty::Nat,
            commitments: vec![PropertyId::Monotone],
        };
        let dropped = Interface {
            expected_type: Ty::Bool,
            commitments: vec![],
        };
        assert!(base.is_refined_by(&more));
        assert!(!base.is_refined_by(&retyped));
        assert!(!base.is_refined_by(&dropped));
    }

    #[test]
    fn ty_view_ignores_realization() {
        let d = DeclId::from_raw(0);
        let mut ir = DesignIr::default();
        ir.decls.insert(
            d,
            Declaration {
                id: d,
                name: "x".into(),
                interface: Interface {
                    expected_type: Ty::Bool,
                    commitments: vec![],
                },
                realization: None,
            },
        );
        assert_eq!(ir.ty_view(d), Some(&Ty::Bool));
        assert_eq!(ir.realization_of(d), None);
        assert!(ir.concepts_well_formed());
    }
}
