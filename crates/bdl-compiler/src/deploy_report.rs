//! The Deploy read model: what a frontend needs to render a target chooser,
//! a three-way status, an assignment table and a failure explanation
//! without knowing what a requirement, a unit relation or a dead end is.
//!
//! It is a *composition* at the read-model edge — the semantic analysis
//! (target-independent) and the deployment analysis (target-relative) are
//! computed as before and joined here with the surface names and the
//! target's wording.  Nothing in it feeds back into either analysis.
//!
//! Four different questions, kept apart (docs/architecture/deployment-read-model.md):
//!
//! | question | answered by | here |
//! |---|---|---|
//! | is the design semantically valid? | `ProjectAnalysis` (typing, causality, clocks) | `design_ready`, `missing[]` of semantic kinds |
//! | are its outputs complete? | `ProjectAnalysis.output_complete` | `design_ready`, `missing[]` |
//! | is the deployment configuration complete? | `DeploymentAnalysis` (`unbound_devices`, `unrealised_outputs`) | `status == Incomplete`, `missing[]` |
//! | do the devices fit *this* target? | `DeploymentAnalysis` (`solve`) | `status`, `rows[]`, `blocker` |
//!
//! `deployable` is the conjunction; `status` alone is never "the answer".

use crate::{DeploymentAnalysis, DeploymentStatus, MappingStatus, ProjectAnalysis};
use bdl_diagnostics::Diagnostic;
use bdl_hardware::boards::{describe, TargetDescriptor};
use bdl_hardware::devices::{device_kind_label, requirement_labels};
use bdl_hardware::{Capability, DeadEndReason, Hardware, RequirementId, ResourceId};
use bdl_model::surface::{DeviceKind, ProjectSnapshot};
use bdl_model::{DeclId, DeviceId, OutputId, Revision};
use bdl_output::OutputState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One requirement of one device and, when placed, the resource that
/// carries it.  Everything a table row shows is here by name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssignmentRow {
    /// The output the device realises, when it is bound to one.
    pub output: Option<OutputId>,
    pub output_name: Option<String>,
    pub device: DeviceId,
    pub device_name: String,
    pub device_kind: DeviceKind,
    pub device_kind_label: String,
    /// The requirement's index within the device (stable; keys manual pins).
    pub requirement_index: u16,
    /// The requirement's role on the device (`PWM`, `direction`, `SDA`).
    pub requirement_label: String,
    pub capability: Capability,
    pub capability_label: String,
    /// The resource chosen by hand, if any.
    pub fixed: Option<ResourceId>,
    /// The resource carrying it in the current placement; `None` when no
    /// placement exists (infeasible).
    pub resource: Option<ResourceId>,
    /// The resource described in the target's terms.
    pub resource_label: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingKind {
    /// A relationship is invalid or open: the design is not executable.
    RelationshipNotChecking,
    /// The design has an instantaneous loop.
    NotCausal,
    /// A relationship reads across timing domains without saying how.
    NotClockConsistent,
    /// An output has no timing domain (it is not yet a sink).
    OutputNoDomain,
    /// A required output has no final target.
    OutputNoDriver,
    /// An output's connection does not fit or is contested.
    OutputConnectionInvalid,
    /// An output with a domain has no device on this target.
    OutputNoDevice,
    /// A device is bound to no output (or to one that no longer exists).
    DeviceNoOutput,
    /// A device's chosen realization is unknown, does not fit its output
    /// or is defective (docs/architecture/output-realization.md).
    RealizationInvalid,
}

impl MissingKind {
    /// Whether the item is about the design itself (true) or about the
    /// deployment configuration (false).
    pub fn is_semantic(self) -> bool {
        !matches!(
            self,
            MissingKind::OutputNoDevice
                | MissingKind::DeviceNoOutput
                | MissingKind::RealizationInvalid
        )
    }
}

/// One thing still missing before the design can be deployed, with the
/// entity it is about, by id and by name.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MissingItem {
    pub kind: MissingKind,
    pub output: Option<OutputId>,
    pub output_name: Option<String>,
    pub device: Option<DeviceId>,
    pub device_name: Option<String>,
    pub mapping: Option<DeclId>,
    pub mapping_name: Option<String>,
    pub message: String,
    pub explanation: String,
}

/// A pin that could have carried the blocked requirement and who holds it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockedCandidate {
    pub resource: ResourceId,
    pub resource_label: String,
    pub held_by_device: DeviceId,
    pub held_by_device_name: String,
    pub held_by_requirement_index: u16,
    pub held_by_requirement_label: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BlockerKind {
    /// Nothing on the target offers the capability.
    NoCapableResource,
    /// The pin chosen by hand is not on the target or lacks the capability.
    FixedUnavailable { pin: ResourceId },
    /// Every capable pin is taken by another requirement, or would violate
    /// a shared-unit relation with one.
    Blocked { candidates: Vec<BlockedCandidate> },
}

/// Why the devices do not fit the target: the solver's first dead end,
/// named.  *A* conflict, not a minimal unsatisfiable core (DI-21).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Blocker {
    pub device: DeviceId,
    pub device_name: String,
    pub requirement_index: u16,
    pub requirement_label: String,
    pub capability: Capability,
    pub capability_label: String,
    pub kind: BlockerKind,
    /// Product language, e.g. "No free pin on Arduino Nano can carry drive PWM."
    pub message: String,
    /// Consequence and remedy.
    pub explanation: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentReport {
    /// The project revision this describes; discard when it moves on.
    pub revision: Revision,
    pub target: TargetDescriptor,
    /// Target-relative: do the bound devices fit?  Unchanged from
    /// `DeploymentAnalysis::status`.
    pub status: DeploymentStatus,
    /// Target-independent: every relationship checks, the design is causal
    /// and clock-consistent, and its outputs are complete.
    pub design_ready: bool,
    /// `design_ready ∧ status == Feasible`: nothing is missing and the
    /// devices fit this target.
    pub deployable: bool,
    /// What is still missing, semantic first, then deployment; empty when
    /// `deployable`.  Also non-empty for an infeasible design whose
    /// configuration is incomplete.
    pub missing: Vec<MissingItem>,
    /// One row per requirement, in (output, device, index) order; the
    /// resource columns are filled when a placement exists.
    pub rows: Vec<AssignmentRow>,
    /// When infeasible.
    pub blocker: Option<Blocker>,
    /// The deployment analysis's diagnostics (product language).
    pub diagnostics: Vec<Diagnostic>,
}

/// Compose the read model.  Pure; deterministic for the same inputs.
pub fn deployment_report(
    snapshot: &ProjectSnapshot,
    analysis: &ProjectAnalysis,
    deployment: &DeploymentAnalysis,
    target: &Hardware,
) -> DeploymentReport {
    let design = &snapshot.design;
    let output_name = |o: OutputId| design.outputs.get(&o).map(|x| x.name.clone());
    let device_name = |d: DeviceId| {
        design
            .devices
            .get(&d)
            .map(|x| x.name.clone())
            .unwrap_or_else(|| d.to_string())
    };
    let mapping_name = |m: DeclId| {
        design
            .mappings
            .get(&m)
            .map(|x| x.name.clone())
            .unwrap_or_else(|| m.to_string())
    };
    // The role of a requirement on its device (`PWM`, `direction`) and its
    // capability, from the requirement list the solver was given.
    let req_role = |id: RequirementId| -> (String, Capability) {
        let cap = deployment
            .requirements
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.capability);
        let role = design.devices.get(&id.device).and_then(|dev| {
            requirement_labels(dev.kind)
                .into_iter()
                .find(|(i, _, _)| *i == id.index)
                .map(|(_, _, label)| label.to_string())
        });
        (
            role.unwrap_or_else(|| format!("requirement {}", id.index)),
            cap.unwrap_or(Capability::DigitalOut),
        )
    };
    let resource_label =
        |r: &ResourceId| target.describe_resource(r).unwrap_or_else(|| r.0.clone());

    // ---- what is missing --------------------------------------------------
    let mut missing = Vec::new();
    let not_checking: Vec<&crate::MappingAnalysis> = analysis
        .mappings
        .values()
        .filter(|m| matches!(m.status, MappingStatus::Open | MappingStatus::Invalid))
        .collect();
    for m in &not_checking {
        let name = mapping_name(m.id);
        let (message, explanation) = match m.status {
            MappingStatus::Open => (
                format!("{name} is not fully defined yet."),
                "Something it needs is still open — a concept without a representation. Complete it before deploying.".to_string(),
            ),
            _ => (
                format!("{name} does not check."),
                "Its definition has errors; see the relationship's diagnostics. The design is not executable until every relationship checks.".to_string(),
            ),
        };
        missing.push(MissingItem {
            kind: MissingKind::RelationshipNotChecking,
            output: None,
            output_name: None,
            device: None,
            device_name: None,
            mapping: Some(m.id),
            mapping_name: Some(name),
            message,
            explanation,
        });
    }
    if !analysis.causality.valid {
        missing.push(MissingItem {
            kind: MissingKind::NotCausal,
            output: None,
            output_name: None,
            device: None,
            device_name: None,
            mapping: None,
            mapping_name: None,
            message: "The design has an instantaneous loop.".into(),
            explanation: "A value cannot be computed from itself in the same instant; carry it across a moment with `previous`, or restructure the relationships.".into(),
        });
    }
    if !analysis.clocks.valid {
        missing.push(MissingItem {
            kind: MissingKind::NotClockConsistent,
            output: None,
            output_name: None,
            device: None,
            device_name: None,
            mapping: None,
            mapping_name: None,
            message: "A relationship reads across timing domains without saying how.".into(),
            explanation: "Choose how the relationship observes the other domain's value, or move it into that domain.".into(),
        });
    }
    for o in &analysis.open_outputs {
        let name = output_name(*o).unwrap_or_else(|| o.to_string());
        missing.push(MissingItem {
            kind: MissingKind::OutputNoDomain,
            output: Some(*o),
            output_name: Some(name.clone()),
            device: None,
            device_name: None,
            mapping: None,
            mapping_name: None,
            message: format!("{name} has no timing domain yet."),
            explanation: "An output commits a value at each activation of a domain; say which one."
                .into(),
        });
    }
    for o in &analysis.outputs.missing_required {
        let name = output_name(*o).unwrap_or_else(|| o.to_string());
        missing.push(MissingItem {
            kind: MissingKind::OutputNoDriver,
            output: Some(*o),
            output_name: Some(name.clone()),
            device: None,
            device_name: None,
            mapping: None,
            mapping_name: None,
            message: format!("{name} has no final target yet."),
            explanation: "Connect exactly one relationship that produces what the output accepts, in the output's domain.".into(),
        });
    }
    for (o, st) in &analysis.outputs.states {
        if !matches!(st, OutputState::IllFormed | OutputState::Conflict) {
            continue;
        }
        let name = output_name(*o).unwrap_or_else(|| o.to_string());
        let (message, explanation) = match st {
            OutputState::Conflict => (
                format!("{name} has more than one final target."),
                "Combine competing values upstream into one relationship, then connect that result.".to_string(),
            ),
            _ => (
                format!("{name}'s connection does not fit."),
                "The connected relationship must produce exactly what the output accepts, in the output's domain; see its diagnostics.".to_string(),
            ),
        };
        missing.push(MissingItem {
            kind: MissingKind::OutputConnectionInvalid,
            output: Some(*o),
            output_name: Some(name),
            device: None,
            device_name: None,
            mapping: None,
            mapping_name: None,
            message,
            explanation,
        });
    }
    for o in &deployment.unrealised_outputs {
        let name = output_name(*o).unwrap_or_else(|| o.to_string());
        missing.push(MissingItem {
            kind: MissingKind::OutputNoDevice,
            output: Some(*o),
            output_name: Some(name.clone()),
            device: None,
            device_name: None,
            mapping: None,
            mapping_name: None,
            message: format!("{name} has no device on {}.", target.display()),
            explanation: "Add the device that carries this output on the board.".into(),
        });
    }
    for d in &deployment.unbound_devices {
        let name = device_name(*d);
        missing.push(MissingItem {
            kind: MissingKind::DeviceNoOutput,
            output: None,
            output_name: None,
            device: Some(*d),
            device_name: Some(name.clone()),
            mapping: None,
            mapping_name: None,
            message: format!("{name} is not connected to any output."),
            explanation:
                "A device realises exactly one output on the board; choose which one this is for."
                    .into(),
        });
    }
    for r in deployment.realizations.values() {
        if !r.check.is_blocking() {
            continue;
        }
        let name = device_name(r.device);
        let d = deployment.diagnostics.iter().find(|d| {
            d.code.as_str().starts_with("deploy.realization_")
                && d.technical.contains(&format!("device {}", r.device))
        });
        missing.push(MissingItem {
            kind: MissingKind::RealizationInvalid,
            output: r.output,
            output_name: r.output.and_then(output_name),
            device: Some(r.device),
            device_name: Some(name.clone()),
            mapping: None,
            mapping_name: None,
            message: d
                .map(|d| d.message.clone())
                .unwrap_or_else(|| format!("{name} has a realization that cannot be used.")),
            explanation: d
                .map(|d| d.explanation.clone())
                .unwrap_or_else(|| "Choose another realization profile on the Deploy page.".into()),
        });
    }
    let design_ready = not_checking.is_empty()
        && analysis.causality.valid
        && analysis.clocks.valid
        && analysis.output_complete;

    // ---- rows -------------------------------------------------------------
    let placement: BTreeMap<RequirementId, &ResourceId> = deployment
        .assignment
        .iter()
        .flat_map(|a| a.iter().map(|(k, v)| (*k, v)))
        .collect();
    let mut rows: Vec<AssignmentRow> = deployment
        .requirements
        .iter()
        .map(|r| {
            let dev = design.devices.get(&r.id.device);
            let output = dev
                .and_then(|d| d.output)
                .filter(|o| design.outputs.contains_key(o));
            let kind = dev.map(|d| d.kind).unwrap_or(DeviceKind::DigitalOutput);
            let (role, _) = req_role(r.id);
            let resource = placement.get(&r.id).map(|x| (*x).clone());
            AssignmentRow {
                output,
                output_name: output.and_then(output_name),
                device: r.id.device,
                device_name: device_name(r.id.device),
                device_kind: kind,
                device_kind_label: device_kind_label(kind).to_string(),
                requirement_index: r.id.index,
                requirement_label: role,
                capability: r.capability,
                capability_label: r.capability.label().to_string(),
                fixed: r.fixed.clone(),
                resource_label: resource.as_ref().map(resource_label),
                resource,
            }
        })
        .collect();
    rows.sort_by(|a, b| {
        (a.output.is_none(), a.output, a.device, a.requirement_index).cmp(&(
            b.output.is_none(),
            b.output,
            b.device,
            b.requirement_index,
        ))
    });

    // ---- blocker ----------------------------------------------------------
    let blocker = deployment.dead_end.as_ref().map(|dead| {
        let (role, cap) = req_role(dead.requirement);
        let who = device_name(dead.requirement.device);
        let (message, explanation) = deployment
            .diagnostics
            .iter()
            .find(|d| d.code.as_str() == "deploy.infeasible")
            .map(|d| (d.message.clone(), d.explanation.clone()))
            .unwrap_or_default();
        let kind = match &dead.reason {
            DeadEndReason::NoCapableResource => BlockerKind::NoCapableResource,
            DeadEndReason::FixedUnavailable { fixed } => {
                BlockerKind::FixedUnavailable { pin: fixed.clone() }
            }
            DeadEndReason::Blocked { candidates } => BlockerKind::Blocked {
                candidates: candidates
                    .iter()
                    .map(|(r, by)| {
                        let (held_role, _) = req_role(*by);
                        BlockedCandidate {
                            resource: r.clone(),
                            resource_label: resource_label(r),
                            held_by_device: by.device,
                            held_by_device_name: device_name(by.device),
                            held_by_requirement_index: by.index,
                            held_by_requirement_label: held_role,
                        }
                    })
                    .collect(),
            },
        };
        Blocker {
            device: dead.requirement.device,
            device_name: who,
            requirement_index: dead.requirement.index,
            requirement_label: role,
            capability: cap,
            capability_label: cap.label().to_string(),
            kind,
            message,
            explanation,
        }
    });

    let deployable = design_ready
        && deployment.status == DeploymentStatus::Feasible
        && !deployment.realization_blocked();
    DeploymentReport {
        revision: deployment.revision,
        target: describe(target),
        status: deployment.status,
        design_ready,
        deployable,
        missing,
        rows,
        blocker,
        diagnostics: deployment.diagnostics.clone(),
    }
}
