//! Conversions between wire messages and the project model.
//!
//! Direction matters: the model → projection direction is total (every
//! snapshot renders); the wire → model direction validates, because a
//! client may send anything.

use crate::pb;
use bdl_model::edit::{EditError, EditKind, EditOp, EditOutcome, Invalidation};
use bdl_model::layout::{Layout, Point};
use bdl_model::surface::{Definition, DeviceKind, ProjectSnapshot, Representation, Signature};
use bdl_model::{ClockId, DeclId, DeviceId, Dim, OutputId, SemanticId};
use bdl_output::OutputState;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ConvertError {
    #[error("missing field `{0}`")]
    Missing(&'static str),
    #[error("invalid value for `{0}`")]
    Invalid(&'static str),
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
    let clock = ClockId::from_raw;
    let output = OutputId::from_raw;
    let device = DeviceId::from_raw;
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
            Op::CreateClockDomain(m) => EditOp::CreateClockDomain {
                name: m.name.clone(),
            },
            Op::RenameClockDomain(m) => EditOp::RenameClockDomain {
                id: clock(m.id),
                name: m.name.clone(),
            },
            Op::DeleteClockDomain(m) => EditOp::DeleteClockDomain { id: clock(m.id) },
            Op::SetMappingClock(m) => EditOp::SetMappingClock {
                id: decl(m.id),
                clock: m.clock_id.map(clock),
            },
            Op::CreateOutput(m) => EditOp::CreateOutput {
                name: m.name.clone(),
                description: m.description.clone(),
                accepts: sem(m.accepts),
                clock: m.clock_id.map(clock),
            },
            Op::RenameOutput(m) => EditOp::RenameOutput {
                id: output(m.id),
                name: m.name.clone(),
            },
            Op::SetOutputAccepts(m) => EditOp::SetOutputAccepts {
                id: output(m.id),
                accepts: sem(m.accepts),
            },
            Op::SetOutputClock(m) => EditOp::SetOutputClock {
                id: output(m.id),
                clock: m.clock_id.map(clock),
            },
            Op::SetOutputRequired(m) => EditOp::SetOutputRequired {
                id: output(m.id),
                required: m.required,
            },
            Op::DeleteOutput(m) => EditOp::DeleteOutput { id: output(m.id) },
            Op::SetMappingDrive(m) => EditOp::SetMappingDrive {
                id: decl(m.id),
                output: m.output_id.map(output),
            },
            Op::CreateDevice(m) => EditOp::CreateDevice {
                name: m.name.clone(),
                kind: device_kind_from_pb(m.kind())?,
                output: m.output_id.map(output),
            },
            Op::RenameDevice(m) => EditOp::RenameDevice {
                id: device(m.id),
                name: m.name.clone(),
            },
            Op::SetDeviceKind(m) => EditOp::SetDeviceKind {
                id: device(m.id),
                kind: device_kind_from_pb(m.kind())?,
            },
            Op::SetDeviceOutput(m) => EditOp::SetDeviceOutput {
                id: device(m.id),
                output: m.output_id.map(output),
            },
            Op::SetDevicePin(m) => EditOp::SetDevicePin {
                id: device(m.id),
                index: u16::try_from(m.index)
                    .map_err(|_| ConvertError::Invalid("set_device_pin.index"))?,
                resource: m.resource.clone(),
            },
            Op::DeleteDevice(m) => EditOp::DeleteDevice { id: device(m.id) },
        },
    )
}

/// The inverse of [`edit_op_from_pb`]; every variant round-trips.
pub fn edit_op_to_pb(op: &EditOp) -> pb::EditOp {
    use pb::edit_op::Op;
    let o = match op {
        EditOp::CreateConcept {
            name,
            description,
            representation,
        } => Op::CreateConcept(pb::CreateConcept {
            name: name.clone(),
            description: description.clone(),
            representation: representation.map(representation_to_pb),
        }),
        EditOp::RenameConcept { id, name } => Op::RenameConcept(pb::RenameConcept {
            id: id.raw(),
            name: name.clone(),
        }),
        EditOp::SetConceptDescription { id, description } => {
            Op::SetConceptDescription(pb::SetConceptDescription {
                id: id.raw(),
                description: description.clone(),
            })
        }
        EditOp::SetConceptRepresentation { id, representation } => {
            Op::SetConceptRepresentation(pb::SetConceptRepresentation {
                id: id.raw(),
                representation: representation.map(representation_to_pb),
            })
        }
        EditOp::DeleteConcept { id } => Op::DeleteConcept(pb::DeleteConcept { id: id.raw() }),
        EditOp::CreateMapping {
            name,
            description,
            signature,
        } => Op::CreateMapping(pb::CreateMapping {
            name: name.clone(),
            description: description.clone(),
            signature: Some(signature_to_pb(signature)),
        }),
        EditOp::RenameMapping { id, name } => Op::RenameMapping(pb::RenameMapping {
            id: id.raw(),
            name: name.clone(),
        }),
        EditOp::SetMappingDescription { id, description } => {
            Op::SetMappingDescription(pb::SetMappingDescription {
                id: id.raw(),
                description: description.clone(),
            })
        }
        EditOp::SetMappingSignature { id, signature } => {
            Op::SetMappingSignature(pb::SetMappingSignature {
                id: id.raw(),
                signature: Some(signature_to_pb(signature)),
            })
        }
        EditOp::AttachDefinition { id, definition } => Op::AttachDefinition(pb::AttachDefinition {
            id: id.raw(),
            definition: Some(definition_to_pb(definition)),
        }),
        EditOp::ReplaceDefinition { id, definition } => {
            Op::ReplaceDefinition(pb::ReplaceDefinition {
                id: id.raw(),
                definition: definition.as_ref().map(definition_to_pb),
            })
        }
        EditOp::DeleteMapping { id } => Op::DeleteMapping(pb::DeleteMapping { id: id.raw() }),
        EditOp::CreateClockDomain { name } => {
            Op::CreateClockDomain(pb::CreateClockDomain { name: name.clone() })
        }
        EditOp::RenameClockDomain { id, name } => Op::RenameClockDomain(pb::RenameClockDomain {
            id: id.raw(),
            name: name.clone(),
        }),
        EditOp::DeleteClockDomain { id } => {
            Op::DeleteClockDomain(pb::DeleteClockDomain { id: id.raw() })
        }
        EditOp::SetMappingClock { id, clock } => Op::SetMappingClock(pb::SetMappingClock {
            id: id.raw(),
            clock_id: clock.map(|c| c.raw()),
        }),
        EditOp::CreateOutput {
            name,
            description,
            accepts,
            clock,
        } => Op::CreateOutput(pb::CreateOutput {
            name: name.clone(),
            description: description.clone(),
            accepts: accepts.raw(),
            clock_id: clock.map(|c| c.raw()),
        }),
        EditOp::RenameOutput { id, name } => Op::RenameOutput(pb::RenameOutput {
            id: id.raw(),
            name: name.clone(),
        }),
        EditOp::SetOutputAccepts { id, accepts } => Op::SetOutputAccepts(pb::SetOutputAccepts {
            id: id.raw(),
            accepts: accepts.raw(),
        }),
        EditOp::SetOutputClock { id, clock } => Op::SetOutputClock(pb::SetOutputClock {
            id: id.raw(),
            clock_id: clock.map(|c| c.raw()),
        }),
        EditOp::SetOutputRequired { id, required } => {
            Op::SetOutputRequired(pb::SetOutputRequired {
                id: id.raw(),
                required: *required,
            })
        }
        EditOp::DeleteOutput { id } => Op::DeleteOutput(pb::DeleteOutput { id: id.raw() }),
        EditOp::SetMappingDrive { id, output } => Op::SetMappingDrive(pb::SetMappingDrive {
            id: id.raw(),
            output_id: output.map(|o| o.raw()),
        }),
        EditOp::CreateDevice { name, kind, output } => Op::CreateDevice(pb::CreateDevice {
            name: name.clone(),
            kind: device_kind_to_pb(*kind).into(),
            output_id: output.map(|o| o.raw()),
        }),
        EditOp::RenameDevice { id, name } => Op::RenameDevice(pb::RenameDevice {
            id: id.raw(),
            name: name.clone(),
        }),
        EditOp::SetDeviceKind { id, kind } => Op::SetDeviceKind(pb::SetDeviceKind {
            id: id.raw(),
            kind: device_kind_to_pb(*kind).into(),
        }),
        EditOp::SetDeviceOutput { id, output } => Op::SetDeviceOutput(pb::SetDeviceOutput {
            id: id.raw(),
            output_id: output.map(|o| o.raw()),
        }),
        EditOp::SetDevicePin {
            id,
            index,
            resource,
        } => Op::SetDevicePin(pb::SetDevicePin {
            id: id.raw(),
            index: u32::from(*index),
            resource: resource.clone(),
        }),
        EditOp::DeleteDevice { id } => Op::DeleteDevice(pb::DeleteDevice { id: id.raw() }),
    };
    pb::EditOp { op: Some(o) }
}

pub fn device_kind_to_pb(k: DeviceKind) -> pb::DeviceKind {
    match k {
        DeviceKind::PwmChannel => pb::DeviceKind::PwmChannel,
        DeviceKind::DigitalOutput => pb::DeviceKind::DigitalOutput,
        DeviceKind::HBridgeChannel => pb::DeviceKind::HBridgeChannel,
        DeviceKind::I2cSensor => pb::DeviceKind::I2cSensor,
        DeviceKind::QuadratureEncoder => pb::DeviceKind::QuadratureEncoder,
        DeviceKind::Uart => pb::DeviceKind::Uart,
    }
}

pub fn device_kind_from_pb(k: pb::DeviceKind) -> Result<DeviceKind, ConvertError> {
    Ok(match k {
        pb::DeviceKind::PwmChannel => DeviceKind::PwmChannel,
        pb::DeviceKind::DigitalOutput => DeviceKind::DigitalOutput,
        pb::DeviceKind::HBridgeChannel => DeviceKind::HBridgeChannel,
        pb::DeviceKind::I2cSensor => DeviceKind::I2cSensor,
        pb::DeviceKind::QuadratureEncoder => DeviceKind::QuadratureEncoder,
        pb::DeviceKind::Uart => DeviceKind::Uart,
        pb::DeviceKind::Unspecified => return Err(ConvertError::Invalid("device_kind")),
    })
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
        created_clock: o.created_clock.map(|c| c.raw()),
        created_output: o.created_output.map(|c| c.raw()),
        created_device: o.created_device.map(|c| c.raw()),
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
        EditError::DuplicateClockName { .. } => "edit.duplicate_clock_name",
        EditError::UnknownClock { .. } => "edit.unknown_clock",
        EditError::ClockInUse { .. } => "edit.clock_in_use",
        EditError::DuplicateOutputName { .. } => "edit.duplicate_output_name",
        EditError::UnknownOutput { .. } => "edit.unknown_output",
        EditError::OutputInUse { .. } => "edit.output_in_use",
        EditError::DuplicateDeviceName { .. } => "edit.duplicate_device_name",
        EditError::UnknownDevice { .. } => "edit.unknown_device",
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
        outputs: l
            .outputs
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
        outputs: l
            .outputs
            .iter()
            .map(|n| (OutputId::from_raw(n.id), Point { x: n.x, y: n.y }))
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
                clock_id: m.clock.map(|c| c.raw()),
                drives_output_id: m.drives.map(|o| o.raw()),
            })
            .collect(),
        layout: Some(layout_to_pb(layout)),
        can_undo: info.can_undo,
        can_redo: info.can_redo,
        dirty: info.dirty,
        clocks: design
            .clocks
            .values()
            .map(|c| pb::ClockView {
                id: c.id.raw(),
                name: c.name.clone(),
            })
            .collect(),
        outputs: design
            .outputs
            .values()
            .map(|o| pb::OutputView {
                id: o.id.raw(),
                name: o.name.clone(),
                description: o.description.clone(),
                accepts: o.accepts.raw(),
                clock_id: o.clock.map(|c| c.raw()),
                required: o.required,
            })
            .collect(),
        devices: design
            .devices
            .values()
            .map(|d| pb::DeviceView {
                id: d.id.raw(),
                name: d.name.clone(),
                kind: device_kind_to_pb(d.kind).into(),
                output_id: d.output.map(|o| o.raw()),
                fixed_pins: d
                    .fixed_pins
                    .iter()
                    .map(|(i, r)| pb::DevicePin {
                        index: u32::from(*i),
                        resource: r.clone(),
                    })
                    .collect(),
                requirements: bdl_hardware::devices::requirement_labels(d.kind)
                    .into_iter()
                    .map(|(i, cap, label)| pb::RequirementLabel {
                        index: u32::from(i),
                        capability: cap.as_str().into(),
                        label: label.into(),
                    })
                    .collect(),
            })
            .collect(),
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

pub fn mapping_analysis_to_pb(m: &bdl_compiler::MappingAnalysis) -> pb::MappingAnalysis {
    use bdl_check::pretty;
    use bdl_compiler::MappingStatus;
    pb::MappingAnalysis {
        id: m.id.raw(),
        status: match m.status {
            MappingStatus::Declared => pb::MappingStatus::Declared,
            MappingStatus::Open => pb::MappingStatus::Open,
            MappingStatus::Invalid => pb::MappingStatus::Invalid,
            MappingStatus::TypeValid => pb::MappingStatus::TypeValid,
            MappingStatus::TemporallyValid => pb::MappingStatus::TemporallyValid,
            MappingStatus::ClockConsistent => pb::MappingStatus::ClockConsistent,
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
    }
}

pub fn analysis_to_pb(a: &bdl_compiler::ProjectAnalysis) -> pb::ProjectAnalysis {
    pb::ProjectAnalysis {
        revision: a.revision.raw(),
        mappings: a.mappings.values().map(mapping_analysis_to_pb).collect(),
        diagnostics: a.diagnostics.iter().map(diagnostic_to_pb).collect(),
        causal: a.causality.valid,
        clock_consistent: a.clocks.valid,
        cycles: a
            .causality
            .cycles
            .iter()
            .map(|c| pb::DeclarationCycle {
                mapping_ids: c.iter().map(|d| d.raw()).collect(),
            })
            .collect(),
        evaluation_order: a.causality.order.iter().map(|d| d.raw()).collect(),
        outputs: a
            .outputs
            .states
            .iter()
            .map(|(o, st)| {
                let claimants: Vec<u64> =
                    a.ir.drives
                        .iter()
                        .filter(|(_, sink)| *sink == o)
                        .map(|(d, _)| d.raw())
                        .collect();
                let driver = a
                    .outputs
                    .valid_bindings
                    .iter()
                    .find(|(_, sink)| *sink == o)
                    .map(|(d, _)| d.raw())
                    .filter(|_| *st == OutputState::Driven);
                pb::OutputAnalysis {
                    id: o.raw(),
                    state: match st {
                        OutputState::Undriven => pb::OutputState::Undriven,
                        OutputState::Driven => pb::OutputState::Driven,
                        OutputState::IllFormed => pb::OutputState::IllFormed,
                        OutputState::Conflict => pb::OutputState::Conflict,
                    }
                    .into(),
                    driver,
                    claimants,
                }
            })
            .collect(),
        open_outputs: a.open_outputs.iter().map(|o| o.raw()).collect(),
        output_complete: a.output_complete,
    }
}

// ---------------------------------------------------------------------------
// Deployment
// ---------------------------------------------------------------------------

pub fn target_view(id: &str, hw: &bdl_hardware::Hardware) -> pb::TargetView {
    pb::TargetView {
        id: id.into(),
        name: hw.name.clone(),
        resource_count: u32::try_from(hw.resources.len()).unwrap_or(u32::MAX),
    }
}

fn placement(id: bdl_hardware::RequirementId, r: &bdl_hardware::ResourceId) -> pb::Placement {
    pb::Placement {
        device_id: id.device.raw(),
        index: u32::from(id.index),
        resource: r.0.clone(),
    }
}

pub fn deployment_to_pb(d: &bdl_compiler::DeploymentAnalysis) -> pb::DeploymentAnalysis {
    use bdl_compiler::DeploymentStatus;
    use bdl_hardware::DeadEndReason;
    pb::DeploymentAnalysis {
        revision: d.revision.raw(),
        target: d.target.clone(),
        status: match d.status {
            DeploymentStatus::Feasible => pb::DeploymentStatus::Feasible,
            DeploymentStatus::Infeasible => pb::DeploymentStatus::Infeasible,
            DeploymentStatus::Incomplete => pb::DeploymentStatus::Incomplete,
        }
        .into(),
        requirements: d
            .requirements
            .iter()
            .map(|r| pb::RequirementView {
                device_id: r.id.device.raw(),
                index: u32::from(r.id.index),
                capability: r.capability.as_str().into(),
                fixed: r.fixed.as_ref().map(|f| f.0.clone()),
                label: r.label.clone(),
            })
            .collect(),
        assignment: d
            .assignment
            .iter()
            .flatten()
            .map(|(id, r)| placement(*id, r))
            .collect(),
        dead_end: d.dead_end.as_ref().map(|e| pb::DeadEnd {
            device_id: e.requirement.device.raw(),
            index: u32::from(e.requirement.index),
            reason: Some(match &e.reason {
                DeadEndReason::NoCapableResource => {
                    pb::dead_end::Reason::NoCapableResource(pb::Unit {})
                }
                DeadEndReason::FixedUnavailable { fixed } => {
                    pb::dead_end::Reason::FixedUnavailable(fixed.0.clone())
                }
                DeadEndReason::Blocked { candidates } => {
                    pb::dead_end::Reason::Blocked(pb::BlockedCandidates {
                        candidates: candidates
                            .iter()
                            .map(|(r, by)| pb::BlockedCandidate {
                                resource: r.0.clone(),
                                held_by_device_id: by.device.raw(),
                                held_by_index: u32::from(by.index),
                            })
                            .collect(),
                    })
                }
            }),
            placed: e.placed.iter().map(|(id, r)| placement(*id, r)).collect(),
        }),
        unbound_devices: d.unbound_devices.iter().map(|x| x.raw()).collect(),
        unrealised_outputs: d.unrealised_outputs.iter().map(|x| x.raw()).collect(),
        diagnostics: d.diagnostics.iter().map(diagnostic_to_pb).collect(),
    }
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

use bdl_reactive::Value;

pub fn value_to_pb(v: &Value) -> pb::Value {
    use pb::value::Kind;
    let kind = match v {
        Value::Bool { value } => Kind::Boolean(*value),
        Value::Nat { value } => Kind::Count(*value),
        Value::Quantity { dim, value } => Kind::Quantity(pb::Quantity {
            dim: Some(dim_to_pb(*dim)),
            value: *value,
        }),
        Value::Semantic { id, repr } => Kind::Semantic(Box::new(pb::SemanticValue {
            concept_id: id.raw(),
            repr: Some(Box::new(value_to_pb(repr))),
        })),
        Value::None => Kind::None(pb::Unit {}),
        Value::Some { value } => Kind::Some(Box::new(value_to_pb(value))),
        Value::Closure { .. } => Kind::Opaque("<function>".into()),
        Value::Prim { prim, .. } => Kind::Opaque(format!("<{prim:?}>")),
    };
    pb::Value { kind: Some(kind) }
}

pub fn value_from_pb(v: &pb::Value) -> Result<Value, ConvertError> {
    use pb::value::Kind;
    Ok(
        match v.kind.as_ref().ok_or(ConvertError::Missing("value.kind"))? {
            Kind::Boolean(b) => Value::Bool { value: *b },
            Kind::Count(n) => Value::Nat { value: *n },
            Kind::Quantity(q) => Value::Quantity {
                dim: dim_from_pb(
                    q.dim
                        .as_ref()
                        .ok_or(ConvertError::Missing("quantity.dim"))?,
                )?,
                value: q.value,
            },
            Kind::Semantic(s) => Value::Semantic {
                id: SemanticId::from_raw(s.concept_id),
                repr: Box::new(value_from_pb(
                    s.repr
                        .as_ref()
                        .ok_or(ConvertError::Missing("semantic.repr"))?,
                )?),
            },
            Kind::None(_) => Value::None,
            Kind::Some(x) => Value::Some {
                value: Box::new(value_from_pb(x)?),
            },
            Kind::Opaque(_) => {
                return Err(ConvertError::Missing(
                    "value: opaque values cannot be supplied",
                ))
            }
        },
    )
}

pub fn tick_sample_to_pb(
    t: &bdl_reactive::TickSample,
    concept_name: &dyn Fn(SemanticId) -> String,
) -> pb::TickSample {
    pb::TickSample {
        tick: t.tick,
        active_clock_ids: t.active.iter().map(|c| c.raw()).collect(),
        values: t
            .values
            .iter()
            .map(|(d, v)| pb::DeclarationSample {
                mapping_id: d.raw(),
                value: Some(value_to_pb(v)),
                rendered: v.render(concept_name),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::surface::Design;

    #[test]
    fn every_edit_op_round_trips_through_pb() {
        use bdl_model::surface::DeviceKind;
        let sem = SemanticId::from_raw;
        let decl = DeclId::from_raw;
        let clock = ClockId::from_raw;
        let output = OutputId::from_raw;
        let device = DeviceId::from_raw;
        let rep = Some(Representation::Quantity { dim: Dim::ANGLE });
        let ops = vec![
            EditOp::CreateConcept {
                name: "Tilt".into(),
                description: "lean".into(),
                representation: rep,
            },
            EditOp::RenameConcept {
                id: sem(1),
                name: "Lean".into(),
            },
            EditOp::SetConceptDescription {
                id: sem(1),
                description: "x".into(),
            },
            EditOp::SetConceptRepresentation {
                id: sem(1),
                representation: None,
            },
            EditOp::DeleteConcept { id: sem(1) },
            EditOp::CreateMapping {
                name: "m".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![sem(1), sem(2)],
                    output: sem(3),
                },
            },
            EditOp::RenameMapping {
                id: decl(4),
                name: "n".into(),
            },
            EditOp::SetMappingDescription {
                id: decl(4),
                description: "d".into(),
            },
            EditOp::SetMappingSignature {
                id: decl(4),
                signature: Signature {
                    inputs: vec![],
                    output: sem(3),
                },
            },
            EditOp::AttachDefinition {
                id: decl(4),
                definition: Definition::Formula {
                    source: "1".into(),
                },
            },
            EditOp::ReplaceDefinition {
                id: decl(4),
                definition: None,
            },
            EditOp::DeleteMapping { id: decl(4) },
            EditOp::CreateClockDomain {
                name: "main".into(),
            },
            EditOp::RenameClockDomain {
                id: clock(5),
                name: "ui".into(),
            },
            EditOp::DeleteClockDomain { id: clock(5) },
            EditOp::SetMappingClock {
                id: decl(4),
                clock: Some(clock(5)),
            },
            EditOp::CreateOutput {
                name: "motor".into(),
                description: String::new(),
                accepts: sem(3),
                clock: None,
            },
            EditOp::RenameOutput {
                id: output(6),
                name: "o".into(),
            },
            EditOp::SetOutputAccepts {
                id: output(6),
                accepts: sem(2),
            },
            EditOp::SetOutputClock {
                id: output(6),
                clock: Some(clock(5)),
            },
            EditOp::SetOutputRequired {
                id: output(6),
                required: true,
            },
            EditOp::DeleteOutput { id: output(6) },
            EditOp::SetMappingDrive {
                id: decl(4),
                output: Some(output(6)),
            },
            EditOp::CreateDevice {
                name: "drive".into(),
                kind: DeviceKind::HBridgeChannel,
                output: Some(output(6)),
            },
            EditOp::RenameDevice {
                id: device(7),
                name: "d".into(),
            },
            EditOp::SetDeviceKind {
                id: device(7),
                kind: DeviceKind::PwmChannel,
            },
            EditOp::SetDeviceOutput {
                id: device(7),
                output: None,
            },
            EditOp::SetDevicePin {
                id: device(7),
                index: 1,
                resource: Some("D3".into()),
            },
            EditOp::DeleteDevice { id: device(7) },
        ];
        for op in ops {
            let back = edit_op_from_pb(&edit_op_to_pb(&op)).unwrap();
            assert_eq!(back, op);
        }
    }

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
