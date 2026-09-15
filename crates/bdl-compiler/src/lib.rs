//! The compiler front end as one pure function:
//!
//! ```text
//! analyze(&ProjectSnapshot) → ProjectAnalysis
//! ```
//!
//! passes, in order: elaboration (`bdl-elab`: concepts → Θ, signatures →
//! interfaces, formulas → Core), checking (`bdl-check`: typing under the
//! declaration's grant, comparison with the interface), then the reactive
//! passes (`bdl-reactive`: dependency graph, causality, clock domains).
//! Dimension checking is not a pass of its own — it falls out of typing.
//! After clocks, the output pass (`bdl-output`: every drive edge well formed,
//! one driver per sink, completeness against the required sinks).
//!
//! Deployment is a second, separate function:
//!
//! ```text
//! analyze_deployment(&ProjectSnapshot, &Hardware) → DeploymentAnalysis
//! ```
//!
//! Semantic analysis never consults a target; deployment analysis is
//! target-relative and is recomputed per target.  A design is not "invalid"
//! because a board is too small — it is infeasible *on that board*.
//!
//! The result is tagged with the snapshot's revision so a consumer can
//! discard it once the project has moved on, and it is deterministic: the
//! same snapshot yields the same analysis, diagnostics in the documented
//! order (entity, span, code).

#![forbid(unsafe_code)]

pub mod backend;
pub mod deploy_report;
pub use backend::{compile, compile_design_ir, readiness, CompileArtifact, CompileOptions};
pub use deploy_report::{
    deployment_report, AssignmentRow, Blocker, BlockerKind, DeploymentReport, MissingItem,
    MissingKind,
};

use bdl_check::{check_realization, pretty, TypeErrorKind};
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity, Severity};
use bdl_elab::{elaborate_design, RealizationOutcome};
use bdl_hardware::{
    devices::requirements_for_all, diagnose, solve, validate, Assignment, DeadEnd, DeadEndReason,
    Hardware, Requirement,
};
use bdl_ir::{DesignIr, Expr, Interface, Ty};
use bdl_model::surface::ProjectSnapshot;
use bdl_model::{DeclId, OutputId, Revision, SemanticId};
use bdl_output::{check_outputs, OutputAnalysis};
use bdl_reactive::{
    analyze_dependencies, check_causality, check_clocks, CausalityAnalysis, ClockAnalysis,
    DependencyGraph,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The paper's workspace states, as far as this slice can establish them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MappingStatus {
    /// Named and typed; no definition.  Others may depend on it.
    Declared,
    /// A definition is attached but something it needs is still open
    /// (a concept without a representation).  Not an error.
    Open,
    /// A definition is attached and does not check.
    Invalid,
    /// The definition produces what the signature promises; the
    /// realization is typed under the declaration's own grant.
    TypeValid,
    /// Type-valid, and no instantaneous loop passes through it: it has a
    /// value at every activation.
    TemporallyValid,
    /// Temporally valid, and every value it reads is in its own timing
    /// domain or transported explicitly.
    ClockConsistent,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MappingAnalysis {
    pub id: DeclId,
    pub interface: Interface,
    pub status: MappingStatus,
    /// The inferred type of the realization when it checks.
    pub inferred_type: Option<Ty>,
    /// The elaborated Core term (for the explanation view / tests).
    pub realization: Option<Expr>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConceptAnalysis {
    pub id: SemanticId,
    pub representation: Option<Ty>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub revision: Revision,
    pub concepts: BTreeMap<SemanticId, ConceptAnalysis>,
    pub mappings: BTreeMap<DeclId, MappingAnalysis>,
    /// Every diagnostic of every entity, in the documented order.
    pub diagnostics: Vec<Diagnostic>,
    /// The Design IR the analysis produced (Θ, Δ, Κ, Ω, β).
    pub ir: DesignIr,
    pub dependencies: DependencyGraph,
    pub causality: CausalityAnalysis,
    pub clocks: ClockAnalysis,
    /// `DriveWF`, `SingleDriver`, `CompleteOutputs` over the outputs that
    /// have a domain.
    pub outputs: OutputAnalysis,
    /// Surface outputs still without a timing domain: not yet sinks the
    /// kernel can see, so neither driven nor missing.
    pub open_outputs: BTreeSet<OutputId>,
    /// Every drive edge well formed, one driver per sink, every required
    /// sink driven, and no output open: the design commits to the world.
    pub output_complete: bool,
}

impl ProjectAnalysis {
    pub fn is_stale_for(&self, current: Revision) -> bool {
        self.revision < current
    }
}

/// Run every implemented pass over one immutable snapshot.
pub fn analyze(snapshot: &ProjectSnapshot) -> ProjectAnalysis {
    let elab = elaborate_design(&snapshot.design);
    let design = &snapshot.design;
    let required: BTreeSet<OutputId> = design
        .outputs
        .values()
        .filter(|o| o.required && o.clock.is_some())
        .map(|o| o.id)
        .collect();
    let open_outputs: BTreeMap<OutputId, String> = design
        .outputs
        .values()
        .filter(|o| o.clock.is_none())
        .map(|o| (o.id, o.name.clone()))
        .collect();
    let mappings = elab
        .mappings
        .into_iter()
        .map(|(id, m)| (id, (m.interface, m.outcome, m.diagnostics)))
        .collect();
    analyze_ir(
        snapshot.revision,
        elab.ir,
        mappings,
        &required,
        &open_outputs,
    )
}

/// Run the passes after elaboration over a Design IR built directly — for
/// tests and tools that construct Core terms the surface cannot express
/// yet (DI-17).  Every realized declaration counts as elaborated; every
/// output in `Ω` is required.
pub fn analyze_design_ir(ir: DesignIr) -> ProjectAnalysis {
    let mappings = ir
        .decls
        .values()
        .map(|d| {
            let outcome = match &d.realization {
                None => RealizationOutcome::Unresolved,
                Some(e) => RealizationOutcome::Elaborated(bdl_elab::Realized {
                    expr: e.clone(),
                    spans: BTreeMap::new(),
                }),
            };
            (d.id, (d.interface.clone(), outcome, Vec::new()))
        })
        .collect();
    let required: BTreeSet<OutputId> = ir.outputs.keys().copied().collect();
    analyze_ir(
        Revision::default(),
        ir,
        mappings,
        &required,
        &BTreeMap::new(),
    )
}

type ElabMappings = BTreeMap<DeclId, (Interface, RealizationOutcome, Vec<Diagnostic>)>;

fn analyze_ir(
    revision: Revision,
    ir: DesignIr,
    elab_mappings: ElabMappings,
    required: &BTreeSet<OutputId>,
    open_outputs: &BTreeMap<OutputId, String>,
) -> ProjectAnalysis {
    let mut all = Vec::new();

    let concepts = ir
        .concepts
        .values()
        .map(|c| {
            (
                c.id,
                ConceptAnalysis {
                    id: c.id,
                    representation: c.representation.clone(),
                },
            )
        })
        .collect();

    let mut mappings = BTreeMap::new();
    for (id, (interface, outcome, diagnostics)) in elab_mappings {
        let mut diagnostics = diagnostics;
        let (status, inferred_type, realization) = match outcome {
            RealizationOutcome::Unresolved => (MappingStatus::Declared, None, None),
            RealizationOutcome::Failed => {
                let status = if diagnostics.iter().any(Diagnostic::is_error) {
                    MappingStatus::Invalid
                } else {
                    MappingStatus::Open
                };
                (status, None, None)
            }
            RealizationOutcome::Elaborated(r) => match check_realization(&ir, id) {
                Ok(ty) => (MappingStatus::TypeValid, ty, Some(r.expr)),
                Err(e) => {
                    let span = r.spans.get(&e.path).copied();
                    let mut d = checker_diagnostic(&ir, id, &e.kind);
                    d.span = span;
                    diagnostics.push(d);
                    (MappingStatus::Invalid, None, Some(r.expr))
                }
            },
        };
        sort_diagnostics(&mut diagnostics);
        all.extend(diagnostics.iter().cloned());
        mappings.insert(
            id,
            MappingAnalysis {
                id,
                interface,
                status,
                inferred_type,
                realization,
                diagnostics,
            },
        );
    }
    // Reactive passes run over the whole design; a declaration in an
    // instantaneous cycle or reading across domains loses the corresponding
    // rung of the ladder and carries the diagnostic.
    let dependencies = analyze_dependencies(&ir);
    let causality = check_causality(&ir, &dependencies);
    let clocks = check_clocks(&ir);
    for (id, m) in mappings.iter_mut() {
        if m.status != MappingStatus::TypeValid {
            continue;
        }
        let mine = |d: &&Diagnostic| d.entity == Entity::Mapping { id: *id };
        if causality.in_cycle(*id) {
            m.status = MappingStatus::Invalid;
            m.diagnostics
                .extend(causality.diagnostics.iter().filter(mine).cloned());
        } else if clocks.ill_clocked.contains(id) {
            m.status = MappingStatus::TemporallyValid;
            m.diagnostics
                .extend(clocks.diagnostics.iter().filter(mine).cloned());
        } else {
            m.status = MappingStatus::ClockConsistent;
        }
        sort_diagnostics(&mut m.diagnostics);
    }
    all.extend(causality.diagnostics.iter().cloned());
    all.extend(clocks.diagnostics.iter().cloned());

    // Output pass.  Which sinks are required is a surface decision.
    let outputs = check_outputs(&ir, required);
    for (id, m) in mappings.iter_mut() {
        let mine: Vec<Diagnostic> = outputs
            .diagnostics
            .iter()
            .filter(|d| d.entity == Entity::Mapping { id: *id })
            .cloned()
            .collect();
        if mine.is_empty() {
            continue;
        }
        // A drive edge that is ill formed or contested is a fault of the
        // mapping's connection, not of its definition: the ladder is
        // untouched, the diagnostics travel with the mapping.
        m.diagnostics.extend(mine);
        sort_diagnostics(&mut m.diagnostics);
    }
    all.extend(outputs.diagnostics.iter().cloned());
    for (o, name) in open_outputs {
        all.push(
            Diagnostic::info(
                "output.clock_unset",
                Entity::Project,
                format!("{name} has no timing domain yet."),
            )
            .explain("An output commits a value at each activation of a domain; say which one.")
            .technical(format!("Ω {o} = none")),
        );
    }
    let output_complete = outputs.executable && open_outputs.is_empty();
    let open_outputs: BTreeSet<OutputId> = open_outputs.keys().copied().collect();
    sort_diagnostics(&mut all);
    ProjectAnalysis {
        revision,
        concepts,
        mappings,
        diagnostics: all,
        ir,
        dependencies,
        causality,
        clocks,
        outputs,
        open_outputs,
        output_complete,
    }
}

// ---- deployment -----------------------------------------------------------

/// Whether the device bindings fit one target.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatus {
    /// Every requirement is placed and every output with a domain is
    /// realised by a bound device; `assignment` is a witness.
    Feasible,
    /// No assignment exists; `dead_end` is one explanation.
    Infeasible,
    /// What is bound fits, but the binding is not finished: an output has
    /// no device on this target, or a device is bound to no output.  Not
    /// an error — a design may stop here.
    Incomplete,
}

/// Target-relative result.  Independent of the semantic analysis: it reads
/// only the device bindings, so it is meaningful for a design that is still
/// open — and equally meaningless as a statement about the design's
/// correctness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentAnalysis {
    pub revision: Revision,
    pub target: String,
    pub status: DeploymentStatus,
    /// Every requirement derived from every device, in device order.
    pub requirements: Vec<Requirement>,
    /// The witness, when feasible.  Deterministic for a given
    /// (snapshot, target).
    pub assignment: Option<Assignment>,
    pub dead_end: Option<DeadEnd>,
    /// Devices bound to no output, or to an output that no longer exists:
    /// still placed, but not yet part of the design's commitment.
    pub unbound_devices: BTreeSet<bdl_model::DeviceId>,
    /// Outputs with a domain that no device realises on this target.
    pub unrealised_outputs: BTreeSet<OutputId>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Run the hardware pass for one target.  Pure and deterministic.
pub fn analyze_deployment(snapshot: &ProjectSnapshot, target: &Hardware) -> DeploymentAnalysis {
    let design = &snapshot.design;
    let requirements = requirements_for_all(design.devices.values());
    let mut diagnostics = Vec::new();
    let device_name = |id: bdl_model::DeviceId| {
        design
            .devices
            .get(&id)
            .map(|d| d.name.clone())
            .unwrap_or_else(|| id.to_string())
    };
    let req_label = |id: bdl_hardware::RequirementId| {
        requirements
            .iter()
            .find(|r| r.id == id)
            .map(|r| r.label.clone())
            .unwrap_or_else(|| device_name(id.device))
    };

    let unbound_devices: BTreeSet<_> = design
        .devices
        .values()
        .filter(|d| d.output.is_none_or(|o| !design.outputs.contains_key(&o)))
        .map(|d| d.id)
        .collect();
    let realised: BTreeSet<OutputId> = design.devices.values().filter_map(|d| d.output).collect();
    let unrealised_outputs: BTreeSet<OutputId> = design
        .outputs
        .values()
        .filter(|o| o.clock.is_some() && !realised.contains(&o.id))
        .map(|o| o.id)
        .collect();
    for d in &unbound_devices {
        diagnostics.push(
            Diagnostic::info(
                "deploy.device_unbound",
                Entity::Project,
                format!("{} is not connected to any output.", device_name(*d)),
            )
            .explain(
                "A device realises exactly one output on the board; choose which one this is for.",
            )
            .technical(format!("device {d} output = none")),
        );
    }
    for o in &unrealised_outputs {
        let name = design
            .outputs
            .get(o)
            .map(|x| x.name.as_str())
            .unwrap_or("?");
        diagnostics.push(
            Diagnostic::info(
                "deploy.output_unrealised",
                Entity::Project,
                format!("{name} has no device on {}.", target.name),
            )
            .explain("Add the device that carries this output on the board.")
            .technical(format!("no DeviceBinding for output {o}")),
        );
    }

    let assignment = solve(target, &requirements);
    let (status, dead_end) = match &assignment {
        Some(a) => {
            debug_assert!(validate(target, &requirements, a).is_empty());
            if unbound_devices.is_empty() && unrealised_outputs.is_empty() {
                (DeploymentStatus::Feasible, None)
            } else {
                (DeploymentStatus::Incomplete, None)
            }
        }
        None => {
            let dead = diagnose(target, &requirements);
            if let Some(dead) = &dead {
                diagnostics.push(dead_end_diagnostic(target, dead, &req_label, &device_name));
            }
            (DeploymentStatus::Infeasible, dead)
        }
    };
    sort_diagnostics(&mut diagnostics);
    DeploymentAnalysis {
        revision: snapshot.revision,
        target: target.name.clone(),
        status,
        requirements,
        assignment,
        dead_end,
        unbound_devices,
        unrealised_outputs,
        diagnostics,
    }
}

fn dead_end_diagnostic(
    target: &Hardware,
    dead: &DeadEnd,
    req_label: &dyn Fn(bdl_hardware::RequirementId) -> String,
    device_name: &dyn Fn(bdl_model::DeviceId) -> String,
) -> Diagnostic {
    let what = req_label(dead.requirement);
    let who = device_name(dead.requirement.device);
    let board = &target.name;
    let d = |msg: String| Diagnostic::error("deploy.infeasible", Entity::Project, msg);
    match &dead.reason {
        DeadEndReason::NoCapableResource => d(format!("{board} has nothing that can carry {what}."))
            .explain(format!("No pin on {board} offers what {who} needs here. Choose a target that has it, or a different device."))
            .technical(format!("no resource with capability for {:?}", dead.requirement)),
        DeadEndReason::FixedUnavailable { fixed } => d(format!("{} cannot carry {what} on {board}.", fixed.0))
            .explain(format!("The pin chosen by hand for {who} is not on this board or lacks the needed function. Pick another pin, or let the placement choose."))
            .technical(format!("fixed {} fails ReqOK for {:?}", fixed.0, dead.requirement)),
        DeadEndReason::Blocked { candidates } => {
            let holders: Vec<String> = candidates
                .iter()
                .map(|(r, by)| format!("{} ({})", r.0, req_label(*by)))
                .collect();
            d(format!("No free pin on {board} can carry {what}."))
                .explain(format!(
                    "Every pin that could serve {who} is already needed by something else: {}. Use fewer devices of this kind, free a pin by moving another device, or choose a larger target. Counting pins is not enough — some pins serve several functions and can be used for only one.",
                    holders.join(", ")
                ))
                .technical(format!(
                    "dead end at {:?} after placing {}; one conflict under solver order, not a minimal core",
                    dead.requirement,
                    dead.placed.len()
                ))
        }
    }
}

/// Product-language rendering of a checker error.  The elaborator normally
/// reports these first with precise spans; the checker is the authority and
/// this covers whatever reaches it.
fn checker_diagnostic(ir: &DesignIr, id: DeclId, kind: &TypeErrorKind) -> Diagnostic {
    let entity = Entity::Mapping { id };
    let name = |s: SemanticId| {
        ir.concepts
            .get(&s)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| s.to_string())
    };
    let d = |code: &'static str, msg: String| Diagnostic::new(code, Severity::Error, entity, msg);
    match kind {
        TypeErrorKind::RealizationMismatch { expected, found } => d(
            "realization.type_mismatch",
            format!("This mapping promises {} but its definition is {}.", pretty::describe(ir, expected), pretty::describe(ir, found)),
        )
        .technical(format!("expected {}, found {}", pretty::kernel(expected), pretty::kernel(found))),
        TypeErrorKind::ConstructionNotGranted { concept } => d(
            "semantic.construction_not_granted",
            format!("This mapping may not produce {}: only the concept in its signature can be constructed here.", name(*concept)),
        )
        .explain("A value of a concept can only be made inside a relationship whose signature announces that concept.")
        .technical(format!("mk {concept} outside Grant.of(signature)")),
        TypeErrorKind::ConstructionMismatch { concept, expected, found } => d(
            "realization.type_mismatch",
            format!("{} is {}, but this produces {}.", name(*concept), pretty::describe(ir, expected), pretty::describe(ir, found)),
        )
        .technical(format!("mk {concept}: expected {}, found {}", pretty::kernel(expected), pretty::kernel(found))),
        TypeErrorKind::UnboundRepresentation { concept } => Diagnostic::info(
            "semantic.unbound_representation",
            entity,
            format!("{} has no representation yet.", name(*concept)),
        ),
        TypeErrorKind::ArgumentMismatch { expected, found } => d(
            "type.argument_mismatch",
            format!("Expected {} here but found {}.", pretty::describe(ir, expected), pretty::describe(ir, found)),
        )
        .technical(format!("expected {}, found {}", pretty::kernel(expected), pretty::kernel(found))),
        TypeErrorKind::ExpectedFunction { found } => {
            d("type.expected_function", format!("This is applied like a relationship, but it is {}.", pretty::describe(ir, found)))
        }
        TypeErrorKind::RepOfNonSemantic { found } => {
            d("type.rep_of_non_semantic", format!("Only a concept's value can be observed; this is {}.", pretty::describe(ir, found)))
        }
        TypeErrorKind::UnboundVariable { index } => d("type.unbound_variable", format!("Internal: unbound variable {index}.")),
        TypeErrorKind::UnknownDeclaration { id } => d("type.unknown_declaration", format!("Refers to a relationship that no longer exists ({id}).")),
        TypeErrorKind::TemporalMismatch { init, value } => d(
            "type.temporal_mismatch",
            format!("The initial value is {} but the remembered value is {}.", pretty::describe(ir, init), pretty::describe(ir, value)),
        ),
        TypeErrorKind::TemporalNotData { found } => {
            d("type.temporal_not_data", format!("Only values can be remembered over time, not relationships ({}).", pretty::describe(ir, found)))
        }
        TypeErrorKind::TemporalUnderBinder => d("type.temporal_under_binder", "Memory belongs to a relationship, not to a formula argument.".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::edit::{apply_edit, EditOp};
    use bdl_model::surface::{Definition, Design, DeviceKind, Representation, Signature};
    use bdl_model::Dim;

    fn lamp() -> (ProjectSnapshot, DeclId) {
        let mut s = ProjectSnapshot::new(Design::empty("lamp"));
        let mut ids = Vec::new();
        for (name, rep) in [
            ("Tilt", Representation::Quantity { dim: Dim::ANGLE }),
            ("Brightness", Representation::Quantity { dim: Dim::ZERO }),
        ] {
            let a = apply_edit(
                &s,
                &EditOp::CreateConcept {
                    name: name.into(),
                    description: String::new(),
                    representation: Some(rep),
                },
            )
            .unwrap();
            ids.push(a.outcome.created_concept.unwrap());
            s = a.snapshot;
        }
        let a = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![ids[0]],
                    output: ids[1],
                },
            },
        )
        .unwrap();
        (a.snapshot, a.outcome.created_mapping.unwrap())
    }

    fn attach(s: &ProjectSnapshot, id: DeclId, f: &str) -> ProjectSnapshot {
        apply_edit(
            s,
            &EditOp::AttachDefinition {
                id,
                definition: Definition::Formula { source: f.into() },
            },
        )
        .unwrap()
        .snapshot
    }

    #[test]
    fn declared_then_valid_then_invalid_with_revisions() {
        let (s, id) = lamp();
        let a = analyze(&s);
        assert_eq!(a.revision, s.revision);
        assert_eq!(a.mappings[&id].status, MappingStatus::Declared);
        assert!(a.diagnostics.is_empty());

        let s2 = attach(&s, id, "Tilt / 90 deg");
        let a2 = analyze(&s2);
        assert_eq!(a2.revision, s2.revision);
        assert_eq!(a2.mappings[&id].status, MappingStatus::ClockConsistent);
        assert!(a2.causality.valid && a2.clocks.valid);
        assert_eq!(a2.causality.order, vec![id]);
        assert_eq!(
            a2.mappings[&id].inferred_type.as_ref(),
            Some(&a2.mappings[&id].interface.expected_type)
        );
        assert!(a2.mappings[&id].realization.is_some());
        assert!(a.is_stale_for(s2.revision));

        let s3 = apply_edit(
            &s2,
            &EditOp::ReplaceDefinition {
                id,
                definition: Some(Definition::Formula {
                    source: "Tilt + 1 s".into(),
                }),
            },
        )
        .unwrap()
        .snapshot;
        let a3 = analyze(&s3);
        assert_eq!(a3.mappings[&id].status, MappingStatus::Invalid);
        assert_eq!(a3.diagnostics[0].code.as_str(), "dimension.mismatch");
    }

    #[test]
    fn open_when_a_representation_is_missing() {
        let (s, id) = lamp();
        let a = apply_edit(
            &s,
            &EditOp::CreateConcept {
                name: "Warmth".into(),
                description: String::new(),
                representation: None,
            },
        )
        .unwrap();
        let warmth = a.outcome.created_concept.unwrap();
        let s = a.snapshot;
        let s = apply_edit(
            &s,
            &EditOp::SetMappingSignature {
                id,
                signature: Signature {
                    inputs: vec![warmth],
                    output: s.design.mappings[&id].signature.output,
                },
            },
        )
        .unwrap()
        .snapshot;
        let s = attach(&s, id, "Warmth / 2");
        let a = analyze(&s);
        assert_eq!(a.mappings[&id].status, MappingStatus::Open);
        assert!(a.diagnostics.iter().all(|d| !d.is_error()));
    }

    #[test]
    fn analysis_is_deterministic_and_serializable() {
        let (s, id) = lamp();
        let s = attach(&s, id, "if Tilt < 10 deg then 0 else Tilt / 90 deg");
        let a = analyze(&s);
        let b = analyze(&s);
        assert_eq!(a, b);
        let json = serde_json::to_string(&a).unwrap();
        let back: ProjectAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(back, a);
    }

    #[test]
    fn cross_domain_read_stops_at_temporally_valid_with_a_clock_diagnostic() {
        // Two nullary mappings in different domains, the second reading the
        // first through a Core term (the surface cannot say this yet).
        let (s, id) = lamp();
        let s = attach(&s, id, "Tilt / 90 deg");
        let s = apply_edit(
            &s,
            &EditOp::CreateClockDomain {
                name: "interaction".into(),
            },
        )
        .unwrap()
        .snapshot;
        let a = analyze(&s);
        assert!(a.clocks.valid);
        // assign the mapping to the domain: still fine (no temporal forms, no crossings)
        let clock = *s.design.clocks.keys().next().unwrap();
        let s = apply_edit(
            &s,
            &EditOp::SetMappingClock {
                id,
                clock: Some(clock),
            },
        )
        .unwrap()
        .snapshot;
        let a = analyze(&s);
        assert_eq!(a.mappings[&id].status, MappingStatus::ClockConsistent);
        assert_eq!(a.ir.clocks[&id], clock);
    }

    proptest::proptest! {
        #[test]
        fn edit_then_analyze_never_panics(src in "\\PC{0,40}") {
            let (s, id) = lamp();
            let s = attach(&s, id, &src);
            let _ = analyze(&s);
        }
    }

    // ---- outputs and deployment ------------------------------------------

    fn edit(s: &ProjectSnapshot, op: EditOp) -> bdl_model::edit::Applied {
        apply_edit(s, &op).unwrap()
    }

    /// The lamp with a domain `main`, a nullary `level : () -> Brightness`
    /// in it, and an output `light : Brightness` in `main`.  Only a
    /// relationship without inputs has the type of a value (DI-20), so only
    /// it can drive an output.
    fn lamp_with_output() -> (ProjectSnapshot, DeclId, OutputId, bdl_model::ClockId) {
        let (s, unary) = lamp();
        let brightness = s.design.mappings[&unary].signature.output;
        let a = edit(
            &s,
            EditOp::CreateMapping {
                name: "level".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output: brightness,
                },
            },
        );
        let id = a.outcome.created_mapping.unwrap();
        let s = attach(&a.snapshot, id, "0.5");
        let a = edit(
            &s,
            EditOp::CreateClockDomain {
                name: "main".into(),
            },
        );
        let clock = a.outcome.created_clock.unwrap();
        let a = edit(
            &a.snapshot,
            EditOp::SetMappingClock {
                id,
                clock: Some(clock),
            },
        );
        let a = edit(
            &a.snapshot,
            EditOp::CreateOutput {
                name: "light".into(),
                description: String::new(),
                accepts: brightness,
                clock: Some(clock),
            },
        );
        let out = a.outcome.created_output.unwrap();
        (a.snapshot, id, out, clock)
    }

    #[test]
    fn output_ladder_undriven_driven_conflict() {
        let (s, id, out, clock) = lamp_with_output();
        let a = analyze(&s);
        assert_eq!(a.outputs.states[&out], bdl_output::OutputState::Undriven);
        assert!(a.outputs.partial_wf && !a.outputs.executable && !a.output_complete);
        assert!(a
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "output.missing_driver" && !d.is_error()));
        assert_eq!(a.mappings[&id].status, MappingStatus::ClockConsistent);

        let s = edit(
            &s,
            EditOp::SetMappingDrive {
                id,
                output: Some(out),
            },
        )
        .snapshot;
        let a = analyze(&s);
        assert_eq!(a.outputs.states[&out], bdl_output::OutputState::Driven);
        assert!(a.output_complete, "{:?}", a.diagnostics);
        assert_eq!(a.ir.drives[&id], out);
        assert!(a.diagnostics.is_empty());

        // a second driver: conflict, no policy, both mappings carry it
        let brightness = s.design.mappings[&id].signature.output;
        let b = edit(
            &s,
            EditOp::CreateMapping {
                name: "pulse".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output: brightness,
                },
            },
        );
        let other = b.outcome.created_mapping.unwrap();
        let s2 = attach(&b.snapshot, other, "1");
        let s2 = edit(
            &s2,
            EditOp::SetMappingClock {
                id: other,
                clock: Some(clock),
            },
        )
        .snapshot;
        let s2 = edit(
            &s2,
            EditOp::SetMappingDrive {
                id: other,
                output: Some(out),
            },
        )
        .snapshot;
        let a = analyze(&s2);
        assert_eq!(a.outputs.states[&out], bdl_output::OutputState::Conflict);
        assert!(!a.outputs.partial_wf && !a.output_complete);
        for m in [id, other] {
            assert_eq!(
                a.mappings[&m].status,
                MappingStatus::ClockConsistent,
                "the ladder is untouched"
            );
            assert!(a.mappings[&m]
                .diagnostics
                .iter()
                .any(|d| d.code.as_str() == "output.multiple_drivers"));
        }
        // a wrong-domain driver is an ill-formed edge, an error, on the driver
        let s3 = edit(&s, EditOp::SetMappingClock { id, clock: None }).snapshot;
        let a = analyze(&s3);
        assert_eq!(a.outputs.states[&out], bdl_output::OutputState::IllFormed);
        assert!(a.mappings[&id]
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "output.clock_mismatch" && d.is_error()));
        // a relationship with inputs is a function, not a value: DI-20
        let unary = *s.design.mappings.keys().next().unwrap();
        assert_ne!(unary, id);
        let s4 = edit(&s, EditOp::SetMappingDrive { id, output: None }).snapshot;
        let s4 = edit(
            &s4,
            EditOp::SetMappingClock {
                id: unary,
                clock: Some(clock),
            },
        )
        .snapshot;
        let s4 = edit(
            &s4,
            EditOp::SetMappingDrive {
                id: unary,
                output: Some(out),
            },
        )
        .snapshot;
        let a = analyze(&s4);
        assert_eq!(a.outputs.states[&out], bdl_output::OutputState::IllFormed);
        let d = a.mappings[&unary]
            .diagnostics
            .iter()
            .find(|d| d.code.as_str() == "output.type_mismatch")
            .unwrap();
        assert!(d
            .explanation
            .contains("A relationship with inputs is not a value"));
    }

    #[test]
    fn an_output_without_a_domain_is_open_not_missing() {
        let (s, id, out, _) = lamp_with_output();
        let s = edit(
            &s,
            EditOp::SetOutputClock {
                id: out,
                clock: None,
            },
        )
        .snapshot;
        let s = edit(
            &s,
            EditOp::SetMappingDrive {
                id,
                output: Some(out),
            },
        )
        .snapshot;
        let a = analyze(&s);
        assert!(a.open_outputs.contains(&out));
        assert!(!a.ir.outputs.contains_key(&out));
        assert!(a.outputs.missing_required.is_empty());
        assert!(!a.output_complete);
        assert!(a.diagnostics.iter().all(|d| !d.is_error()));
        assert!(a
            .diagnostics
            .iter()
            .any(|d| d.code.as_str() == "output.clock_unset" && d.message.contains("light")));
    }

    #[test]
    fn deployment_is_target_relative_and_separate_from_semantics() {
        let (s, id, out, _) = lamp_with_output();
        let s = edit(
            &s,
            EditOp::SetMappingDrive {
                id,
                output: Some(out),
            },
        )
        .snapshot;
        // the same output realised by different device kinds on two targets
        let pwm = edit(
            &s,
            EditOp::CreateDevice {
                name: "lamp".into(),
                kind: bdl_model::surface::DeviceKind::PwmChannel,
                output: Some(out),
            },
        )
        .snapshot;
        let dig = edit(
            &s,
            EditOp::CreateDevice {
                name: "lamp".into(),
                kind: bdl_model::surface::DeviceKind::DigitalOutput,
                output: Some(out),
            },
        )
        .snapshot;
        let sem_pwm = analyze(&pwm);
        let sem_dig = analyze(&dig);
        assert_eq!(
            sem_pwm.outputs, sem_dig.outputs,
            "semantic output analysis ignores devices"
        );
        assert_eq!(sem_pwm.ir, sem_dig.ir);
        assert!(sem_pwm.output_complete);

        let nano = bdl_hardware::boards::arduino_nano();
        let gpio_only = Hardware {
            name: "gpio_only".into(),
            display_name: String::new(),
            description: String::new(),
            family: String::new(),
            resources: vec![bdl_hardware::Resource {
                id: bdl_hardware::ResourceId::new("P0"),
                capabilities: [bdl_hardware::Capability::DigitalOut].into_iter().collect(),
                units: BTreeMap::new(),
            }],
            shareable: BTreeSet::new(),
        };
        let d = analyze_deployment(&pwm, &nano);
        assert_eq!(d.status, DeploymentStatus::Feasible);
        assert_eq!(
            d.assignment.as_ref().unwrap().values().next().unwrap().0,
            "D3"
        );
        assert!(d.diagnostics.is_empty());
        let d = analyze_deployment(&pwm, &gpio_only);
        assert_eq!(d.status, DeploymentStatus::Infeasible);
        assert_eq!(
            d.dead_end.as_ref().unwrap().reason,
            DeadEndReason::NoCapableResource
        );
        assert_eq!(d.diagnostics[0].code.as_str(), "deploy.infeasible");
        assert!(d.diagnostics[0]
            .message
            .contains("gpio_only has nothing that can carry lamp"));
        let d = analyze_deployment(&dig, &gpio_only);
        assert_eq!(d.status, DeploymentStatus::Feasible);
        // determinism
        assert_eq!(
            analyze_deployment(&pwm, &nano),
            analyze_deployment(&pwm, &nano)
        );
        let json = serde_json::to_string(&analyze_deployment(&pwm, &nano)).unwrap();
        let back: DeploymentAnalysis = serde_json::from_str(&json).unwrap();
        assert_eq!(back, analyze_deployment(&pwm, &nano));
    }

    #[test]
    fn deployment_reports_unbound_devices_and_unrealised_outputs() {
        let (s, _, out, _) = lamp_with_output();
        let s = edit(
            &s,
            EditOp::CreateDevice {
                name: "spare".into(),
                kind: bdl_model::surface::DeviceKind::DigitalOutput,
                output: None,
            },
        )
        .snapshot;
        let d = analyze_deployment(&s, &bdl_hardware::boards::arduino_nano());
        assert_eq!(d.status, DeploymentStatus::Incomplete);
        assert_eq!(d.unbound_devices.len(), 1);
        assert_eq!(d.unrealised_outputs, [out].into_iter().collect());
        let codes: Vec<&str> = d.diagnostics.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, ["deploy.device_unbound", "deploy.output_unrealised"]);
        assert!(d.diagnostics.iter().all(|d| !d.is_error()));
    }

    #[test]
    fn a_manual_pin_can_make_a_feasible_design_infeasible() {
        let (s, _, out, _) = lamp_with_output();
        let a = edit(
            &s,
            EditOp::CreateDevice {
                name: "lamp".into(),
                kind: bdl_model::surface::DeviceKind::PwmChannel,
                output: Some(out),
            },
        );
        let dev = a.outcome.created_device.unwrap();
        let s = a.snapshot;
        let nano = bdl_hardware::boards::arduino_nano();
        assert_eq!(
            analyze_deployment(&s, &nano).status,
            DeploymentStatus::Feasible
        );
        let s = edit(
            &s,
            EditOp::SetDevicePin {
                id: dev,
                index: 0,
                resource: Some("D4".into()),
            },
        )
        .snapshot;
        let d = analyze_deployment(&s, &nano);
        assert_eq!(d.status, DeploymentStatus::Infeasible);
        assert!(d.diagnostics[0].message.contains("D4 cannot carry lamp"));
    }

    // ---- the Deploy read model ----------------------------------------------

    fn report(s: &ProjectSnapshot, hw: &Hardware) -> DeploymentReport {
        let a = analyze(s);
        let d = analyze_deployment(s, hw);
        deployment_report(s, &a, &d, hw)
    }

    #[test]
    fn report_names_what_is_missing_before_anything_is_bound() {
        use deploy_report::MissingKind::*;
        let (s, id, out, _) = lamp_with_output();
        let nano = bdl_hardware::boards::arduino_nano();
        // driven, no device: deployment incomplete, design ready
        let s1 = edit(
            &s,
            EditOp::SetMappingDrive {
                id,
                output: Some(out),
            },
        )
        .snapshot;
        let r = report(&s1, &nano);
        assert!(r.design_ready && !r.deployable);
        assert_eq!(r.status, DeploymentStatus::Incomplete);
        assert_eq!(
            r.missing.iter().map(|m| m.kind).collect::<Vec<_>>(),
            vec![OutputNoDevice]
        );
        assert_eq!(r.missing[0].output_name.as_deref(), Some("light"));
        assert_eq!(r.missing[0].message, "light has no device on Arduino Nano.");
        assert!(r.rows.is_empty() && r.blocker.is_none());
        assert_eq!(r.target.display_name, "Arduino Nano");
        // undriven, with a device: the device fits, the design is not ready
        let a = edit(
            &s,
            EditOp::CreateDevice {
                name: "lamp".into(),
                kind: DeviceKind::PwmChannel,
                output: Some(out),
            },
        );
        let r = report(&a.snapshot, &nano);
        assert_eq!(
            r.status,
            DeploymentStatus::Feasible,
            "target-relative status is unchanged by semantics"
        );
        assert!(!r.design_ready && !r.deployable);
        assert_eq!(
            r.missing.iter().map(|m| m.kind).collect::<Vec<_>>(),
            vec![OutputNoDriver]
        );
        assert_eq!(r.rows.len(), 1);
        assert_eq!(
            r.rows[0].resource.as_ref().map(|x| x.0.as_str()),
            Some("D3")
        );
        // an unbound device and an open output are both listed, semantic first
        let b = edit(
            &a.snapshot,
            EditOp::CreateDevice {
                name: "spare".into(),
                kind: DeviceKind::DigitalOutput,
                output: None,
            },
        );
        let b = edit(
            &b.snapshot,
            EditOp::SetOutputClock {
                id: out,
                clock: None,
            },
        )
        .snapshot;
        let r = report(&b, &nano);
        assert_eq!(
            r.missing.iter().map(|m| m.kind).collect::<Vec<_>>(),
            vec![OutputNoDomain, DeviceNoOutput]
        );
        assert!(r.missing[0].kind.is_semantic() && !r.missing[1].kind.is_semantic());
        assert_eq!(r.missing[1].device_name.as_deref(), Some("spare"));
        // an invalid relationship is named
        let (s2, unary) = lamp();
        let s2 = attach(&s2, unary, "Tilt + 1 s");
        let r = report(&s2, &nano);
        assert_eq!(r.missing[0].kind, RelationshipNotChecking);
        assert_eq!(r.missing[0].mapping_name.as_deref(), Some("dimByTilt"));
        assert_eq!(r.missing[0].message, "dimByTilt does not check.");
    }

    #[test]
    fn report_rows_and_blocker_are_usable_without_solver_internals() {
        let (s, id, out, _) = lamp_with_output();
        let s = edit(
            &s,
            EditOp::SetMappingDrive {
                id,
                output: Some(out),
            },
        )
        .snapshot;
        let a = edit(
            &s,
            EditOp::CreateDevice {
                name: "drive".into(),
                kind: DeviceKind::HBridgeChannel,
                output: Some(out),
            },
        );
        let dev = a.outcome.created_device.unwrap();
        let s = a.snapshot;
        let nano = bdl_hardware::boards::arduino_nano();
        let r = report(&s, &nano);
        assert!(r.deployable && r.missing.is_empty());
        let rows: Vec<String> = r
            .rows
            .iter()
            .map(|x| {
                format!(
                    "{} | {} | {} | {} | {} | {}",
                    x.output_name.as_deref().unwrap_or("-"),
                    x.device_name,
                    x.device_kind_label,
                    x.requirement_label,
                    x.capability_label,
                    x.resource.as_ref().map(|r| r.0.as_str()).unwrap_or("-")
                )
            })
            .collect();
        assert_eq!(
            rows,
            vec![
                "light | drive | H-bridge channel | PWM | PWM | D3",
                "light | drive | H-bridge channel | direction | digital out | D0",
            ]
        );
        assert_eq!(
            r.rows[0].resource_label.as_deref(),
            Some("D3: digital in, digital out, PWM (timer 2), interrupt")
        );
        assert!(r.rows.iter().all(|x| x.fixed.is_none()));
        // deterministic
        assert_eq!(r, report(&s, &nano));
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(serde_json::from_str::<DeploymentReport>(&json).unwrap(), r);
        // a fixed pin that cannot do PWM: the blocker names the pin, the device, the role
        let s2 = edit(
            &s,
            EditOp::SetDevicePin {
                id: dev,
                index: 0,
                resource: Some("D4".into()),
            },
        )
        .snapshot;
        let r = report(&s2, &nano);
        assert_eq!(r.status, DeploymentStatus::Infeasible);
        assert!(r.design_ready && !r.deployable);
        let b = r.blocker.as_ref().unwrap();
        assert_eq!(
            (
                b.device_name.as_str(),
                b.requirement_label.as_str(),
                b.capability_label.as_str()
            ),
            ("drive", "PWM", "PWM")
        );
        assert_eq!(
            b.kind,
            BlockerKind::FixedUnavailable {
                pin: bdl_hardware::ResourceId::new("D4")
            }
        );
        assert_eq!(b.message, "D4 cannot carry drive PWM on arduino_nano.");
        assert!(r.rows.iter().all(|x| x.resource.is_none()));
        assert_eq!(r.rows[0].fixed.as_ref().map(|x| x.0.as_str()), Some("D4"));
        // PWM exhaustion: seven channels, the blocked candidates name their holders
        let mut s3 = s.clone();
        for n in 1..=6 {
            s3 = edit(
                &s3,
                EditOp::CreateDevice {
                    name: format!("L{n}"),
                    kind: DeviceKind::PwmChannel,
                    output: None,
                },
            )
            .snapshot;
        }
        let r = report(&s3, &nano);
        assert_eq!(r.status, DeploymentStatus::Infeasible);
        let b = r.blocker.as_ref().unwrap();
        let BlockerKind::Blocked { candidates } = &b.kind else {
            panic!("{:?}", b.kind)
        };
        assert_eq!(candidates.len(), 6);
        assert!(candidates
            .iter()
            .all(|c| !c.held_by_device_name.is_empty() && !c.resource_label.is_empty()));
        assert!(b
            .message
            .starts_with("No free pin on arduino_nano can carry"));
        // the same design fits the bigger board (the six extra channels are
        // bound to no output, so the configuration is incomplete, not infeasible)
        let big = report(&s3, &bdl_hardware::boards::big_board());
        assert_eq!(big.status, DeploymentStatus::Incomplete);
        assert!(big.blocker.is_none() && big.rows.iter().all(|x| x.resource.is_some()));
        assert_eq!(
            big.missing
                .iter()
                .filter(|m| m.kind == deploy_report::MissingKind::DeviceNoOutput)
                .count(),
            6
        );
    }
}
