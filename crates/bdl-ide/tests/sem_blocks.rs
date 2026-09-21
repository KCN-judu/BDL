//! The concept ladder on the IDE service (ADR-0044): a mapping block's
//! references are exactly the Sem blocks and rules its definition names
//! (FV `Reads` = `DependsOn`, `reads_iff_dependsOn`); a read edge is taken
//! away by a text edit (`ComposeOp::Unreference`), never a signature edit;
//! two Sem blocks of one concept raise nothing.

mod support;

use bdl_ide::*;
use bdl_model::surface::{ProjectSnapshot, Representation};
use bdl_model::DeclId;
use std::collections::BTreeSet;
use support::*;

struct Picture {
    snapshot: ProjectSnapshot,
    pressed: DeclId,
    pressed_b: DeclId,
    lit_rule: DeclId,
    lit: DeclId,
    lit_b: DeclId,
}

/// FV `lamp_picture` and `rule_template`: two Source Sem blocks of
/// `Pressed`, one rule `lit : Pressed -> Lit`, two Sem blocks of `Lit`
/// each produced by one mapping block applying the rule.
fn picture() -> Picture {
    let s = ProjectSnapshot::new(bdl_model::surface::Design::empty("lamp"));
    let s = edit(&s, concept("Pressed", Some(Representation::Boolean)));
    let s = edit(&s, concept("Lit", Some(Representation::Boolean)));
    let pressed_c = s
        .design
        .concepts
        .values()
        .find(|c| c.name == "Pressed")
        .expect("c")
        .id;
    let lit_c = s
        .design
        .concepts
        .values()
        .find(|c| c.name == "Lit")
        .expect("c")
        .id;
    let s = edit(&s, mapping("pressed", vec![], pressed_c));
    let s = edit(&s, mapping("pressedB", vec![], pressed_c));
    let s = edit(&s, mapping("litRule", vec![pressed_c], lit_c));
    let s = edit(&s, mapping("lit", vec![], lit_c));
    let s = edit(&s, mapping("litB", vec![], lit_c));
    let id = |s: &ProjectSnapshot, n: &str| {
        s.design
            .mappings
            .values()
            .find(|m| m.name == n)
            .expect("m")
            .id
    };
    let (pressed, pressed_b, lit_rule, lit, lit_b) = (
        id(&s, "pressed"),
        id(&s, "pressedB"),
        id(&s, "litRule"),
        id(&s, "lit"),
        id(&s, "litB"),
    );
    let s = edit(&s, formula(lit_rule, "Pressed"));
    let s = edit(&s, formula(lit, "litRule(pressed)"));
    let s = edit(&s, formula(lit_b, "litRule(pressedB)"));
    Picture {
        snapshot: s,
        pressed,
        pressed_b,
        lit_rule,
        lit,
        lit_b,
    }
}

#[test]
fn a_mapping_blocks_references_are_the_sem_blocks_and_rules_it_names() {
    let p = picture();
    let mut host = IdeHost::new(p.snapshot.clone());
    let snap = host.snapshot();
    let deps = &snap.analysis().dependencies.all;
    let reads = |d: DeclId| -> BTreeSet<DeclId> { deps.get(&d).cloned().unwrap_or_default() };
    // `Lit := litRule(pressed)` reads `pressed` and the rule, nothing else
    assert_eq!(reads(p.lit), BTreeSet::from([p.pressed, p.lit_rule]));
    assert_eq!(reads(p.lit_b), BTreeSet::from([p.pressed_b, p.lit_rule]));
    // a Source reads nothing; the rule's body names its parameter, no
    // declaration
    assert!(reads(p.pressed).is_empty());
    assert!(reads(p.lit_rule).is_empty());
    // two Sem blocks of one concept, a rule applied twice: nothing to say
    let codes: Vec<String> = snap
        .analysis()
        .mappings
        .values()
        .flat_map(|m| m.diagnostics.iter().map(|d| d.code.as_str().to_owned()))
        .collect();
    assert!(codes.is_empty(), "{codes:?}");
}

#[test]
fn a_read_edge_is_taken_away_by_a_text_edit_that_leaves_a_slot() {
    let p = picture();
    let mut host = IdeHost::new(p.snapshot.clone());
    let r = compose(
        &host.snapshot(),
        p.lit,
        "litRule(pressed)",
        &ComposeOp::Unreference { decl: p.pressed },
    )
    .expect("compose");
    assert_eq!(r.source, "litRule(?)");
    assert_eq!(r.select.as_deref(), Some("r.0"));
    // every occurrence goes; the edits are in source order
    let r = compose(
        &host.snapshot(),
        p.lit,
        "if pressed then pressed else litRule(pressed)",
        &ComposeOp::Unreference { decl: p.pressed },
    )
    .expect("compose");
    assert_eq!(r.source, "if ? then ? else litRule(?)");
    assert_eq!(r.edits.len(), 3);
    assert!(r
        .edits
        .windows(2)
        .all(|w| w[0].range.start < w[1].range.start));
    // a declaration the formula does not name: refused, the text untouched
    assert!(compose(
        &host.snapshot(),
        p.lit,
        "litRule(pressed)",
        &ComposeOp::Unreference { decl: p.pressed_b },
    )
    .is_err());
    // the rule is named by the call, not read through an edge: no edge,
    // nothing to unreference — refused, and the signature never changes
    assert!(compose(
        &host.snapshot(),
        p.lit,
        "litRule(pressed)",
        &ComposeOp::Unreference { decl: p.lit_rule },
    )
    .is_err());
    let snap = host.snapshot();
    let block = &snap.effective().design.mappings[&p.lit];
    assert!(
        block.signature.is_unit_domain(),
        "a Sem block never has inputs"
    );
}

/// A Sem block dropped on a mapping block fills the definition's first
/// slot with its name; a definition without a slot refuses the drop.
#[test]
fn a_sem_block_dropped_on_a_mapping_block_fills_its_first_slot() {
    let p = picture();
    let mut host = IdeHost::new(p.snapshot.clone());
    let r = compose(
        &host.snapshot(),
        p.lit,
        "litRule(?)",
        &ComposeOp::Read { decl: p.pressed_b },
    )
    .expect("compose");
    assert_eq!(r.source, "litRule(pressedB)");
    let r = compose(
        &host.snapshot(),
        p.lit,
        "",
        &ComposeOp::Read { decl: p.pressed },
    )
    .expect("compose");
    assert_eq!(r.source, "pressed");
    assert!(compose(
        &host.snapshot(),
        p.lit,
        "litRule(pressed)",
        &ComposeOp::Read { decl: p.pressed_b },
    )
    .is_err());
}

/// A new Sem block of a concept already carried by others changes no
/// verdict (FV `new_sem_transparent`).
#[test]
fn creating_another_sem_block_of_a_concept_changes_nothing() {
    let p = picture();
    let mut before = IdeHost::new(p.snapshot.clone());
    let lit_c = p.snapshot.design.mappings[&p.lit].signature.output;
    let s = edit(&p.snapshot, mapping("litC", vec![], lit_c));
    let mut after = IdeHost::new(s);
    let (b, a) = (before.snapshot(), after.snapshot());
    for id in [p.lit, p.lit_b, p.pressed, p.lit_rule] {
        assert_eq!(
            b.analysis().mappings[&id].status,
            a.analysis().mappings[&id].status
        );
    }
}
