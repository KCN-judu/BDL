//! The entity/reference index of one snapshot: which entities exist,
//! what they are called, and who refers to whom — all by identity.
//!
//! References are *semantic* facts of the model (a mapping's signature
//! mentions a concept; a drive edge names an output; an output accepts a
//! concept; a formula body reads an input), computed once here.  Surfaces
//! then *place* them through the projection map.  Nothing in this module
//! searches text.

use crate::entity::{EntityKind, EntityRef, EntityRole};
use crate::text::TextRange;
use bdl_elab::names::{InputEnv, Lookup};
use bdl_model::surface::{Definition, Design};
use bdl_model::{DeclId, SemanticId};
use bdl_syntax::ast::{self, AstNode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// One semantic reference: `referrer` mentions `target` in `role`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticReference {
    pub target: EntityRef,
    pub referrer: EntityRef,
    pub role: EntityRole,
    /// For a reference from inside a formula body: the body-relative
    /// range of the name, so it can be placed in whichever document holds
    /// the formula (or the Studio definition field).
    pub in_formula: Option<TextRange>,
}

/// The index: names and references of every entity in the effective
/// design.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityIndex {
    names: BTreeMap<EntityRef, String>,
    /// `target → references to it`, each list in `(referrer, role)` order.
    incoming: BTreeMap<EntityRef, Vec<SemanticReference>>,
    /// `referrer → what it references`.
    outgoing: BTreeMap<EntityRef, Vec<SemanticReference>>,
    /// Concept-name resolution inside each formula body: body-relative
    /// range → concept, for hover/navigation inside formulas.
    formula_names: BTreeMap<DeclId, Vec<(TextRange, SemanticId)>>,
}

impl EntityIndex {
    pub fn build(design: &Design) -> EntityIndex {
        let mut ix = EntityIndex::default();
        for c in design.concepts.values() {
            ix.names.insert(EntityRef::Concept(c.id), c.name.clone());
        }
        for m in design.mappings.values() {
            ix.names.insert(EntityRef::Mapping(m.id), m.name.clone());
        }
        for c in design.clocks.values() {
            ix.names.insert(EntityRef::Clock(c.id), c.name.clone());
        }
        for o in design.outputs.values() {
            ix.names.insert(EntityRef::Output(o.id), o.name.clone());
        }
        for d in design.devices.values() {
            ix.names.insert(EntityRef::Device(d.id), d.name.clone());
        }

        for m in design.mappings.values() {
            let me = EntityRef::Mapping(m.id);
            for (i, c) in m.signature.inputs.iter().enumerate() {
                ix.add(SemanticReference {
                    target: EntityRef::Concept(*c),
                    referrer: me,
                    role: EntityRole::Input { index: i as u16 },
                    in_formula: None,
                });
            }
            ix.add(SemanticReference {
                target: EntityRef::Concept(m.signature.output),
                referrer: me,
                role: EntityRole::Output,
                in_formula: None,
            });
            if let Some(c) = m.clock {
                ix.add(SemanticReference {
                    target: EntityRef::Clock(c),
                    referrer: me,
                    role: EntityRole::ClockBinding,
                    in_formula: None,
                });
            }
            if let Some(o) = m.drives {
                ix.add(SemanticReference {
                    target: EntityRef::Output(o),
                    referrer: me,
                    role: EntityRole::DriveEdge,
                    in_formula: None,
                });
            }
            if let Some(Definition::Formula { source }) = &m.definition {
                let names = formula_input_names(design, &m.signature.inputs, &m.parameters, source);
                for n in &names {
                    ix.add(SemanticReference {
                        target: EntityRef::Concept(n.concept),
                        referrer: me,
                        role: EntityRole::Definition,
                        in_formula: Some(n.range),
                    });
                }
                ix.formula_names
                    .insert(m.id, names.iter().map(|n| (n.range, n.concept)).collect());
            }
        }
        for o in design.outputs.values() {
            let me = EntityRef::Output(o.id);
            ix.add(SemanticReference {
                target: EntityRef::Concept(o.accepts),
                referrer: me,
                role: EntityRole::Output,
                in_formula: None,
            });
            if let Some(c) = o.clock {
                ix.add(SemanticReference {
                    target: EntityRef::Clock(c),
                    referrer: me,
                    role: EntityRole::ClockBinding,
                    in_formula: None,
                });
            }
        }
        for d in design.devices.values() {
            if let Some(o) = d.output {
                ix.add(SemanticReference {
                    target: EntityRef::Output(o),
                    referrer: EntityRef::Device(d.id),
                    role: EntityRole::DeviceBinding,
                    in_formula: None,
                });
            }
        }
        for v in ix.incoming.values_mut() {
            v.sort();
        }
        for v in ix.outgoing.values_mut() {
            v.sort();
        }
        ix
    }

    fn add(&mut self, r: SemanticReference) {
        self.outgoing.entry(r.referrer).or_default().push(r.clone());
        self.incoming.entry(r.target).or_default().push(r);
    }

    pub fn name(&self, entity: EntityRef) -> Option<&str> {
        self.names.get(&entity).map(String::as_str)
    }

    pub fn exists(&self, entity: EntityRef) -> bool {
        entity == EntityRef::Project || self.names.contains_key(&entity)
    }

    /// Every entity, in `EntityRef` order.
    pub fn entities(&self) -> impl Iterator<Item = (EntityRef, &str)> {
        self.names.iter().map(|(e, n)| (*e, n.as_str()))
    }

    pub fn entities_of_kind(&self, kind: EntityKind) -> impl Iterator<Item = (EntityRef, &str)> {
        self.entities().filter(move |(e, _)| e.kind() == kind)
    }

    /// The entity of `kind` with exactly this display name, if any.
    pub fn lookup(&self, kind: EntityKind, name: &str) -> Option<EntityRef> {
        self.entities_of_kind(kind)
            .find(|(_, n)| *n == name)
            .map(|(e, _)| e)
    }

    /// References *to* an entity.
    pub fn references_to(&self, entity: EntityRef) -> &[SemanticReference] {
        self.incoming.get(&entity).map_or(&[], Vec::as_slice)
    }

    /// References *from* an entity.
    pub fn references_from(&self, entity: EntityRef) -> &[SemanticReference] {
        self.outgoing.get(&entity).map_or(&[], Vec::as_slice)
    }

    /// The concept named at a body-relative offset of a mapping's formula.
    pub fn formula_name_at(&self, mapping: DeclId, offset: u32) -> Option<(TextRange, SemanticId)> {
        self.formula_names
            .get(&mapping)?
            .iter()
            .find(|(r, _)| r.contains(offset))
            .copied()
    }

    pub fn formula_names(&self, mapping: DeclId) -> &[(TextRange, SemanticId)] {
        self.formula_names.get(&mapping).map_or(&[], Vec::as_slice)
    }
}

/// The input-name occurrences of a formula, resolved to concepts by the
/// elaborator's own rule (`bdl_elab::names`, ADR-0013 / TEXTUAL_SYNTAX
/// §14.4).  Total: an unparsable formula yields the names the tree still
/// has.  Body-relative ranges, in source order; `by_parameter` is `true`
/// when the occurrence is a textual parameter name rather than the
/// concept's own name — a rename of the concept must leave it alone.
pub fn formula_input_names(
    design: &Design,
    inputs: &[SemanticId],
    parameters: &[String],
    source: &str,
) -> Vec<FormulaInputName> {
    let env = InputEnv::with_parameters(design, inputs, parameters);
    let parse = bdl_syntax::parse_formula(source);
    let mut out = Vec::new();
    for node in parse.syntax_node().descendants() {
        let Some(name) = ast::NameExpr::cast(node) else {
            continue;
        };
        let Some(r) = name.name() else { continue };
        if let Lookup::Input(i) = env.resolve(design, &r.as_str()) {
            if let Some(c) = inputs.get(i) {
                out.push(FormulaInputName {
                    range: TextRange::from(r.span()),
                    concept: *c,
                    by_parameter: parameters.get(i).is_some_and(|p| !p.is_empty()),
                });
            }
        }
    }
    out
}

/// One occurrence of an input name inside a formula body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormulaInputName {
    pub range: TextRange,
    pub concept: SemanticId,
    pub by_parameter: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::surface::{Concept, MappingBlock, Signature};
    use bdl_model::Dim;

    fn lamp() -> (Design, SemanticId, SemanticId, DeclId) {
        let mut d = Design::empty("lamp");
        let (tilt, ids) = d.ids.fresh_semantic();
        let (bright, ids) = ids.fresh_semantic();
        let (dim, ids) = ids.fresh_decl();
        d.ids = ids;
        for (id, name) in [(tilt, "Tilt"), (bright, "Brightness")] {
            d.concepts.insert(
                id,
                Concept {
                    id,
                    name: name.into(),
                    description: String::new(),
                    representation: Some(bdl_model::Representation::Quantity { dim: Dim::ANGLE }),
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
                definition: Some(Definition::Formula {
                    source: "Tilt / (90 deg) // Tilt in a comment".into(),
                }),
                clock: None,
                drives: None,
                parameters: Vec::new(),
            },
        );
        (d, tilt, bright, dim)
    }

    #[test]
    fn references_are_by_identity_and_skip_comments() {
        let (d, tilt, bright, dim) = lamp();
        let ix = EntityIndex::build(&d);
        let refs = ix.references_to(EntityRef::Concept(tilt));
        assert_eq!(refs.len(), 2, "{refs:?}");
        assert_eq!(refs[0].role, EntityRole::Input { index: 0 });
        assert_eq!(refs[0].referrer, EntityRef::Mapping(dim));
        assert_eq!(refs[1].role, EntityRole::Definition);
        assert_eq!(refs[1].in_formula, Some(TextRange::new(0, 4)));
        assert_eq!(ix.references_to(EntityRef::Concept(bright)).len(), 1);
        assert_eq!(
            ix.formula_name_at(dim, 2),
            Some((TextRange::new(0, 4), tilt))
        );
        assert_eq!(
            ix.formula_name_at(dim, 20),
            None,
            "comments are not references"
        );
        assert_eq!(
            ix.lookup(EntityKind::Mapping, "dimByTilt"),
            Some(EntityRef::Mapping(dim))
        );
        assert_eq!(ix.lookup(EntityKind::Concept, "dimByTilt"), None);
    }
}
