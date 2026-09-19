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

pub fn representation_to_pb(r: &Representation) -> pb::Representation {
    use pb::representation::Kind;
    let kind = match r {
        Representation::Quantity { dim } => Kind::Quantity(dim_to_pb(*dim)),
        Representation::Boolean => Kind::Boolean(pb::Unit {}),
        Representation::Count => Kind::Count(pb::Unit {}),
        Representation::Optional { inner } => Kind::Optional(Box::new(representation_to_pb(inner))),
        Representation::List { element } => Kind::List(Box::new(representation_to_pb(element))),
        Representation::Pair { first, second } => Kind::Pair(Box::new(pb::PairRepresentation {
            first: Some(Box::new(representation_to_pb(first))),
            second: Some(Box::new(representation_to_pb(second))),
        })),
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
        Kind::Optional(inner) => Ok(Representation::optional(representation_from_pb(inner)?)),
        Kind::List(element) => Ok(Representation::list(representation_from_pb(element)?)),
        Kind::Pair(p) => Ok(Representation::pair(
            representation_from_pb(
                p.first
                    .as_ref()
                    .ok_or(ConvertError::Missing("representation.pair.first"))?,
            )?,
            representation_from_pb(
                p.second
                    .as_ref()
                    .ok_or(ConvertError::Missing("representation.pair.second"))?,
            )?,
        )),
    }
}

pub fn definition_to_pb(d: &Definition) -> pb::Definition {
    match d {
        Definition::Formula { source } => pb::Definition {
            kind: Some(pb::definition::Kind::Formula(source.clone())),
        },
        // The wire keeps the designer's text; the pinned scope is an
        // elaboration detail of a derived design.
        Definition::ScopedFormula { source, .. } => pb::Definition {
            kind: Some(pb::definition::Kind::Formula(source.clone())),
        },
        Definition::Reference { target, transport } => pb::Definition {
            kind: Some(pb::definition::Kind::Reference(pb::ReferenceDefinition {
                target: target.raw(),
                transport: transport.as_ref().map(|t| pb::TransportSpec {
                    source_clock_id: t.source.raw(),
                    init: t.init.clone(),
                }),
            })),
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
        pb::definition::Kind::Reference(r) => Ok(Definition::Reference {
            target: DeclId::from_raw(r.target),
            transport: r.transport.as_ref().map(|t| bdl_model::surface::Transport {
                source: ClockId::from_raw(t.source_clock_id),
                init: t.init.clone(),
            }),
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
            Op::SetConceptOrdered(m) => EditOp::SetConceptOrdered {
                id: sem(m.id),
                ordered: m.ordered,
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
                definition: m.definition.as_ref().map(definition_from_pb).transpose()?,
                clock: m.clock_id.map(ClockId::from_raw),
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
            representation: representation.as_ref().map(representation_to_pb),
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
                representation: representation.as_ref().map(representation_to_pb),
            })
        }
        EditOp::SetConceptOrdered { id, ordered } => Op::SetConceptOrdered(pb::SetConceptOrdered {
            id: id.raw(),
            ordered: *ordered,
        }),
        EditOp::DeleteConcept { id } => Op::DeleteConcept(pb::DeleteConcept { id: id.raw() }),
        EditOp::CreateMapping {
            name,
            description,
            signature,
            definition,
            clock,
        } => Op::CreateMapping(pb::CreateMapping {
            name: name.clone(),
            description: description.clone(),
            signature: Some(signature_to_pb(signature)),
            definition: definition.as_ref().map(definition_to_pb),
            clock_id: clock.map(|c| c.raw()),
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
        EditError::OrderNeedsQuantity { .. } => "edit.order_needs_quantity",
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
        instances: l
            .instances
            .iter()
            .map(|(id, p)| pb::NodePosition {
                id: *id,
                x: p.x,
                y: p.y,
            })
            .collect(),
        groups: l
            .groups
            .iter()
            .map(|(id, g)| pb::GroupBox {
                id: *id,
                x: g.x,
                y: g.y,
                width: g.width,
                height: g.height,
                collapsed: g.collapsed,
            })
            .collect(),
        components: l
            .components
            .iter()
            .map(|(id, inner)| pb::ComponentLayout {
                id: *id,
                layout: Some(layout_to_pb(inner)),
            })
            .collect(),
        viewport: l.viewport.map(|v| pb::Viewport {
            x: v.x,
            y: v.y,
            zoom: v.zoom,
        }),
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
        instances: l
            .instances
            .iter()
            .map(|n| (n.id, Point { x: n.x, y: n.y }))
            .collect(),
        groups: l
            .groups
            .iter()
            .map(|g| {
                (
                    g.id,
                    bdl_model::layout::GroupBox {
                        x: g.x,
                        y: g.y,
                        width: g.width,
                        height: g.height,
                        collapsed: g.collapsed,
                    },
                )
            })
            .collect(),
        components: l
            .components
            .iter()
            .map(|c| {
                (
                    c.id,
                    c.layout.as_ref().map(layout_from_pb).unwrap_or_default(),
                )
            })
            .collect(),
        viewport: l.viewport.as_ref().map(|v| bdl_model::layout::Viewport {
            x: v.x,
            y: v.y,
            zoom: v.zoom,
        }),
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
    /// The snapshot is the derived flattening of a behaviour system.
    pub derived: bool,
    /// The system is loaded from and written as text (ADR-0020).
    pub textual: bool,
}

/// Render a snapshot for the editor.  Deterministic: ordered maps in, ordered
/// lists out.
pub fn projection(
    snapshot: &ProjectSnapshot,
    layout: &Layout,
    info: &SessionInfo,
) -> pb::ProjectProjection {
    let mut p = design_projection(&snapshot.design);
    p.revision = snapshot.revision.raw();
    p.root_path = info.root_path.clone();
    p.layout = Some(layout_to_pb(layout));
    p.can_undo = info.can_undo;
    p.can_redo = info.can_redo;
    p.dirty = info.dirty;
    // One kind of project (ADR-0023): the field stays for clients that
    // still read it and always says the unified kind.
    let _ = (info.textual, info.derived);
    p.set_kind(pb::ProjectKind::Text);
    p
}

/// A design's views alone — no revision, layout or session fields.  Used
/// for the open project and for a component body.
pub fn design_projection(design: &bdl_model::surface::Design) -> pb::ProjectProjection {
    pb::ProjectProjection {
        revision: 0,
        name: design.name.clone(),
        root_path: String::new(),
        kind: pb::ProjectKind::Flat.into(),
        concepts: design
            .concepts
            .values()
            .map(|c| pb::ConceptView {
                id: c.id.raw(),
                name: c.name.clone(),
                description: c.description.clone(),
                representation: c.representation.as_ref().map(representation_to_pb),
                ordered: c.ordered,
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
        layout: None,
        can_undo: false,
        can_redo: false,
        dirty: false,
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
        references: Vec::new(),
    }
}

pub fn analysis_to_pb(a: &bdl_compiler::ProjectAnalysis) -> pb::ProjectAnalysis {
    pb::ProjectAnalysis {
        revision: a.revision.raw(),
        mappings: a
            .mappings
            .values()
            .map(|m| pb::MappingAnalysis {
                references: a
                    .dependencies
                    .all
                    .get(&m.id)
                    .into_iter()
                    .flatten()
                    .map(|d| d.raw())
                    .collect(),
                ..mapping_analysis_to_pb(m)
            })
            .collect(),
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
// Concept libraries
// ---------------------------------------------------------------------------

pub fn role_hint_to_pb(r: bdl_library::RoleHint) -> pb::RoleHint {
    match r {
        bdl_library::RoleHint::Input => pb::RoleHint::Input,
        bdl_library::RoleHint::Output => pb::RoleHint::Output,
        bdl_library::RoleHint::Either => pb::RoleHint::Either,
    }
}

#[allow(deprecated)] // the 0.16 fields are set empty for 0.16 clients
pub fn concept_template_view(t: &bdl_library::ConceptTemplate) -> pb::ConceptTemplateView {
    pb::ConceptTemplateView {
        id: t.id.clone(),
        display_name: t.display_name.clone(),
        default_name: t.default_name.clone(),
        description: t.description.clone(),
        category: t.category.clone(),
        role_hint: role_hint_to_pb(t.role_hint).into(),
        representation: t.representation().as_ref().map(representation_to_pb),
        type_name: t.type_name().unwrap_or("").to_owned(),
        unit: t.unit_symbol().to_owned(),
        keywords: t.keywords.clone(),
        icon: t.icon.clone(),
        // deprecated since 0.17: never set
        source_default_name: String::new(),
        display_names: Default::default(),
        descriptions: Default::default(),
    }
}

pub fn concept_library_view(l: &bdl_library::Library) -> pb::ConceptLibraryView {
    pb::ConceptLibraryView {
        id: l.info.id.clone(),
        name: l.info.name.clone(),
        schema_version: l.info.schema_version,
        version: l.info.version.clone(),
        templates: l.templates().iter().map(concept_template_view).collect(),
    }
}

/// Every library served plus the shared quantity vocabulary.
pub fn concept_templates_response(set: &bdl_library::LibrarySet) -> pb::ConceptTemplatesResponse {
    pb::ConceptTemplatesResponse {
        libraries: set.libraries().iter().map(concept_library_view).collect(),
        quantities: bdl_model::quantity::QUANTITIES
            .iter()
            .map(|q| pb::QuantityView {
                id: q.id.to_owned(),
                type_name: q.type_name.to_owned(),
                unit: q.unit.to_owned(),
                dim: Some(dim_to_pb(q.dim)),
            })
            .collect(),
    }
}

pub fn library_object_view(o: &bdl_library::CreatedObject) -> pb::LibraryObjectView {
    pb::LibraryObjectView {
        kind: o.kind.clone(),
        key: o.key.clone(),
        name: o.name.clone(),
        type_name: o.type_name.clone(),
        signature: o.signature.clone(),
        description: o.description.clone(),
        representation: o.representation.as_ref().map(representation_to_pb),
        unit: o.unit.clone(),
    }
}

pub fn library_item_view(i: &bdl_library::LibraryItem) -> pb::LibraryItemView {
    pb::LibraryItemView {
        id: i.id.clone(),
        category: i.category.as_str().to_owned(),
        display_name: i.display_name.clone(),
        description: i.description.clone(),
        group: i.group.clone(),
        keywords: i.keywords.clone(),
        icon: i.icon.clone(),
        creates: i.creates().iter().map(library_object_view).collect(),
        concept: i.as_concept_template().as_ref().map(concept_template_view),
    }
}

pub fn library_view(l: &bdl_library::Library) -> pb::LibraryView {
    pb::LibraryView {
        id: l.info.id.clone(),
        name: l.info.name.clone(),
        schema_version: l.info.schema_version,
        version: l.info.version.clone(),
        items: l.items().iter().map(library_item_view).collect(),
    }
}

/// Every library served, as items, plus the shared quantity vocabulary.
pub fn library_items_response(set: &bdl_library::LibrarySet) -> pb::LibraryItemsResponse {
    pb::LibraryItemsResponse {
        libraries: set.libraries().iter().map(library_view).collect(),
        quantities: concept_templates_response(set).quantities,
    }
}

// ---------------------------------------------------------------------------

pub fn target_view(id: &str, hw: &bdl_hardware::Hardware) -> pb::TargetView {
    let t = bdl_hardware::boards::describe(hw);
    pb::TargetView {
        id: id.into(),
        name: t.display_name.clone(),
        resource_count: u32::try_from(t.resource_count).unwrap_or(u32::MAX),
        display_name: t.display_name,
        description: t.description,
        family: t.family,
        capabilities: t
            .capabilities
            .iter()
            .map(|c| pb::CapabilitySummary {
                capability: c.capability.as_str().into(),
                label: c.capability.label().into(),
                resource_count: u32::try_from(c.resource_count).unwrap_or(u32::MAX),
                shareable: c.shareable,
            })
            .collect(),
    }
}

/// The read-model fields of a deployment message, from a composed report.
pub fn report_to_pb(r: &bdl_compiler::DeploymentReport) -> pb::DeploymentAnalysis {
    use bdl_compiler::{BlockerKind, MissingKind};
    let mut m = pb::DeploymentAnalysis {
        revision: r.revision.raw(),
        target: r.target.id.clone(),
        target_display_name: r.target.display_name.clone(),
        design_ready: r.design_ready,
        deployable: r.deployable,
        ..Default::default()
    };
    m.set_status(deployment_status_to_pb(r.status));
    m.missing = r
        .missing
        .iter()
        .map(|x| {
            let mut item = pb::MissingItem {
                output_id: x.output.map(|o| o.raw()),
                output_name: x.output_name.clone().unwrap_or_default(),
                device_id: x.device.map(|d| d.raw()),
                device_name: x.device_name.clone().unwrap_or_default(),
                mapping_id: x.mapping.map(|d| d.raw()),
                mapping_name: x.mapping_name.clone().unwrap_or_default(),
                message: x.message.clone(),
                explanation: x.explanation.clone(),
                ..Default::default()
            };
            item.set_kind(match x.kind {
                MissingKind::RelationshipNotChecking => pb::MissingKind::RelationshipNotChecking,
                MissingKind::NotCausal => pb::MissingKind::NotCausal,
                MissingKind::NotClockConsistent => pb::MissingKind::NotClockConsistent,
                MissingKind::OutputNoDomain => pb::MissingKind::OutputNoDomain,
                MissingKind::OutputNoDriver => pb::MissingKind::OutputNoDriver,
                MissingKind::OutputConnectionInvalid => pb::MissingKind::OutputConnectionInvalid,
                MissingKind::OutputNoDevice => pb::MissingKind::OutputNoDevice,
                MissingKind::DeviceNoOutput => pb::MissingKind::DeviceNoOutput,
            });
            item
        })
        .collect();
    m.rows = r
        .rows
        .iter()
        .map(|x| {
            let mut row = pb::AssignmentRow {
                output_id: x.output.map(|o| o.raw()),
                output_name: x.output_name.clone().unwrap_or_default(),
                device_id: x.device.raw(),
                device_name: x.device_name.clone(),
                device_kind_label: x.device_kind_label.clone(),
                requirement_index: u32::from(x.requirement_index),
                requirement_label: x.requirement_label.clone(),
                capability: x.capability.as_str().into(),
                capability_label: x.capability_label.clone(),
                fixed: x.fixed.as_ref().map(|f| f.0.clone()),
                resource: x.resource.as_ref().map(|f| f.0.clone()),
                resource_label: x.resource_label.clone(),
                ..Default::default()
            };
            row.set_device_kind(device_kind_to_pb(x.device_kind));
            row
        })
        .collect();
    m.blocker = r.blocker.as_ref().map(|b| pb::Blocker {
        device_id: b.device.raw(),
        device_name: b.device_name.clone(),
        requirement_index: u32::from(b.requirement_index),
        requirement_label: b.requirement_label.clone(),
        capability: b.capability.as_str().into(),
        capability_label: b.capability_label.clone(),
        kind: Some(match &b.kind {
            BlockerKind::NoCapableResource => pb::blocker::Kind::NoCapableResource(pb::Unit {}),
            BlockerKind::FixedUnavailable { pin } => {
                pb::blocker::Kind::FixedUnavailable(pin.0.clone())
            }
            BlockerKind::Blocked { candidates } => {
                pb::blocker::Kind::Blocked(pb::BlockerCandidates {
                    candidates: candidates
                        .iter()
                        .map(|c| pb::BlockerCandidate {
                            resource: c.resource.0.clone(),
                            resource_label: c.resource_label.clone(),
                            held_by_device_id: c.held_by_device.raw(),
                            held_by_device_name: c.held_by_device_name.clone(),
                            held_by_requirement_index: u32::from(c.held_by_requirement_index),
                            held_by_requirement_label: c.held_by_requirement_label.clone(),
                        })
                        .collect(),
                })
            }
        }),
        message: b.message.clone(),
        explanation: b.explanation.clone(),
    });
    m.diagnostics = r.diagnostics.iter().map(diagnostic_to_pb).collect();
    m
}

fn deployment_status_to_pb(s: bdl_compiler::DeploymentStatus) -> pb::DeploymentStatus {
    use bdl_compiler::DeploymentStatus;
    match s {
        DeploymentStatus::Feasible => pb::DeploymentStatus::Feasible,
        DeploymentStatus::Infeasible => pb::DeploymentStatus::Infeasible,
        DeploymentStatus::Incomplete => pb::DeploymentStatus::Incomplete,
    }
}

/// Both layers: the analysis as computed plus the composed read model.
pub fn deployment_with_report_to_pb(
    d: &bdl_compiler::DeploymentAnalysis,
    r: &bdl_compiler::DeploymentReport,
) -> pb::DeploymentAnalysis {
    let mut m = deployment_to_pb(d);
    let report = report_to_pb(r);
    m.target_display_name = report.target_display_name;
    m.design_ready = report.design_ready;
    m.deployable = report.deployable;
    m.missing = report.missing;
    m.rows = report.rows;
    m.blocker = report.blocker;
    m
}

fn placement(id: bdl_hardware::RequirementId, r: &bdl_hardware::ResourceId) -> pb::Placement {
    pb::Placement {
        device_id: id.device.raw(),
        index: u32::from(id.index),
        resource: r.0.clone(),
    }
}

pub fn deployment_to_pb(d: &bdl_compiler::DeploymentAnalysis) -> pb::DeploymentAnalysis {
    use bdl_hardware::DeadEndReason;
    pb::DeploymentAnalysis {
        revision: d.revision.raw(),
        target: d.target.clone(),
        status: deployment_status_to_pb(d.status).into(),
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
        ..Default::default()
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
        Value::List { items } => Kind::List(pb::ValueList {
            items: items.iter().map(value_to_pb).collect(),
        }),
        Value::Pair { fst, snd } => Kind::Pair(Box::new(pb::ValuePair {
            fst: Some(Box::new(value_to_pb(fst))),
            snd: Some(Box::new(value_to_pb(snd))),
        })),
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
            Kind::List(l) => Value::List {
                items: l
                    .items
                    .iter()
                    .map(value_from_pb)
                    .collect::<Result<bdl_reactive::value::List, _>>()?,
            },
            Kind::Pair(p) => Value::Pair {
                fst: Box::new(value_from_pb(
                    p.fst.as_ref().ok_or(ConvertError::Missing("pair.fst"))?,
                )?),
                snd: Box::new(value_from_pb(
                    p.snd.as_ref().ok_or(ConvertError::Missing("pair.snd"))?,
                )?),
            },
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

// ---------------------------------------------------------------------------
// Behaviour systems
// ---------------------------------------------------------------------------

pub mod system {
    use super::*;
    use bdl_system::{
        Acceptance, BehaviorComponent, BehaviorGroupId, BehaviorSystem, BindingEnd,
        BindingTransport, ClockContract, ComponentId, ComponentInstanceId, ExtractionChoices,
        ExtractionPreview, GroupBoundary, GroupEditOp, GroupScope, LocalEntity, OriginMap,
        ParameterValue, PortContract, PortId, PortKind, PortRef, PortStatus, SystemAnalysis,
        SystemEditOp, SystemEditOutcome, SystemSnapshot,
    };
    use std::collections::BTreeMap;

    pub fn contract_to_pb(c: &BehaviorComponent, k: &PortContract) -> pb::PortContractView {
        let name = |s: SemanticId| {
            c.body
                .concepts
                .get(&s)
                .map(|x| x.name.clone())
                .unwrap_or_default()
        };
        let mut v = pb::PortContractView {
            signature: Some(signature_to_pb(&k.signature)),
            input_names: k.signature.inputs.iter().map(|s| name(*s)).collect(),
            output_name: name(k.signature.output),
            shared: k
                .signature
                .inputs
                .iter()
                .chain([&k.signature.output])
                .filter_map(|s| {
                    c.shared_concepts.get(s).map(|g| pb::IdPair {
                        local: s.raw(),
                        system: g.raw(),
                    })
                })
                .collect(),
            clock_id: k.clock.local().map(|x| x.raw()),
            clock_name: k
                .clock
                .local()
                .and_then(|x| c.body.clocks.get(&x))
                .map(|x| x.name.clone())
                .unwrap_or_default(),
            commitments: k
                .commitments
                .iter()
                .map(|p| format!("{p:?}").to_lowercase())
                .collect(),
            ..Default::default()
        };
        v.set_clock_kind(match k.clock {
            ClockContract::Agnostic => pb::ClockContractKind::Agnostic,
            ClockContract::Parameter { .. } => pb::ClockContractKind::Parameter,
            ClockContract::Private { .. } => pb::ClockContractKind::Private,
        });
        v
    }

    pub fn contract_from_pb(v: &pb::PortContractView) -> Result<PortContract, ConvertError> {
        let signature = signature_from_pb(
            v.signature
                .as_ref()
                .ok_or(ConvertError::Missing("port_contract.signature"))?,
        );
        let local = || {
            v.clock_id
                .map(ClockId::from_raw)
                .ok_or(ConvertError::Missing("port_contract.clock_id"))
        };
        let clock = match v.clock_kind() {
            pb::ClockContractKind::Agnostic => ClockContract::Agnostic,
            pb::ClockContractKind::Parameter => ClockContract::Parameter { clock: local()? },
            pb::ClockContractKind::Private => ClockContract::Private { clock: local()? },
            pb::ClockContractKind::Unspecified => {
                return Err(ConvertError::Invalid("port_contract.clock_kind"))
            }
        };
        let mut commitments = Vec::new();
        for c in &v.commitments {
            commitments.push(match c.as_str() {
                "monotone" => bdl_ir::PropertyId::Monotone,
                "deterministic" => bdl_ir::PropertyId::Deterministic,
                "total" => bdl_ir::PropertyId::Total,
                "boundedrange" | "bounded_range" => bdl_ir::PropertyId::BoundedRange,
                _ => return Err(ConvertError::Invalid("port_contract.commitments")),
            });
        }
        Ok(PortContract {
            signature,
            commitments,
            clock,
        })
    }

    pub fn port_kind_to_pb(k: PortKind) -> pb::PortKind {
        match k {
            PortKind::Required => pb::PortKind::Required,
            PortKind::Provided => pb::PortKind::Provided,
            PortKind::Parameter => pb::PortKind::Parameter,
        }
    }

    pub fn port_kind_from_pb(k: pb::PortKind) -> Result<PortKind, ConvertError> {
        Ok(match k {
            pb::PortKind::Required => PortKind::Required,
            pb::PortKind::Provided => PortKind::Provided,
            pb::PortKind::Parameter => PortKind::Parameter,
            pb::PortKind::Unspecified => return Err(ConvertError::Invalid("port_kind")),
        })
    }

    fn port_ref_to_pb(r: PortRef) -> pb::PortRefView {
        pb::PortRefView {
            instance: r.instance.raw(),
            port: r.port.raw(),
            base_decl: None,
        }
    }

    fn port_ref_from_pb(
        r: Option<&pb::PortRefView>,
        field: &'static str,
    ) -> Result<PortRef, ConvertError> {
        let r = r.ok_or(ConvertError::Missing(field))?;
        if r.base_decl.is_some() {
            return Err(ConvertError::Invalid(field));
        }
        Ok(PortRef {
            instance: ComponentInstanceId::from_raw(r.instance),
            port: PortId::from_raw(r.port),
        })
    }

    fn end_to_pb(e: BindingEnd) -> pb::PortRefView {
        match e {
            BindingEnd::Port(r) => port_ref_to_pb(r),
            BindingEnd::Base { decl } => pb::PortRefView {
                instance: 0,
                port: 0,
                base_decl: Some(decl.raw()),
            },
        }
    }

    fn end_from_pb(
        r: Option<&pb::PortRefView>,
        field: &'static str,
    ) -> Result<BindingEnd, ConvertError> {
        let r = r.ok_or(ConvertError::Missing(field))?;
        Ok(match r.base_decl {
            Some(d) => BindingEnd::Base {
                decl: DeclId::from_raw(d),
            },
            None => BindingEnd::Port(PortRef {
                instance: ComponentInstanceId::from_raw(r.instance),
                port: PortId::from_raw(r.port),
            }),
        })
    }

    fn pairs<'a, A: Copy + 'a, B: Copy + 'a>(
        it: impl Iterator<Item = (&'a A, &'a B)>,
        raw: impl Fn(A) -> u64,
        raw_b: impl Fn(B) -> u64,
    ) -> Vec<pb::IdPair> {
        it.map(|(a, b)| pb::IdPair {
            local: raw(*a),
            system: raw_b(*b),
        })
        .collect()
    }

    fn origin_view(
        flat: u64,
        sort: pb::LocalSort,
        o: &bdl_system::Origin,
        port: Option<PortId>,
    ) -> pb::OriginView {
        let local = match o.local {
            LocalEntity::Decl(d) => d.raw(),
            LocalEntity::Sem(s) => s.raw(),
            LocalEntity::Clock(c) => c.raw(),
            LocalEntity::Output(o) => o.raw(),
            LocalEntity::Device(d) => d.raw(),
        };
        let mut v = pb::OriginView {
            flat,
            instance: o.instance.raw(),
            component: o.component.raw(),
            local,
            port: port.map(|p| p.raw()),
            ..Default::default()
        };
        v.set_sort(sort);
        v
    }

    pub fn origins_to_pb(origins: &OriginMap) -> Vec<pb::OriginView> {
        let mut out = Vec::new();
        for (d, o) in &origins.decls {
            out.push(origin_view(
                d.raw(),
                pb::LocalSort::Decl,
                o,
                origins.ports.get(d).map(|p| p.port),
            ));
        }
        for (s, o) in &origins.sems {
            out.push(origin_view(s.raw(), pb::LocalSort::Sem, o, None));
        }
        for (c, o) in &origins.clocks {
            out.push(origin_view(c.raw(), pb::LocalSort::Clock, o, None));
        }
        for (x, o) in &origins.outputs {
            out.push(origin_view(x.raw(), pb::LocalSort::Output, o, None));
        }
        for (d, o) in &origins.devices {
            out.push(origin_view(d.raw(), pb::LocalSort::Device, o, None));
        }
        out
    }

    pub fn system_view(
        snapshot: &SystemSnapshot,
        origins: &OriginMap,
        authoring_generation: u64,
        boundaries: &BTreeMap<BehaviorGroupId, GroupBoundary>,
        dirty: bool,
        definition_drafts: &BTreeMap<(Option<ComponentId>, DeclId), String>,
    ) -> pb::SystemView {
        let s: &BehaviorSystem = &snapshot.system;
        pb::SystemView {
            definition_drafts: definition_drafts
                .iter()
                .map(|((component, mapping), source)| pb::DefinitionDraftView {
                    component: component.map(|c| c.raw()),
                    mapping_id: mapping.raw(),
                    source: source.clone(),
                })
                .collect(),
            revision: snapshot.revision.raw(),
            name: s.base.name.clone(),
            base: Some(design_projection(&s.base)),
            components: s
                .components
                .values()
                .map(|c| pb::ComponentView {
                    id: c.id.raw(),
                    name: c.name.clone(),
                    description: c.description.clone(),
                    ports: c
                        .interface
                        .ports
                        .values()
                        .map(|p| {
                            let mut v = pb::PortView {
                                id: p.id.raw(),
                                name: p.name.clone(),
                                description: p.description.clone(),
                                decl: p.decl.raw(),
                                contract: Some(contract_to_pb(c, &p.contract)),
                                ..Default::default()
                            };
                            v.set_kind(port_kind_to_pb(p.kind));
                            v
                        })
                        .collect(),
                    clock_params: c.interface.clock_params.iter().map(|k| k.raw()).collect(),
                    shared_concepts: pairs(
                        c.shared_concepts.iter(),
                        |a: SemanticId| a.raw(),
                        |b: SemanticId| b.raw(),
                    ),
                    external_outputs: pairs(
                        c.external_outputs.iter(),
                        |a: OutputId| a.raw(),
                        |b: OutputId| b.raw(),
                    ),
                    body: Some(design_projection(&c.body)),
                    stamp: c.body_stamp,
                    interface_stamp: c.interface_stamp,
                })
                .collect(),
            instances: s
                .instances
                .values()
                .map(|i| pb::ComponentInstanceView {
                    id: i.id.raw(),
                    component: i.component.raw(),
                    name: i.name.clone(),
                    clock_bindings: pairs(
                        i.clock_bindings.iter(),
                        |a: ClockId| a.raw(),
                        |b: ClockId| b.raw(),
                    ),
                    parameter_bindings: i
                        .parameter_bindings
                        .iter()
                        .map(|(p, v)| pb::ParameterBindingView {
                            port: p.raw(),
                            source: v.source.clone(),
                        })
                        .collect(),
                })
                .collect(),
            bindings: s
                .bindings
                .values()
                .map(|b| pb::BindingView {
                    id: b.id.raw(),
                    source: Some(end_to_pb(b.source)),
                    destination: Some(end_to_pb(b.destination)),
                    transport_init: b.transport.as_ref().map(|t| t.init.clone()),
                })
                .collect(),
            exports: s
                .exports
                .values()
                .map(|e| pb::ExportView {
                    id: e.id.raw(),
                    port: Some(port_ref_to_pb(e.port)),
                    name: e.name.clone(),
                })
                .collect(),
            origins: origins_to_pb(origins),
            is_flat: s.is_flat(),
            groups: s
                .groups
                .values()
                .map(|g| pb::BehaviorGroupView {
                    id: g.id.raw(),
                    name: g.name.clone(),
                    description: g.description.clone(),
                    members: g.members.iter().map(|d| d.raw()).collect(),
                    component: match g.scope {
                        GroupScope::SystemBase => None,
                        GroupScope::Component { component } => Some(component.raw()),
                    },
                })
                .collect(),
            authoring_generation,
            dirty,
            boundaries: boundaries
                .iter()
                .map(|(id, b)| boundary_to_pb(*id, b))
                .collect(),
        }
    }

    pub fn group_edit_op_from_pb(op: &pb::GroupEditOp) -> Result<GroupEditOp, ConvertError> {
        use pb::group_edit_op::Op;
        let g = BehaviorGroupId::from_raw;
        let d = DeclId::from_raw;
        Ok(
            match op
                .op
                .as_ref()
                .ok_or(ConvertError::Missing("group_edit_op.op"))?
            {
                Op::CreateGroup(m) => GroupEditOp::CreateGroup {
                    scope: match m.component {
                        None => GroupScope::SystemBase,
                        Some(c) => GroupScope::Component {
                            component: ComponentId::from_raw(c),
                        },
                    },
                    name: m.name.clone(),
                    description: m.description.clone(),
                    members: m.members.iter().map(|x| d(*x)).collect(),
                },
                Op::RenameGroup(m) => GroupEditOp::RenameGroup {
                    id: g(m.id),
                    name: m.name.clone(),
                },
                Op::SetGroupDescription(m) => GroupEditOp::SetGroupDescription {
                    id: g(m.id),
                    description: m.description.clone(),
                },
                Op::DeleteGroup(m) => GroupEditOp::DeleteGroup { id: g(m.id) },
                Op::AddMember(m) => GroupEditOp::AddMember {
                    group: g(m.group),
                    decl: d(m.decl),
                },
                Op::RemoveMember(m) => GroupEditOp::RemoveMember {
                    group: g(m.group),
                    decl: d(m.decl),
                },
                Op::MoveMember(m) => GroupEditOp::MoveMember {
                    decl: d(m.decl),
                    to: g(m.to),
                },
                Op::MergeGroups(m) => GroupEditOp::MergeGroups {
                    into: g(m.into),
                    from: g(m.from),
                },
                Op::SplitGroup(m) => GroupEditOp::SplitGroup {
                    id: g(m.id),
                    name: m.name.clone(),
                    members: m.members.iter().map(|x| d(*x)).collect(),
                },
            },
        )
    }

    pub fn choices_from_pb(c: Option<&pb::ExtractionChoices>) -> ExtractionChoices {
        match c {
            None => ExtractionChoices::default(),
            Some(c) => ExtractionChoices {
                name: c.name.clone(),
                instance_name: c.instance_name.clone(),
                keep_internal: c
                    .keep_internal
                    .iter()
                    .map(|d| DeclId::from_raw(*d))
                    .collect(),
                internalize_sinks: c
                    .internalize_sinks
                    .iter()
                    .map(|o| OutputId::from_raw(*o))
                    .collect(),
            },
        }
    }

    fn raws(v: &[DeclId]) -> Vec<u64> {
        v.iter().map(|d| d.raw()).collect()
    }

    pub fn boundary_to_pb(id: BehaviorGroupId, b: &GroupBoundary) -> pb::BehaviorGroupBoundaryView {
        pb::BehaviorGroupBoundaryView {
            id: id.raw(),
            members: raws(&b.members),
            crossing_in: raws(&b.crossing_in),
            crossing_out: raws(&b.crossing_out),
            open_members: raws(&b.open_members),
            driven_members: raws(&b.driven_members),
            private_candidates: raws(&b.private_candidates),
            external_inputs: raws(&b.external_inputs),
            external_outputs: raws(&b.external_outputs),
            clocks: b.clocks.iter().map(|c| c.raw()).collect(),
            internal_edges: b
                .internal_edges
                .iter()
                .map(|(a, b)| pb::DeclEdge {
                    from: a.raw(),
                    to: b.raw(),
                })
                .collect(),
            crossing_edges: b
                .crossing_edges
                .iter()
                .map(|(a, b)| pb::DeclEdge {
                    from: a.raw(),
                    to: b.raw(),
                })
                .collect(),
        }
    }

    pub fn preview_to_pb(p: &ExtractionPreview) -> pb::ExtractionPreviewView {
        let port = |x: &bdl_system::PreviewPort| {
            let mut v = pb::PreviewPortView {
                decl: x.decl.raw(),
                name: x.name.clone(),
                concept: x.concept.raw(),
                ..Default::default()
            };
            v.set_kind(port_kind_to_pb(x.kind));
            v
        };
        pb::ExtractionPreviewView {
            group: p.group.raw(),
            name: p.name.clone(),
            instance_name: p.instance_name.clone(),
            boundary: Some(boundary_to_pb(p.group, &p.boundary)),
            required: p.required.iter().map(port).collect(),
            provided: p.provided.iter().map(port).collect(),
            open_members: p
                .open_members
                .iter()
                .map(|o| pb::OpenMemberDecisionView {
                    decl: o.decl.raw(),
                    name: o.name.clone(),
                    as_input: o.as_input,
                })
                .collect(),
            clocks: p.clocks.iter().map(|c| c.raw()).collect(),
            sinks: p
                .sinks
                .iter()
                .map(|d| pb::SinkDecisionView {
                    output: d.output.raw(),
                    name: d.name.clone(),
                    drivers: raws(&d.drivers),
                    internal: d.internal,
                })
                .collect(),
            private: raws(&p.private),
            warnings: p.warnings.iter().map(diagnostic_to_pb).collect(),
        }
    }

    pub fn system_edit_op_from_pb(op: &pb::SystemEditOp) -> Result<SystemEditOp, ConvertError> {
        use pb::system_edit_op::Op;
        let comp = ComponentId::from_raw;
        let inst = ComponentInstanceId::from_raw;
        let port = PortId::from_raw;
        Ok(
            match op
                .op
                .as_ref()
                .ok_or(ConvertError::Missing("system_edit_op.op"))?
            {
                Op::Base(e) => SystemEditOp::Base {
                    op: edit_op_from_pb(e)?,
                },
                Op::CreateComponent(m) => SystemEditOp::CreateComponent {
                    name: m.name.clone(),
                    description: m.description.clone(),
                },
                Op::RenameComponent(m) => SystemEditOp::RenameComponent {
                    id: comp(m.id),
                    name: m.name.clone(),
                },
                Op::SetComponentDescription(m) => SystemEditOp::SetComponentDescription {
                    id: comp(m.id),
                    description: m.description.clone(),
                },
                Op::DeleteComponent(m) => SystemEditOp::DeleteComponent { id: comp(m.id) },
                Op::EditComponentBody(m) => SystemEditOp::EditComponentBody {
                    component: comp(m.component),
                    op: edit_op_from_pb(
                        m.op.as_ref()
                            .ok_or(ConvertError::Missing("edit_component_body.op"))?,
                    )?,
                },
                Op::DeclarePort(m) => SystemEditOp::DeclarePort {
                    component: comp(m.component),
                    decl: DeclId::from_raw(m.decl),
                    kind: port_kind_from_pb(m.kind())?,
                    name: m.name.clone(),
                    description: m.description.clone(),
                },
                Op::RenamePort(m) => SystemEditOp::RenamePort {
                    component: comp(m.component),
                    port: port(m.port),
                    name: m.name.clone(),
                },
                Op::RetirePort(m) => SystemEditOp::RetirePort {
                    component: comp(m.component),
                    port: port(m.port),
                },
                Op::SetClockParameter(m) => SystemEditOp::SetClockParameter {
                    component: comp(m.component),
                    clock: ClockId::from_raw(m.clock),
                    parameter: m.parameter,
                },
                Op::ShareConcept(m) => SystemEditOp::ShareConcept {
                    component: comp(m.component),
                    local: SemanticId::from_raw(m.local),
                    system: m.system.map(SemanticId::from_raw),
                },
                Op::ExternalizeOutput(m) => SystemEditOp::ExternalizeOutput {
                    component: comp(m.component),
                    local: OutputId::from_raw(m.local),
                    system: m.system.map(OutputId::from_raw),
                },
                Op::CreateInstance(m) => SystemEditOp::CreateInstance {
                    component: comp(m.component),
                    name: m.name.clone(),
                },
                Op::RenameInstance(m) => SystemEditOp::RenameInstance {
                    id: inst(m.id),
                    name: m.name.clone(),
                },
                Op::DeleteInstance(m) => SystemEditOp::DeleteInstance { id: inst(m.id) },
                Op::SetClockArgument(m) => SystemEditOp::SetClockArgument {
                    instance: inst(m.instance),
                    parameter: ClockId::from_raw(m.parameter),
                    clock: m.clock.map(ClockId::from_raw),
                },
                Op::SetParameterArgument(m) => SystemEditOp::SetParameterArgument {
                    instance: inst(m.instance),
                    port: port(m.port),
                    value: m.value.as_ref().map(|source| ParameterValue {
                        source: source.clone(),
                    }),
                },
                Op::BindPorts(m) => SystemEditOp::BindPorts {
                    source: end_from_pb(m.source.as_ref(), "bind_ports.source")?,
                    destination: end_from_pb(m.destination.as_ref(), "bind_ports.destination")?,
                    transport: m
                        .transport_init
                        .as_ref()
                        .map(|init| BindingTransport { init: init.clone() }),
                },
                Op::UnbindPorts(m) => SystemEditOp::UnbindPorts {
                    binding: bdl_system::BindingId::from_raw(m.binding),
                },
                Op::ExtractGroupAsComponent(m) => SystemEditOp::ExtractGroupAsComponent {
                    group: BehaviorGroupId::from_raw(m.group),
                    choices: choices_from_pb(m.choices.as_ref()),
                },
                Op::DeleteGroupWithMembers(m) => SystemEditOp::DeleteGroupWithMembers {
                    group: BehaviorGroupId::from_raw(m.group),
                },
                Op::ExportPort(m) => SystemEditOp::ExportPort {
                    port: port_ref_from_pb(m.port.as_ref(), "export_port.port")?,
                    name: m.name.clone(),
                },
                Op::HidePort(m) => SystemEditOp::HidePort {
                    export: bdl_system::ExportId::from_raw(m.export),
                },
                Op::ChangePortContract(m) => SystemEditOp::ChangePortContract {
                    component: comp(m.component),
                    port: port(m.port),
                    contract: contract_from_pb(
                        m.contract
                            .as_ref()
                            .ok_or(ConvertError::Missing("change_port_contract.contract"))?,
                    )?,
                },
                Op::RebindPortDeclaration(m) => SystemEditOp::RebindPortDeclaration {
                    component: comp(m.component),
                    port: port(m.port),
                    decl: DeclId::from_raw(m.decl),
                },
                Op::DuplicateComponent(m) => SystemEditOp::DuplicateComponent {
                    id: comp(m.id),
                    name: m.name.clone(),
                },
                Op::ReplaceInstanceComponent(m) => SystemEditOp::ReplaceInstanceComponent {
                    instance: inst(m.instance),
                    component: comp(m.component),
                },
            },
        )
    }

    pub fn system_outcome_to_pb(o: &SystemEditOutcome) -> pb::SystemEditOutcome {
        let flat = outcome_to_pb(&EditOutcome {
            kind: o.kind,
            invalidates: o.invalidates.clone(),
            origin_decls: o.origin_decls.clone(),
            ..Default::default()
        });
        pb::SystemEditOutcome {
            kind: flat.kind,
            invalidates: flat.invalidates,
            origin_decls: flat.origin_decls,
            instances: o.instances.iter().map(|i| i.raw()).collect(),
            created_component: o.created_component.map(|c| c.raw()),
            created_instance: o.created_instance.map(|c| c.raw()),
            created_port: o.created_port.map(|c| c.raw()),
            created_binding: o.created_binding.map(|c| c.raw()),
            created_export: o.created_export.map(|c| c.raw()),
            inner: o.inner.as_ref().map(outcome_to_pb),
            bindings: o.bindings.iter().map(|b| b.raw()).collect(),
        }
    }

    pub fn system_analysis_to_pb(a: &SystemAnalysis) -> pb::SystemAnalysisView {
        let mut v = pb::SystemAnalysisView {
            revision: a.revision.raw(),
            analysis: Some(analysis_to_pb(&a.analysis)),
            composition: a.composition.iter().map(diagnostic_to_pb).collect(),
            ports: a
                .ports
                .iter()
                .map(|(r, st)| {
                    let mut p = pb::PortStatusView {
                        port: Some(port_ref_to_pb(*r)),
                        ..Default::default()
                    };
                    match st {
                        PortStatus::Provided => p.set_status(pb::PortStatusKind::Provided),
                        PortStatus::Bound { binding } => {
                            p.set_status(pb::PortStatusKind::Bound);
                            p.binding = Some(binding.raw());
                        }
                        PortStatus::Exported { export } => {
                            p.set_status(pb::PortStatusKind::Exported);
                            p.export = Some(export.raw());
                        }
                        PortStatus::Valued => p.set_status(pb::PortStatusKind::Valued),
                        PortStatus::Open => p.set_status(pb::PortStatusKind::Open),
                    }
                    p
                })
                .collect(),
            projected: a
                .projected
                .iter()
                .map(|p| pb::ProjectedDiagnostic {
                    diagnostic: Some(diagnostic_to_pb(&p.diagnostic)),
                    origin: p.origin.as_ref().map(|o| {
                        let (flat, sort) = match p.diagnostic.entity {
                            bdl_diagnostics::Entity::Mapping { id } => {
                                (id.raw(), pb::LocalSort::Decl)
                            }
                            bdl_diagnostics::Entity::Concept { id } => {
                                (id.raw(), pb::LocalSort::Sem)
                            }
                            bdl_diagnostics::Entity::Project => (0, pb::LocalSort::Unspecified),
                        };
                        origin_view(flat, sort, o, p.port.map(|r| r.port))
                    }),
                    label: p.label.clone().unwrap_or_default(),
                })
                .collect(),
            components: a
                .components
                .iter()
                .map(|(id, realizes)| pb::ComponentStatusView {
                    id: id.raw(),
                    realizes: *realizes,
                })
                .collect(),
            groups: a
                .groups
                .iter()
                .map(|(id, b)| boundary_to_pb(*id, b))
                .collect(),
            component_analyses: a
                .component_analyses
                .iter()
                .map(|(id, an)| pb::ComponentAnalysisView {
                    id: id.raw(),
                    analysis: Some(analysis_to_pb(an)),
                })
                .collect(),
            ..Default::default()
        };
        v.set_acceptance(match a.acceptance {
            Acceptance::Invalid => pb::SystemAcceptance::Invalid,
            Acceptance::Open => pb::SystemAcceptance::Open,
            Acceptance::Executable => pb::SystemAcceptance::Executable,
        });
        v
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
                definition: None,
                clock: None,
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
                definition: Definition::Formula { source: "1".into() },
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
                ..Default::default()
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
                definition: None,
                clock: None,
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
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        let p = projection(&a.snapshot, &Layout::default(), &SessionInfo::default());
        assert_eq!(p.revision, 2);
        assert_eq!(p.mappings[0].state(), pb::AcceptanceState::Declared);
    }

    #[test]
    fn analysis_projects_the_relationships_a_definition_applies() {
        // brightness = twice(half): the value's analysis names the rule it
        // applies (and the value it reads), by identity, so Studio can point
        // from the rule to it without reading formula text.
        let mut s = ProjectSnapshot::new(Design::empty("p"));
        let step = |s: &ProjectSnapshot, op: EditOp| bdl_model::apply_edit(s, &op).unwrap();
        let a = step(
            &s,
            EditOp::CreateConcept {
                name: "Brightness".into(),
                description: String::new(),
                representation: Some(Representation::Quantity { dim: Dim::ZERO }),
            },
        );
        let bright = a.outcome.created_concept.unwrap();
        s = a.snapshot;
        let a = step(
            &s,
            EditOp::CreateMapping {
                name: "twice".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![bright],
                    output: bright,
                },
                definition: None,
                clock: None,
            },
        );
        let twice = a.outcome.created_mapping.unwrap();
        s = a.snapshot;
        s = step(
            &s,
            EditOp::AttachDefinition {
                id: twice,
                definition: Definition::Formula {
                    source: "Brightness * 2".into(),
                },
            },
        )
        .snapshot;
        let a = step(
            &s,
            EditOp::CreateMapping {
                name: "half".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output: bright,
                },
                definition: None,
                clock: None,
            },
        );
        let half = a.outcome.created_mapping.unwrap();
        s = a.snapshot;
        s = step(
            &s,
            EditOp::AttachDefinition {
                id: half,
                definition: Definition::Formula {
                    source: "0.5".into(),
                },
            },
        )
        .snapshot;
        let a = step(
            &s,
            EditOp::CreateMapping {
                name: "brightness".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![],
                    output: bright,
                },
                definition: None,
                clock: None,
            },
        );
        let value = a.outcome.created_mapping.unwrap();
        s = a.snapshot;
        s = step(
            &s,
            EditOp::AttachDefinition {
                id: value,
                definition: Definition::Formula {
                    source: "twice(half)".into(),
                },
            },
        )
        .snapshot;
        let a = analysis_to_pb(&bdl_compiler::analyze(&s));
        let refs = |id: DeclId| {
            a.mappings
                .iter()
                .find(|m| m.id == id.raw())
                .unwrap()
                .references
                .clone()
        };
        assert_eq!(refs(value), vec![twice.raw(), half.raw()]);
        assert!(refs(twice).is_empty() && refs(half).is_empty());
    }

    /// The canvas's reference edges come from the wire, never from the
    /// formula text (ADR-0034): a value's analysis names every relationship
    /// its definition references, a rule's and a Source's name none.
    #[test]
    fn analysis_carries_the_realization_refs() {
        let mut s = ProjectSnapshot::new(Design::empty("ac"));
        let apply = |s: &mut ProjectSnapshot, op: EditOp| {
            let a = bdl_model::apply_edit(s, &op).unwrap();
            *s = a.snapshot;
            a.outcome
        };
        let concept = |name: &str| EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: Some(Representation::Boolean),
        };
        let temp = apply(&mut s, concept("RoomTemp")).created_concept.unwrap();
        let held = apply(&mut s, concept("ButtonHeld"))
            .created_concept
            .unwrap();
        let switch = apply(&mut s, concept("SwitchState"))
            .created_concept
            .unwrap();
        let mapping =
            |name: &str, inputs: Vec<bdl_model::SemanticId>, output| EditOp::CreateMapping {
                name: name.into(),
                description: String::new(),
                signature: Signature { inputs, output },
                definition: None,
                clock: None,
            };
        let sensor = apply(&mut s, mapping("TempSensor", vec![], temp))
            .created_mapping
            .unwrap();
        let button = apply(&mut s, mapping("ButtonInput", vec![], held))
            .created_mapping
            .unwrap();
        let rule = apply(
            &mut s,
            mapping("AirConditionerCtrl", vec![temp, held], switch),
        )
        .created_mapping
        .unwrap();
        apply(
            &mut s,
            EditOp::AttachDefinition {
                id: rule,
                definition: Definition::Formula {
                    source: "RoomTemp and ButtonHeld".into(),
                },
            },
        );
        let value = apply(&mut s, mapping("acOn", vec![], switch))
            .created_mapping
            .unwrap();
        apply(
            &mut s,
            EditOp::AttachDefinition {
                id: value,
                definition: Definition::Formula {
                    source: "AirConditionerCtrl(TempSensor, ButtonInput)".into(),
                },
            },
        );
        let a = analysis_to_pb(&bdl_compiler::analyze(&s));
        let refs = |id: DeclId| {
            a.mappings
                .iter()
                .find(|m| m.id == id.raw())
                .map(|m| m.references.clone())
                .unwrap()
        };
        assert_eq!(
            refs(value),
            vec![sensor.raw(), button.raw(), rule.raw()],
            "ascending, each once"
        );
        assert!(
            refs(rule).is_empty(),
            "a rule reads its inputs; it references nothing"
        );
        assert!(refs(sensor).is_empty() && refs(button).is_empty());
        let draft = mapping_analysis_to_pb(&bdl_compiler::analyze(&s).mappings[&value]);
        assert!(
            draft.references.is_empty(),
            "a draft verdict carries no edges"
        );
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

    // ---- deployment read model ------------------------------------------

    #[test]
    fn target_view_carries_the_chooser_fields() {
        let hw = bdl_hardware::boards::arduino_nano();
        let t = target_view("arduino_nano", &hw);
        assert_eq!(
            (
                t.id.as_str(),
                t.display_name.as_str(),
                t.name.as_str(),
                t.family.as_str()
            ),
            ("arduino_nano", "Arduino Nano", "Arduino Nano", "avr")
        );
        assert_eq!(t.resource_count, 22);
        let pwm = t
            .capabilities
            .iter()
            .find(|c| c.capability == "pwm")
            .unwrap();
        assert_eq!(
            (pwm.label.as_str(), pwm.resource_count, pwm.shareable),
            ("PWM", 6, false)
        );
        assert!(
            t.capabilities
                .iter()
                .find(|c| c.capability == "i2c_scl")
                .unwrap()
                .shareable
        );
        assert!(t.capabilities.iter().all(|c| !c.label.is_empty()));
    }

    #[test]
    fn every_read_model_variant_reaches_the_wire() {
        use bdl_compiler::deploy_report::*;
        use bdl_compiler::DeploymentStatus;
        use bdl_hardware::{Capability, ResourceId};
        use bdl_model::surface::DeviceKind;
        let rid = |s: &str| ResourceId::new(s);
        let item = |kind: MissingKind| MissingItem {
            kind,
            output: Some(OutputId::from_raw(4)),
            output_name: Some("motor".into()),
            device: Some(DeviceId::from_raw(2)),
            device_name: Some("drive".into()),
            mapping: Some(DeclId::from_raw(7)),
            mapping_name: Some("cruise".into()),
            message: "m".into(),
            explanation: "e".into(),
        };
        let kinds = [
            MissingKind::RelationshipNotChecking,
            MissingKind::NotCausal,
            MissingKind::NotClockConsistent,
            MissingKind::OutputNoDomain,
            MissingKind::OutputNoDriver,
            MissingKind::OutputConnectionInvalid,
            MissingKind::OutputNoDevice,
            MissingKind::DeviceNoOutput,
        ];
        let mut report = DeploymentReport {
            revision: bdl_model::Revision::default(),
            target: bdl_hardware::boards::describe(&bdl_hardware::boards::big_board()),
            status: DeploymentStatus::Infeasible,
            design_ready: true,
            deployable: false,
            missing: kinds.iter().map(|k| item(*k)).collect(),
            rows: vec![AssignmentRow {
                output: Some(OutputId::from_raw(4)),
                output_name: Some("motor".into()),
                device: DeviceId::from_raw(2),
                device_name: "drive".into(),
                device_kind: DeviceKind::HBridgeChannel,
                device_kind_label: "H-bridge channel".into(),
                requirement_index: 1,
                requirement_label: "direction".into(),
                capability: Capability::DigitalOut,
                capability_label: "digital out".into(),
                fixed: Some(rid("D4")),
                resource: Some(rid("D4")),
                resource_label: Some("D4: digital in, digital out".into()),
            }],
            blocker: Some(Blocker {
                device: DeviceId::from_raw(9),
                device_name: "L7".into(),
                requirement_index: 0,
                requirement_label: "PWM".into(),
                capability: Capability::Pwm,
                capability_label: "PWM".into(),
                kind: BlockerKind::Blocked {
                    candidates: vec![BlockedCandidate {
                        resource: rid("D3"),
                        resource_label: "D3: …".into(),
                        held_by_device: DeviceId::from_raw(3),
                        held_by_device_name: "L1".into(),
                        held_by_requirement_index: 0,
                        held_by_requirement_label: "PWM".into(),
                    }],
                },
                message: "No free pin".into(),
                explanation: "Every pin…".into(),
            }),
            diagnostics: vec![],
        };
        let m = report_to_pb(&report);
        assert_eq!(m.status(), pb::DeploymentStatus::Infeasible);
        assert_eq!(m.target, "big_board");
        assert_eq!(m.target_display_name, "Big board (mock)");
        assert!(m.design_ready && !m.deployable);
        let wire_kinds: Vec<pb::MissingKind> = m.missing.iter().map(|x| x.kind()).collect();
        assert_eq!(
            wire_kinds,
            [
                pb::MissingKind::RelationshipNotChecking,
                pb::MissingKind::NotCausal,
                pb::MissingKind::NotClockConsistent,
                pb::MissingKind::OutputNoDomain,
                pb::MissingKind::OutputNoDriver,
                pb::MissingKind::OutputConnectionInvalid,
                pb::MissingKind::OutputNoDevice,
                pb::MissingKind::DeviceNoOutput,
            ]
        );
        assert!(wire_kinds
            .iter()
            .all(|k| *k != pb::MissingKind::Unspecified));
        let x = &m.missing[0];
        assert_eq!(
            (
                x.output_id,
                x.output_name.as_str(),
                x.device_id,
                x.device_name.as_str(),
                x.mapping_id,
                x.mapping_name.as_str()
            ),
            (Some(4), "motor", Some(2), "drive", Some(7), "cruise")
        );
        let r = &m.rows[0];
        assert_eq!(r.device_kind(), pb::DeviceKind::HBridgeChannel);
        assert_eq!(
            (
                r.output_id,
                r.output_name.as_str(),
                r.device_id,
                r.device_name.as_str(),
                r.device_kind_label.as_str()
            ),
            (Some(4), "motor", 2, "drive", "H-bridge channel")
        );
        assert_eq!(
            (
                r.requirement_index,
                r.requirement_label.as_str(),
                r.capability.as_str(),
                r.capability_label.as_str()
            ),
            (1, "direction", "digital_out", "digital out")
        );
        assert_eq!(
            (
                r.fixed.as_deref(),
                r.resource.as_deref(),
                r.resource_label.as_deref()
            ),
            (Some("D4"), Some("D4"), Some("D4: digital in, digital out"))
        );
        let b = m.blocker.as_ref().unwrap();
        assert_eq!(
            (
                b.device_id,
                b.device_name.as_str(),
                b.requirement_index,
                b.requirement_label.as_str(),
                b.capability.as_str(),
                b.capability_label.as_str()
            ),
            (9, "L7", 0, "PWM", "pwm", "PWM")
        );
        let Some(pb::blocker::Kind::Blocked(c)) = &b.kind else {
            panic!()
        };
        let c = &c.candidates[0];
        assert_eq!(
            (
                c.resource.as_str(),
                c.resource_label.as_str(),
                c.held_by_device_id,
                c.held_by_device_name.as_str(),
                c.held_by_requirement_index,
                c.held_by_requirement_label.as_str()
            ),
            ("D3", "D3: …", 3, "L1", 0, "PWM")
        );
        assert_eq!(
            (b.message.as_str(), b.explanation.as_str()),
            ("No free pin", "Every pin…")
        );
        // the other blocker kinds
        report.blocker.as_mut().unwrap().kind = BlockerKind::NoCapableResource;
        assert_eq!(
            report_to_pb(&report).blocker.unwrap().kind,
            Some(pb::blocker::Kind::NoCapableResource(pb::Unit {}))
        );
        report.blocker.as_mut().unwrap().kind = BlockerKind::FixedUnavailable { pin: rid("D9") };
        assert_eq!(
            report_to_pb(&report).blocker.unwrap().kind,
            Some(pb::blocker::Kind::FixedUnavailable("D9".into()))
        );
        // feasible: no blocker, rows without fixed pins, optional fields absent
        report.status = DeploymentStatus::Feasible;
        report.deployable = true;
        report.blocker = None;
        report.missing.clear();
        report.rows[0].fixed = None;
        report.rows[0].output = None;
        report.rows[0].output_name = None;
        let m = report_to_pb(&report);
        assert!(m.deployable && m.blocker.is_none() && m.missing.is_empty());
        assert!(
            m.rows[0].fixed.is_none()
                && m.rows[0].output_id.is_none()
                && m.rows[0].output_name.is_empty()
        );
    }

    #[test]
    fn port_contracts_round_trip_through_pb() {
        use bdl_model::surface::Signature;
        use bdl_system::{ClockContract, PortContract};
        use std::collections::BTreeMap;
        let sem = SemanticId::from_raw;
        let mut c = bdl_system::BehaviorComponent {
            id: bdl_system::ComponentId::from_raw(0),
            name: "Lamp".into(),
            description: String::new(),
            body: Design::empty("Lamp"),
            interface: Default::default(),
            shared_concepts: BTreeMap::new(),
            external_outputs: BTreeMap::new(),
            body_stamp: 0,
            interface_stamp: 0,
        };
        c.body.concepts.insert(
            sem(0),
            bdl_model::surface::Concept {
                id: sem(0),
                name: "Tilt".into(),
                description: String::new(),
                representation: None,
                ordered: false,
            },
        );
        c.body.clocks.insert(
            ClockId::from_raw(2),
            bdl_model::surface::ClockDomain {
                id: ClockId::from_raw(2),
                name: "tick".into(),
            },
        );
        c.shared_concepts.insert(sem(0), sem(9));
        for clock in [
            ClockContract::Agnostic,
            ClockContract::Parameter {
                clock: ClockId::from_raw(2),
            },
            ClockContract::Private {
                clock: ClockId::from_raw(2),
            },
        ] {
            let k = PortContract {
                signature: Signature {
                    inputs: vec![sem(0)],
                    output: sem(0),
                },
                commitments: vec![bdl_ir::PropertyId::Monotone],
                clock,
            };
            let v = system::contract_to_pb(&c, &k);
            assert_eq!(v.input_names, vec!["Tilt"]);
            assert_eq!(v.shared.len(), 2);
            if clock != ClockContract::Agnostic {
                assert_eq!(v.clock_name, "tick");
            }
            assert_eq!(system::contract_from_pb(&v).unwrap(), k);
        }
    }
}
