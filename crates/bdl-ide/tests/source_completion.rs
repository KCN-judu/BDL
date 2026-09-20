//! Completion in a source file as the Code view asks for it
//! (`CompletionContext::Document` over a text workspace): the slot the
//! line is filling, the scope the position is in, and — inside a
//! formula body — the formula engine with the body's own names.  The
//! candidates and their insertion text are the service's; a client
//! composes nothing.

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

/// Candidates at the end of `needle`'s first occurrence in the buffer
/// `text` of `path`, set as that document's overlay.
fn complete_at(
    host: &mut IdeHost,
    path: &str,
    text: &str,
    needle: &str,
) -> Vec<SemanticCompletion> {
    host.set_text_document(&file_uri(path), text.to_owned());
    let doc = host.file_document(path).expect("document");
    let offset = (text.find(needle).expect("needle") + needle.len()) as u32;
    let snap = host.snapshot();
    completion(
        &snap,
        &CompletionContext::Document {
            document: doc,
            offset,
        },
    )
}

fn labels(items: &[SemanticCompletion]) -> Vec<&str> {
    items.iter().map(|i| i.label.as_str()).collect()
}

fn find<'a>(items: &'a [SemanticCompletion], label: &str) -> &'a SemanticCompletion {
    items
        .iter()
        .find(|i| i.label == label)
        .unwrap_or_else(|| panic!("no {label:?} in {:?}", labels(items)))
}

const A: &str = "\
concept Tilt : Angle
concept Brightness : Scalar
concept Readings : List<Tilt>
clock main
mapping tilt : () -> Tilt @main
mapping readings : () -> Readings @main
mapping dimByTilt : Tilt -> Brightness
dimByTilt(t) = clamp(t / (90 deg), 0, 1)
mapping brightness : () -> Brightness @main
brightness() = dimByTilt(tilt)
";

#[test]
fn after_a_colon_the_candidates_are_concepts_never_values() {
    let mut h = host(&[("src/a.bdl", A)]);
    let text = format!("{A}mapping level : Bri");
    let items = complete_at(&mut h, "src/a.bdl", &text, "level : Bri");
    assert_eq!(labels(&items), vec!["Brightness"]);
    let c = find(&items, "Brightness");
    assert_eq!(c.kind, CompletionKind::Concept);
    assert_eq!(c.insert, "Brightness");
    assert_eq!(
        &text[c.replace.start as usize..c.replace.end as usize],
        "Bri"
    );
    // with no prefix: every concept, no relationship, no unit, no keyword
    let text = format!("{A}mapping level : ");
    let items = complete_at(&mut h, "src/a.bdl", &text, "level : ");
    assert!(
        items.iter().all(|i| i.kind == CompletionKind::Concept),
        "{:?}",
        labels(&items)
    );
    assert!(labels(&items).contains(&"Tilt"));
    assert!(!labels(&items).contains(&"tilt"));
}

#[test]
fn in_a_formula_a_source_and_a_value_are_references_and_a_rule_is_a_call() {
    let mut h = host(&[("src/a.bdl", A)]);
    let text = format!("{A}mapping level : () -> Brightness @main\nlevel() = ");
    let items = complete_at(&mut h, "src/a.bdl", &text, "level() = ");
    // the Source and the value: their names, as values
    let tilt = find(&items, "tilt");
    assert_eq!(tilt.kind, CompletionKind::Mapping);
    assert_eq!(tilt.insert, "tilt");
    let b = find(&items, "brightness");
    assert_eq!(b.insert, "brightness");
    assert_eq!(
        b.resulting_type.as_deref(),
        Some("a dimensionless quantity")
    );
    // the rule: a call, never a bare name
    let dim = find(&items, "dimByTilt(Tilt)");
    assert_eq!(dim.kind, CompletionKind::Mapping);
    assert_eq!(dim.insert, "dimByTilt(");
    assert!(!labels(&items).contains(&"dimByTilt"));
    // the equation library, as calls
    let clamp = find(&items, "clamp(x, low, high)");
    assert_eq!(clamp.kind, CompletionKind::Equation);
    assert_eq!(clamp.insert, "clamp(");
    // the value's own name is not offered (an instantaneous cycle)
    assert!(!labels(&items).contains(&"level"));
    // the result type ranks what fits first: a dimensionless value above an angle
    let pos = |l: &str| items.iter().position(|i| i.label == l).unwrap();
    assert!(pos("brightness") < pos("tilt"));
}

#[test]
fn after_a_number_the_candidates_are_units_of_the_expected_dimension_first() {
    let mut h = host(&[("src/a.bdl", A)]);
    let text = format!("{A}mapping lean : () -> Tilt @main\nlean() = 90 ");
    let items = complete_at(&mut h, "src/a.bdl", &text, "lean() = 90 ");
    assert!(!items.is_empty());
    assert!(
        items.iter().all(|i| i.kind == CompletionKind::Unit),
        "{:?}",
        labels(&items)
    );
    assert_eq!(items[0].kind, CompletionKind::Unit);
    assert_eq!(items[0].resulting_type.as_deref(), Some("an angle"));
    let deg = find(&items, "deg");
    assert_eq!(deg.insert, "deg");
    // a prefix narrows to units, still by position
    let text = format!("{A}mapping lean : () -> Tilt @main\nlean() = 90 d");
    let items = complete_at(&mut h, "src/a.bdl", &text, "lean() = 90 d");
    assert!(labels(&items).contains(&"deg"));
    assert!(!labels(&items).contains(&"dimByTilt(Tilt)"));
}

#[test]
fn inside_a_binder_the_local_is_offered_and_shadows_the_design() {
    let mut h = host(&[("src/a.bdl", A)]);
    let text =
        format!("{A}mapping anyFlat : () -> Brightness @main\nanyFlat() = any r in readings: r");
    let items = complete_at(&mut h, "src/a.bdl", &text, "readings: r");
    let r = find(&items, "r");
    assert_eq!(r.kind, CompletionKind::Local);
    assert!(
        items.iter().position(|i| i.label == "r").unwrap()
            < items.iter().position(|i| i.label == "readings").unwrap()
    );
    // a local spelled like a value shadows it: the local ranks first
    let text = format!(
        "{A}mapping anyFlat : () -> Brightness @main\nanyFlat() = any tilt in readings: tilt"
    );
    let items = complete_at(&mut h, "src/a.bdl", &text, "readings: tilt");
    let firsts: Vec<_> = items
        .iter()
        .filter(|i| i.label == "tilt")
        .map(|i| i.kind)
        .collect();
    assert_eq!(firsts.first(), Some(&CompletionKind::Local), "{firsts:?}");
    // outside the body the local does not exist
    let text = format!("{A}mapping anyFlat : () -> Brightness @main\nanyFlat() = r");
    let items = complete_at(&mut h, "src/a.bdl", &text, "anyFlat() = r");
    assert!(!items
        .iter()
        .any(|i| i.label == "r" && i.kind == CompletionKind::Local));
}

#[test]
fn a_draft_that_does_not_build_still_completes_from_what_the_last_build_knows() {
    let mut h = host(&[("src/a.bdl", A)]);
    // an unclosed call earlier in the file, then a new value being typed
    let broken = A.replace("dimByTilt(tilt)", "dimByTilt(tilt");
    let text = format!("{broken}mapping level : () -> Brightness @main\nlevel() = dim");
    let items = complete_at(&mut h, "src/a.bdl", &text, "level() = dim");
    assert!(
        labels(&items).iter().any(|l| l.starts_with("dimByTilt")),
        "{:?}",
        labels(&items)
    );
    // and at an item start, the item keywords
    let text = format!("{broken}\nma");
    let items = complete_at(&mut h, "src/a.bdl", &text, "\nma");
    assert_eq!(labels(&items), vec!["mapping"]);
    assert_eq!(items[0].insert, "mapping ");
}

#[test]
fn ranges_are_byte_offsets_on_non_ascii_text() {
    // names are ASCII; the text around them is not
    let base = "/// 傾き — how far the lamp head is tilted\nconcept Tilt : Angle\n";
    let text = format!("{base}/// 明るさ\nmapping lean : () -> Ti");
    let mut h = host(&[("src/a.bdl", base)]);
    let items = complete_at(&mut h, "src/a.bdl", &text, "-> Ti");
    let c = find(&items, "Tilt");
    let start = text.find("-> Ti").unwrap() + 3;
    assert_eq!(c.replace, TextRange::new(start as u32, text.len() as u32));
    assert!(text.is_char_boundary(c.replace.start as usize));
    assert_eq!(
        &text[c.replace.start as usize..c.replace.end as usize],
        "Ti"
    );
}

#[test]
fn clocks_after_at_and_items_in_a_component_body() {
    let sys = include_str!("../../bdl-syntax/test_data/valid/system.bdl");
    let mut h = host(&[("src/system.bdl", sys)]);
    let text = format!("{sys}mapping late : () -> Brightness @");
    let items = complete_at(&mut h, "src/system.bdl", &text, "late : () -> Brightness @");
    assert_eq!(labels(&items), vec!["display", "interaction"]);
    // inside the body: the body's items, the body's own clock
    let text = sys.replacen("  provides brightness", "  pro\n  provides brightness", 1);
    let items = complete_at(&mut h, "src/system.bdl", &text, "  pro");
    assert_eq!(labels(&items), vec!["provides"]);
    let text = sys.replacen(
        "  provides brightness",
        "  mapping late : () -> Brightness @\n  provides brightness",
        1,
    );
    let items = complete_at(&mut h, "src/system.bdl", &text, "late : () -> Brightness @");
    assert_eq!(labels(&items), vec!["blink", "main"]);
    // an instance's component name after the colon
    let text = format!("{sys}instance third : Ad");
    let items = complete_at(&mut h, "src/system.bdl", &text, "third : Ad");
    assert_eq!(labels(&items), vec!["AdaptiveLamp"]);
}
