//! Studio and the LSP share semantics: the same candidate definition,
//! reaching the service as a `MappingDefinitionDraft` overlay (Studio's
//! formula editor) or inside a `TextDocument` overlay (an unsaved `.bdl`
//! buffer), gets the same semantic diagnostics — identity, code, severity,
//! entity — and the same ladder status.  Presentation (spans in a formula
//! vs. ranges in a document) is projected afterwards and not compared.

mod support;

use bdl_compiler::MappingStatus;
use bdl_ide::*;
use bdl_ide_db::textual::render_module;
use bdl_ide_db::DocumentUri;
use bdl_model::edit::EditOp;
use bdl_model::surface::{Definition, Representation};
use bdl_model::{Dim, SemanticId};
use support::*;

/// (code, severity, primary entity, primary role) — what both surfaces
/// must agree on.
fn identity(d: &SemanticDiagnostic) -> (String, SemanticSeverity, EntityRef, EntityRole) {
    (d.code.clone(), d.severity, d.primary.entity, d.primary.role)
}

/// The document a text editor would hold: the committed module with the
/// mapping's definition body replaced by `formula`.
fn module_with(lamp: &Lamp, formula: &str) -> String {
    let mut design = lamp.snapshot.design.clone();
    design
        .mappings
        .get_mut(&lamp.dim_by_tilt)
        .expect("fixture mapping")
        .definition = Some(Definition::Formula {
        source: formula.into(),
    });
    render_module(&design)
}

fn both_surfaces(
    lamp: &Lamp,
    formula: &str,
) -> (DraftVerdict, Vec<SemanticDiagnostic>, MappingStatus) {
    // Studio: the draft overlay.
    let mut studio = IdeHost::new(lamp.snapshot.clone());
    studio.set_definition_draft(lamp.dim_by_tilt, formula);
    let s = studio.snapshot();
    let verdict = draft_verdict(&s, lamp.dim_by_tilt).expect("draft verdict");

    // LSP: the unsaved buffer.
    let mut lsp = IdeHost::new(lamp.snapshot.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    lsp.set_text_document(&uri, module_with(lamp, formula));
    let t = lsp.snapshot();
    let mapping = EntityRef::Mapping(lamp.dim_by_tilt);
    let text_diags: Vec<SemanticDiagnostic> = diagnostics(&t, DiagnosticScope::Project)
        .items
        .into_iter()
        .filter(|d| d.primary.entity == mapping)
        .collect();
    let text_status = t
        .analysis()
        .mappings
        .get(&lamp.dim_by_tilt)
        .expect("mapping in the text world")
        .status;
    (verdict, text_diags, text_status)
}

fn assert_same(lamp: &Lamp, formula: &str, expected: MappingStatus) {
    let (verdict, text_diags, text_status) = both_surfaces(lamp, formula);
    assert_eq!(verdict.status, expected, "{formula}: Studio status");
    assert_eq!(text_status, expected, "{formula}: LSP status");
    let mut a: Vec<_> = verdict.diagnostics.iter().map(identity).collect();
    let mut b: Vec<_> = text_diags.iter().map(identity).collect();
    a.sort();
    b.sort();
    assert_eq!(a, b, "{formula}: the two surfaces disagree");
    // Not vacuous: an invalid draft carries at least one error on both sides.
    if expected == MappingStatus::Invalid {
        assert!(
            verdict.diagnostics.iter().any(|d| d.is_error()),
            "{formula}: Studio"
        );
        assert!(text_diags.iter().any(|d| d.is_error()), "{formula}: LSP");
    }
}

#[test]
fn valid_invalid_and_syntax_broken_drafts_agree_on_both_surfaces() {
    let lamp = lamp();
    assert_same(&lamp, "Tilt / 90 deg", MappingStatus::ClockConsistent);
    assert_same(&lamp, "Tilt + 1 s", MappingStatus::Invalid);
    assert_same(&lamp, "Tilt / 90 deg + 2 m", MappingStatus::Invalid);
    assert_same(&lamp, "Tilt / ", MappingStatus::Invalid);
    assert_same(&lamp, "Tilt * 2 cd", MappingStatus::Invalid);
}

#[test]
fn open_drafts_are_open_on_both_surfaces() {
    // A concept without a representation, read by the mapping.
    let lamp = lamp();
    let s = &lamp.snapshot;
    let a = bdl_model::edit::apply_edit(s, &concept("Warmth", None)).expect("edit");
    let warmth: SemanticId = a.outcome.created_concept.expect("created");
    let s = edit(
        &a.snapshot,
        EditOp::SetMappingSignature {
            id: lamp.dim_by_tilt,
            signature: bdl_model::surface::Signature {
                inputs: vec![warmth],
                output: lamp.brightness,
            },
        },
    );
    let lamp = Lamp {
        snapshot: s,
        ..lamp
    };
    assert_same(&lamp, "Warmth / 2", MappingStatus::Open);
    let (verdict, text_diags, _) = both_surfaces(&lamp, "Warmth / 2");
    assert!(verdict.diagnostics.iter().all(|d| !d.is_error()));
    assert!(text_diags.iter().all(|d| !d.is_error()));
    assert!(verdict
        .diagnostics
        .iter()
        .any(|d| d.code == "semantic.unbound_representation"));
}

#[test]
fn a_representation_change_moves_both_surfaces_the_same_way() {
    let lamp = lamp();
    let retyped = edit(
        &lamp.snapshot,
        EditOp::SetConceptRepresentation {
            id: lamp.tilt,
            representation: Some(Representation::Quantity { dim: Dim::TIME }),
        },
    );
    let lamp = Lamp {
        snapshot: retyped,
        ..lamp
    };
    assert_same(&lamp, "Tilt / 90 deg", MappingStatus::Invalid);
    assert_same(&lamp, "Tilt / 90 s", MappingStatus::ClockConsistent);
}
