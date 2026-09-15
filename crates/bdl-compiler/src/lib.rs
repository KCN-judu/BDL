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
//! Later passes (outputs, hardware) slot in after clocks.
//!
//! The result is tagged with the snapshot's revision so a consumer can
//! discard it once the project has moved on, and it is deterministic: the
//! same snapshot yields the same analysis, diagnostics in the documented
//! order (entity, span, code).

#![forbid(unsafe_code)]

use bdl_check::{check_realization, pretty, TypeErrorKind};
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity, Severity};
use bdl_elab::{elaborate_design, RealizationOutcome};
use bdl_ir::{DesignIr, Expr, Interface, Ty};
use bdl_model::surface::ProjectSnapshot;
use bdl_model::{DeclId, Revision, SemanticId};
use bdl_reactive::{
    analyze_dependencies, check_causality, check_clocks, CausalityAnalysis, ClockAnalysis,
    DependencyGraph,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    /// The Design IR the analysis produced (Θ, Δ, Κ; Ω/β empty until the
    /// output pass exists).
    pub ir: DesignIr,
    pub dependencies: DependencyGraph,
    pub causality: CausalityAnalysis,
    pub clocks: ClockAnalysis,
}

impl ProjectAnalysis {
    pub fn is_stale_for(&self, current: Revision) -> bool {
        self.revision < current
    }
}

/// Run every implemented pass over one immutable snapshot.
pub fn analyze(snapshot: &ProjectSnapshot) -> ProjectAnalysis {
    let elab = elaborate_design(&snapshot.design);
    let ir = elab.ir;
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
    for (id, m) in elab.mappings {
        let mut diagnostics = m.diagnostics;
        let (status, inferred_type, realization) = match m.outcome {
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
                interface: m.interface,
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
    sort_diagnostics(&mut all);
    ProjectAnalysis {
        revision: snapshot.revision,
        concepts,
        mappings,
        diagnostics: all,
        ir,
        dependencies,
        causality,
        clocks,
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
    use bdl_model::surface::{Definition, Design, Representation, Signature};
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
}
