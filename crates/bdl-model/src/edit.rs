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

use crate::ids::{ClockId, DeclId, DeviceId, OutputId, SemanticId};
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
        id: SemanticId,
        name: String,
    },
    SetConceptDescription {
        id: SemanticId,
        description: String,
    },
    /// Bind (write-once) or rebind a concept's representation.  Binding an
    /// unbound concept is a refinement; rebinding is an edit.
    SetConceptRepresentation {
        id: SemanticId,
        representation: Option<Representation>,
    },
    /// Declare or undeclare the concept ordered (`Concept::ordered`).  An
    /// edit, not a refinement: a formula that compares two values of the
    /// concept types only while it is ordered.
    SetConceptOrdered {
        id: SemanticId,
        ordered: bool,
    },
    /// Refused while any mapping mentions the concept.
    DeleteConcept {
        id: SemanticId,
    },
    CreateMapping {
        name: String,
        #[serde(default)]
        description: String,
        signature: Signature,
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
        accepts: SemanticId,
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
        accepts: SemanticId,
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
    pub created_concept: Option<SemanticId>,
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
    UnknownConcept { id: SemanticId },
    #[error("unknown mapping {id}")]
    UnknownMapping { id: DeclId },
    #[error("concept {id} is still used by {} mapping(s)", used_by.len())]
    ConceptInUse {
        id: SemanticId,
        used_by: Vec<DeclId>,
    },
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
            let (id, ids) = design.ids.fresh_semantic();
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
        } => {
            let name = valid_name(name)?;
            if design.mappings.values().any(|m| m.name == name) {
                return Err(EditError::DuplicateMappingName { name });
            }
            check_signature(&design, signature)?;
            let (id, ids) = design.ids.fresh_decl();
            design.ids = ids;
            design.mappings.insert(
                id,
                MappingBlock {
                    id,
                    name,
                    description: description.clone(),
                    signature: signature.clone(),
                    definition: None,
                    clock: None,
                    drives: None,
                    parameters: Vec::new(),
                },
            );
            EditOutcome {
                created_mapping: Some(id),
                ..EditOutcome::refinement()
            }
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
            let m = mapping_mut(&mut design, *id)?;
            m.signature = signature.clone();
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
            EditOutcome::edit([Invalidation::Deployment])
        }
        EditOp::SetDeviceOutput { id, output } => {
            if let Some(o) = output {
                design
                    .outputs
                    .get(o)
                    .ok_or(EditError::UnknownOutput { id: *o })?;
            }
            device_mut(&mut design, *id)?.output = *output;
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
    id: SemanticId,
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

    fn create_concept(s: &ProjectSnapshot, name: &str) -> (ProjectSnapshot, SemanticId) {
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

    #[test]
    fn create_concept_bumps_revision_and_allocates_id() {
        let s0 = empty();
        let (s1, tilt) = create_concept(&s0, "Tilt");
        assert_eq!(s1.revision, Revision::from_raw(1));
        assert_eq!(s1.design.concepts[&tilt].name, "Tilt");
        // input untouched
        assert!(s0.design.concepts.is_empty());
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
        let ghost = SemanticId::from_raw(99);
        let err = apply_edit(
            &s,
            &EditOp::CreateMapping {
                name: "m".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: ghost,
                },
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
                inputs: vec![SemanticId::from_raw(0)],
                output: SemanticId::from_raw(1),
            },
        };
        let json = serde_json::to_string(&op).unwrap();
        let back: EditOp = serde_json::from_str(&json).unwrap();
        assert_eq!(op, back);
    }
}
