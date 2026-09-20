//! The backend entry: readiness check → reactive lowering → Rust code
//! generation, as one function over an analysed design.
//!
//! ```text
//! compile(snapshot, options)  =  analyze  →  readiness  →  lower  →  generate
//!                                   ↓             ↓          ↓          ↓
//!                              ProjectAnalysis  backend.*  ExecIr  GeneratedCrate
//! ```
//!
//! Readiness is decided on the authoritative analysis — nothing is
//! re-checked ad hoc downstream.  A design that is not ready yields a
//! `backend.not_ready` diagnostic and no artefact; lowering refusals
//! (`backend.unsupported_*`, `backend.internal_lowering`) likewise.  The
//! generated core is target-independent: deployment feasibility is not a
//! readiness condition here (a later, platform-adapter stage may require it).
//!
//! Collections are the one deployment fact decided here, because they
//! are decided on the lowered plan: the [`crate::collections`] report is
//! always computed, and under [`MemoryPolicy::Bounded`] an unbounded
//! remembered collection refuses the artefact (`deployment.*`).
//!
//! Output realizations (docs/architecture/output-realization.md) are
//! deployment data that the artefact *does* carry: every device binding
//! with a chosen profile whose encoder is well formed and fits its output
//! becomes a machine sink below the behavior plan; one whose profile is
//! unknown, incompatible or defective refuses the artefact
//! (`backend.realization_invalid`) — the board's feasibility stays with
//! `analyze_deployment`, as before.

use crate::collections::{
    collections_diagnostics, collections_report, CollectionsReport, MemoryPolicy,
};
use crate::{analyze, analyze_design_ir, MappingStatus, ProjectAnalysis};
use bdl_codegen_rust::{CodegenOptions, GeneratedCrate};
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_exec_ir::ExecIr;
use bdl_ir::{DesignIr, Expr, Ty};
use bdl_lower::{Provision, Provisions, Realization, Realizations};
use bdl_model::surface::{Design, ProjectSnapshot};
use bdl_model::{RelationshipRole, SemanticId};
use bdl_output::provision::{check_provider, provision_body};
use bdl_output::realization::{check_binding, encoder_body, Catalogue};
use bdl_reactive::Schedule;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CompileOptions {
    /// Require `output_complete` (every required sink driven, no output
    /// open) — what an executable, firmware-oriented artefact needs.  Off,
    /// a partially bound design still generates (its outputs are whatever
    /// is validly driven).
    pub require_complete: bool,
    pub codegen: CodegenOptions,
    /// Whether an unbounded remembered collection refuses the artefact.
    pub memory: MemoryPolicy,
    /// The deployment schedule, when known: decides what each
    /// cross-domain window requires (`deployment.window_capacity`).
    pub schedule: Option<Schedule>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompileArtifact {
    pub analysis: ProjectAnalysis,
    pub exec_ir: Option<ExecIr>,
    pub generated: Option<GeneratedCrate>,
    /// The collections report of the lowered plan (`None` when lowering
    /// did not happen).
    pub collections: Option<CollectionsReport>,
    /// Backend diagnostics only (`backend.*`, `deployment.*`); the
    /// analysis carries its own.
    pub diagnostics: Vec<Diagnostic>,
}

impl CompileArtifact {
    pub fn succeeded(&self) -> bool {
        self.generated.is_some()
    }
}

/// Why a design cannot be generated yet, as one product-language
/// diagnostic listing every unmet condition; empty when ready.
pub fn readiness(a: &ProjectAnalysis, require_complete: bool) -> Vec<Diagnostic> {
    let mut reasons: Vec<String> = Vec::new();
    let mut technical: Vec<String> = Vec::new();
    let not_checking: Vec<_> = a
        .mappings
        .values()
        .filter(|m| matches!(m.status, MappingStatus::Open | MappingStatus::Invalid))
        .collect();
    if !not_checking.is_empty() {
        let names: Vec<String> = not_checking
            .iter()
            .map(|m| {
                a.ir.decls
                    .get(&m.id)
                    .map(|d| d.name.clone())
                    .unwrap_or_else(|| m.id.to_string())
            })
            .collect();
        reasons.push(format!(
            "{} relationship{} {} not fully defined or do{} not check ({}).",
            names.len(),
            if names.len() == 1 { "" } else { "s" },
            if names.len() == 1 { "is" } else { "are" },
            if names.len() == 1 { "es" } else { "" },
            names.join(", ")
        ));
        technical.extend(
            not_checking
                .iter()
                .map(|m| format!("{} is {:?}", m.id, m.status)),
        );
    }
    if !a.causality.valid {
        reasons.push("The design has an instantaneous loop.".into());
        technical.push(format!("Causal fails: cycles {:?}", a.causality.cycles));
    }
    if !a.clocks.valid {
        reasons.push("A relationship reads across timing domains without saying how.".into());
        technical.push(format!("Clocked fails for {:?}", a.clocks.ill_clocked));
    }
    if !a.outputs.partial_wf {
        reasons.push(
            "An output has a connection that does not fit, or more than one final target.".into(),
        );
        technical.push(format!(
            "DriveWF/SingleDriver fail: faults {:?}, conflicts {:?}",
            a.outputs.faults.keys().collect::<Vec<_>>(),
            a.outputs.conflicts.keys().collect::<Vec<_>>()
        ));
    }
    if require_complete && !a.output_complete {
        reasons.push(
            "Not every required output has a final target, or an output has no timing domain."
                .into(),
        );
        technical.push(format!(
            "CompleteOutputs fails: missing {:?}, open {:?}",
            a.outputs.missing_required, a.open_outputs
        ));
    }
    if reasons.is_empty() {
        return Vec::new();
    }
    vec![Diagnostic::error("backend.not_ready", Entity::Project, "The design is not ready to generate code.")
        .explain(format!(
            "Code is generated only from a design whose relationships all check, that has no instantaneous loop, that is consistent across timing domains{}. {}",
            if require_complete { ", and whose outputs are complete" } else { " and whose outputs are well connected" },
            reasons.join(" ")
        ))
        .technical(technical.join("; "))]
}

pub fn compile(snapshot: &ProjectSnapshot, options: &CompileOptions) -> CompileArtifact {
    let analysis = analyze(snapshot);
    let name = snapshot.design.name.clone();
    let (realizations, mut refusals) = realizations_of(&snapshot.design, &analysis);
    let (provisions, more) = provisions_of(&snapshot.design, &analysis);
    refusals.extend(more);
    compile_analysis(
        analysis,
        &name,
        options,
        realizations,
        provisions,
        refusals,
        None,
    )
}

/// [`compile`] with the platform adapter for one solved deployment
/// (`crate::target::compile_for_target` is the entry; this is its body).
/// The adapter plan is built after lowering, from the machine sinks and
/// the placement; a plan that cannot be built refuses the artefact with
/// `adapter.*` diagnostics and leaves the exec IR in place.
pub(crate) fn compile_with_target(
    snapshot: &ProjectSnapshot,
    options: &CompileOptions,
    deployment: &crate::DeploymentAnalysis,
    target: &bdl_hardware::Hardware,
    target_options: &crate::target::TargetOptions,
) -> CompileArtifact {
    let analysis = analyze(snapshot);
    let name = snapshot.design.name.clone();
    let (realizations, mut refusals) = realizations_of(&snapshot.design, &analysis);
    let (provisions, more) = provisions_of(&snapshot.design, &analysis);
    refusals.extend(more);
    compile_analysis(
        analysis,
        &name,
        options,
        realizations,
        provisions,
        refusals,
        Some((deployment, target, target_options)),
    )
}

/// [`compile`] for a Design IR built directly (see [`analyze_design_ir`]).
/// A Design IR carries no device bindings, so no sink is lowered.
pub fn compile_design_ir(ir: DesignIr, name: &str, options: &CompileOptions) -> CompileArtifact {
    let analysis = analyze_design_ir(ir);
    compile_analysis(
        analysis,
        name,
        options,
        Realizations::new(),
        Provisions::new(),
        Vec::new(),
        None,
    )
}

/// The providers to lower — one per Source whose providing device has a
/// valid profile — and the refusals for chosen profiles that are not
/// valid.  A Source without a device, or a device without a profile,
/// lowers nothing: the Source stays a plain input (the simulation
/// supplies it; a board refuses it in `adapter_plan`).  The judgment is
/// the deployment analysis's (`check_provider`), repeated here on the
/// design alone so the core can be generated with providers and no board.
fn provisions_of(design: &Design, analysis: &ProjectAnalysis) -> (Provisions, Vec<Diagnostic>) {
    let ir = &analysis.ir;
    let theta = |s: SemanticId| ir.representation_of(s).cloned();
    let catalogue = Catalogue::builtin();
    let mut provisions = Provisions::new();
    let mut refusals = Vec::new();
    for d in design.devices.values() {
        let (Some(source), Some(_)) = (d.source, &d.provider) else {
            continue;
        };
        let Some(m) = design.mappings.get(&source) else {
            continue;
        };
        if m.role() != RelationshipRole::Source {
            continue;
        }
        if provisions.contains_key(&source) {
            // contested: the deployment analysis reports it; the first
            // device in id order provides
            continue;
        }
        let check = check_provider(&catalogue, d, Some(m.signature.output), theta);
        if check.is_blocking() {
            let profile = d.provider.as_ref().map(|p| p.0.clone()).unwrap_or_default();
            refusals.push(
                Diagnostic::error(
                    "backend.provider_invalid",
                    Entity::Project,
                    format!("{} cannot provide {} with `{profile}`.", d.name, m.name),
                )
                .explain("The provider chosen for this device is unknown, does not fit what the Source carries, or is defective. The Deploy page names which; the design's behavior is unaffected.")
                .technical(format!("device {} provider {profile}: {:?}", d.id, check.status)),
            );
            continue;
        }
        let Some(profile) = check.profile else {
            continue;
        };
        provisions.insert(
            source,
            Provision {
                source,
                device: d.id,
                device_name: d.name.clone(),
                profile: profile.id.clone(),
                raw: profile.transducer.raw.clone(),
                body: provision_body(m.signature.output, &profile.transducer, Expr::var(0)),
            },
        );
    }
    (provisions, refusals)
}

/// The machine sinks to lower — one per device binding whose chosen
/// profile is valid and whose output is validly driven — and the refusals
/// for chosen profiles that are not valid.  A binding without a profile
/// lowers nothing (the design behaves exactly as before realization
/// existed); a valid profile on an output that is not driven yet has
/// nothing to encode and lowers nothing either.
fn realizations_of(design: &Design, analysis: &ProjectAnalysis) -> (Realizations, Vec<Diagnostic>) {
    let ir = &analysis.ir;
    let theta = |s: SemanticId| ir.representation_of(s).cloned();
    let mut realizations = Realizations::new();
    let mut refusals = Vec::new();
    for d in design.devices.values() {
        if d.realization.is_none() {
            continue;
        }
        let output = d.output.filter(|o| design.outputs.contains_key(o));
        let accepts = output.map(|o| Ty::sem(design.outputs[&o].accepts));
        let check = check_binding(d, accepts.as_ref(), theta);
        if check.is_blocking() {
            let profile = d
                .realization
                .as_ref()
                .map(|p| p.0.clone())
                .unwrap_or_default();
            refusals.push(
                Diagnostic::error(
                    "backend.realization_invalid",
                    Entity::Project,
                    format!("{} cannot generate a raw command with `{profile}`.", d.name),
                )
                .explain("The realization chosen for this device is unknown, does not fit what its output carries, or is defective. The Deploy page names which; the design's behavior is unaffected.")
                .technical(format!("device {} realization {profile}: {:?}", d.id, check.status)),
            );
            continue;
        }
        let (Some(profile), Some(output), Some(accepts)) = (check.profile, output, accepts) else {
            continue;
        };
        let Some((driver, _)) = analysis
            .outputs
            .valid_bindings
            .iter()
            .find(|(_, o)| **o == output)
        else {
            continue;
        };
        realizations.insert(
            d.id,
            Realization {
                device: d.id,
                device_name: d.name.clone(),
                output,
                profile: profile.id.clone(),
                raw: profile.encoder.raw.clone(),
                body: encoder_body(&accepts, &profile.encoder, *driver),
            },
        );
    }
    (realizations, refusals)
}

type Target<'a> = (
    &'a crate::DeploymentAnalysis,
    &'a bdl_hardware::Hardware,
    &'a crate::target::TargetOptions,
);

fn compile_analysis(
    analysis: ProjectAnalysis,
    name: &str,
    options: &CompileOptions,
    realizations: Realizations,
    provisions: Provisions,
    refusals: Vec<Diagnostic>,
    target: Option<Target<'_>>,
) -> CompileArtifact {
    let mut diagnostics = readiness(&analysis, options.require_complete);
    diagnostics.extend(refusals);
    sort_diagnostics(&mut diagnostics);
    if !diagnostics.is_empty() {
        return CompileArtifact {
            analysis,
            exec_ir: None,
            generated: None,
            collections: None,
            diagnostics,
        };
    }
    let exec_ir = match bdl_lower::lower_with_provisions(
        &analysis.ir,
        name,
        &analysis.outputs.valid_bindings,
        &realizations,
        &provisions,
    ) {
        Ok(e) => e,
        Err(ds) => {
            diagnostics.extend(ds);
            sort_diagnostics(&mut diagnostics);
            return CompileArtifact {
                analysis,
                exec_ir: None,
                generated: None,
                collections: None,
                diagnostics,
            };
        }
    };
    let collections = collections_report(&exec_ir, options.schedule.as_ref());
    diagnostics.extend(collections_diagnostics(&collections, options.memory));
    if diagnostics.iter().any(Diagnostic::is_error) {
        sort_diagnostics(&mut diagnostics);
        return CompileArtifact {
            analysis,
            exec_ir: Some(exec_ir),
            generated: None,
            collections: Some(collections),
            diagnostics,
        };
    }
    let plan = match target {
        None => None,
        Some((deployment, hw, target_options)) => {
            match crate::target::adapter_plan(
                &exec_ir,
                hw,
                deployment,
                &collections,
                options.schedule.as_ref(),
                target_options,
            ) {
                Ok(plan) => Some(plan),
                Err(ds) => {
                    diagnostics.extend(ds);
                    sort_diagnostics(&mut diagnostics);
                    return CompileArtifact {
                        analysis,
                        exec_ir: Some(exec_ir),
                        generated: None,
                        collections: Some(collections),
                        diagnostics,
                    };
                }
            }
        }
    };
    let generated = match bdl_codegen_rust::generate_with_adapter(
        &exec_ir,
        &options.codegen,
        plan.as_ref(),
    ) {
        Ok(g) => Some(g),
        Err(e) => {
            diagnostics.push(
                Diagnostic::error("backend.internal_lowering", Entity::Project, "Code generation hit an internal inconsistency.")
                    .explain("The lowered plan could not be printed as Rust. This is a compiler bug; the design itself is fine.")
                    .technical(e.0),
            );
            None
        }
    };
    sort_diagnostics(&mut diagnostics);
    CompileArtifact {
        analysis,
        exec_ir: Some(exec_ir),
        generated,
        collections: Some(collections),
        diagnostics,
    }
}
