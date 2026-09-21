//! Invalidation preview: what an edit would reopen, before it is made.
//!
//! BDL's refinement-vs-edit distinction is a model, not UI folklore
//! (`docs/architecture/overview.md`): every [`EditOp`] classifies itself and names
//! the [`Invalidation`] categories and origin declarations it touches.
//! This query applies the edit to a *copy* of the effective snapshot,
//! reads that classification, follows the dependency graph to the
//! declarations that would have to be re-validated, re-analyses the
//! candidate to report status changes, and reports what is *preserved*
//! — identities above all.  Nothing is committed.

use bdl_compiler::MappingStatus;
use bdl_ide_db::{AnalysisSnapshot, EntityRef};
use bdl_model::edit::{apply_edit, EditKind, EditOp, Invalidation};
use bdl_model::DeclId;
use bdl_reactive::DependencyGraph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// A kind of established fact.  The compiler's own categories plus the
/// two the tool adds on top (simulation results, identities).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Fact {
    Interface,
    Realization,
    Semantic,
    Reactive,
    Clock,
    Output,
    Deployment,
    /// A simulation run belongs to one revision; any commit drops it.
    SimulationResults,
    /// Stable ids: never invalidated by any edit.
    Identity,
}

impl Fact {
    fn of(i: Invalidation) -> Fact {
        match i {
            Invalidation::Interface => Fact::Interface,
            Invalidation::Realization => Fact::Realization,
            Invalidation::Semantic => Fact::Semantic,
            Invalidation::Reactive => Fact::Reactive,
            Invalidation::Clock => Fact::Clock,
            Invalidation::Output => Fact::Output,
            Invalidation::Deployment => Fact::Deployment,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Invalidated {
    pub entity: EntityRef,
    pub facts: BTreeSet<Fact>,
    /// Why: the origin declaration it depends on, when it is a dependent
    /// rather than an origin itself.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub through: Option<EntityRef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusChange {
    pub mapping: DeclId,
    pub before: MappingStatus,
    pub after: MappingStatus,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvalidationPreview {
    /// `None` when the edit could not be applied (see `error`) or when
    /// there is nothing to preview.
    pub kind: Option<EditKind>,
    pub categories: BTreeSet<Fact>,
    pub origins: BTreeSet<EntityRef>,
    pub invalidates: Vec<Invalidated>,
    /// Facts and entities the edit leaves alone.  Identities are always
    /// here: an edit never re-identifies anything.
    pub preserves: BTreeSet<Fact>,
    pub preserved_entities: BTreeSet<EntityRef>,
    /// Mapping statuses that would move, from re-analysing the candidate.
    pub status_changes: Vec<StatusChange>,
    /// The edit is refused by the model (a duplicate name, an unknown id).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl InvalidationPreview {
    /// Nothing is invalidated (a pure refinement, or a plan with no model
    /// operation).
    pub fn none() -> InvalidationPreview {
        InvalidationPreview {
            kind: None,
            categories: BTreeSet::new(),
            origins: BTreeSet::new(),
            invalidates: Vec::new(),
            preserves: [Fact::Identity].into_iter().collect(),
            preserved_entities: BTreeSet::new(),
            status_changes: Vec::new(),
            error: None,
        }
    }

    pub fn is_refinement(&self) -> bool {
        matches!(self.kind, Some(EditKind::Refinement) | None) && self.error.is_none()
    }

    /// Merge another preview (a plan with several model operations).
    pub fn merge(&mut self, other: InvalidationPreview) {
        if other.kind == Some(EditKind::Edit) || self.kind.is_none() {
            self.kind = other.kind.or(self.kind);
        }
        self.categories.extend(other.categories);
        self.origins.extend(other.origins);
        for inv in other.invalidates {
            match self.invalidates.iter_mut().find(|i| i.entity == inv.entity) {
                Some(mine) => mine.facts.extend(inv.facts),
                None => self.invalidates.push(inv),
            }
        }
        self.preserves.retain(|f| !self.categories.contains(f));
        self.preserved_entities
            .retain(|e| !self.invalidates.iter().any(|i| i.entity == *e));
        self.status_changes.extend(other.status_changes);
        if other.error.is_some() {
            self.error = other.error;
        }
    }
}

/// What `op` would invalidate, applied to the effective project of
/// `snapshot`.  Pure; the snapshot is untouched.
pub fn preview_change(snapshot: &AnalysisSnapshot, op: &EditOp) -> InvalidationPreview {
    let before = snapshot.effective();
    let applied = match apply_edit(before, op) {
        Ok(a) => a,
        Err(e) => {
            return InvalidationPreview {
                error: Some(e.to_string()),
                ..InvalidationPreview::none()
            }
        }
    };
    let outcome = &applied.outcome;
    let categories: BTreeSet<Fact> = outcome.invalidates.iter().map(|i| Fact::of(*i)).collect();
    let origins: BTreeSet<EntityRef> = outcome
        .origin_decls
        .iter()
        .map(|d| EntityRef::Mapping(*d))
        .collect();

    let mut invalidates: Vec<Invalidated> = Vec::new();
    let mut invalidated_ids: BTreeSet<DeclId> = BTreeSet::new();
    let mut origin_ids: Vec<DeclId> = outcome.origin_decls.iter().copied().collect();
    // The edited entity itself is an origin for the categories it touches
    // even when the model reports none (a signature change names no
    // dependents by itself).
    if let Some(d) = edited_mapping(op) {
        if !origin_ids.contains(&d) {
            origin_ids.push(d);
        }
    }
    let graph = &snapshot.analysis().dependencies;
    for d in &origin_ids {
        if categories.is_empty() {
            break;
        }
        invalidated_ids.insert(*d);
        invalidates.push(Invalidated {
            entity: EntityRef::Mapping(*d),
            facts: categories.clone(),
            through: None,
        });
        // Dependents (who references d) must re-validate their
        // realizations when d's interface or realization changed.
        let dependents_matter = categories.iter().any(|f| {
            matches!(
                f,
                Fact::Interface | Fact::Semantic | Fact::Realization | Fact::Reactive | Fact::Clock
            )
        });
        if dependents_matter {
            for dep in reverse_reachable(graph, *d) {
                if invalidated_ids.insert(dep) {
                    invalidates.push(Invalidated {
                        entity: EntityRef::Mapping(dep),
                        facts: [Fact::Realization, Fact::Reactive].into_iter().collect(),
                        through: Some(EntityRef::Mapping(*d)),
                    });
                }
            }
        }
    }
    if let Some(c) = edited_concept(op) {
        if !categories.is_empty() {
            invalidates.push(Invalidated {
                entity: EntityRef::Concept(c),
                facts: categories.clone(),
                through: None,
            });
        }
    }
    if let Some(o) = edited_output(op) {
        if !categories.is_empty() {
            invalidates.push(Invalidated {
                entity: EntityRef::Output(o),
                facts: categories.clone(),
                through: None,
            });
        }
    }

    // Status changes, from the compiler.
    let after = bdl_compiler::analyze(&applied.snapshot);
    let before_status: BTreeMap<DeclId, MappingStatus> = snapshot
        .analysis()
        .mappings
        .iter()
        .map(|(id, m)| (*id, m.status))
        .collect();
    let mut status_changes = Vec::new();
    for (id, m) in &after.mappings {
        if let Some(b) = before_status.get(id) {
            if *b != m.status {
                status_changes.push(StatusChange {
                    mapping: *id,
                    before: *b,
                    after: m.status,
                });
            }
        }
    }

    let mut preserves: BTreeSet<Fact> = [
        Fact::Identity,
        Fact::Interface,
        Fact::Realization,
        Fact::Semantic,
        Fact::Reactive,
        Fact::Clock,
        Fact::Output,
        Fact::Deployment,
        Fact::SimulationResults,
    ]
    .into_iter()
    .collect();
    let mut categories = categories;
    if outcome.kind == Some(EditKind::Edit) || !outcome.invalidates.is_empty() {
        categories.insert(Fact::SimulationResults);
    }
    preserves.retain(|f| !categories.contains(f));
    let preserved_entities: BTreeSet<EntityRef> = before
        .design
        .mappings
        .keys()
        .filter(|d| !invalidated_ids.contains(d))
        .map(|d| EntityRef::Mapping(*d))
        .collect();

    InvalidationPreview {
        kind: outcome.kind,
        categories,
        origins,
        invalidates,
        preserves,
        preserved_entities,
        status_changes,
        error: None,
    }
}

/// Everything that (transitively) references `d`.
fn reverse_reachable(g: &DependencyGraph, d: DeclId) -> Vec<DeclId> {
    let mut seen = BTreeSet::new();
    let mut stack = vec![d];
    while let Some(x) = stack.pop() {
        if let Some(users) = g.reverse_all.get(&x) {
            for u in users {
                if *u != d && seen.insert(*u) {
                    stack.push(*u);
                }
            }
        }
    }
    seen.into_iter().collect()
}

fn edited_mapping(op: &EditOp) -> Option<DeclId> {
    match op {
        EditOp::RenameMapping { id, .. }
        | EditOp::SetMappingDescription { id, .. }
        | EditOp::SetMappingSignature { id, .. }
        | EditOp::AttachDefinition { id, .. }
        | EditOp::ReplaceDefinition { id, .. }
        | EditOp::DeleteMapping { id }
        | EditOp::SetMappingClock { id, .. }
        | EditOp::SetMappingDrive { id, .. } => Some(*id),
        _ => None,
    }
}

fn edited_concept(op: &EditOp) -> Option<bdl_model::ConceptId> {
    match op {
        EditOp::RenameConcept { id, .. }
        | EditOp::SetConceptDescription { id, .. }
        | EditOp::SetConceptRepresentation { id, .. }
        | EditOp::DeleteConcept { id } => Some(*id),
        _ => None,
    }
}

fn edited_output(op: &EditOp) -> Option<bdl_model::OutputId> {
    match op {
        EditOp::RenameOutput { id, .. }
        | EditOp::SetOutputAccepts { id, .. }
        | EditOp::SetOutputClock { id, .. }
        | EditOp::SetOutputRequired { id, .. }
        | EditOp::DeleteOutput { id } => Some(*id),
        _ => None,
    }
}
