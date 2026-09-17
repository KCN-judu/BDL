//! Rename by identity.
//!
//! A name is a projection of a stable id (D-14, DI-10): renaming changes
//! what every surface *shows*, not what anything *is*.  The plan therefore
//! has one model operation — the rename itself, a refinement with empty
//! invalidation — plus the text edits that keep the projections in step:
//! every anchored name site in open documents, and, because formula
//! bodies name their inputs by display name (ADR-0013), every formula that
//! reads the renamed concept.  Formula occurrences are found by resolving
//! each name to its input (the elaborator's rule), never by searching for
//! the old spelling: a comment that says `Tilt` stays as it is.

use crate::edit_plan::{SemanticEditPlan, SemanticOperation};
use bdl_ide_db::{
    index::formula_input_names, AnalysisSnapshot, EntityKind, EntityRef, EntityRole, TextEdit,
};
use bdl_model::surface::Definition;
use bdl_model::EditOp;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum RenameError {
    #[error("unknown entity {entity}")]
    UnknownEntity { entity: EntityRef },
    #[error("{entity} cannot be renamed")]
    NotRenamable { entity: EntityRef },
    #[error("a name is required")]
    EmptyName,
    #[error("`{name}` is not a valid name: use letters, digits and `_`, starting with a letter")]
    NotAnIdentifier { name: String },
    #[error("another {kind:?} is already named `{name}`")]
    Duplicate { kind: EntityKind, name: String },
}

/// Plan renaming `entity` to `new_name`.  Nothing is applied.
pub fn plan_rename(
    snapshot: &AnalysisSnapshot,
    entity: EntityRef,
    new_name: &str,
) -> Result<SemanticEditPlan, RenameError> {
    let new_name = new_name.trim();
    if !snapshot.exists(entity) {
        return Err(RenameError::UnknownEntity { entity });
    }
    if new_name.is_empty() {
        return Err(RenameError::EmptyName);
    }
    // Studio display names are free text; the textual surface needs an
    // identifier.  A name that cannot be spelled in text is refused only
    // while some document projects the entity.
    let projected_in_text = snapshot
        .projections()
        .anchors_of(entity)
        .iter()
        .any(|a| a.document().is_some());
    if projected_in_text && !is_identifier(new_name) {
        return Err(RenameError::NotAnIdentifier {
            name: new_name.to_owned(),
        });
    }
    let kind = entity.kind();
    if let Some(other) = snapshot.index().lookup(kind, new_name) {
        if other != entity {
            return Err(RenameError::Duplicate {
                kind,
                name: new_name.to_owned(),
            });
        }
    }
    let edit = match entity {
        EntityRef::Concept(id) => EditOp::RenameConcept {
            id,
            name: new_name.to_owned(),
        },
        EntityRef::Mapping(id) => EditOp::RenameMapping {
            id,
            name: new_name.to_owned(),
        },
        EntityRef::Clock(id) => EditOp::RenameClockDomain {
            id,
            name: new_name.to_owned(),
        },
        EntityRef::Output(id) => EditOp::RenameOutput {
            id,
            name: new_name.to_owned(),
        },
        EntityRef::Device(id) => EditOp::RenameDevice {
            id,
            name: new_name.to_owned(),
        },
        EntityRef::Project | EntityRef::Requirement { .. } => {
            return Err(RenameError::NotRenamable { entity })
        }
    };
    let old_name = snapshot.name_of(entity).unwrap_or("").to_owned();
    let mut plan = SemanticEditPlan::new(
        format!("Rename `{old_name}` to `{new_name}`"),
        snapshot.stamp(),
    );
    plan.affected_entities.insert(entity);
    plan.operations.push(SemanticOperation::Model { edit });

    // Text projections: every name and reference site in open documents.
    let mut per_document: BTreeMap<_, Vec<TextEdit>> = BTreeMap::new();
    for a in snapshot.projections().anchors_of(entity) {
        if !matches!(a.role, EntityRole::Name | EntityRole::Reference) {
            continue;
        }
        if let (Some(doc), Some(range)) = (a.document(), a.text_range()) {
            per_document
                .entry(doc)
                .or_default()
                .push(TextEdit::replace(range, new_name));
        }
    }
    for (document, mut edits) in per_document {
        edits.sort_by_key(|e| e.range);
        edits.dedup();
        plan.operations
            .push(SemanticOperation::Text { document, edits });
    }

    // Formula bodies that read a renamed concept: committed formulas become
    // model operations; a draft becomes draft edits; a text-declared
    // definition was already covered by its document's anchors.
    if let EntityRef::Concept(c) = entity {
        let committed = &snapshot.committed().design;
        for r in snapshot.index().references_to(entity) {
            let Some(m) = r.referrer.as_mapping() else {
                continue;
            };
            if r.role != EntityRole::Definition || r.in_formula.is_none() {
                continue;
            }
            if snapshot.declaring_document(r.referrer).is_some() {
                continue;
            }
            if plan.affected_entities.contains(&r.referrer) {
                continue;
            }
            plan.affected_entities.insert(r.referrer);
            let block = snapshot.effective().design.mappings.get(&m);
            let Some(block) = block else { continue };
            let Some(Definition::Formula { source }) = &block.definition else {
                continue;
            };
            let occurrences = formula_input_names(
                &snapshot.effective().design,
                &block.signature.inputs,
                &block.parameters,
                source,
            );
            // A textual parameter name is the body's own; only the
            // concept's name spelled in the body follows the rename.
            let edits: Vec<TextEdit> = occurrences
                .into_iter()
                .filter(|n| n.concept == c && !n.by_parameter)
                .map(|n| TextEdit::replace(n.range, new_name))
                .collect();
            if edits.is_empty() {
                continue;
            }
            if snapshot.is_drafted(m) {
                plan.operations
                    .push(SemanticOperation::DraftText { mapping: m, edits });
                continue;
            }
            let is_committed_formula = committed
                .mappings
                .get(&m)
                .and_then(|b| b.definition.as_ref())
                .is_some_and(|d| matches!(d, Definition::Formula { source: s } if s == source));
            if is_committed_formula {
                if let Ok(rewritten) = TextEdit::apply_all(source, &edits) {
                    plan.operations.push(SemanticOperation::Model {
                        edit: EditOp::ReplaceDefinition {
                            id: m,
                            definition: Some(Definition::Formula { source: rewritten }),
                        },
                    });
                }
            }
        }
    }
    Ok(plan)
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}
