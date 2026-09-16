//! `analyze_system`: flatten, then the existing compiler; then say what the
//! flat result means in the system's terms.  There is no system type
//! checker, evaluator, clock judgment or code generator here — only
//! projection (§34 of the brief).

use crate::flatten::{flatten, FlattenedSystem, Origin};
use crate::ids::{BindingId, ExportId};
use crate::model::*;
use crate::validate::validate_composition;
use bdl_compiler::{analyze, readiness, ProjectAnalysis};
use bdl_diagnostics::{Diagnostic, Entity};
use bdl_model::{DeclId, Revision};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Where a port of an instance stands.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PortStatus {
    /// A provided port: offered, whether or not anyone takes it.
    Provided,
    /// A required port or parameter taken from another instance's port.
    Bound { binding: BindingId },
    /// A required port declared an input of the whole system.
    Exported { export: ExportId },
    /// A parameter given a closed value.
    Valued,
    /// A required port or parameter nobody has bound: an ordinary
    /// unresolved declaration (FV Theorem H).  Not an error.
    Open,
}

/// Acceptance levels of a system (§40 of the brief).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Acceptance {
    /// A composition error or a flat error.
    Invalid,
    /// Structurally valid with at least one open port, or not yet
    /// executable for the flat reasons (`backend.not_ready`).
    Open,
    /// Every required port bound or exported; the flat design is causal,
    /// clock-consistent, output-complete and backend-ready.
    Executable,
}

/// A flat diagnostic with its system origin, when it has one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectedDiagnostic {
    pub diagnostic: Diagnostic,
    /// The instance-local entity the diagnostic is about; `None` for the
    /// system's own (base) entities and project-level diagnostics.
    pub origin: Option<Origin>,
    /// The port, when the entity is a port declaration.
    pub port: Option<PortRef>,
    /// `instance.local` when the entity is an instance's, else the base
    /// entity's own name.
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SystemAnalysis {
    pub revision: Revision,
    pub flattened: FlattenedSystem,
    /// The existing compiler's analysis of the flattened design.
    pub analysis: ProjectAnalysis,
    /// Composition-level diagnostics (`system.*`).
    pub composition: Vec<Diagnostic>,
    pub ports: BTreeMap<PortRef, PortStatus>,
    pub acceptance: Acceptance,
    /// Every diagnostic — composition and flat — projected to its origin.
    pub projected: Vec<ProjectedDiagnostic>,
}

pub fn analyze_system(snapshot: &SystemSnapshot) -> SystemAnalysis {
    let s = &snapshot.system;
    let flattened = flatten(snapshot);
    let mut composition = flattened.diagnostics.clone();
    composition.extend(validate_composition(s, &flattened));
    bdl_diagnostics::sort_diagnostics(&mut composition);
    let analysis = analyze(&flattened.snapshot);

    let mut ports = BTreeMap::new();
    for inst in s.instances.values() {
        let Some(c) = s.components.get(&inst.component) else {
            continue;
        };
        for p in c.interface.ports.values() {
            let r = PortRef {
                instance: inst.id,
                port: p.id,
            };
            let status = match p.kind {
                PortKind::Provided => PortStatus::Provided,
                PortKind::Required | PortKind::Parameter => {
                    if let Some(b) = s.binding_into(r) {
                        PortStatus::Bound { binding: b.id }
                    } else if let Some(e) = s.export_of(r) {
                        PortStatus::Exported { export: e.id }
                    } else if inst.parameter_bindings.contains_key(&p.id) {
                        PortStatus::Valued
                    } else {
                        PortStatus::Open
                    }
                }
            };
            ports.insert(r, status);
        }
    }

    let has_error = composition.iter().any(Diagnostic::is_error)
        || analysis.diagnostics.iter().any(Diagnostic::is_error);
    let open = ports.values().any(|p| *p == PortStatus::Open);
    let acceptance = if has_error {
        Acceptance::Invalid
    } else if open || !readiness(&analysis, true).is_empty() {
        Acceptance::Open
    } else {
        Acceptance::Executable
    };

    let projected = composition
        .iter()
        .chain(analysis.diagnostics.iter())
        .map(|d| project(&flattened, d))
        .collect();

    SystemAnalysis {
        revision: snapshot.revision,
        flattened,
        analysis,
        composition,
        ports,
        acceptance,
        projected,
    }
}

/// Project one flat diagnostic to the system.
pub fn project(flat: &FlattenedSystem, d: &Diagnostic) -> ProjectedDiagnostic {
    let design = &flat.snapshot.design;
    let (origin, port, label) = match d.entity {
        Entity::Mapping { id } => {
            let origin = flat.origins.decls.get(&id).cloned();
            let port = flat.origins.ports.get(&id).copied();
            let label = design.mappings.get(&id).map(|m| m.name.clone());
            (origin, port, label)
        }
        Entity::Concept { id } => (
            flat.origins.sems.get(&id).cloned(),
            None,
            design.concepts.get(&id).map(|c| c.name.clone()),
        ),
        Entity::Project => (None, None, None),
    };
    ProjectedDiagnostic {
        diagnostic: d.clone(),
        origin,
        port,
        label,
    }
}

/// The flat declaration behind a port reference, for simulation probes and
/// traces.
pub fn decl_of_port(flat: &FlattenedSystem, r: PortRef) -> Option<DeclId> {
    flat.origins.decl_of_port(r)
}
