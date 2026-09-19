//! The Source role is derived, never stored (ADR-0032, FV Phase 12): an
//! unresolved `() -> A` at the environment boundary is a Source; a
//! resolved `() -> A` is an ordinary relationship; `A -> B` is a
//! relationship; a port-backed body declaration presents its port; hover
//! and Explain say so in the formal vocabulary and keep the canonical type.

mod support;

use bdl_ide::*;
use bdl_model::edit::apply_edit;
use bdl_model::DeclId;
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
        Some(RelationshipRole::Mapping),
        "a resolved () -> A computes internally"
    );
    assert_eq!(
        relationship_role(&snap, lamp.dim_by_tilt),
        Some(RelationshipRole::Mapping),
        "A -> B, unresolved or not, is a relationship"
    );
    assert_eq!(relationship_role(&snap, DeclId::from_raw(999)), None);

    // hover: the role beside the name, the canonical type, no call
    let h = hover(&snap, EntityRef::Mapping(tilt_src)).unwrap();
    assert_eq!(detail(&h, "role"), Some("Source"));
    assert_eq!(detail(&h, "type"), Some("() -> Tilt"));
    assert_eq!(h.signature.as_deref(), Some("mapping tilt : () -> Tilt"));
    assert!(detail(&h, "definition").unwrap().contains("environment"));
    let h = hover(&snap, EntityRef::Mapping(pulse)).unwrap();
    assert_eq!(detail(&h, "role"), Some("Mapping"));
    assert_eq!(detail(&h, "type"), Some("() -> Brightness"));

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
    assert_eq!(line(&e, "Semantics", "role"), Some("Mapping"));
    assert_eq!(line(&e, "Semantics", "provision"), None);
    assert!(line(&e, "Semantics", "reading")
        .unwrap()
        .contains("resolved_not_source"));
}

#[test]
fn a_port_backed_body_declaration_presents_its_port_not_source() {
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
    assert_eq!(
        relationship_role(&snap, tilt_src),
        Some(RelationshipRole::Port(PortKind::Required))
    );
    assert_eq!(
        relationship_role(&snap, other),
        Some(RelationshipRole::Source),
        "an unexposed open declaration of the body is provided by the environment"
    );
    let h = hover(&snap, EntityRef::Mapping(tilt_src)).unwrap();
    assert_eq!(detail(&h, "role"), Some("required port"));
    assert_eq!(detail(&h, "definition"), Some("none — an open declaration"));

    // a provided port's declaration is resolved by its body: still the port
    let s2 = edit(&lamp.snapshot, mapping("level", vec![], lamp.brightness));
    let level = *s2.design.mappings.keys().last().unwrap();
    let s2 = edit(&s2, formula(level, "1"));
    let mut host = IdeHost::new(s2);
    host.set_port_backed(BTreeMap::from([(level, PortKind::Provided)]));
    assert_eq!(
        relationship_role(&host.snapshot(), level),
        Some(RelationshipRole::Port(PortKind::Provided))
    );
}
