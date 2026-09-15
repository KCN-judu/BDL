//! Conversions between wire messages and the project model.
//!
//! Direction matters: the model → projection direction is total (every
//! snapshot renders); the wire → model direction validates, because a
//! client may send anything.

use crate::pb;
use bdl_model::edit::{EditError, EditKind, EditOp, EditOutcome, Invalidation};
use bdl_model::layout::{Layout, Point};
use bdl_model::surface::{Definition, ProjectSnapshot, Representation, Signature};
use bdl_model::{DeclId, Dim, SemanticId};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConvertError {
    #[error("missing field `{0}`")]
    Missing(&'static str),
    #[error("dimension exponent out of range")]
    DimOutOfRange,
}

// ---------------------------------------------------------------------------
// Dim / Representation / Definition / Signature
// ---------------------------------------------------------------------------

pub fn dim_to_pb(d: Dim) -> pb::Dim {
    pb::Dim {
        length: d.length.into(),
        mass: d.mass.into(),
        time: d.time.into(),
        current: d.current.into(),
        temperature: d.temperature.into(),
        amount: d.amount.into(),
        luminous: d.luminous.into(),
        angle: d.angle.into(),
    }
}

pub fn dim_from_pb(d: &pb::Dim) -> Result<Dim, ConvertError> {
    let c = |v: i32| i8::try_from(v).map_err(|_| ConvertError::DimOutOfRange);
    Ok(Dim {
        length: c(d.length)?,
        mass: c(d.mass)?,
        time: c(d.time)?,
        current: c(d.current)?,
        temperature: c(d.temperature)?,
        amount: c(d.amount)?,
        luminous: c(d.luminous)?,
        angle: c(d.angle)?,
    })
}

pub fn representation_to_pb(r: Representation) -> pb::Representation {
    use pb::representation::Kind;
    let kind = match r {
        Representation::Quantity { dim } => Kind::Quantity(dim_to_pb(dim)),
        Representation::Boolean => Kind::Boolean(pb::Unit {}),
        Representation::Count => Kind::Count(pb::Unit {}),
    };
    pb::Representation { kind: Some(kind) }
}

pub fn representation_from_pb(r: &pb::Representation) -> Result<Representation, ConvertError> {
    use pb::representation::Kind;
    match r
        .kind
        .as_ref()
        .ok_or(ConvertError::Missing("representation.kind"))?
    {
        Kind::Quantity(d) => Ok(Representation::Quantity {
            dim: dim_from_pb(d)?,
        }),
        Kind::Boolean(_) => Ok(Representation::Boolean),
        Kind::Count(_) => Ok(Representation::Count),
    }
}

pub fn definition_to_pb(d: &Definition) -> pb::Definition {
    match d {
        Definition::Formula { source } => pb::Definition {
            kind: Some(pb::definition::Kind::Formula(source.clone())),
        },
    }
}

pub fn definition_from_pb(d: &pb::Definition) -> Result<Definition, ConvertError> {
    match d
        .kind
        .as_ref()
        .ok_or(ConvertError::Missing("definition.kind"))?
    {
        pb::definition::Kind::Formula(source) => Ok(Definition::Formula {
            source: source.clone(),
        }),
    }
}

pub fn signature_to_pb(s: &Signature) -> pb::Signature {
    pb::Signature {
        inputs: s.inputs.iter().map(|i| i.raw()).collect(),
        output: s.output.raw(),
    }
}

pub fn signature_from_pb(s: &pb::Signature) -> Signature {
    Signature {
        inputs: s.inputs.iter().map(|i| SemanticId::from_raw(*i)).collect(),
        output: SemanticId::from_raw(s.output),
    }
}

// ---------------------------------------------------------------------------
// Edits
// ---------------------------------------------------------------------------

pub fn edit_op_from_pb(op: &pb::EditOp) -> Result<EditOp, ConvertError> {
    use pb::edit_op::Op;
    let sem = SemanticId::from_raw;
    let decl = DeclId::from_raw;
    Ok(
        match op.op.as_ref().ok_or(ConvertError::Missing("edit_op.op"))? {
            Op::CreateConcept(m) => EditOp::CreateConcept {
                name: m.name.clone(),
                description: m.description.clone(),
                representation: m
                    .representation
                    .as_ref()
                    .map(representation_from_pb)
                    .transpose()?,
            },
            Op::RenameConcept(m) => EditOp::RenameConcept {
                id: sem(m.id),
                name: m.name.clone(),
            },
            Op::SetConceptDescription(m) => EditOp::SetConceptDescription {
                id: sem(m.id),
                description: m.description.clone(),
            },
            Op::SetConceptRepresentation(m) => EditOp::SetConceptRepresentation {
                id: sem(m.id),
                representation: m
                    .representation
                    .as_ref()
                    .map(representation_from_pb)
                    .transpose()?,
            },
            Op::DeleteConcept(m) => EditOp::DeleteConcept { id: sem(m.id) },
            Op::CreateMapping(m) => EditOp::CreateMapping {
                name: m.name.clone(),
                description: m.description.clone(),
                signature: signature_from_pb(
                    m.signature
                        .as_ref()
                        .ok_or(ConvertError::Missing("create_mapping.signature"))?,
                ),
            },
            Op::RenameMapping(m) => EditOp::RenameMapping {
                id: decl(m.id),
                name: m.name.clone(),
            },
            Op::SetMappingDescription(m) => EditOp::SetMappingDescription {
                id: decl(m.id),
                description: m.description.clone(),
            },
            Op::SetMappingSignature(m) => EditOp::SetMappingSignature {
                id: decl(m.id),
                signature: signature_from_pb(
                    m.signature
                        .as_ref()
                        .ok_or(ConvertError::Missing("set_mapping_signature.signature"))?,
                ),
            },
            Op::AttachDefinition(m) => EditOp::AttachDefinition {
                id: decl(m.id),
                definition: definition_from_pb(
                    m.definition
                        .as_ref()
                        .ok_or(ConvertError::Missing("attach_definition.definition"))?,
                )?,
            },
            Op::ReplaceDefinition(m) => EditOp::ReplaceDefinition {
                id: decl(m.id),
                definition: m.definition.as_ref().map(definition_from_pb).transpose()?,
            },
            Op::DeleteMapping(m) => EditOp::DeleteMapping { id: decl(m.id) },
        },
    )
}

pub fn outcome_to_pb(o: &EditOutcome) -> pb::EditOutcome {
    pb::EditOutcome {
        kind: match o.kind {
            Some(EditKind::Refinement) => pb::EditKind::Refinement,
            Some(EditKind::Edit) => pb::EditKind::Edit,
            None => pb::EditKind::Unspecified,
        }
        .into(),
        invalidates: o
            .invalidates
            .iter()
            .map(|i| {
                let v: pb::Invalidation = match i {
                    Invalidation::Interface => pb::Invalidation::Interface,
                    Invalidation::Realization => pb::Invalidation::Realization,
                    Invalidation::Semantic => pb::Invalidation::Semantic,
                    Invalidation::Reactive => pb::Invalidation::Reactive,
                    Invalidation::Clock => pb::Invalidation::Clock,
                    Invalidation::Output => pb::Invalidation::Output,
                    Invalidation::Deployment => pb::Invalidation::Deployment,
                };
                v.into()
            })
            .collect(),
        origin_decls: o.origin_decls.iter().map(|d| d.raw()).collect(),
        created_concept: o.created_concept.map(|c| c.raw()),
        created_mapping: o.created_mapping.map(|m| m.raw()),
    }
}

/// Stable machine-readable code + designer-readable message for an edit
/// failure.  The typed error travels in `details_json` for the expert view.
pub fn edit_error_to_pb(e: &EditError) -> pb::Error {
    let code = match e {
        EditError::EmptyName => "edit.empty_name",
        EditError::DuplicateConceptName { .. } => "edit.duplicate_concept_name",
        EditError::DuplicateMappingName { .. } => "edit.duplicate_mapping_name",
        EditError::UnknownConcept { .. } => "edit.unknown_concept",
        EditError::UnknownMapping { .. } => "edit.unknown_mapping",
        EditError::ConceptInUse { .. } => "edit.concept_in_use",
        EditError::AlreadyDefined { .. } => "edit.already_defined",
        EditError::NotDefined { .. } => "edit.not_defined",
    };
    pb::Error {
        code: code.to_owned(),
        message: e.to_string(),
        details_json: serde_json::to_string(e).unwrap_or_default(),
    }
}

// ---------------------------------------------------------------------------
// Layout
// ---------------------------------------------------------------------------

pub fn layout_to_pb(l: &Layout) -> pb::Layout {
    pb::Layout {
        concepts: l
            .concepts
            .iter()
            .map(|(id, p)| pb::NodePosition {
                id: id.raw(),
                x: p.x,
                y: p.y,
            })
            .collect(),
        mappings: l
            .mappings
            .iter()
            .map(|(id, p)| pb::NodePosition {
                id: id.raw(),
                x: p.x,
                y: p.y,
            })
            .collect(),
    }
}

pub fn layout_from_pb(l: &pb::Layout) -> Layout {
    Layout {
        concepts: l
            .concepts
            .iter()
            .map(|n| (SemanticId::from_raw(n.id), Point { x: n.x, y: n.y }))
            .collect(),
        mappings: l
            .mappings
            .iter()
            .map(|n| (DeclId::from_raw(n.id), Point { x: n.x, y: n.y }))
            .collect(),
    }
}

// ---------------------------------------------------------------------------
// Projection
// ---------------------------------------------------------------------------

/// Session facts that accompany a snapshot in the projection.
#[derive(Clone, Debug, Default)]
pub struct SessionInfo {
    pub root_path: String,
    pub can_undo: bool,
    pub can_redo: bool,
    pub dirty: bool,
}

/// Render a snapshot for the editor.  Deterministic: ordered maps in, ordered
/// lists out.
pub fn projection(
    snapshot: &ProjectSnapshot,
    layout: &Layout,
    info: &SessionInfo,
) -> pb::ProjectProjection {
    let design = &snapshot.design;
    pb::ProjectProjection {
        revision: snapshot.revision.raw(),
        name: design.name.clone(),
        root_path: info.root_path.clone(),
        concepts: design
            .concepts
            .values()
            .map(|c| pb::ConceptView {
                id: c.id.raw(),
                name: c.name.clone(),
                description: c.description.clone(),
                representation: c.representation.map(representation_to_pb),
            })
            .collect(),
        mappings: design
            .mappings
            .values()
            .map(|m| pb::MappingView {
                id: m.id.raw(),
                name: m.name.clone(),
                description: m.description.clone(),
                signature: Some(signature_to_pb(&m.signature)),
                definition: m.definition.as_ref().map(definition_to_pb),
                state: if m.is_unresolved() {
                    pb::AcceptanceState::Declared
                } else {
                    pb::AcceptanceState::Defined
                }
                .into(),
            })
            .collect(),
        layout: Some(layout_to_pb(layout)),
        can_undo: info.can_undo,
        can_redo: info.can_redo,
        dirty: info.dirty,
    }
}

// ---------------------------------------------------------------------------
// Analysis
// ---------------------------------------------------------------------------

pub fn diagnostic_to_pb(d: &bdl_diagnostics::Diagnostic) -> pb::Diagnostic {
    use bdl_diagnostics::{Entity, Severity};
    let severity = match d.severity {
        Severity::Error => pb::DiagnosticSeverity::Error,
        Severity::Warning => pb::DiagnosticSeverity::Warning,
        Severity::Info => pb::DiagnosticSeverity::Info,
    };
    let entity = match d.entity {
        Entity::Project => pb::diagnostic::Entity::Project(pb::Unit {}),
        Entity::Concept { id } => pb::diagnostic::Entity::ConceptId(id.raw()),
        Entity::Mapping { id } => pb::diagnostic::Entity::MappingId(id.raw()),
    };
    pb::Diagnostic {
        code: d.code.as_str().to_owned(),
        severity: severity.into(),
        entity: Some(entity),
        span: d.span.map(|s| pb::SourceSpan {
            start: s.start,
            end: s.end,
        }),
        message: d.message.clone(),
        explanation: d.explanation.clone(),
        technical: d.technical.clone(),
        fixes: d.fixes.clone(),
    }
}

pub fn analysis_to_pb(a: &bdl_compiler::ProjectAnalysis) -> pb::ProjectAnalysis {
    use bdl_check::pretty;
    use bdl_compiler::MappingStatus;
    pb::ProjectAnalysis {
        revision: a.revision.raw(),
        mappings: a
            .mappings
            .values()
            .map(|m| pb::MappingAnalysis {
                id: m.id.raw(),
                status: match m.status {
                    MappingStatus::Declared => pb::MappingStatus::Declared,
                    MappingStatus::Open => pb::MappingStatus::Open,
                    MappingStatus::Invalid => pb::MappingStatus::Invalid,
                    MappingStatus::TypeValid => pb::MappingStatus::TypeValid,
                }
                .into(),
                interface: pretty::kernel(&m.interface.expected_type),
                inferred_type: m
                    .inferred_type
                    .as_ref()
                    .map(pretty::kernel)
                    .unwrap_or_default(),
                core_expr: m.realization.as_ref().map(pretty::expr).unwrap_or_default(),
                diagnostics: m.diagnostics.iter().map(diagnostic_to_pb).collect(),
            })
            .collect(),
        diagnostics: a.diagnostics.iter().map(diagnostic_to_pb).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::surface::Design;

    #[test]
    fn edit_op_round_trip_for_create_mapping() {
        let op = pb::EditOp {
            op: Some(pb::edit_op::Op::CreateMapping(pb::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Some(pb::Signature {
                    inputs: vec![0],
                    output: 1,
                }),
            })),
        };
        let m = edit_op_from_pb(&op).unwrap();
        assert_eq!(
            m,
            EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![SemanticId::from_raw(0)],
                    output: SemanticId::from_raw(1)
                },
            }
        );
    }

    #[test]
    fn missing_fields_are_errors_not_panics() {
        let op = pb::EditOp {
            op: Some(pb::edit_op::Op::CreateMapping(pb::CreateMapping::default())),
        };
        assert_eq!(
            edit_op_from_pb(&op),
            Err(ConvertError::Missing("create_mapping.signature"))
        );
        assert_eq!(
            edit_op_from_pb(&pb::EditOp::default()),
            Err(ConvertError::Missing("edit_op.op"))
        );
    }

    #[test]
    fn projection_marks_unresolved_as_declared() {
        let s = ProjectSnapshot::new(Design::empty("p"));
        let a = bdl_model::apply_edit(
            &s,
            &EditOp::CreateConcept {
                name: "Tilt".into(),
                description: String::new(),
                representation: None,
            },
        )
        .unwrap();
        let tilt = a.outcome.created_concept.unwrap();
        let a = bdl_model::apply_edit(
            &a.snapshot,
            &EditOp::CreateMapping {
                name: "m".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output: tilt,
                },
            },
        )
        .unwrap();
        let p = projection(&a.snapshot, &Layout::default(), &SessionInfo::default());
        assert_eq!(p.revision, 2);
        assert_eq!(p.mappings[0].state(), pb::AcceptanceState::Declared);
    }

    #[test]
    fn dim_round_trip() {
        let d = Dim::LENGTH - Dim::TIME;
        assert_eq!(dim_from_pb(&dim_to_pb(d)).unwrap(), d);
        assert_eq!(
            dim_from_pb(&pb::Dim {
                length: 1000,
                ..Default::default()
            }),
            Err(ConvertError::DimOutOfRange)
        );
    }
}
