//! A concept rename is one edit to the designer and several to the model:
//! the concept's name, the same name where a component body shares the
//! concept (`use concept`, one concept under two local ids), and every
//! formula that reads the concept *by its display name* — a parameter
//! name is lexical and stays (ADR-0020, TEXTUAL_SYNTAX §14.4).  The
//! occurrences are found by the elaborator's own resolution rule through
//! `bdl_ide_db::index::formula_input_names`, never by string matching.
//!
//! The expansion is applied as one revision, so undo takes the rename
//! back whole and no snapshot ever has a body naming a concept that no
//! longer exists.

use bdl_ide_db::index::formula_input_names;
use bdl_model::surface::{Definition, Design};
use bdl_model::{EditOp, SemanticId};
use bdl_system::{BehaviorSystem, ComponentId, SystemEditOp};

/// The definition rewrites a design needs when `concept` becomes `name`.
fn formula_rewrites(design: &Design, concept: SemanticId, name: &str) -> Vec<EditOp> {
    let mut ops = Vec::new();
    for m in design.mappings.values() {
        if !m.signature.inputs.contains(&concept) {
            continue;
        }
        let Some(Definition::Formula { source }) = &m.definition else {
            continue;
        };
        let mut occurrences: Vec<_> =
            formula_input_names(design, &m.signature.inputs, &m.parameters, source)
                .into_iter()
                .filter(|n| n.concept == concept && !n.by_parameter)
                .collect();
        if occurrences.is_empty() {
            continue;
        }
        // Later occurrences first, so earlier offsets stay valid.
        occurrences.sort_by_key(|n| std::cmp::Reverse(n.range.start));
        let mut text = source.clone();
        for n in occurrences {
            text.replace_range(n.range.start as usize..n.range.end as usize, name);
        }
        ops.push(EditOp::ReplaceDefinition {
            id: m.id,
            definition: Some(Definition::Formula { source: text }),
        });
    }
    ops
}

/// A system's rename: the base concept and every body copy of it are one
/// concept to the designer and to the text, so they move together.
pub fn expand_system(system: &BehaviorSystem, op: &SystemEditOp) -> Vec<SystemEditOp> {
    let (base, name) = match op {
        SystemEditOp::Base {
            op: EditOp::RenameConcept { id, name },
        } => (Some(*id), name.clone()),
        SystemEditOp::EditComponentBody {
            component,
            op: EditOp::RenameConcept { id, name },
        } => {
            let shared = system
                .components
                .get(component)
                .and_then(|c| c.shared_concepts.get(id).copied());
            match shared {
                Some(sys) => (Some(sys), name.clone()),
                None => {
                    // A private body concept: the body's formulas follow.
                    let Some(c) = system.components.get(component) else {
                        return vec![op.clone()];
                    };
                    let mut ops: Vec<SystemEditOp> = formula_rewrites(&c.body, *id, name)
                        .into_iter()
                        .map(|op| SystemEditOp::EditComponentBody {
                            component: *component,
                            op,
                        })
                        .collect();
                    ops.push(op.clone());
                    return ops;
                }
            }
        }
        _ => return vec![op.clone()],
    };
    let Some(sys) = base else {
        return vec![op.clone()];
    };
    let mut ops: Vec<SystemEditOp> = formula_rewrites(&system.base, sys, &name)
        .into_iter()
        .map(|op| SystemEditOp::Base { op })
        .collect();
    for c in system.components.values() {
        let locals: Vec<SemanticId> = c
            .shared_concepts
            .iter()
            .filter(|(_, s)| **s == sys)
            .map(|(l, _)| *l)
            .collect();
        for local in locals {
            let body_ops = formula_rewrites(&c.body, local, &name);
            ops.extend(component_ops(c.id, body_ops));
            ops.push(SystemEditOp::EditComponentBody {
                component: c.id,
                op: EditOp::RenameConcept {
                    id: local,
                    name: name.clone(),
                },
            });
        }
    }
    ops.push(SystemEditOp::Base {
        op: EditOp::RenameConcept {
            id: sys,
            name: name.clone(),
        },
    });
    ops
}

fn component_ops(component: ComponentId, ops: Vec<EditOp>) -> Vec<SystemEditOp> {
    ops.into_iter()
        .map(|op| SystemEditOp::EditComponentBody { component, op })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::surface::{Concept, MappingBlock, Representation, Signature};
    use bdl_model::Dim;

    fn lamp() -> (Design, SemanticId) {
        let mut d = Design::empty("lamp");
        let (tilt, ids) = d.ids.fresh_semantic();
        let (bright, ids) = ids.fresh_semantic();
        let (dim, ids) = ids.fresh_decl();
        let (by_param, ids) = ids.fresh_decl();
        d.ids = ids;
        for (id, name, dim) in [
            (tilt, "Tilt", Dim::ANGLE),
            (bright, "Brightness", Dim::ZERO),
        ] {
            d.concepts.insert(
                id,
                Concept {
                    id,
                    name: name.into(),
                    description: String::new(),
                    representation: Some(Representation::Quantity { dim }),
                    ordered: false,
                },
            );
        }
        d.mappings.insert(
            dim,
            MappingBlock {
                id: dim,
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: bright,
                },
                parameters: vec![],
                definition: Some(Definition::Formula {
                    source: "tilt / (90 deg) + Tilt / (90 deg)".into(),
                }),
                clock: None,
                drives: None,
            },
        );
        d.mappings.insert(
            by_param,
            MappingBlock {
                id: by_param,
                name: "byParam".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: bright,
                },
                parameters: vec!["t".into()],
                definition: Some(Definition::Formula {
                    source: "t / (90 deg)".into(),
                }),
                clock: None,
                drives: None,
            },
        );
        (d, tilt)
    }

    #[test]
    fn concept_named_occurrences_follow_and_parameters_stay() {
        let (d, tilt) = lamp();
        let system = BehaviorSystem::from_flat(d);
        let ops = expand_system(
            &system,
            &SystemEditOp::Base {
                op: EditOp::RenameConcept {
                    id: tilt,
                    name: "Lean".into(),
                },
            },
        );
        assert_eq!(ops.len(), 2, "{ops:?}");
        match &ops[0] {
            SystemEditOp::Base {
                op:
                    EditOp::ReplaceDefinition {
                        definition: Some(Definition::Formula { source }),
                        ..
                    },
            } => assert_eq!(source, "Lean / (90 deg) + Lean / (90 deg)"),
            other => panic!("{other:?}"),
        }
        assert!(matches!(
            &ops[1],
            SystemEditOp::Base {
                op: EditOp::RenameConcept { .. }
            }
        ));
    }
}
