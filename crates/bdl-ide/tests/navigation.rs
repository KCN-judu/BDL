//! What is at a position, for the Code view and the language server
//! alike (`docs/architecture/ide-service.md`): hover, definition and
//! references by identity across the files of a workspace, inside a
//! component's authored source, on a document that does not build, and
//! on a formula draft — never by spelling.

use bdl_ide::*;
use bdl_ide_db::workspace::file_uri;
use bdl_text::{IdentityTable, SourceFile};

fn host(files: &[(&str, &str)]) -> IdeHost {
    IdeHost::text_workspace(
        "ws",
        files
            .iter()
            .map(|(p, t)| SourceFile {
                path: (*p).into(),
                text: (*t).into(),
            })
            .collect(),
        IdentityTable::default(),
    )
}

fn doc(host: &IdeHost, path: &str) -> DocumentId {
    host.file_document(path).expect("document")
}

/// Byte offset of the `n`th occurrence of `needle` in `text`.
fn at(text: &str, needle: &str, n: usize) -> u32 {
    text.match_indices(needle)
        .nth(n)
        .map(|(i, _)| i as u32)
        .expect("occurrence")
}

fn slice(text: &str, r: TextRange) -> &str {
    &text[r.start as usize..r.end as usize]
}

const A: &str = "\
concept Tilt : Angle
concept Brightness : Scalar
clock main
mapping tilt : () -> Tilt @main
mapping dimByTilt : Tilt -> Brightness
dimByTilt(t) = clamp(t / (90 deg), 0, 1)
";

const B: &str = "\
mapping brightness : () -> Brightness @main
brightness() = dimByTilt(tilt)
output light : Brightness @main
drive light = brightness
";

#[test]
fn definition_and_references_cross_files_by_identity() {
    let mut host = host(&[("src/a.bdl", A), ("src/b.bdl", B)]);
    let snap = host.snapshot();
    let (a, b) = (doc(&host, "src/a.bdl"), doc(&host, "src/b.bdl"));

    // `dimByTilt` applied in b → declared in a
    let def = definition_at(&snap, b, at(B, "dimByTilt", 0) + 2);
    assert_eq!(def.len(), 1, "{def:?}");
    assert_eq!(def[0].document(), Some(a));
    assert_eq!(slice(A, def[0].text_range().unwrap()), "dimByTilt");
    // `tilt`, a Source named in b's formula → declared in a
    let def = definition_at(&snap, b, at(B, "tilt)", 0));
    assert_eq!(def[0].document(), Some(a));
    assert_eq!(slice(A, def[0].text_range().unwrap()), "tilt");
    // `Brightness` in b's signature → the concept in a
    let def = definition_at(&snap, b, at(B, "Brightness", 0));
    assert_eq!(slice(A, def[0].text_range().unwrap()), "Brightness");

    // references to `Brightness` from a: the two signatures in b, the
    // output's accepts, and the concept's own name when asked
    let refs = references_at(&snap, a, at(A, "Brightness", 0), false);
    let in_b: Vec<&str> = refs
        .iter()
        .filter(|r| r.document() == Some(b))
        .map(|r| slice(B, r.text_range().unwrap()))
        .collect();
    assert_eq!(in_b, vec!["Brightness", "Brightness"], "{refs:?}");
    assert!(refs
        .iter()
        .all(|r| r.document() != Some(a) || r.role != EntityRole::Name));
    let with_decl = references_at(&snap, a, at(A, "Brightness", 0), true);
    assert_eq!(with_decl.len(), refs.len() + 1);
    // references to the rule: its application in b
    let refs = references_at(&snap, a, at(A, "dimByTilt", 0), false);
    assert!(
        refs.iter()
            .any(|r| r.document() == Some(b) && slice(B, r.text_range().unwrap()) == "dimByTilt"),
        "{refs:?}"
    );

    // hover on the reference in b: the rule's card with its role
    let h = hover_at(&snap, b, at(B, "dimByTilt", 0)).expect("hover");
    assert_eq!(slice(B, h.range), "dimByTilt");
    let HoverContent::Entity(e) = h.content else {
        panic!("entity")
    };
    assert_eq!(e.title, "dimByTilt");
    assert!(
        e.details
            .iter()
            .any(|d| d.label == "role" && d.value == "Rule"),
        "{:?}",
        e.details
    );
    // hover on the Source
    let h = hover_at(&snap, b, at(B, "tilt)", 0)).expect("hover");
    let HoverContent::Entity(e) = h.content else {
        panic!("entity")
    };
    assert!(
        e.details
            .iter()
            .any(|d| d.label == "role" && d.value == "Source"),
        "{:?}",
        e.details
    );
}

#[test]
fn equations_hover_with_the_library_words_and_have_no_definition() {
    let mut host = host(&[("src/a.bdl", A)]);
    let snap = host.snapshot();
    let a = doc(&host, "src/a.bdl");
    let h = hover_at(&snap, a, at(A, "clamp", 0) + 1).expect("hover");
    assert_eq!(slice(A, h.range), "clamp");
    let HoverContent::Equation {
        name,
        shape,
        documentation,
    } = h.content
    else {
        panic!("equation")
    };
    assert_eq!(name, "clamp");
    assert!(shape.starts_with("clamp("));
    assert!(!documentation.is_empty());
    assert!(definition_at(&snap, a, at(A, "clamp", 0)).is_empty());
    assert!(references_at(&snap, a, at(A, "clamp", 0), true).is_empty());
}

#[test]
fn nothing_semantic_answers_none_cleanly() {
    let mut host = host(&[("src/a.bdl", A)]);
    let snap = host.snapshot();
    let a = doc(&host, "src/a.bdl");
    // a keyword, a number, a unit, whitespace, a parameter
    for needle in ["concept", "90", "deg", "\n", "(t)"] {
        let off = at(A, needle, 0) + if needle == "(t)" { 1 } else { 0 };
        assert!(hover_at(&snap, a, off).is_none(), "{needle:?}");
        assert!(definition_at(&snap, a, off).is_empty(), "{needle:?}");
    }
    assert!(hover_at(&snap, a, A.len() as u32 + 10).is_none());
}

#[test]
fn nominal_concepts_with_one_representation_never_collide() {
    let text = "\
concept Tilt : Angle
concept Heading : Angle
mapping tilt : () -> Tilt
mapping heading : () -> Heading
";
    let mut host = host(&[("src/a.bdl", text)]);
    let snap = host.snapshot();
    let a = doc(&host, "src/a.bdl");
    let tilt = references_at(&snap, a, at(text, "Tilt", 0), true);
    let heading = references_at(&snap, a, at(text, "Heading", 0), true);
    let names = |v: &[ProjectionAnchor]| -> Vec<&str> {
        v.iter()
            .map(|r| slice(text, r.text_range().unwrap()))
            .collect()
    };
    assert_eq!(names(&tilt), vec!["Tilt", "Tilt"]);
    assert_eq!(names(&heading), vec!["Heading", "Heading"]);
    // definition from a reference goes to its own declaration
    let d = definition_at(&snap, a, at(text, "-> Heading", 0) + 3);
    assert_eq!(d[0].text_range().unwrap().start, at(text, "Heading", 0));
}

#[test]
fn a_component_body_navigates_the_authored_source_never_a_flattened_copy() {
    let text = include_str!("../../bdl-syntax/test_data/valid/system.bdl");
    let mut host = host(&[("src/system.bdl", text)]);
    let snap = host.snapshot();
    let d = doc(&host, "src/system.bdl");
    // `tiltValue` inside the body's formula → the `requires` port line
    let body_use = at(text, "dimByTilt(tiltValue)", 0) + "dimByTilt(".len() as u32;
    let def = definition_at(&snap, d, body_use);
    assert!(!def.is_empty());
    let requires_line = at(text, "requires tiltValue", 0);
    assert!(
        def.iter().all(|a| {
            let r = a.text_range().unwrap();
            r.start > requires_line && r.start < requires_line + "requires tiltValue".len() as u32
        }),
        "{def:?}"
    );
    // the hover there is the port — the authored thing — not a copy
    let h = hover_at(&snap, d, body_use).expect("hover");
    let HoverContent::Entity(e) = h.content else {
        panic!("entity")
    };
    assert_eq!(e.kind, EntityKind::Port);
    assert_eq!(e.title, "AdaptiveLamp.tiltValue");
    assert_eq!(
        e.signature.as_deref(),
        Some("required port of AdaptiveLamp")
    );
    // a body relationship (no port): the copy is presented by its
    // authored name, and its declaration is the body's line
    let h = hover_at(&snap, d, at(text, "dimByTilt(tiltValue)", 0)).expect("hover");
    let HoverContent::Entity(e) = h.content else {
        panic!("entity")
    };
    assert_eq!(e.title, "dimByTilt", "{e:?}");
    assert!(
        e.signature
            .as_deref()
            .is_some_and(|s| s.starts_with("mapping dimByTilt :")),
        "{:?}",
        e.signature
    );
    // `dimByTilt` applied in the body → the body's own declaration
    let def = definition_at(&snap, d, at(text, "dimByTilt(tiltValue)", 0));
    let decl = at(text, "mapping dimByTilt", 0);
    assert!(
        def.iter()
            .all(|a| a.text_range().unwrap().start == decl + "mapping ".len() as u32),
        "{def:?}"
    );
    // an instance's port in a bind line → the port in the component
    let refs = definition_at(
        &snap,
        d,
        at(text, "lampA.tiltValue", 0) + "lampA.".len() as u32,
    );
    assert!(
        refs.iter()
            .all(|a| a.text_range().unwrap().start > requires_line),
        "{refs:?}"
    );
    // `AdaptiveLamp` at an instance → the component
    let def = definition_at(&snap, d, at(text, ": AdaptiveLamp", 0) + 2);
    assert_eq!(slice(text, def[0].text_range().unwrap()), "AdaptiveLamp");
    assert_eq!(
        def[0].text_range().unwrap().start,
        at(text, "component AdaptiveLamp", 0) + "component ".len() as u32
    );
}

#[test]
fn a_document_that_does_not_build_keeps_what_still_resolves() {
    // b is broken (an unclosed call); a's declarations still resolve from
    // b's references outside the broken item, and the broken formula's
    // names resolve where the last build knows them
    let broken = B.replace("dimByTilt(tilt)", "dimByTilt(tilt");
    let mut host = host(&[("src/a.bdl", A), ("src/b.bdl", B)]);
    host.set_text_document(&file_uri("src/b.bdl"), broken.clone());
    let snap = host.snapshot();
    let (a, b) = (doc(&host, "src/a.bdl"), doc(&host, "src/b.bdl"));
    assert!(snap.document(b).is_some_and(|d| d.source == broken));
    // the concept in the signature still navigates
    let def = definition_at(&snap, b, at(&broken, "Brightness", 0));
    assert!(def.iter().any(|d| d.document() == Some(a)), "{def:?}");
    // a name nothing resolves any more answers nothing, not a guess
    let h = hover_at(&snap, b, at(&broken, "dimByTilt(tilt", 0) + 11);
    assert!(
        h.is_none()
            || matches!(&h, Some(HoverAt { content: HoverContent::Entity(e), .. }) if e.title == "tilt")
    );
}

#[test]
fn a_formula_draft_hovers_its_inputs_relationships_and_equations() {
    let mut host = host(&[("src/a.bdl", A), ("src/b.bdl", B)]);
    let snap = host.snapshot();
    let dim = snap
        .effective()
        .design
        .mappings
        .values()
        .find(|m| m.name == "dimByTilt")
        .map(|m| m.id)
        .expect("dimByTilt");
    let src = "clamp(t / (90 deg), 0, 1)";
    // a parameter name is the input concept
    let h = formula_hover_at(&snap, dim, 6).expect("t");
    assert_eq!(slice(src, h.range), "t");
    let HoverContent::Entity(e) = h.content else {
        panic!("entity")
    };
    assert_eq!(e.title, "Tilt");
    // the equation
    let h = formula_hover_at(&snap, dim, 0).expect("clamp");
    assert!(matches!(h.content, HoverContent::Equation { ref name, .. } if name == "clamp"));
    // a relationship named in a value's draft
    let brightness = snap
        .effective()
        .design
        .mappings
        .values()
        .find(|m| m.name == "brightness")
        .map(|m| m.id)
        .expect("brightness");
    let h = formula_hover_at(&snap, brightness, 2).expect("dimByTilt");
    let HoverContent::Entity(e) = h.content else {
        panic!("entity")
    };
    assert_eq!(e.title, "dimByTilt");
    // nothing on an operator or a number
    assert!(formula_hover_at(&snap, dim, 8).is_none());
}
