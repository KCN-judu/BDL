//! The Sem-block model (ADR-0044; BDL_FV Phase 21, FVD-0163): a Sem block
//! is a unit-domain declaration with one value per tick; its mapping block
//! is its definition; a rule is the template a mapping block applies.  The
//! analysis reports the read edges of a mapping block as `dependencies.all`
//! — the Sem blocks and rules its definition names — which is the kernel's
//! `dependsOn` (FV `reads_iff_dependsOn`); nothing is resolved by concept;
//! two Sem blocks of one concept raise nothing.

#![allow(clippy::unwrap_used)]

use bdl_compiler::{analyze, MappingStatus};
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{ConceptId, DeclId, Dim};
use std::collections::BTreeSet;

fn concept(s: &ProjectSnapshot, name: &str, rep: Representation) -> (ProjectSnapshot, ConceptId) {
    let a = apply_edit(
        s,
        &EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: Some(rep),
        },
    )
    .unwrap();
    let id = a.outcome.created_concept.unwrap();
    (a.snapshot, id)
}

fn mapping(
    s: &ProjectSnapshot,
    name: &str,
    inputs: Vec<ConceptId>,
    output: ConceptId,
    formula: Option<&str>,
) -> (ProjectSnapshot, DeclId) {
    let a = apply_edit(
        s,
        &EditOp::CreateMapping {
            name: name.into(),
            description: String::new(),
            signature: Signature { inputs, output },
            definition: formula.map(|f| Definition::Formula { source: f.into() }),
            clock: None,
        },
    )
    .unwrap();
    let id = a.outcome.created_mapping.unwrap();
    (a.snapshot, id)
}

fn set(ids: &[DeclId]) -> BTreeSet<DeclId> {
    ids.iter().copied().collect()
}

/// The picture of the brief (FV `lamp_picture`): `pressed` a Source Sem
/// block, `lit` a rule, `Lit` a Sem block whose mapping block applies the
/// rule to `pressed`.  The read edges of `Lit` are `pressed` and `lit`;
/// the rule and the Source read nothing; `applied_by` is the inverse.
#[test]
fn a_mapping_block_reads_the_sem_blocks_and_rules_its_definition_names() {
    let s = ProjectSnapshot::new(Design::empty("lamp"));
    let (s, pressed_c) = concept(&s, "Pressed", Representation::Boolean);
    let (s, lit_c) = concept(&s, "Lit", Representation::Boolean);
    let (s, pressed) = mapping(&s, "pressed", vec![], pressed_c, None);
    let (s, lit) = mapping(&s, "lit", vec![pressed_c], lit_c, Some("Pressed"));
    let (s, lit_v) = mapping(&s, "litV", vec![], lit_c, Some("lit(pressed)"));
    let a = analyze(&s);
    let reads = |d: DeclId| a.dependencies.all.get(&d).cloned().unwrap_or_default();
    assert_eq!(reads(lit_v), set(&[pressed, lit]));
    assert_eq!(reads(lit), set(&[]));
    assert_eq!(reads(pressed), set(&[]));
    assert_eq!(a.mappings[&pressed].applied_by, set(&[lit_v]));
    assert_eq!(a.mappings[&lit].applied_by, set(&[lit_v]));
    assert_eq!(a.mappings[&lit_v].status, MappingStatus::ClockConsistent);
    assert_eq!(a.mappings[&lit].role, bdl_model::RelationshipRole::Rule);
    assert_eq!(a.mappings[&lit_v].role, bdl_model::RelationshipRole::Value);
    assert_eq!(
        a.mappings[&pressed].role,
        bdl_model::RelationshipRole::Source
    );
}

/// A rule is a template (FV `rule_template`): applied twice it makes two
/// Sem blocks of one concept, each reading its own Source.  Two Sem blocks
/// of one concept — and two Sources of one concept — raise no diagnostic.
#[test]
fn a_rule_applied_twice_gives_two_sem_blocks_of_one_concept_and_nothing_complains() {
    let s = ProjectSnapshot::new(Design::empty("lamp"));
    let (s, pressed_c) = concept(&s, "Pressed", Representation::Boolean);
    let (s, lit_c) = concept(&s, "Lit", Representation::Boolean);
    let (s, pressed_a) = mapping(&s, "pressedA", vec![], pressed_c, None);
    let (s, pressed_b) = mapping(&s, "pressedB", vec![], pressed_c, None);
    let (s, lit) = mapping(&s, "lit", vec![pressed_c], lit_c, Some("Pressed"));
    let (s, lit_a) = mapping(&s, "litA", vec![], lit_c, Some("lit(pressedA)"));
    let (s, lit_b) = mapping(&s, "litB", vec![], lit_c, Some("lit(pressedB)"));
    let a = analyze(&s);
    let reads = |d: DeclId| a.dependencies.all.get(&d).cloned().unwrap_or_default();
    assert_eq!(reads(lit_a), set(&[pressed_a, lit]));
    assert_eq!(reads(lit_b), set(&[pressed_b, lit]));
    assert_eq!(a.mappings[&lit].applied_by, set(&[lit_a, lit_b]));
    for d in [pressed_a, pressed_b, lit, lit_a, lit_b] {
        assert!(
            a.mappings[&d].diagnostics.is_empty(),
            "{d:?}: {:?}",
            a.mappings[&d].diagnostics
        );
    }
    assert!(a.diagnostics.is_empty(), "{:?}", a.diagnostics);
}

/// Nothing is resolved by concept: a formula naming a concept that is not
/// an input is `formula.name.not_an_input`, not a lookup of "the value of
/// that concept" — even when exactly one Sem block of it exists.
#[test]
fn a_concept_name_in_a_formula_is_not_a_value() {
    let s = ProjectSnapshot::new(Design::empty("lamp"));
    let (s, tilt_c) = concept(&s, "Tilt", Representation::Quantity { dim: Dim::ANGLE });
    let (s, level_c) = concept(&s, "Level", Representation::Quantity { dim: Dim::ZERO });
    let (s, _tilt) = mapping(&s, "tilt", vec![], tilt_c, None);
    let (s, level) = mapping(&s, "level", vec![], level_c, Some("Tilt / 90 deg"));
    let a = analyze(&s);
    let codes: Vec<&str> = a.mappings[&level]
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(codes, vec!["formula.name.not_an_input"]);
    assert!(a.dependencies.all.get(&level).is_none_or(|r| r.is_empty()));
}
