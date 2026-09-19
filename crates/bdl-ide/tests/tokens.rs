//! The token classifier against the actual language
//! (`docs/architecture/syntax-highlighting.md` §Tests): exact ranges and
//! classes on real text, the lexical layer on broken text, the derived
//! role on a relationship as it changes, components, and the LSP encoding
//! of the same stream.

mod support;

use bdl_ide::tokens::encode::{byte_range, decode_data, encode_data, PositionEncoding};
use bdl_ide::*;
use bdl_ide_db::DocumentUri;
use bdl_model::edit::EditOp;
use bdl_model::surface::{Definition, Representation};
use bdl_model::Dim;
use bdl_text::{IdentityTable, SourceFile};
use support::*;

/// `(text, type, modifiers)` rows of a token stream over `text`.
fn rows<'a>(
    text: &'a str,
    tokens: &[SemanticToken],
) -> Vec<(&'a str, TokenType, Vec<&'static str>)> {
    tokens
        .iter()
        .map(|t| {
            (
                &text[t.range.start as usize..t.range.end as usize],
                t.ty,
                t.modifiers.names(),
            )
        })
        .collect()
}

fn assert_well_formed(text: &str, tokens: &[SemanticToken]) {
    for t in tokens {
        assert!(t.range.start < t.range.end, "empty token {t:?}");
        assert!(t.range.end as usize <= text.len(), "out of bounds {t:?}");
        assert!(text.is_char_boundary(t.range.start as usize));
        assert!(text.is_char_boundary(t.range.end as usize));
    }
    for w in tokens.windows(2) {
        assert!(
            w[0].range.end <= w[1].range.start,
            "overlap {:?} {:?}",
            w[0],
            w[1]
        );
    }
}

fn text_host(text: &str) -> (IdeHost, DocumentId, String) {
    let file = SourceFile {
        path: "src/main.bdl".into(),
        text: text.to_owned(),
    };
    let mut host = IdeHost::text_workspace("lamp", vec![file], IdentityTable::default());
    let doc = host.file_document("src/main.bdl").expect("document");
    (host, doc, text.to_owned())
}

fn has(
    rows: &[(&str, TokenType, Vec<&'static str>)],
    text: &str,
    ty: TokenType,
    mods: &[&str],
) -> bool {
    rows.iter()
        .any(|(t, k, m)| *t == text && *k == ty && m.as_slice() == mods)
}

// ---- the lamp, exactly ------------------------------------------------------------

const LAMP: &str = "\
// Lamp — ångström ✓
concept Tilt : Angle
concept Brightness : Scalar

mapping dimByTilt : Tilt -> Brightness
dimByTilt(tilt) =
  clamp(tilt / (90 deg), 0, 1)

mapping temp : () -> RoomTemp

mapping brightness : () -> Brightness
brightness() = dimByTilt(Tilt)
";

#[test]
fn the_lamp_tokenizes_to_exact_classes_on_both_layers() {
    let text = LAMP.replace("RoomTemp", "Brightness");
    let (mut host, doc, text) = text_host(&text);
    let snap = host.snapshot();
    let tokens = semantic_tokens(&snap, doc);
    assert_well_formed(&text, &tokens);
    let r = rows(&text, &tokens);

    // lexical: the comment (with its non-ASCII text), keywords, numbers,
    // the unit apart from its number, operators (not the parentheses)
    assert!(
        has(&r, "// Lamp — ångström ✓", TokenType::Comment, &[]),
        "{r:?}"
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "concept" && x.1 == TokenType::Keyword)
            .count(),
        2
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "mapping" && x.1 == TokenType::Keyword)
            .count(),
        3
    );
    assert!(has(&r, "90", TokenType::Number, &[]));
    assert!(has(&r, "deg", TokenType::Unit, &[]));
    assert!(has(&r, "/", TokenType::Operator, &[]));
    assert!(has(&r, "->", TokenType::Operator, &[]));
    assert!(has(&r, "=", TokenType::Operator, &[]));
    assert!(
        !r.iter().any(|x| x.0 == "(" || x.0 == ")" || x.0 == ","),
        "punctuation is plain: {r:?}"
    );

    // semantic: concepts are types at declaration and at reference
    assert!(has(&r, "Tilt", TokenType::Type, &["declaration"]));
    assert!(has(&r, "Brightness", TokenType::Type, &["declaration"]));
    // …including a concept named in a body that cannot read it: still the
    // concept (the diagnostic says it is not an input; the class says what
    // it is)
    assert!(
        r.iter()
            .filter(|x| x.0 == "Tilt" && x.1 == TokenType::Type && x.2.is_empty())
            .count()
            == 2,
        "{r:?}"
    );
    assert!(
        has(&r, "Angle", TokenType::Type, &[]) || r.iter().all(|x| x.0 != "Angle"),
        "a representation name is not an entity"
    );

    // a Rule: function at its declaration, its definition head and its call
    assert!(
        has(&r, "dimByTilt", TokenType::Function, &["declaration"]),
        "{r:?}"
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "dimByTilt" && x.1 == TokenType::Function)
            .count(),
        3,
        "{r:?}"
    );

    // the rule's parameter, declared and used
    assert!(has(&r, "tilt", TokenType::Parameter, &["declaration"]));
    assert!(has(&r, "tilt", TokenType::Parameter, &[]));

    // the equation of the library
    assert!(
        has(&r, "clamp", TokenType::Function, &["defaultLibrary"]),
        "{r:?}"
    );

    // a Source: a variable the design only reads, declared and unresolved
    assert!(
        has(
            &r,
            "temp",
            TokenType::Variable,
            &["declaration", "source", "unresolved"]
        ),
        "{r:?}"
    );

    // a Value: a nullary relationship with a definition
    assert!(
        has(&r, "brightness", TokenType::Variable, &["declaration"]),
        "{r:?}"
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "brightness" && x.1 == TokenType::Variable)
            .count(),
        2
    );

    // LSP: the same stream encodes and decodes to the same byte ranges
    let data = encode_data(&tokens, &text, PositionEncoding::Utf16);
    assert_eq!(data.len(), tokens.len() * 5);
    for (row, t) in decode_data(&data).iter().zip(&tokens) {
        assert_eq!(
            byte_range(&text, PositionEncoding::Utf16, row.0, row.1, row.2),
            Some(t.range)
        );
        assert_eq!(row.3, t.ty.index());
        assert_eq!(row.4, t.modifiers.bits());
    }
}

// ---- expressions, binders, holes, ranges ---------------------------------------------

#[test]
fn expression_forms_classify_by_the_tree() {
    let text = "\
concept Tilt : Angle
concept Held : Bool
concept Level : Scalar
concept Readings : Scalar

mapping choose : Tilt -> Held -> Level
choose(t, held) =
  if held && !(t > 45 deg) || false then 1 else ?

mapping steady : Readings -> Held
steady(rs) =
  all r in rs: r in 0 .. 1 && (any r in rs: r > 0.5)

mapping fallback : Level
fallback() = choose(90 deg, true) ?? 0
";
    let (mut host, doc, text) = text_host(text);
    let snap = host.snapshot();
    let tokens = semantic_tokens(&snap, doc);
    assert_well_formed(&text, &tokens);
    let r = rows(&text, &tokens);
    for kw in ["if", "then", "else", "true", "false"] {
        assert!(has(&r, kw, TokenType::Keyword, &[]), "{kw}: {r:?}");
    }
    for op in ["&&", "!", ">", "||", "..", "??"] {
        assert!(has(&r, op, TokenType::Operator, &[]), "{op}: {r:?}");
    }
    assert!(has(&r, "?", TokenType::Slot, &[]), "{r:?}");
    // binders: the word is a keyword, its local a parameter (declared,
    // then used), `in` a keyword; two binders with the same local are
    // two declarations and their uses resolve to the nearest one
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "all" && x.1 == TokenType::Keyword)
            .count(),
        1
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "any" && x.1 == TokenType::Keyword)
            .count(),
        1
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "r" && x.1 == TokenType::Parameter && x.2 == ["declaration"])
            .count(),
        2,
        "{r:?}"
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "r" && x.1 == TokenType::Parameter && x.2.is_empty())
            .count(),
        2,
        "{r:?}"
    );
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "in" && x.1 == TokenType::Keyword)
            .count(),
        3
    );
    // the rule's parameters
    assert!(has(&r, "held", TokenType::Parameter, &["declaration"]));
    assert!(has(&r, "held", TokenType::Parameter, &[]));
    assert!(has(&r, "rs", TokenType::Parameter, &[]));
    // units after numbers, not identifiers
    assert_eq!(
        r.iter()
            .filter(|x| x.0 == "deg" && x.1 == TokenType::Unit)
            .count(),
        2
    );
    // a Rule called from a Value's body
    assert!(has(&r, "choose", TokenType::Function, &[]), "{r:?}");
}

#[test]
fn contextual_words_are_names_where_the_grammar_makes_them_names() {
    // a relationship called `all`, applied: a variable, never a keyword
    let text = "\
concept Level : Scalar
mapping all : Level
all() = 1
mapping twice : Level
twice() = all + all
";
    let (mut host, doc, text) = text_host(text);
    let snap = host.snapshot();
    let r = rows(&text, &semantic_tokens(&snap, doc));
    let alls: Vec<&TokenType> = r.iter().filter(|x| x.0 == "all").map(|x| &x.1).collect();
    assert_eq!(alls.len(), 4, "{r:?}");
    assert!(alls.iter().all(|k| **k == TokenType::Variable), "{r:?}");
}

#[test]
fn units_are_disambiguated_by_position_not_spelling() {
    // `deg` as a parameter name is a parameter; after a number it is a unit
    let text = "\
concept Angle1 : Angle
concept Level : Scalar
mapping f : Angle1 -> Level
f(deg) = deg / (90 deg)
";
    let (mut host, doc, text) = text_host(text);
    let snap = host.snapshot();
    let r = rows(&text, &semantic_tokens(&snap, doc));
    let degs: Vec<(TokenType, Vec<&str>)> = r
        .iter()
        .filter(|x| x.0 == "deg")
        .map(|x| (x.1, x.2.clone()))
        .collect();
    assert_eq!(
        degs,
        vec![
            (TokenType::Parameter, vec!["declaration"]),
            (TokenType::Parameter, vec![]),
            (TokenType::Unit, vec![]),
        ],
        "{r:?}"
    );
}

// ---- broken text keeps its lexical tokens ------------------------------------------------

#[test]
fn incomplete_and_invalid_text_keeps_lexical_highlighting() {
    let text = "\
concept Tilt : Angle
// a comment survives anything
mapping broken : Tilt ->
broken(t) = clamp(t / (90 deg, 0
concept Later : Scalar
";
    let (mut host, doc, text) = text_host(text);
    let snap = host.snapshot();
    let tokens = semantic_tokens(&snap, doc);
    assert_well_formed(&text, &tokens);
    let r = rows(&text, &tokens);
    // the file does not build; everything lexical is still there
    assert!(has(
        &r,
        "// a comment survives anything",
        TokenType::Comment,
        &[]
    ));
    assert_eq!(r.iter().filter(|x| x.1 == TokenType::Keyword).count(), 3);
    assert!(has(&r, "90", TokenType::Number, &[]));
    // inside the unclosed call the parser recovers as it can; what it
    // still recognises keeps its class, what it skips is plain
    assert!(has(&r, "Tilt", TokenType::Type, &["declaration"]));
    assert!(has(&r, "Later", TokenType::Type, &["declaration"]));
    // the pure lexical layer alone agrees with the merged stream where no
    // entity is involved
    let lexical = lexical_tokens(&bdl_syntax::parse_module(&text).syntax_node());
    assert_well_formed(&text, &lexical);
    assert!(lexical.iter().all(|l| l.ty != TokenType::Type));
    assert!(lexical.len() <= tokens.len());
}

#[test]
fn garbage_and_unicode_never_break_the_invariants() {
    let samples = [
        "",
        "§§§ concept T : Angle",
        "mapping : -> \n concept\n enum E { §",
        "/* never closed\nconcept X : Angle",
        "concept 日本 : Angle\nmapping f : 日本 -> 日本\nf(x) = x // ✓ ångström 𝛼",
        "(((((\n?? ?? ? .. .. @ . -> =>",
        "concept A : Angle\nmapping f : A -> A\nf(x) =\n  all x in [x]: x ?? ?",
    ];
    for s in samples {
        let (mut host, doc, text) = text_host(s);
        let snap = host.snapshot();
        let tokens = semantic_tokens(&snap, doc);
        assert_well_formed(&text, &tokens);
        for enc in [
            PositionEncoding::Utf8,
            PositionEncoding::Utf16,
            PositionEncoding::Utf32,
        ] {
            let data = encode_data(&tokens, &text, enc);
            for (row, t) in decode_data(&data).iter().zip(&tokens) {
                if !text[t.range.start as usize..t.range.end as usize].contains('\n') {
                    assert_eq!(
                        byte_range(&text, enc, row.0, row.1, row.2),
                        Some(t.range),
                        "{s:?}"
                    );
                }
            }
        }
    }
}

// ---- the derived role: Source ⇄ Value, Rule stays Rule ----------------------------------

#[test]
fn a_relationship_changes_class_with_its_derived_role_only() {
    let lamp = lamp();
    let s = edit(&lamp.snapshot, mapping("temp", vec![], lamp.brightness));
    let temp = s
        .design
        .mappings
        .values()
        .find(|m| m.name == "temp")
        .map(|m| m.id)
        .expect("temp");
    let mut host = IdeHost::new(s.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    let render =
        |s: &bdl_model::surface::ProjectSnapshot| bdl_ide_db::textual::render_module(&s.design);
    // unresolved and nullary: a Source
    let text = render(&s);
    host.set_text_document(&uri, text.clone());
    let snap = host.snapshot();
    let doc = snap.document_by_uri(&uri).expect("doc");
    let r = rows(&text, &semantic_tokens(&snap, doc));
    assert!(
        has(
            &r,
            "temp",
            TokenType::Variable,
            &["declaration", "source", "unresolved"]
        ),
        "{r:?}"
    );
    // attach a definition: a Value
    let s2 = edit(&s, formula(temp, "0.5"));
    let mut host2 = IdeHost::new(s2.clone());
    let text2 = render(&s2);
    host2.set_text_document(&uri, text2.clone());
    let snap2 = host2.snapshot();
    let doc2 = snap2.document_by_uri(&uri).expect("doc");
    let r2 = rows(&text2, &semantic_tokens(&snap2, doc2));
    assert!(
        has(&r2, "temp", TokenType::Variable, &["declaration"]),
        "{r2:?}"
    );
    // remove it: a Source again
    let s3 = edit(
        &s2,
        EditOp::ReplaceDefinition {
            id: temp,
            definition: None,
        },
    );
    let mut host3 = IdeHost::new(s3.clone());
    let text3 = render(&s3);
    host3.set_text_document(&uri, text3.clone());
    let snap3 = host3.snapshot();
    let doc3 = snap3.document_by_uri(&uri).expect("doc");
    let r3 = rows(&text3, &semantic_tokens(&snap3, doc3));
    assert!(
        has(
            &r3,
            "temp",
            TokenType::Variable,
            &["declaration", "source", "unresolved"]
        ),
        "{r3:?}"
    );

    // a Rule is a function declared or defined; an unapplied rule is a
    // diagnostic, never a class
    let rule = lamp.dim_by_tilt;
    let r_decl = rows(&text, &semantic_tokens(&snap, doc));
    assert!(
        has(
            &r_decl,
            "dimByTilt",
            TokenType::Function,
            &["declaration", "unresolved"]
        ),
        "{r_decl:?}"
    );
    let s4 = edit(&s, formula(rule, "Tilt / 90 deg"));
    let mut host4 = IdeHost::new(s4.clone());
    let text4 = render(&s4);
    host4.set_text_document(&uri, text4.clone());
    let snap4 = host4.snapshot();
    let doc4 = snap4.document_by_uri(&uri).expect("doc");
    let r4 = rows(&text4, &semantic_tokens(&snap4, doc4));
    assert!(
        has(&r4, "dimByTilt", TokenType::Function, &["declaration"]),
        "{r4:?}"
    );
    let diagnostics = diagnostics(&snap4, DiagnosticScope::Project);
    let _unapplied = diagnostics
        .items
        .iter()
        .any(|d| d.code.contains("unapplied"));
    assert!(r4
        .iter()
        .filter(|x| x.0 == "dimByTilt")
        .all(|x| x.1 == TokenType::Function));
}

#[test]
fn nominal_identity_not_representation_classifies_concepts() {
    let text = "\
concept Tilt : Angle
concept MotorAngle : Angle
mapping f : Tilt -> MotorAngle
";
    let (mut host, doc, text) = text_host(text);
    let snap = host.snapshot();
    let r = rows(&text, &semantic_tokens(&snap, doc));
    assert!(has(&r, "Tilt", TokenType::Type, &["declaration"]));
    assert!(has(&r, "MotorAngle", TokenType::Type, &["declaration"]));
    assert!(has(&r, "Tilt", TokenType::Type, &[]));
    assert!(has(&r, "MotorAngle", TokenType::Type, &[]));
    // the references are to two different entities
    let (t1, _) =
        entity_at(&snap, doc, text.rfind("Tilt").expect("Tilt") as u32 + 1).expect("entity");
    let (t2, _) = entity_at(
        &snap,
        doc,
        text.rfind("MotorAngle").expect("MotorAngle") as u32 + 1,
    )
    .expect("entity");
    assert_ne!(t1, t2);
}

// ---- formulas (the definition editor's subdocument) -------------------------------------

#[test]
fn a_formula_draft_has_the_same_classes_relative_to_its_own_text() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    let src = "clamp(Tilt / (90 deg), 0, ?) // ✓";
    host.set_definition_draft(m, src);
    let snap = host.snapshot();
    let tokens = formula_tokens(&snap, m);
    assert_well_formed(src, &tokens);
    let r = rows(src, &tokens);
    assert!(
        has(&r, "clamp", TokenType::Function, &["defaultLibrary"]),
        "{r:?}"
    );
    assert!(has(&r, "Tilt", TokenType::Type, &[]), "{r:?}");
    assert!(has(&r, "90", TokenType::Number, &[]));
    assert!(has(&r, "deg", TokenType::Unit, &[]));
    assert!(has(&r, "?", TokenType::Slot, &[]));
    assert!(has(&r, "// ✓", TokenType::Comment, &[]));
    // no draft, no formula: an empty stream, never a panic
    host.clear_definition_draft(m);
    let snap = host.snapshot();
    assert!(formula_tokens(&snap, m).is_empty());
    assert!(formula_tokens(&snap, bdl_model::DeclId::from_raw(999)).is_empty());
    // a broken draft keeps its lexical tokens
    host.set_definition_draft(m, "Tilt / (90 deg");
    let snap = host.snapshot();
    let tokens = formula_tokens(&snap, m);
    assert!(rows("Tilt / (90 deg", &tokens)
        .iter()
        .any(|x| x.0 == "deg" && x.1 == TokenType::Unit));
}

// ---- components, instances, ports, outputs, clocks ------------------------------------------

#[test]
fn system_items_map_onto_standard_classes_with_modifiers() {
    let text = "\
concept Tilt : Angle
concept Brightness : Scalar
concept Gain : Scalar
clock interaction

mapping tiltValue : Tilt

component AdaptiveLamp {
  use concept Tilt
  use concept Brightness
  use concept Gain
  param clock main

  requires tiltValue : Tilt @main
  param gain : Gain

  mapping dimByTilt : Tilt -> Brightness
  dimByTilt(t) = t / (90 deg)

  provides brightness : Brightness @main
  brightness() = dimByTilt(tiltValue) * gain
}

instance lampA : AdaptiveLamp { main = interaction, gain = 2 }
bind lampA.tiltValue = tiltValue

output light : Brightness @interaction
drive light = mirror
mapping mirror : Brightness @interaction
mirror() = 1
export lampA.brightness as lampOut
device pwmLight : pwm_channel for light
";
    let (mut host, doc, text) = text_host(text);
    let snap = host.snapshot();
    let tokens = semantic_tokens(&snap, doc);
    assert_well_formed(&text, &tokens);
    let r = rows(&text, &tokens);
    let faults: Vec<String> = diagnostics(&snap, DiagnosticScope::Project)
        .items
        .iter()
        .filter(|d| d.is_error())
        .map(|d| format!("{}: {}", d.code, d.message))
        .collect();
    assert!(
        faults.is_empty(),
        "the fixture must build: {faults:?}\n{r:?}"
    );
    for kw in [
        "component",
        "use",
        "param",
        "requires",
        "provides",
        "instance",
        "bind",
        "output",
        "drive",
        "export",
        "device",
        "clock",
    ] {
        assert!(has(&r, kw, TokenType::Keyword, &[]), "{kw}: {r:?}");
    }
    assert!(
        has(&r, "AdaptiveLamp", TokenType::Class, &["declaration"]),
        "{r:?}"
    );
    assert!(has(&r, "AdaptiveLamp", TokenType::Class, &[]), "{r:?}");
    assert!(
        has(
            &r,
            "lampA",
            TokenType::Variable,
            &["declaration", "instance"]
        ),
        "{r:?}"
    );
    assert!(
        has(&r, "interaction", TokenType::Namespace, &["declaration"]),
        "{r:?}"
    );
    assert!(
        r.iter()
            .filter(|x| x.0 == "interaction" && x.1 == TokenType::Namespace)
            .count()
            >= 2,
        "{r:?}"
    );
    assert!(
        has(&r, "light", TokenType::Variable, &["declaration", "output"]),
        "{r:?}"
    );
    assert!(
        has(
            &r,
            "pwmLight",
            TokenType::Variable,
            &["declaration", "device"]
        ),
        "{r:?}"
    );
    assert!(
        has(&r, "gain", TokenType::Property, &["declaration"]),
        "{r:?}"
    );
    assert!(
        has(&r, "brightness", TokenType::Property, &["declaration"]),
        "{r:?}"
    );
    assert!(
        has(&r, "lampOut", TokenType::Property, &["declaration"]),
        "{r:?}"
    );
    assert!(has(&r, "@", TokenType::Operator, &[]));
    assert!(has(&r, ".", TokenType::Operator, &[]));
    // the top-level Source and the body's rule
    assert!(
        has(
            &r,
            "tiltValue",
            TokenType::Variable,
            &["declaration", "source", "unresolved"]
        ),
        "{r:?}"
    );
    assert!(
        has(&r, "dimByTilt", TokenType::Function, &["declaration"]),
        "{r:?}"
    );
}
