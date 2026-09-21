//! Revisioned edits.
//!
//! ```text
//! ProjectSnapshot @ N  +  EditOp  ─apply_edit─▶  ProjectSnapshot @ N+1  +  EditOutcome
//! ```
//!
//! [`apply_edit`] is pure: it never mutates its input and never touches the
//! file system.  Every edit is classified as a *refinement* (preserves
//! everything previously established about other declarations) or an *edit*
//! (legal, but reopens the validation of dependents) — the distinction the
//! paper makes precise for the kernel, carried into the tool as an explicit
//! [`Invalidation`] record so that incremental analysis is a model, not UI
//! folklore.

use crate::ids::{ClockId, ConceptId, DeclId, DeviceId, OutputId};
use crate::surface::{
    ClockDomain, Concept, Definition, DeviceBinding, DeviceKind, MappingBlock, PhysicalOutput,
    ProjectSnapshot, Representation, Signature,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A semantic edit operation.  Serializable so the protocol, undo history
/// and future project logs all speak the same vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EditOp {
    CreateConcept {
        name: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        representation: Option<Representation>,
    },
    RenameConcept {
        id: ConceptId,
        name: String,
    },
    SetConceptDescription {
        id: ConceptId,
        description: String,
    },
    /// Bind (write-once) or rebind a concept's representation.  Binding an
    /// unbound concept is a refinement; rebinding is an edit.  Refused
    /// while the concept is declared ordered and the new representation
    /// is not a quantity (`EditError::OrderNeedsQuantity`): undeclare the
    /// order first.
    SetConceptRepresentation {
        id: ConceptId,
        representation: Option<Representation>,
    },
    /// Declare or undeclare the concept ordered (`Concept::ordered`).  An
    /// edit, not a refinement: a formula that compares two values of the
    /// concept types only while it is ordered.  Refused unless the
    /// representation is a quantity (`EditError::OrderNeedsQuantity`).
    SetConceptOrdered {
        id: ConceptId,
        ordered: bool,
    },
    /// Refused while any mapping mentions the concept.
    DeleteConcept {
        id: ConceptId,
    },
    /// Create a mapping: declared, and — when `definition` is given —
    /// defined in the same step, in `clock` if one is named.  A creation
    /// is a refinement whatever it carries: nothing existing depends on
    /// the new declaration yet.
    CreateMapping {
        name: String,
        #[serde(default)]
        description: String,
        signature: Signature,
        #[serde(default)]
        definition: Option<Definition>,
        #[serde(default)]
        clock: Option<ClockId>,
    },
    RenameMapping {
        id: DeclId,
        name: String,
    },
    SetMappingDescription {
        id: DeclId,
        description: String,
    },
    /// Changing the expected type of a declaration: always an edit.
    SetMappingSignature {
        id: DeclId,
        signature: Signature,
    },
    /// Attach a definition to an *unresolved* mapping (refinement).  Use
    /// [`EditOp::ReplaceDefinition`] to change an existing one.
    AttachDefinition {
        id: DeclId,
        definition: Definition,
    },
    /// Replace or detach an existing definition (edit).
    ReplaceDefinition {
        id: DeclId,
        definition: Option<Definition>,
    },
    DeleteMapping {
        id: DeclId,
    },
    CreateClockDomain {
        name: String,
    },
    RenameClockDomain {
        id: ClockId,
        name: String,
    },
    /// Refused while any mapping is assigned to the domain.
    DeleteClockDomain {
        id: ClockId,
    },
    /// Assign (or clear) a mapping's domain.  Always an edit: every client's
    /// domain judgment depends on it.
    SetMappingClock {
        id: DeclId,
        clock: Option<ClockId>,
    },
    CreateOutput {
        name: String,
        #[serde(default)]
        description: String,
        accepts: ConceptId,
        #[serde(default)]
        clock: Option<ClockId>,
    },
    RenameOutput {
        id: OutputId,
        name: String,
    },
    /// Changing what a sink accepts or when it updates invalidates its driver.
    SetOutputAccepts {
        id: OutputId,
        accepts: ConceptId,
    },
    SetOutputClock {
        id: OutputId,
        clock: Option<ClockId>,
    },
    SetOutputRequired {
        id: OutputId,
        required: bool,
    },
    /// Refused while a mapping drives the sink or a device realises it.
    DeleteOutput {
        id: OutputId,
    },
    /// Bind a mapping as the final driver of a sink (`Some`, a refinement
    /// when the mapping drove nothing) or detach it (`None`, an edit).
    SetMappingDrive {
        id: DeclId,
        output: Option<OutputId>,
    },
    CreateDevice {
        name: String,
        kind: DeviceKind,
        #[serde(default)]
        output: Option<OutputId>,
    },
    RenameDevice {
        id: DeviceId,
        name: String,
    },
    SetDeviceKind {
        id: DeviceId,
        kind: DeviceKind,
    },
    SetDeviceOutput {
        id: DeviceId,
        output: Option<OutputId>,
    },
    /// Choose the realization profile of a device (or release it), and with
    /// it the hardware requirement template the profile prescribes: one
    /// deployment decision, applied whole.  The design's behaviour is
    /// untouched (FV Phase 14: the logical output keeps its meaning).
    SetDeviceRealization {
        id: DeviceId,
        profile: Option<crate::surface::OutputProfileId>,
        kind: DeviceKind,
    },
    /// Bind the device to a Source it provides (or release it), by the
    /// declaration's stable id.  A deployment assignment (FV Phase 16,
    /// `assignSource`): the Source keeps its declaration and its meaning;
    /// only what fills it at run time is named.  Releases the device's
    /// output, since a device consumes or provides, never both.
    SetDeviceSource {
        id: DeviceId,
        source: Option<DeclId>,
    },
    /// Choose the provider profile of a device (or release it), and with
    /// it the hardware requirement template the profile prescribes: one
    /// deployment decision, applied whole.
    SetDeviceProvider {
        id: DeviceId,
        profile: Option<crate::surface::InputProfileId>,
        kind: DeviceKind,
    },
    /// Pin one of the device's requirements to a named board resource, or
    /// release it.  A deployment constraint, never a design change.
    SetDevicePin {
        id: DeviceId,
        index: u16,
        resource: Option<String>,
    },
    DeleteDevice {
        id: DeviceId,
    },
}

/// Whether an edit preserves what dependents previously established.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EditKind {
    /// Everything established about other declarations remains valid.
    Refinement,
    /// Dependents' validation must be reopened (see [`Invalidation`]).
    Edit,
}

/// Which kind of established fact an edit can invalidate.  Analyses subscribe
/// to categories; an edit reports the categories it touches and the
/// declarations at which the change originates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Invalidation {
    /// Expected type or commitments of a declaration changed.
    Interface,
    /// A realization was attached, replaced or detached.
    Realization,
    /// Concept identity/representation changed.
    Semantic,
    /// Temporal structure changed (delays, initial values).
    Reactive,
    /// Clock-domain assignment changed.
    Clock,
    /// Output bindings changed.
    Output,
    /// Board or manual resource constraints changed.
    Deployment,
}

/// What an edit did, beyond the new snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EditOutcome {
    pub kind: Option<EditKind>,
    pub invalidates: BTreeSet<Invalidation>,
    /// Declarations at which invalidation originates (dependents are
    /// computed by the analyses, not guessed here).
    pub origin_decls: BTreeSet<DeclId>,
    pub created_concept: Option<ConceptId>,
    pub created_mapping: Option<DeclId>,
    pub created_clock: Option<ClockId>,
    pub created_output: Option<OutputId>,
    pub created_device: Option<DeviceId>,
}

impl EditOutcome {
    fn refinement() -> Self {
        EditOutcome {
            kind: Some(EditKind::Refinement),
            ..Default::default()
        }
    }
    fn edit(invalidates: impl IntoIterator<Item = Invalidation>) -> Self {
        EditOutcome {
            kind: Some(EditKind::Edit),
            invalidates: invalidates.into_iter().collect(),
            ..Default::default()
        }
    }
    fn at(mut self, decl: DeclId) -> Self {
        self.origin_decls.insert(decl);
        self
    }
    fn touching(mut self, i: Invalidation) -> Self {
        self.invalidates.insert(i);
        self
    }
}

/// The result of a successful edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Applied {
    pub snapshot: ProjectSnapshot,
    pub outcome: EditOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum EditError {
    #[error("a name is required")]
    EmptyName,
    #[error("a concept named `{name}` already exists")]
    DuplicateConceptName { name: String },
    #[error("a mapping named `{name}` already exists")]
    DuplicateMappingName { name: String },
    #[error("unknown concept {id}")]
    UnknownConcept { id: ConceptId },
    #[error("unknown mapping {id}")]
    UnknownMapping { id: DeclId },
    #[error("concept {id} can only be ordered while its value form is a quantity; clear the order before choosing another value form")]
    OrderNeedsQuantity { id: ConceptId },
    #[error("concept {id} is still used by {} mapping(s)", used_by.len())]
    ConceptInUse { id: ConceptId, used_by: Vec<DeclId> },
    #[error("mapping {id} already has a definition; replace it explicitly")]
    AlreadyDefined { id: DeclId },
    #[error("mapping {id} has no definition to replace")]
    NotDefined { id: DeclId },
    #[error("a timing domain named `{name}` already exists")]
    DuplicateClockName { name: String },
    #[error("unknown timing domain {id}")]
    UnknownClock { id: ClockId },
    #[error("timing domain {id} is still used by {} mapping(s)", used_by.len())]
    ClockInUse { id: ClockId, used_by: Vec<DeclId> },
    #[error("an output named `{name}` already exists")]
    DuplicateOutputName { name: String },
    #[error("unknown output {id}")]
    UnknownOutput { id: OutputId },
    #[error("output {id} is still driven by {} mapping(s) or realised by {} device(s)", drivers.len(), devices.len())]
    OutputInUse {
        id: OutputId,
        drivers: Vec<DeclId>,
        devices: Vec<DeviceId>,
    },
    #[error("a device named `{name}` already exists")]
    DuplicateDeviceName { name: String },
    #[error("unknown device {id}")]
    UnknownDevice { id: DeviceId },
    #[error("relationship {id} is not a Source: only a Source is provided by a device")]
    NotASource { id: DeclId },
}

/// Apply one edit to a snapshot, producing the next revision.
///
/// Errors leave the caller with the unchanged input; nothing is partially
/// applied.
pub fn apply_edit(snapshot: &ProjectSnapshot, op: &EditOp) -> Result<Applied, EditError> {
    let mut design = snapshot.design.clone();
    let outcome = match op {
        EditOp::CreateConcept {
            name,
            description,
            representation,
        } => {
            let name = valid_name(name)?;
            if design.concepts.values().any(|c| c.name == name) {
                return Err(EditError::DuplicateConceptName { name });
            }
            let (id, ids) = design.ids.fresh_concept();
            design.ids = ids;
            design.concepts.insert(
                id,
                Concept {
                    id,
                    name,
                    description: description.clone(),
                    representation: representation.clone(),
                    ordered: false,
                },
            );
            EditOutcome {
                created_concept: Some(id),
                ..EditOutcome::refinement()
            }
        }
        EditOp::RenameConcept { id, name } => {
            let name = valid_name(name)?;
            if design
                .concepts
                .values()
                .any(|c| c.id != *id && c.name == name)
            {
                return Err(EditError::DuplicateConceptName { name });
            }
            concept_mut(&mut design, *id)?.name = name;
            EditOutcome::refinement()
        }
        EditOp::SetConceptDescription { id, description } => {
            concept_mut(&mut design, *id)?.description = description.clone();
            EditOutcome::refinement()
        }
        EditOp::SetConceptRepresentation { id, representation } => {
            let concept = concept_mut(&mut design, *id)?;
            if concept.ordered
                && !representation
                    .as_ref()
                    .is_some_and(Representation::supports_order)
            {
                return Err(EditError::OrderNeedsQuantity { id: *id });
            }
            let was_bound = concept.representation.is_some();
            concept.representation = representation.clone();
            if was_bound {
                // Rebinding breaks every realization typed against the old
                // representation.
                let users: Vec<DeclId> = design.mappings_using(*id).map(|m| m.id).collect();
                let mut o = EditOutcome::edit([Invalidation::Semantic, Invalidation::Realization]);
                o.origin_decls.extend(users);
                o
            } else {
                EditOutcome::refinement()
            }
        }
        EditOp::SetConceptOrdered { id, ordered } => {
            let concept = concept_mut(&mut design, *id)?;
            if *ordered
                && !concept
                    .representation
                    .as_ref()
                    .is_some_and(Representation::supports_order)
            {
                return Err(EditError::OrderNeedsQuantity { id: *id });
            }
            if concept.ordered == *ordered {
                EditOutcome::refinement()
            } else {
                concept.ordered = *ordered;
                let users: Vec<DeclId> = design.mappings_using(*id).map(|m| m.id).collect();
                let mut o = EditOutcome::edit([Invalidation::Semantic, Invalidation::Realization]);
                o.origin_decls.extend(users);
                o
            }
        }
        EditOp::DeleteConcept { id } => {
            concept_mut(&mut design, *id)?;
            let used_by: Vec<DeclId> = design.mappings_using(*id).map(|m| m.id).collect();
            if !used_by.is_empty() {
                return Err(EditError::ConceptInUse { id: *id, used_by });
            }
            design.concepts.remove(id);
            EditOutcome::edit([Invalidation::Semantic])
        }
        EditOp::CreateMapping {
            name,
            description,
            signature,
            definition,
            clock,
        } => {
            let name = valid_name(name)?;
            if design.mappings.values().any(|m| m.name == name) {
                return Err(EditError::DuplicateMappingName { name });
            }
            check_signature(&design, signature)?;
            if let Some(c) = clock {
                if !design.clocks.contains_key(c) {
                    return Err(EditError::UnknownClock { id: *c });
                }
            }
            let parameters = derived_parameters(&design, signature, &[]);
            let (id, ids) = design.ids.fresh_decl();
            design.ids = ids;
            design.mappings.insert(
                id,
                MappingBlock {
                    id,
                    name,
                    description: description.clone(),
                    signature: signature.clone(),
                    definition: definition.clone(),
                    clock: *clock,
                    drives: None,
                    parameters,
                },
            );
            let mut o = EditOutcome {
                created_mapping: Some(id),
                ..EditOutcome::refinement()
            };
            if definition.is_some() {
                o = o
                    .touching(Invalidation::Realization)
                    .touching(Invalidation::Reactive);
            }
            if clock.is_some() {
                o = o.touching(Invalidation::Clock);
            }
            o
        }
        EditOp::RenameMapping { id, name } => {
            let name = valid_name(name)?;
            if design
                .mappings
                .values()
                .any(|m| m.id != *id && m.name == name)
            {
                return Err(EditError::DuplicateMappingName { name });
            }
            mapping_mut(&mut design, *id)?.name = name;
            EditOutcome::refinement()
        }
        EditOp::SetMappingDescription { id, description } => {
            mapping_mut(&mut design, *id)?.description = description.clone();
            EditOutcome::refinement()
        }
        EditOp::SetMappingSignature { id, signature } => {
            check_signature(&design, signature)?;
            let current = mapping_mut(&mut design, *id)?.parameters.clone();
            let parameters = derived_parameters(&design, signature, &current);
            let m = mapping_mut(&mut design, *id)?;
            m.signature = signature.clone();
            m.parameters = parameters;
            let mut o =
                EditOutcome::edit([Invalidation::Interface, Invalidation::Semantic]).at(*id);
            if m.definition.is_some() {
                o = o.touching(Invalidation::Realization);
            }
            o
        }
        EditOp::AttachDefinition { id, definition } => {
            let m = mapping_mut(&mut design, *id)?;
            if m.definition.is_some() {
                return Err(EditError::AlreadyDefined { id: *id });
            }
            m.definition = Some(definition.clone());
            // A refinement for dependents; the declaration's own realization
            // and the simulation now need (re)checking.
            EditOutcome::refinement()
                .touching(Invalidation::Realization)
                .touching(Invalidation::Reactive)
                .at(*id)
        }
        EditOp::ReplaceDefinition { id, definition } => {
            let m = mapping_mut(&mut design, *id)?;
            if m.definition.is_none() {
                return Err(EditError::NotDefined { id: *id });
            }
            m.definition = definition.clone();
            EditOutcome::edit([Invalidation::Realization, Invalidation::Reactive]).at(*id)
        }
        EditOp::DeleteMapping { id } => {
            mapping_mut(&mut design, *id)?;
            design.mappings.remove(id);
            // a device that provided the deleted Source now provides nothing
            for d in design.devices.values_mut() {
                if d.source == Some(*id) {
                    d.source = None;
                }
            }
            EditOutcome::edit([
                Invalidation::Interface,
                Invalidation::Realization,
                Invalidation::Reactive,
            ])
            .at(*id)
        }
        EditOp::CreateClockDomain { name } => {
            let name = valid_name(name)?;
            if design.clocks.values().any(|c| c.name == name) {
                return Err(EditError::DuplicateClockName { name });
            }
            let (id, ids) = design.ids.fresh_clock();
            design.ids = ids;
            design.clocks.insert(id, ClockDomain { id, name });
            EditOutcome {
                created_clock: Some(id),
                ..EditOutcome::refinement()
            }
        }
        EditOp::RenameClockDomain { id, name } => {
            let name = valid_name(name)?;
            if design
                .clocks
                .values()
                .any(|c| c.id != *id && c.name == name)
            {
                return Err(EditError::DuplicateClockName { name });
            }
            design
                .clocks
                .get_mut(id)
                .ok_or(EditError::UnknownClock { id: *id })?
                .name = name;
            EditOutcome::refinement()
        }
        EditOp::DeleteClockDomain { id } => {
            design
                .clocks
                .get(id)
                .ok_or(EditError::UnknownClock { id: *id })?;
            let used_by: Vec<DeclId> = design
                .mappings
                .values()
                .filter(|m| m.clock == Some(*id))
                .map(|m| m.id)
                .collect();
            if !used_by.is_empty() {
                return Err(EditError::ClockInUse { id: *id, used_by });
            }
            design.clocks.remove(id);
            EditOutcome::edit([Invalidation::Clock])
        }
        EditOp::SetMappingClock { id, clock } => {
            if let Some(c) = clock {
                design
                    .clocks
                    .get(c)
                    .ok_or(EditError::UnknownClock { id: *c })?;
            }
            mapping_mut(&mut design, *id)?.clock = *clock;
            EditOutcome::edit([Invalidation::Clock, Invalidation::Reactive]).at(*id)
        }
        EditOp::CreateOutput {
            name,
            description,
            accepts,
            clock,
        } => {
            let name = valid_name(name)?;
            if design.outputs.values().any(|o| o.name == name) {
                return Err(EditError::DuplicateOutputName { name });
            }
            design
                .concepts
                .get(accepts)
                .ok_or(EditError::UnknownConcept { id: *accepts })?;
            if let Some(c) = clock {
                design
                    .clocks
                    .get(c)
                    .ok_or(EditError::UnknownClock { id: *c })?;
            }
            let (id, ids) = design.ids.fresh_output();
            design.ids = ids;
            design.outputs.insert(
                id,
                PhysicalOutput {
                    id,
                    name,
                    description: description.clone(),
                    accepts: *accepts,
                    clock: *clock,
                    required: true,
                },
            );
            EditOutcome {
                created_output: Some(id),
                ..EditOutcome::refinement()
            }
        }
        EditOp::RenameOutput { id, name } => {
            let name = valid_name(name)?;
            if design
                .outputs
                .values()
                .any(|o| o.id != *id && o.name == name)
            {
                return Err(EditError::DuplicateOutputName { name });
            }
            output_mut(&mut design, *id)?.name = name;
            EditOutcome::refinement()
        }
        EditOp::SetOutputAccepts { id, accepts } => {
            design
                .concepts
                .get(accepts)
                .ok_or(EditError::UnknownConcept { id: *accepts })?;
            output_mut(&mut design, *id)?.accepts = *accepts;
            let drivers: Vec<DeclId> = design.drivers_of(*id).map(|m| m.id).collect();
            let mut o = EditOutcome::edit([Invalidation::Output]);
            o.origin_decls.extend(drivers);
            o
        }
        EditOp::SetOutputClock { id, clock } => {
            if let Some(c) = clock {
                design
                    .clocks
                    .get(c)
                    .ok_or(EditError::UnknownClock { id: *c })?;
            }
            let out = output_mut(&mut design, *id)?;
            let was_set = out.clock.is_some();
            out.clock = *clock;
            let drivers: Vec<DeclId> = design.drivers_of(*id).map(|m| m.id).collect();
            if was_set {
                let mut o = EditOutcome::edit([Invalidation::Output, Invalidation::Clock]);
                o.origin_decls.extend(drivers);
                o
            } else {
                EditOutcome::refinement().touching(Invalidation::Output)
            }
        }
        EditOp::SetOutputRequired { id, required } => {
            output_mut(&mut design, *id)?.required = *required;
            EditOutcome::refinement().touching(Invalidation::Output)
        }
        EditOp::DeleteOutput { id } => {
            output_mut(&mut design, *id)?;
            let drivers: Vec<DeclId> = design.drivers_of(*id).map(|m| m.id).collect();
            let devices: Vec<DeviceId> = design
                .devices
                .values()
                .filter(|d| d.output == Some(*id))
                .map(|d| d.id)
                .collect();
            if !drivers.is_empty() || !devices.is_empty() {
                return Err(EditError::OutputInUse {
                    id: *id,
                    drivers,
                    devices,
                });
            }
            design.outputs.remove(id);
            EditOutcome::edit([Invalidation::Output, Invalidation::Deployment])
        }
        EditOp::SetMappingDrive { id, output } => {
            if let Some(o) = output {
                design
                    .outputs
                    .get(o)
                    .ok_or(EditError::UnknownOutput { id: *o })?;
            }
            let m = mapping_mut(&mut design, *id)?;
            let had = m.drives.is_some();
            m.drives = *output;
            if had {
                EditOutcome::edit([Invalidation::Output]).at(*id)
            } else {
                EditOutcome::refinement()
                    .touching(Invalidation::Output)
                    .at(*id)
            }
        }
        EditOp::CreateDevice { name, kind, output } => {
            let name = valid_name(name)?;
            if design.devices.values().any(|d| d.name == name) {
                return Err(EditError::DuplicateDeviceName { name });
            }
            if let Some(o) = output {
                design
                    .outputs
                    .get(o)
                    .ok_or(EditError::UnknownOutput { id: *o })?;
            }
            let (id, ids) = design.ids.fresh_device();
            design.ids = ids;
            design.devices.insert(
                id,
                DeviceBinding {
                    id,
                    name,
                    kind: *kind,
                    output: *output,
                    realization: None,
                    source: None,
                    provider: None,
                    fixed_pins: BTreeMap::new(),
                },
            );
            EditOutcome {
                created_device: Some(id),
                ..EditOutcome::refinement().touching(Invalidation::Deployment)
            }
        }
        EditOp::RenameDevice { id, name } => {
            let name = valid_name(name)?;
            if design
                .devices
                .values()
                .any(|d| d.id != *id && d.name == name)
            {
                return Err(EditError::DuplicateDeviceName { name });
            }
            device_mut(&mut design, *id)?.name = name;
            EditOutcome::refinement()
        }
        EditOp::SetDeviceKind { id, kind } => {
            let d = device_mut(&mut design, *id)?;
            d.kind = *kind;
            d.fixed_pins.clear();
            // a kind chosen by hand is no longer the profile's: the
            // realization or provider is released with it
            d.realization = None;
            d.provider = None;
            EditOutcome::edit([Invalidation::Deployment])
        }
        EditOp::SetDeviceRealization { id, profile, kind } => {
            let d = device_mut(&mut design, *id)?;
            if d.kind != *kind {
                d.fixed_pins.clear();
            }
            d.kind = *kind;
            d.realization = profile.clone();
            EditOutcome::edit([Invalidation::Deployment])
        }
        EditOp::SetDeviceOutput { id, output } => {
            if let Some(o) = output {
                design
                    .outputs
                    .get(o)
                    .ok_or(EditError::UnknownOutput { id: *o })?;
            }
            let d = device_mut(&mut design, *id)?;
            d.output = *output;
            if output.is_some() {
                d.source = None;
                d.provider = None;
            }
            EditOutcome::edit([Invalidation::Deployment])
        }
        EditOp::SetDeviceSource { id, source } => {
            if let Some(s) = source {
                let m = design
                    .mappings
                    .get(s)
                    .ok_or(EditError::UnknownMapping { id: *s })?;
                if m.role() != crate::surface::RelationshipRole::Source {
                    return Err(EditError::NotASource { id: *s });
                }
            }
            let d = device_mut(&mut design, *id)?;
            d.source = *source;
            if source.is_some() {
                d.output = None;
                d.realization = None;
            }
            EditOutcome::edit([Invalidation::Deployment])
        }
        EditOp::SetDeviceProvider { id, profile, kind } => {
            let d = device_mut(&mut design, *id)?;
            if d.kind != *kind {
                d.fixed_pins.clear();
            }
            d.kind = *kind;
            d.provider = profile.clone();
            EditOutcome::edit([Invalidation::Deployment])
        }
        EditOp::SetDevicePin {
            id,
            index,
            resource,
        } => {
            let d = device_mut(&mut design, *id)?;
            match resource {
                Some(r) => {
                    d.fixed_pins.insert(*index, r.clone());
                }
                None => {
                    d.fixed_pins.remove(index);
                }
            }
            EditOutcome::edit([Invalidation::Deployment])
        }
        EditOp::DeleteDevice { id } => {
            device_mut(&mut design, *id)?;
            design.devices.remove(id);
            EditOutcome::edit([Invalidation::Deployment])
        }
    };
    Ok(Applied {
        snapshot: ProjectSnapshot {
            revision: snapshot.revision.next(),
            design,
        },
        outcome,
    })
}

fn valid_name(name: &str) -> Result<String, EditError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        Err(EditError::EmptyName)
    } else {
        Ok(trimmed.to_owned())
    }
}

fn concept_mut(
    design: &mut crate::surface::Design,
    id: ConceptId,
) -> Result<&mut Concept, EditError> {
    design
        .concepts
        .get_mut(&id)
        .ok_or(EditError::UnknownConcept { id })
}

fn mapping_mut(
    design: &mut crate::surface::Design,
    id: DeclId,
) -> Result<&mut MappingBlock, EditError> {
    design
        .mappings
        .get_mut(&id)
        .ok_or(EditError::UnknownMapping { id })
}

fn output_mut(
    design: &mut crate::surface::Design,
    id: OutputId,
) -> Result<&mut PhysicalOutput, EditError> {
    design
        .outputs
        .get_mut(&id)
        .ok_or(EditError::UnknownOutput { id })
}

fn device_mut(
    design: &mut crate::surface::Design,
    id: DeviceId,
) -> Result<&mut DeviceBinding, EditError> {
    design
        .devices
        .get_mut(&id)
        .ok_or(EditError::UnknownDevice { id })
}

/// The parameter names a rule's inputs get from a structured edit
/// (ADR-0044 amending ADR-0013).  A rule is a template over its inputs;
/// an input is named after its concept unless the same concept is read
/// twice, when the concept's name would be ambiguous
/// (`bdl_elab::names::Lookup::Ambiguous`): those inputs are then named
/// `tilt1`, `tilt2`, … — the concept's name with a lower-case initial
/// and its ordinal among the repeats — and every other input keeps the
/// empty entry that falls back to its concept's name.  Names the text
/// gave (`f(t, held) = …`) are kept as long as they still cover the
/// signature; a unit domain has none.
fn derived_parameters(
    design: &crate::surface::Design,
    sig: &Signature,
    current: &[String],
) -> Vec<String> {
    if sig.inputs.is_empty() {
        return Vec::new();
    }
    let authored = current.iter().any(|p| !p.is_empty());
    if authored && current.len() == sig.inputs.len() {
        return current.to_vec();
    }
    let repeated: BTreeSet<ConceptId> = sig
        .inputs
        .iter()
        .filter(|c| sig.inputs.iter().filter(|d| d == c).count() > 1)
        .copied()
        .collect();
    if repeated.is_empty() {
        return Vec::new();
    }
    let mut seen: BTreeMap<ConceptId, usize> = BTreeMap::new();
    sig.inputs
        .iter()
        .map(|c| {
            if !repeated.contains(c) {
                return String::new();
            }
            let n = seen.entry(*c).or_insert(0);
            *n += 1;
            let name = design
                .concepts
                .get(c)
                .map(|k| k.name.as_str())
                .unwrap_or("input");
            let mut chars = name.chars();
            let head: String = chars
                .next()
                .map(|h| h.to_lowercase().collect())
                .unwrap_or_default();
            format!("{head}{}{n}", chars.as_str())
        })
        .collect()
}

fn check_signature(design: &crate::surface::Design, sig: &Signature) -> Result<(), EditError> {
    for c in sig.inputs.iter().chain(std::iter::once(&sig.output)) {
        if !design.concepts.contains_key(c) {
            return Err(EditError::UnknownConcept { id: *c });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Revision;
    use crate::surface::Design;

    fn empty() -> ProjectSnapshot {
        ProjectSnapshot::new(Design::empty("lamp"))
    }

    fn create_concept(s: &ProjectSnapshot, name: &str) -> (ProjectSnapshot, ConceptId) {
        let a = apply_edit(
            s,
            &EditOp::CreateConcept {
                name: name.into(),
                description: String::new(),
                representation: None,
            },
        )
        .unwrap();
        let id = a.outcome.created_concept.unwrap();
        (a.snapshot, id)
    }

    /// The concept ladder (ADR-0044): a Sem block is a unit-domain
    /// declaration and never has inputs or parameters; a rule is a
    /// template whose inputs are named after their concepts unless one
    /// concept is read twice, when the repeats get stable names so the
    /// formula can tell them apart (`Lookup::Ambiguous` otherwise).
    #[test]
    fn a_sem_block_has_no_inputs_and_same_concept_inputs_get_parameter_names() {
        let (s, tilt) = create_concept(&empty(), "Tilt");
        let (s, held) = create_concept(&s, "Held");
        let create = |s: &ProjectSnapshot, name: &str, inputs: Vec<ConceptId>| {
            let a = apply_edit(
                s,
                &EditOp::CreateMapping {
                    name: name.into(),
                    description: String::new(),
                    signature: Signature {
                        inputs,
                        output: tilt,
                    },
                    definition: None,
                    clock: None,
                },
            )
            .unwrap();
            let id = a.outcome.created_mapping.unwrap();
            (a.snapshot, id)
        };
        let (s, sem) = create(&s, "tilt", vec![]);
        assert!(s.design.mappings[&sem].signature.is_unit_domain());
        assert!(s.design.mappings[&sem].parameters.is_empty());
        // one input per concept: named after the concept, nothing stored
        let (s, rule) = create(&s, "dim", vec![tilt, held]);
        assert!(s.design.mappings[&rule].parameters.is_empty());
        // the same concept twice: the repeats are told apart
        let (s, diff) = create(&s, "diff", vec![tilt, held, tilt]);
        assert_eq!(
            s.design.mappings[&diff].parameters,
            vec!["tilt1".to_owned(), String::new(), "tilt2".to_owned()]
        );
        // the same through a signature change; names the text gave are
        // kept while they still cover the inputs
        let s = apply_edit(
            &s,
            &EditOp::SetMappingSignature {
                id: rule,
                signature: Signature {
                    inputs: vec![tilt, tilt],
                    output: tilt,
                },
            },
        )
        .unwrap()
        .snapshot;
        assert_eq!(
            s.design.mappings[&rule].parameters,
            vec!["tilt1".to_owned(), "tilt2".to_owned()]
        );
        let mut authored = s.clone();
        authored.design.mappings.get_mut(&rule).unwrap().parameters = vec!["a".into(), "b".into()];
        let kept = apply_edit(
            &authored,
            &EditOp::SetMappingSignature {
                id: rule,
                signature: Signature {
                    inputs: vec![tilt, held],
                    output: tilt,
                },
            },
        )
        .unwrap()
        .snapshot;
        assert_eq!(
            kept.design.mappings[&rule].parameters,
            vec!["a".to_owned(), "b".to_owned()]
        );
    }

    #[test]
    fn create_concept_bumps_revision_and_allocates_id() {
        let s0 = empty();
        let (s1, tilt) = create_concept(&s0, "Tilt");
        assert_eq!(s1.revision, Revision::from_raw(1));
        assert_eq!(s1.design.concepts[&tilt].name, "Tilt");
        // input untouched
        assert!(s0.design.concepts.is_empty());
    }

    /// The order invariant (`Concept::order_is_valid`): no edit leaves a
    /// concept ordered over a value form that has no order.
    #[test]
    fn an_order_declaration_needs_a_quantity_and_keeps_it() {
        let (s, mode) = create_concept(&empty(), "Mode");
        let set = |s: &ProjectSnapshot, r: Option<Representation>| {
            apply_edit(
                s,
                &EditOp::SetConceptRepresentation {
                    id: mode,
                    representation: r,
                },
            )
        };
        let order = |s: &ProjectSnapshot, ordered: bool| {
            apply_edit(s, &EditOp::SetConceptOrdered { id: mode, ordered })
        };
        // unbound: cannot be ordered
        assert_eq!(
            order(&s, true).unwrap_err(),
            EditError::OrderNeedsQuantity { id: mode }
        );
        // a count, a truth value, a collection: no order to declare
        for r in [
            Representation::Count,
            Representation::Boolean,
            Representation::list(Representation::quantity(crate::Dim::ZERO)),
        ] {
            let s = set(&s, Some(r)).unwrap().snapshot;
            assert_eq!(
                order(&s, true).unwrap_err(),
                EditError::OrderNeedsQuantity { id: mode }
            );
            // undeclaring is always fine
            assert!(order(&s, false).is_ok());
        }
        // a quantity: ordered, and the representation may then only
        // change to another quantity
        let s = set(&s, Some(Representation::quantity(crate::Dim::ZERO)))
            .unwrap()
            .snapshot;
        let s = order(&s, true).unwrap().snapshot;
        assert!(s.design.concepts[&mode].ordered);
        assert!(s.design.concepts[&mode].order_is_valid());
        assert_eq!(
            set(&s, Some(Representation::Count)).unwrap_err(),
            EditError::OrderNeedsQuantity { id: mode }
        );
        assert_eq!(
            set(&s, None).unwrap_err(),
            EditError::OrderNeedsQuantity { id: mode }
        );
        let s = set(&s, Some(Representation::quantity(crate::Dim::TIME)))
            .unwrap()
            .snapshot;
        assert!(s.design.concepts[&mode].ordered);
        // undeclare, then any form again
        let s = order(&s, false).unwrap().snapshot;
        assert!(set(&s, Some(Representation::Count)).is_ok());
    }

    #[test]
    fn duplicate_and_empty_names_are_refused() {
        let (s1, _) = create_concept(&empty(), "Tilt");
        let err = apply_edit(
            &s1,
            &EditOp::CreateConcept {
                name: " Tilt ".into(),
                description: String::new(),
                representation: None,
            },
        )
        .unwrap_err();
        assert_eq!(
            err,
            EditError::DuplicateConceptName {
                name: "Tilt".into()
            }
        );
        let err = apply_edit(
            &s1,
            &EditOp::CreateConcept {
                name: "   ".into(),
                description: String::new(),
                representation: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, EditError::EmptyName);
    }

    #[test]
    fn unresolved_mapping_is_a_legal_state() {
        let (s, tilt) = create_concept(&empty(), "Tilt");
        let (s, bright) = create_concept(&s, "Brightness");
        let a = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: bright,
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        let id = a.outcome.created_mapping.unwrap();
        let m = &a.snapshot.design.mappings[&id];
        assert!(m.is_unresolved());
        assert_eq!(a.outcome.kind, Some(EditKind::Refinement));
        assert!(a.outcome.invalidates.is_empty());
    }

    #[test]
    fn signature_must_reference_existing_concepts() {
        let (s, tilt) = create_concept(&empty(), "Tilt");
        let ghost = ConceptId::from_raw(99);
        let err = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "m".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: ghost,
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap_err();
        assert_eq!(err, EditError::UnknownConcept { id: ghost });
    }

    #[test]
    fn attach_is_refinement_replace_is_edit() {
        let (s, tilt) = create_concept(&empty(), "Tilt");
        let (s, bright) = create_concept(&s, "Brightness");
        let a = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: bright,
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        let id = a.outcome.created_mapping.unwrap();
        let def = Definition::Formula {
            source: "clamp(0.2 + 0.8 * Tilt / 60deg, 0, 1)".into(),
        };
        let attached = apply_edit(
            &a.snapshot,
            &EditOp::AttachDefinition {
                id,
                definition: def.clone(),
            },
        )
        .unwrap();
        assert_eq!(attached.outcome.kind, Some(EditKind::Refinement));
        assert!(attached
            .outcome
            .invalidates
            .contains(&Invalidation::Realization));
        assert_eq!(
            attached
                .outcome
                .origin_decls
                .iter()
                .copied()
                .collect::<Vec<_>>(),
            vec![id]
        );

        // attaching twice is refused; the designer must replace explicitly
        let err = apply_edit(
            &attached.snapshot,
            &EditOp::AttachDefinition {
                id,
                definition: def,
            },
        )
        .unwrap_err();
        assert_eq!(err, EditError::AlreadyDefined { id });

        let replaced = apply_edit(
            &attached.snapshot,
            &EditOp::ReplaceDefinition {
                id,
                definition: None,
            },
        )
        .unwrap();
        assert_eq!(replaced.outcome.kind, Some(EditKind::Edit));
        assert!(replaced.snapshot.design.mappings[&id].is_unresolved());
    }

    #[test]
    fn delete_concept_refused_while_in_use() {
        let (s, tilt) = create_concept(&empty(), "Tilt");
        let (s, bright) = create_concept(&s, "Brightness");
        let a = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: bright,
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        let id = a.outcome.created_mapping.unwrap();
        let err = apply_edit(&a.snapshot, &EditOp::DeleteConcept { id: tilt }).unwrap_err();
        assert_eq!(
            err,
            EditError::ConceptInUse {
                id: tilt,
                used_by: vec![id]
            }
        );
    }

    #[test]
    fn rename_keeps_identity() {
        let (s, tilt) = create_concept(&empty(), "Tilt");
        let r = apply_edit(
            &s,
            &EditOp::RenameConcept {
                id: tilt,
                name: "Orientation".into(),
            },
        )
        .unwrap();
        assert_eq!(r.snapshot.design.concepts[&tilt].name, "Orientation");
        assert_eq!(r.outcome.kind, Some(EditKind::Refinement));
    }

    #[test]
    fn edit_ops_round_trip_through_json() {
        let op = EditOp::CreateMapping {
            name: "m".into(),
            description: String::new(),
            signature: Signature {
                inputs: vec![ConceptId::from_raw(0)],
                output: ConceptId::from_raw(1),
            },
            definition: None,
            clock: None,
        };
        let json = serde_json::to_string(&op).unwrap();
        let back: EditOp = serde_json::from_str(&json).unwrap();
        assert_eq!(op, back);
    }

    /// `CreateMapping` with a definition and a domain is one authored step
    /// (protocol 0.19): the structure is checked — the name, the signature,
    /// a domain that exists — and refused whole when wrong; the definition
    /// text is authoring state the project keeps as typed, checked by the
    /// compiler, never here (a text project saves what does not build).
    #[test]
    fn create_mapping_with_definition_and_clock_checks_structure_and_keeps_the_text() {
        let (s, tilt) = create_concept(&empty(), "Tilt");
        let clock = apply_edit(
            &s,
            &EditOp::CreateClockDomain {
                name: "main".into(),
            },
        )
        .unwrap();
        let main = clock.outcome.created_clock.unwrap();
        let s = clock.snapshot;
        let create = |definition: Option<Definition>, clock: Option<ClockId>| {
            apply_edit(
                &s,
                &EditOp::CreateMapping {
                    name: "reading".into(),
                    description: String::new(),
                    signature: Signature {
                        inputs: vec![],
                        output: tilt,
                    },
                    definition,
                    clock,
                },
            )
        };
        // a domain that does not exist: refused, nothing created
        let err = create(None, Some(ClockId::from_raw(99))).unwrap_err();
        assert_eq!(
            err,
            EditError::UnknownClock {
                id: ClockId::from_raw(99)
            }
        );
        // a definition the language cannot read: kept as typed — a Value
        // in the state invalid, the compiler's to say — with its domain
        let a = create(
            Some(Definition::Formula {
                source: "this is not a formula ((".into(),
            }),
            Some(main),
        )
        .unwrap();
        let id = a.outcome.created_mapping.unwrap();
        let m = &a.snapshot.design.mappings[&id];
        assert_eq!(
            m.definition,
            Some(Definition::Formula {
                source: "this is not a formula ((".into()
            })
        );
        assert_eq!(m.clock, Some(main));
        assert_eq!(m.role(), crate::RelationshipRole::Value);
        assert_eq!(a.outcome.kind, Some(EditKind::Refinement));
        assert_eq!(
            a.snapshot.revision,
            s.revision.next(),
            "one step, one revision"
        );
        // the same shape without a definition is a Source; with reads a Rule
        let src = create(None, None).unwrap();
        assert_eq!(
            src.snapshot.design.mappings[&src.outcome.created_mapping.unwrap()].role(),
            crate::RelationshipRole::Source
        );
        let rule = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "scale".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: tilt,
                },
                definition: None,
                clock: None,
            },
        )
        .unwrap();
        assert_eq!(
            rule.snapshot.design.mappings[&rule.outcome.created_mapping.unwrap()].role(),
            crate::RelationshipRole::Rule
        );
    }
}
