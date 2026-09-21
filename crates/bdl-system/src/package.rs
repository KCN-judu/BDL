//! Hierarchy is packaging (FV `toComponent`, D-72): a flattened system is
//! an ordinary design, hence again a component body.  No recursive system
//! type, no nested evaluation.

use crate::analyze::analyze_system;
use crate::ids::{ComponentId, PortId};
use crate::model::*;
use bdl_diagnostics::Diagnostic;
use bdl_model::{ClockId, ConceptId, DeclId, OutputId};
use std::collections::BTreeMap;

/// A port chosen for the package, by the flat declaration it exposes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackagePort {
    /// An inner port (of an instance) or an inner base relationship.
    pub decl: PackageDecl,
    pub kind: PortKind,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageDecl {
    Port(PortRef),
    Base(DeclId),
}

/// What the package exposes and what stays shared with the enclosing
/// system.  Everything not listed is private to the package.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PackageInterface {
    pub ports: Vec<PackagePort>,
    /// Inner base clocks that become parameters of the package.
    pub clock_params: Vec<ClockId>,
    /// Inner base concept → the enclosing system's concept it stands for.
    pub shared_concepts: BTreeMap<ConceptId, ConceptId>,
    /// Inner base sink → the enclosing system's sink it stands for.
    pub external_outputs: BTreeMap<OutputId, OutputId>,
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum PackageError {
    #[error("the system does not compose: {} error(s)", errors.len())]
    NotComposable { errors: Vec<Diagnostic> },
    #[error("port `{name}` names a declaration the flattened system does not have")]
    UnknownPortDeclaration { name: String },
    #[error("port `{name}` is {kind:?} but the declaration is not of that shape")]
    PortShape { name: String, kind: PortKind },
    #[error("a port named `{name}` is listed twice")]
    DuplicatePortName { name: String },
    #[error("clock parameter {clock} is not a domain of the system")]
    UnknownClock { clock: ClockId },
}

/// Package a system as a component.  The component's body is the flattened
/// design (a copy — the inner system stays authored truth of its own
/// project); its ports are fresh, numbered from 0, and renumbered by the
/// enclosing system's `InstallComponent`.  Validation: the composition has
/// no error, and every chosen port is realised by the flattened design in
/// the advertised shape (the FV's open `Realizes` obligation, discharged
/// here by construction: required ⇒ unresolved, provided ⇒ present,
/// parameter ⇒ unresolved with the unit domain).
pub fn package_system(
    snapshot: &SystemSnapshot,
    name: &str,
    interface: PackageInterface,
) -> Result<BehaviorComponent, PackageError> {
    let a = analyze_system(snapshot);
    let errors: Vec<Diagnostic> = a
        .composition
        .iter()
        .filter(|d| d.is_error())
        .cloned()
        .collect();
    if !errors.is_empty() {
        return Err(PackageError::NotComposable { errors });
    }
    let design = a.flattened.snapshot.design.clone();
    let mut ports = BTreeMap::new();
    let mut seen = std::collections::BTreeSet::new();
    for (i, p) in interface.ports.iter().enumerate() {
        if !seen.insert(p.name.clone()) {
            return Err(PackageError::DuplicatePortName {
                name: p.name.clone(),
            });
        }
        let decl = match &p.decl {
            PackageDecl::Port(r) => a.flattened.origins.decl_of_port(*r),
            PackageDecl::Base(d) => Some(*d).filter(|d| !a.flattened.origins.decls.contains_key(d)),
        }
        .ok_or_else(|| PackageError::UnknownPortDeclaration {
            name: p.name.clone(),
        })?;
        let m = design
            .mappings
            .get(&decl)
            .ok_or_else(|| PackageError::UnknownPortDeclaration {
                name: p.name.clone(),
            })?;
        let ok = match p.kind {
            PortKind::Required => m.definition.is_none(),
            PortKind::Provided => true,
            PortKind::Parameter => m.definition.is_none() && m.signature.is_unit_domain(),
        };
        if !ok {
            return Err(PackageError::PortShape {
                name: p.name.clone(),
                kind: p.kind,
            });
        }
        let id = PortId::from_raw(i as u64);
        let interface_so_far = BehaviorInterface {
            ports: BTreeMap::new(),
            clock_params: interface.clock_params.clone(),
        };
        let contract = PortContract::of_declaration(m, &interface_so_far);
        ports.insert(
            id,
            Port {
                id,
                name: p.name.clone(),
                description: String::new(),
                kind: p.kind,
                decl,
                contract,
            },
        );
    }
    for c in &interface.clock_params {
        if !design.clocks.contains_key(c) {
            return Err(PackageError::UnknownClock { clock: *c });
        }
    }
    Ok(BehaviorComponent {
        id: ComponentId::from_raw(0),
        name: name.to_owned(),
        description: String::new(),
        body: design,
        interface: BehaviorInterface {
            ports,
            clock_params: interface.clock_params,
        },
        shared_concepts: interface.shared_concepts,
        external_outputs: interface.external_outputs,
        body_stamp: 0,
        interface_stamp: 0,
    })
}
