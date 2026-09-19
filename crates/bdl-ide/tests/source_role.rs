//! The role is derived, never stored (ADR-0032, FV Phase 12), and one of
//! three: an unresolved `() -> A` is a Source; a realized `() -> A` — a
//! formula, memory, a binding — is a Value; `A -> B` is a Rule with or
//! without a definition.  A port-backed body declaration keeps its role
//! and carries the port as a fact; a draft that is not committed changes
//! nothing; hover and Explain say so in the formal vocabulary and keep the
//! canonical type.

mod support;

use bdl_ide::*;
use bdl_model::edit::apply_edit;
use bdl_model::{DeclId, EditOp};
use bdl_system::PortKind;
use std::collections::BTreeMap;
use support::*;

fn detail<'a>(h: &'a SemanticHover, key: &str) -> Option<&'a str> {
    h.details
        .iter()
        .find(|d| d.label == key)
        .map(|d| d.value.as_str())
}

fn line<'a>(e: &'a Explanation, heading: &str, key: &str) -> Option<&'a str> {
    e.sections
        .iter()
        .find(|s| s.heading == heading)?
        .lines
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

#[test]
fn unresolved_unit_domain_is_a_source_and_resolved_is_not() {
    let lamp = lamp();
    // tilt : () -> Tilt, unresolved: the environment provides it
    let (s, tilt_src) = {
        let a = apply_edit(&lamp.snapshot, &mapping("tilt", vec![], lamp.tilt)).unwrap();
        (a.snapshot, a.outcome.created_mapping.unwrap())
    };
    // pulse : () -> Brightness, resolved by memory: not a Source
    let (s, pulse) = {
        let a = apply_edit(&s, &mapping("pulse", vec![], lamp.brightness)).unwrap();
        let id = a.outcome.created_mapping.unwrap();
        (edit(&a.snapshot, formula(id, "delay(0, 1)")), id)
    };
    let mut host = IdeHost::new(s);
    let snap = host.snapshot();

    assert_eq!(
        relationship_role(&snap, tilt_src),
        Some(RelationshipRole::Source)
    );
    assert_eq!(
        relationship_role(&snap, pulse),
        Some(RelationshipRole::Value),
        "a resolved () -> A computes internally: a Value"
    );
    assert_eq!(
        relationship_role(&snap, lamp.dim_by_tilt),
        Some(RelationshipRole::Rule),
        "A -> B, unresolved or not, is a Rule"
    );
    assert_eq!(provider(&snap, tilt_src), Some(Provider::Environment));
    assert_eq!(provider(&snap, pulse), None);
    assert_eq!(provider(&snap, lamp.dim_by_tilt), None);
    assert_eq!(relationship_role(&snap, DeclId::from_raw(999)), None);

    // hover: the role beside the name, the canonical type, no call
    let h = hover(&snap, EntityRef::Mapping(tilt_src)).unwrap();
    assert_eq!(detail(&h, "role"), Some("Source"));
    assert_eq!(detail(&h, "type"), Some("() -> Tilt"));
    assert_eq!(h.signature.as_deref(), Some("mapping tilt : () -> Tilt"));
    assert!(detail(&h, "definition").unwrap().contains("environment"));
    let h = hover(&snap, EntityRef::Mapping(pulse)).unwrap();
    assert_eq!(detail(&h, "role"), Some("Value"));
    assert_eq!(detail(&h, "type"), Some("() -> Brightness"));
    assert_eq!(detail(&h, "port"), None);
    let h = hover(&snap, EntityRef::Mapping(lamp.dim_by_tilt)).unwrap();
    assert_eq!(detail(&h, "role"), Some("Rule"));
    assert_eq!(detail(&h, "definition"), Some("none — an open declaration"));

    // Explain: canonical type, kernel interface, role, provision, the
    // empty-product reading — and for the resolved one, why it is not a Source
    let e = explain(&snap, EntityRef::Mapping(tilt_src)).unwrap();
    assert_eq!(line(&e, "Semantics", "canonical type"), Some("() -> Tilt"));
    assert_eq!(line(&e, "Semantics", "role"), Some("Source"));
    assert_eq!(line(&e, "Semantics", "provision"), Some("environment"));
    let reading = line(&e, "Semantics", "reading").unwrap();
    assert!(reading.contains("once per activation"), "{reading}");
    assert!(reading.contains("not an effectful"), "{reading}");
    assert!(line(&e, "Semantics", "domain")
        .unwrap()
        .contains("empty product"));
    let e = explain(&snap, EntityRef::Mapping(pulse)).unwrap();
    assert_eq!(line(&e, "Semantics", "role"), Some("Value"));
    assert_eq!(line(&e, "Semantics", "provision"), None);
    assert!(line(&e, "Semantics", "reading")
        .unwrap()
        .contains("resolved_not_source"));
    let e = explain(&snap, EntityRef::Mapping(lamp.dim_by_tilt)).unwrap();
    assert_eq!(line(&e, "Semantics", "role"), Some("Rule"));
    assert!(line(&e, "Semantics", "reading")
        .unwrap()
        .contains("no value of its own"));
}

/// Role transitions follow the committed realization state and nothing
/// else: a Source with a definition is a Value, without it a Source again;
/// a Rule stays a Rule declared, defined or invalid; a Value with an
/// invalid definition stays a Value (the definition is authored, its
/// checking is a state); a draft that is not committed changes no role.
#[test]
fn roles_follow_the_committed_realization_and_never_a_draft() {
    let lamp = lamp();
    let (s, src) = {
        let a = apply_edit(&lamp.snapshot, &mapping("tilt", vec![], lamp.tilt)).unwrap();
        (a.snapshot, a.outcome.created_mapping.unwrap())
    };
    let role = |s: &bdl_model::ProjectSnapshot, id| {
        relationship_role(&IdeHost::new(s.clone()).snapshot(), id)
    };
    assert_eq!(role(&s, src), Some(RelationshipRole::Source));
    // Source + realization -> Value; remove it -> Source
    let defined = edit(&s, formula(src, "1 deg"));
    assert_eq!(role(&defined, src), Some(RelationshipRole::Value));
    let detached = edit(
        &defined,
        EditOp::ReplaceDefinition {
            id: src,
            definition: None,
        },
    );
    assert_eq!(role(&detached, src), Some(RelationshipRole::Source));
    // a stateful realization is a Value, never a Source by shape
    let memory = edit(&s, formula(src, "delay(0 deg, tilt)"));
    assert_eq!(role(&memory, src), Some(RelationshipRole::Value));
    // an invalid definition is still a definition: Value, in the state invalid
    let invalid = edit(&s, formula(src, "true + 1"));
    let snap = IdeHost::new(invalid.clone()).snapshot();
    assert_eq!(relationship_role(&snap, src), Some(RelationshipRole::Value));
    assert_eq!(
        snap.analysis().mappings[&src].status,
        bdl_compiler::MappingStatus::Invalid
    );
    assert_eq!(
        snap.analysis().mappings[&src].role,
        RelationshipRole::Value,
        "the compiler states the same role"
    );
    // a Rule: declared, defined, invalid — a Rule throughout
    assert_eq!(role(&s, lamp.dim_by_tilt), Some(RelationshipRole::Rule));
    let defined = edit(&s, formula(lamp.dim_by_tilt, "Tilt / 90 deg"));
    assert_eq!(
        role(&defined, lamp.dim_by_tilt),
        Some(RelationshipRole::Rule)
    );
    let invalid = edit(&s, formula(lamp.dim_by_tilt, "Tilt + true"));
    assert_eq!(
        role(&invalid, lamp.dim_by_tilt),
        Some(RelationshipRole::Rule)
    );
    // a draft on the Source: the role is the committed one until it commits
    let mut host = IdeHost::new(s.clone());
    host.set_definition_draft(src, "1 deg");
    let snap = host.snapshot();
    assert!(snap.is_drafted(src));
    assert_eq!(
        relationship_role(&snap, src),
        Some(RelationshipRole::Source)
    );
    assert_eq!(provider(&snap, src), Some(Provider::Environment));
    assert_eq!(
        hover(&snap, EntityRef::Mapping(src))
            .unwrap()
            .details
            .iter()
            .find(|d| d.label == "role")
            .map(|d| d.value.as_str()),
        Some("Source")
    );
}

#[test]
fn a_port_backed_body_declaration_keeps_its_role_and_carries_the_port() {
    let lamp = lamp();
    let (s, tilt_src) = {
        let a = apply_edit(&lamp.snapshot, &mapping("tiltValue", vec![], lamp.tilt)).unwrap();
        (a.snapshot, a.outcome.created_mapping.unwrap())
    };
    let (s, other) = {
        let a = apply_edit(&s, &mapping("ambient", vec![], lamp.brightness)).unwrap();
        (a.snapshot, a.outcome.created_mapping.unwrap())
    };
    // as a component body host: tiltValue backs a required port
    let mut host = IdeHost::new(s);
    host.set_port_backed(BTreeMap::from([(tilt_src, PortKind::Required)]));
    let snap = host.snapshot();
    // a Source of the body, provided through the port: the instance binds it
    assert_eq!(
        relationship_role(&snap, tilt_src),
        Some(RelationshipRole::Source)
    );
    assert_eq!(port_backed(&snap, tilt_src), Some(PortKind::Required));
    assert_eq!(
        provider(&snap, tilt_src),
        Some(Provider::Port(PortKind::Required))
    );
    assert_eq!(
        relationship_role(&snap, other),
        Some(RelationshipRole::Source),
        "an unexposed open declaration of the body is provided by the environment"
    );
    assert_eq!(provider(&snap, other), Some(Provider::Environment));
    assert_eq!(port_backed(&snap, other), None);
    let h = hover(&snap, EntityRef::Mapping(tilt_src)).unwrap();
    assert_eq!(detail(&h, "role"), Some("Source"));
    assert_eq!(detail(&h, "port"), Some("required"));
    assert!(detail(&h, "definition")
        .unwrap()
        .contains("through the port"));
    let e = explain(&snap, EntityRef::Mapping(tilt_src)).unwrap();
    assert_eq!(line(&e, "Semantics", "port"), Some("required"));
    assert!(line(&e, "Semantics", "provision")
        .unwrap()
        .contains("binding"));

    // a provided port's declaration is resolved by its body: still the port
    let s2 = edit(&lamp.snapshot, mapping("level", vec![], lamp.brightness));
    let level = *s2.design.mappings.keys().last().unwrap();
    let s2 = edit(&s2, formula(level, "1"));
    let mut host = IdeHost::new(s2);
    host.set_port_backed(BTreeMap::from([(level, PortKind::Provided)]));
    let snap = host.snapshot();
    assert_eq!(
        relationship_role(&snap, level),
        Some(RelationshipRole::Value)
    );
    assert_eq!(port_backed(&snap, level), Some(PortKind::Provided));
    assert_eq!(
        provider(&snap, level),
        None,
        "a Value is provided by nobody"
    );
}

/// The Formula Composer offers a Source by reference — `ambient`, never
/// `ambient()`: a `() -> A` is read as a value (`refForms_agree`), and the
/// consumer cannot tell a Source from a resolved relationship.
#[test]
fn the_composer_inserts_a_source_by_reference_never_as_a_call() {
    let lamp = lamp();
    let (s, ambient) = {
        let a = apply_edit(&lamp.snapshot, &mapping("ambient", vec![], lamp.brightness)).unwrap();
        (a.snapshot, a.outcome.created_mapping.unwrap())
    };
    let (s, pulse) = {
        let a = apply_edit(&s, &mapping("pulse", vec![], lamp.brightness)).unwrap();
        let id = a.outcome.created_mapping.unwrap();
        (edit(&a.snapshot, formula(id, "delay(0, 1)")), id)
    };
    let mut host = IdeHost::new(s);
    host.set_definition_draft(lamp.dim_by_tilt, "?");
    let slot = formula_slot(&host.snapshot(), lamp.dim_by_tilt, "r").expect("slot");
    let by_entity = |id: DeclId| {
        slot.references
            .iter()
            .find(|r| r.entity == Some(EntityRef::Mapping(id)))
            .expect("offered")
    };
    let src = by_entity(ambient);
    assert_eq!(src.label, "ambient");
    assert_eq!(src.insert, "ambient", "by reference, not a call");
    assert!(src.produces.starts_with("Brightness"));
    // the resolved () -> Brightness is offered the same way: consumers are
    // indistinguishable (`consumers_indistinguishable`)
    let res = by_entity(pulse);
    assert_eq!(res.insert, "pulse");
}
