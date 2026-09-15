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

use crate::{analyze, analyze_design_ir, MappingStatus, ProjectAnalysis};
use bdl_codegen_rust::{CodegenOptions, GeneratedCrate};
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_exec_ir::ExecIr;
use bdl_ir::DesignIr;
use bdl_model::surface::ProjectSnapshot;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct CompileOptions {
    /// Require `output_complete` (every required sink driven, no output
    /// open) — what an executable, firmware-oriented artefact needs.  Off,
    /// a partially bound design still generates (its outputs are whatever
    /// is validly driven).
    pub require_complete: bool,
    pub codegen: CodegenOptions,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompileArtifact {
    pub analysis: ProjectAnalysis,
    pub exec_ir: Option<ExecIr>,
    pub generated: Option<GeneratedCrate>,
    /// Backend diagnostics only (`backend.*`); the analysis carries its own.
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
    compile_analysis(analysis, &name, options)
}

/// [`compile`] for a Design IR built directly (see [`analyze_design_ir`]).
pub fn compile_design_ir(ir: DesignIr, name: &str, options: &CompileOptions) -> CompileArtifact {
    let analysis = analyze_design_ir(ir);
    compile_analysis(analysis, name, options)
}

fn compile_analysis(
    analysis: ProjectAnalysis,
    name: &str,
    options: &CompileOptions,
) -> CompileArtifact {
    let mut diagnostics = readiness(&analysis, options.require_complete);
    if !diagnostics.is_empty() {
        return CompileArtifact {
            analysis,
            exec_ir: None,
            generated: None,
            diagnostics,
        };
    }
    let exec_ir = match bdl_lower::lower(&analysis.ir, name, &analysis.outputs.valid_bindings) {
        Ok(e) => e,
        Err(ds) => {
            diagnostics.extend(ds);
            sort_diagnostics(&mut diagnostics);
            return CompileArtifact {
                analysis,
                exec_ir: None,
                generated: None,
                diagnostics,
            };
        }
    };
    let generated = match bdl_codegen_rust::generate(&exec_ir, &options.codegen) {
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
    CompileArtifact {
        analysis,
        exec_ir: Some(exec_ir),
        generated,
        diagnostics,
    }
}
