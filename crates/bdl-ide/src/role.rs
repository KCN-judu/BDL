//! The designer-facing role of a relationship (ADR-0032).
//!
//! A relationship is one ordinary declaration whatever its role; the role is
//! derived, never stored, from three existing facts: its canonical domain,
//! its realization state, and whether it backs a port.  It corresponds to
//! FV Phase 12 (`BDL/Surface/UnitDomain.lean`): `Source Δ d` is
//! "no realization", `SimulationInput` is `Source ∧ UnitDomain`, and
//! `resolved_not_source` says a realized `() -> B` never consults the
//! environment.  Production narrows the word *Source* once more, to the
//! environment boundary of the design on screen: a declaration that backs a
//! port presents the port's role (required / provided / parameter), because
//! its value comes from a binding, not from the environment.

use bdl_ide_db::AnalysisSnapshot;
use bdl_model::DeclId;
use bdl_system::PortKind;

/// What a relationship is, as a designer reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RelationshipRole {
    /// A value entering the behavior model from the environment: unit
    /// domain, no realization, not a port.  Observed once per activation;
    /// never a call.
    Source,
    /// A value transformed inside the behavior model — including a resolved
    /// `() -> B` that computes internally (memory, a constant): the domain
    /// shape alone makes nothing a Source.
    Mapping,
    /// A component body's declaration that backs a port: its value is the
    /// binding's (required, parameter) or is offered by it (provided).
    Port(PortKind),
}

impl RelationshipRole {
    /// The word Explain and hover use.
    pub fn word(self) -> &'static str {
        match self {
            RelationshipRole::Source => "Source",
            RelationshipRole::Mapping => "Mapping",
            RelationshipRole::Port(PortKind::Required) => "required port",
            RelationshipRole::Port(PortKind::Provided) => "provided port",
            RelationshipRole::Port(PortKind::Parameter) => "parameter port",
        }
    }
}

/// The role of `mapping` in `snapshot`, or `None` when there is no such
/// relationship.
pub fn relationship_role(snapshot: &AnalysisSnapshot, mapping: DeclId) -> Option<RelationshipRole> {
    let m = snapshot.effective().design.mappings.get(&mapping)?;
    if let Some(kind) = snapshot.port_of(mapping) {
        return Some(RelationshipRole::Port(kind));
    }
    Some(if m.definition.is_none() && m.signature.is_unit_domain() {
        RelationshipRole::Source
    } else {
        RelationshipRole::Mapping
    })
}

/// How a Source's value is provided, in the words of the formal account:
/// the environment (`I d t`), or — once deployment realizes it — a device.
/// The IDE service knows no devices bound to relationships; it says
/// "environment".
pub fn provision(role: RelationshipRole) -> Option<&'static str> {
    match role {
        RelationshipRole::Source => Some("environment"),
        _ => None,
    }
}
