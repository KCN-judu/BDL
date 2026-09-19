//! The designer-facing role of a relationship (ADR-0032), and who provides
//! a Source.
//!
//! The role is [`bdl_model::RelationshipRole`] — Source, Rule or Value —
//! read off the *committed* authoring state of the snapshot: a definition
//! draft that is not committed changes nothing (ADR-0030 keeps drafts as
//! authoring state, not as definitions).  The compiler states the same
//! role in `MappingAnalysis.role`; this module is the IDE's reading of it
//! for hover and Explain, plus the one fact those add: for a Source, the
//! boundary that provides it.
//!
//! FV Phase 12 (`BDL/Surface/UnitDomain.lean`): `Source Δ d` is "no
//! realization", `SimulationInput` is `Source ∧ UnitDomain`, and
//! `resolved_not_source` says a realized `() -> B` never consults the
//! environment.  Inside a component body the environment of the design is
//! the instance that binds its ports: a declaration that backs a required
//! or parameter port is a Source *of the body*, provided through the port
//! ([`Provider::Port`]); the system's flattening realizes the instance's
//! copy by a binding (a Value) or leaves it unresolved (a Source of the
//! system, FV Theorem H).

use bdl_ide_db::AnalysisSnapshot;
use bdl_model::DeclId;
pub use bdl_model::RelationshipRole;
use bdl_system::PortKind;

/// Who provides a Source's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Provider {
    /// The environment of the design: a simulation input; at deployment, a
    /// device (ISS-0016, PRP-0001).
    Environment,
    /// The port the declaration backs in its component: the instance's
    /// binding (required) or its argument (parameter) supplies the value.
    Port(PortKind),
}

impl Provider {
    /// The words hover and Explain use.
    pub fn word(self) -> &'static str {
        match self {
            Provider::Environment => "environment",
            Provider::Port(PortKind::Required) => "the instance's binding of this required port",
            Provider::Port(PortKind::Parameter) => "the instance's argument for this parameter",
            Provider::Port(PortKind::Provided) => "this provided port",
        }
    }
}

/// The role of `mapping` at the snapshot's committed revision, or `None`
/// when there is no such relationship.
pub fn relationship_role(snapshot: &AnalysisSnapshot, mapping: DeclId) -> Option<RelationshipRole> {
    snapshot
        .committed()
        .design
        .mappings
        .get(&mapping)
        .map(bdl_model::MappingBlock::role)
}

/// The port `mapping` backs, when the snapshot is a component body's (or
/// the flattening of one): a fact beside the role, never a role.
pub fn port_backed(snapshot: &AnalysisSnapshot, mapping: DeclId) -> Option<PortKind> {
    snapshot.port_of(mapping)
}

/// Who provides `mapping`'s value: `Some` for a Source only.
pub fn provider(snapshot: &AnalysisSnapshot, mapping: DeclId) -> Option<Provider> {
    match relationship_role(snapshot, mapping)? {
        RelationshipRole::Source => Some(match port_backed(snapshot, mapping) {
            Some(kind) => Provider::Port(kind),
            None => Provider::Environment,
        }),
        RelationshipRole::Rule | RelationshipRole::Value => None,
    }
}
