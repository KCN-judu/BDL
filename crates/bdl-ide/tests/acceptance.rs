//! The milestone's acceptance scenarios (`docs/architecture/ide-service.md`
//! §Tests): unresolved mappings stay legal and queryable; one entity is
//! the same entity on both surfaces; a semantic diagnostic is made once
//! and projected twice; drafts and unsaved documents are overlays; stale
//! results cannot win; cancellation stops obsolete work; rename is by
//! identity.

mod support;

use bdl_ide::*;
use bdl_ide_db::textual::render_module;
use bdl_ide_db::{CancelScope, DocumentUri, ResultGate};
use bdl_model::edit::{apply_edit, EditKind, EditOp};
use bdl_model::surface::{Definition, Representation};
use bdl_model::Dim;
use std::sync::{Arc, Mutex};
use support::*;

// ---- §52 unresolved mapping ------------------------------------------------

#[test]
fn unresolved_mapping_is_legal_and_fully_queryable() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    host.set_text_document(&uri, render_module(&lamp.snapshot.design));
    let snap = host.snapshot();
    let doc = snap.document_by_uri(&uri).expect("document");
    let m = EntityRef::Mapping(lamp.dim_by_tilt);

    // No error anywhere: not in the project, not in the document.
    let all = diagnostics(&snap, DiagnosticScope::Project);
    assert!(
        all.errors().next().is_none(),
        "an unresolved mapping is not an error: {:?}",
        all.codes()
    );
    let in_doc = diagnostics(&snap, DiagnosticScope::Document(doc));
    assert!(in_doc.errors().next().is_none());
    for d in &in_doc.items {
        let t = project_to_document(&snap, d, doc);
        assert!(t.is_none_or(|t| t.severity != SemanticSeverity::Error));
    }

    // Hover says "declared", references and definition work, rename plans.
    let h = hover(&snap, m).expect("hover");
    assert_eq!(h.status, EntityStatus::Declared);
    assert!(h.status.is_open());
    assert_eq!(
        h.signature.as_deref(),
        Some("mapping dimByTilt : Tilt -> Brightness")
    );
    let refs = references(&snap, EntityRef::Concept(lamp.tilt));
    assert!(refs.references.iter().any(|r| r.referrer == m));
    assert!(!definition_of(&snap, m).is_empty());
    let plan = plan_rename(&snap, m, "dim").expect("rename plans");
    assert!(plan.model_edits().count() == 1);
    assert!(!plan.text_edits(doc).is_empty());
    let e = explain(&snap, m).expect("explain");
    assert!(e.sections.iter().any(|s| s.heading == "Semantics"));
}

// ---- §53 shared entity identity ------------------------------------------------

#[test]
fn the_same_mapping_is_one_entity_on_both_surfaces_and_rename_hits_both() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    let text = render_module(&lamp.snapshot.design);
    host.set_text_document(&uri, text.clone());
    let snap = host.snapshot();
    let doc = snap.document_by_uri(&uri).expect("document");

    // Textual position → entity; visual element → entity: the same ref.
    let at = text.find("dimByTilt").expect("name in text") as u32 + 2;
    let (from_text, role) = entity_at(&snap, doc, at).expect("entity under cursor");
    assert_eq!(role, EntityRole::Name);
    let from_visual = snap
        .projections()
        .iter()
        .find(|a| a.visual_element() == Some(VisualElementRef::MappingNode(lamp.dim_by_tilt)))
        .map(|a| a.entity)
        .expect("visual anchor");
    assert_eq!(from_text, from_visual);
    assert_eq!(from_text, EntityRef::Mapping(lamp.dim_by_tilt));

    // Rename planned from the text side and from the visual side is the
    // same plan: one model operation on decl#N, one text edit set.
    let a = plan_rename(&snap, from_text, "dim").expect("plan");
    let b = plan_rename(&snap, from_visual, "dim").expect("plan");
    assert_eq!(a, b);
    assert!(matches!(
        a.model_edits().next(),
        Some(EditOp::RenameMapping { id, name }) if *id == lamp.dim_by_tilt && name == "dim"
    ));
    let edits = a.text_edits(doc);
    assert_eq!(edits.len(), 1, "the declaration name (no definition yet)");
    assert!(a.invalidation.is_refinement(), "a rename reopens nothing");
}

// ---- §54 diagnostic multi-projection --------------------------------------------

#[test]
fn one_conflict_diagnostic_projects_to_text_and_canvas() {
    let c = contested_output();
    let mut host = IdeHost::new(c.snapshot.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    let text = render_module(&c.snapshot.design);
    host.set_text_document(&uri, text.clone());
    let snap = host.snapshot();
    let doc = snap.document_by_uri(&uri).expect("document");

    let set = diagnostics(&snap, DiagnosticScope::Project);
    let conflicts: Vec<&SemanticDiagnostic> = set
        .items
        .iter()
        .filter(|d| d.code == "output.multiple_drivers")
        .collect();
    assert_eq!(conflicts.len(), 1, "created once, not once per driver");
    let d = conflicts[0];
    assert_eq!(d.primary.role, EntityRole::DriveEdge);
    assert_eq!(d.primary.entity, EntityRef::Mapping(c.level_a));
    let related: Vec<(EntityRef, EntityRole)> =
        d.related.iter().map(|(a, _)| (a.entity, a.role)).collect();
    assert_eq!(
        related,
        vec![
            (EntityRef::Mapping(c.level_b), EntityRole::DriveEdge),
            (EntityRef::Output(c.light), EntityRole::Output),
        ]
    );

    // Text: lands on levelA's declaration (the drive edge has no textual
    // syntax yet), with the other claimant as related information.
    let t = project_to_document(&snap, d, doc).expect("placed in text");
    assert_eq!(
        &text[t.range.start as usize..t.range.end as usize],
        "levelA"
    );
    assert!(t
        .related
        .iter()
        .any(|r| &text[r.range.start as usize..r.range.end as usize] == "levelB"));

    // Canvas: the two edges and the terminal.
    let v = project_to_visual(&snap, d);
    assert_eq!(
        v.highlights,
        vec![
            VisualElementRef::DriveEdge {
                mapping: c.level_a,
                output: c.light
            },
            VisualElementRef::DriveEdge {
                mapping: c.level_b,
                output: c.light
            },
            VisualElementRef::OutputTerminal(c.light),
        ]
    );

    // Actions: detach either claimant, or create an upstream combination.
    let acts = actions_for(&snap, d);
    let titles: Vec<&str> = acts.iter().map(|a| a.title.as_str()).collect();
    assert!(titles.contains(&"Detach `levelA` from `light`"));
    assert!(titles.contains(&"Detach `levelB` from `light`"));
    assert!(titles.iter().any(|t| t.contains("combine")));
    let detach = acts
        .iter()
        .find(|a| a.title.contains("levelA"))
        .expect("detach");
    assert!(detach.is_ready());
    let plan = detach.plan.as_ref().expect("plan");
    assert!(matches!(
        plan.model_edits().next(),
        Some(EditOp::SetMappingDrive { id, output: None }) if *id == c.level_a
    ));
    assert!(plan.invalidation.categories.contains(&Fact::Output));
}

// ---- §55 formula overlay ------------------------------------------------------

#[test]
fn formula_draft_is_an_overlay_until_the_commit_lands() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;

    host.set_definition_draft(m, "Tilt / 90 deg");
    let snap = host.snapshot();
    assert!(snap.committed().design.mappings[&m].is_unresolved());
    assert!(!snap.effective().design.mappings[&m].is_unresolved());
    let v = draft_verdict(&snap, m).expect("verdict");
    assert_eq!(v.status, bdl_compiler::MappingStatus::ClockConsistent);
    assert!(v.parse_ok);
    assert!(v.diagnostics.iter().all(|d| !d.is_error()));

    // Diagnostics and completion see the overlay.
    host.set_definition_draft(m, "Tilt + 1 s");
    let snap = host.snapshot();
    let v = draft_verdict(&snap, m).expect("verdict");
    assert_eq!(v.status, bdl_compiler::MappingStatus::Invalid);
    let dim = v
        .diagnostics
        .iter()
        .find(|d| d.code == "dimension.mismatch")
        .expect("dimension error on the draft");
    let span = dim.primary.source.expect("span in the draft");
    assert_eq!(span.origin, SourceOrigin::Formula { mapping: m });
    let items = completion(
        &snap,
        &CompletionContext::Formula {
            mapping: m,
            offset: 1,
        },
    );
    assert_eq!(items[0].label, "Tilt");
    assert_eq!(items[0].kind, CompletionKind::Input);

    // Committing is one ordinary edit against the committed revision; the
    // overlay clears only once that commit is the committed state.
    let op = EditOp::AttachDefinition {
        id: m,
        definition: Definition::Formula {
            source: "Tilt + 1 s".into(),
        },
    };
    let committed = apply_edit(host.committed(), &op).expect("commit").snapshot;
    assert_eq!(
        host.overlays().len(),
        1,
        "still an overlay before the commit lands"
    );
    let effect = host.set_committed(committed);
    assert_eq!(effect.dropped.len(), 1);
    assert!(
        host.overlays().is_empty(),
        "the draft is now the committed definition"
    );
    let snap = host.snapshot();
    assert!(!snap.is_drafted(m));
    assert!(
        draft_verdict(&snap, m).is_err(),
        "no draft, no draft verdict"
    );
}

// ---- §56 text document overlay ---------------------------------------------------

#[test]
fn unsaved_document_is_an_overlay_and_close_reverts() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let uri = DocumentUri::new("file:///extra.bdl");
    let before = host.snapshot();
    assert_eq!(before.effective().design.concepts.len(), 2);

    let (doc, _) = host.set_text_document(
        &uri,
        "concept Held : Bool\nmapping hold : Held -> Brightness\nhold(held) =\n  if held then 1 else 0\n",
    );
    let snap = host.snapshot();
    assert_eq!(
        snap.committed().design.concepts.len(),
        2,
        "disk model unchanged"
    );
    assert_eq!(snap.effective().design.concepts.len(), 3);
    assert_eq!(snap.effective().design.mappings.len(), 2);
    let syms = document_symbols(&snap, doc);
    let names: Vec<&str> = syms.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["Held", "hold"]);
    let hold = syms[1].entity;
    assert_eq!(
        hover(&snap, hold).map(|h| h.status),
        Some(EntityStatus::ClockConsistent)
    );

    host.close_text_document(&uri);
    let after = host.snapshot();
    assert_eq!(after.effective().design.concepts.len(), 2);
    assert_eq!(after.effective().design, before.effective().design);
    assert!(
        hover(&after, hold).is_none(),
        "the overlay's entity is gone"
    );
    assert!(after.stamp() > before.stamp());
}

// ---- §57 stale results ----------------------------------------------------------

#[test]
fn out_of_order_verdicts_cannot_overwrite_the_newest() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    let mut stamps = Vec::new();
    for text in ["T", "Ti", "Til"] {
        host.set_definition_draft(m, text);
        let snap = host.snapshot();
        stamps.push((text, draft_verdict(&snap, m).expect("verdict").stamp));
    }
    let mut gate = ResultGate::new();
    let mut shown = None;
    for i in [1, 0, 2] {
        let (text, stamp) = stamps[i];
        if gate.offer(stamp) {
            shown = Some(text);
        }
    }
    assert_eq!(shown, Some("Til"));
    assert_eq!(gate.shown(), Some(stamps[2].1));
    assert!(host.is_current(stamps[2].1));
    assert!(!host.is_current(stamps[1].1));
}

// ---- §58 cancellation -----------------------------------------------------------

/// A query that runs "for a long time": it stops at checkpoints the test
/// releases one by one, polling its token at each.
struct SlowQuery {
    token: CancellationToken,
    checkpoints: Arc<Mutex<Vec<bool>>>,
}

impl SlowQuery {
    fn step(&self) -> Result<(), Cancelled> {
        self.token.check()?;
        self.checkpoints.lock().expect("lock").push(true);
        Ok(())
    }
}

#[test]
fn changing_the_overlay_cancels_the_running_query_and_its_result_is_rejected() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let m = lamp.dim_by_tilt;
    host.set_definition_draft(m, "Til");
    let key = OverlayKey::MappingDefinition { mapping: m };
    let (req, token) = host.begin_request(CancelScope::Overlay(key));
    let started_at = host.stamp();
    let q = SlowQuery {
        token,
        checkpoints: Arc::new(Mutex::new(Vec::new())),
    };
    q.step().expect("first checkpoint passes");

    // The designer keeps typing.
    host.set_definition_draft(m, "Tilt");
    assert_eq!(
        host.live_requests(),
        0,
        "the request was retired by the change"
    );
    assert_eq!(
        q.step(),
        Err(Cancelled),
        "the query notices at its next checkpoint"
    );
    assert_eq!(q.checkpoints.lock().expect("lock").len(), 1);

    // Even a query that never polled cannot publish: its stamp is stale.
    let mut gate = ResultGate::new();
    assert!(!gate.offer_current(started_at, host.stamp()));
    host.end_request(req);

    // A fresh request for the new text runs to completion.
    let (_, token) = host.begin_request(CancelScope::Overlay(key));
    let snap = host.snapshot_cancellable(&token).expect("not cancelled");
    assert_eq!(snap.stamp(), host.stamp());
    assert!(gate.offer_current(snap.stamp(), host.stamp()));
}

// ---- §60 rename by identity -----------------------------------------------------

#[test]
fn rename_follows_identity_not_spelling() {
    let lamp = lamp();
    // Two mappings: one reads Tilt in its formula and mentions "Tilt" in a
    // comment; another is named `tiltGuard` and never uses the concept.
    let s = edit(
        &lamp.snapshot,
        formula(lamp.dim_by_tilt, "Tilt / 90 deg // Tilt is the lean angle"),
    );
    let s = edit(&s, mapping("tiltGuard", vec![], lamp.brightness));
    let guard = s
        .design
        .mappings
        .values()
        .find(|m| m.name == "tiltGuard")
        .map(|m| m.id)
        .expect("guard");
    let s = edit(&s, formula(guard, "1"));
    let mut host = IdeHost::new(s.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    let text = render_module(&s.design);
    host.set_text_document(&uri, text.clone());
    let snap = host.snapshot();
    let doc = snap.document_by_uri(&uri).expect("document");

    let plan = plan_rename(&snap, EntityRef::Concept(lamp.tilt), "Lean").expect("plan");
    // Model: the rename itself.  The text document declares both mappings,
    // so their formulas are covered by text edits, not model operations.
    let model: Vec<&EditOp> = plan.model_edits().collect();
    assert_eq!(model.len(), 1);
    assert!(
        matches!(model[0], EditOp::RenameConcept { id, name } if *id == lamp.tilt && name == "Lean")
    );
    let edits = plan.text_edits(doc);
    let replaced: Vec<&str> = edits
        .iter()
        .map(|e| &text[e.range.start as usize..e.range.end as usize])
        .collect();
    // `concept Tilt`, `: Tilt ->`, `dimByTilt(Tilt)` is a parameter (not
    // anchored), and `Tilt / 90 deg` in the body — but never `dimByTilt`,
    // `tiltGuard`, or the comment.
    assert_eq!(replaced, vec!["Tilt", "Tilt", "Tilt"], "{replaced:?}");
    let new_text = TextEdit::apply_all(
        &text,
        &edits.iter().map(|e| (*e).clone()).collect::<Vec<_>>(),
    )
    .expect("apply");
    assert!(new_text.contains("concept Lean : Angle"));
    assert!(new_text.contains("mapping dimByTilt : Lean -> Brightness"));
    assert!(new_text.contains("Lean / 90 deg // Tilt is the lean angle"));
    assert!(new_text.contains("mapping tiltGuard"));

    // Without the document, the committed formula is rewritten through
    // the model instead — still by identity.
    let mut host2 = IdeHost::new(s.clone());
    let snap2 = host2.snapshot();
    let plan2 = plan_rename(&snap2, EntityRef::Concept(lamp.tilt), "Lean").expect("plan");
    let ops: Vec<&EditOp> = plan2.model_edits().collect();
    assert_eq!(ops.len(), 2);
    assert!(matches!(
        ops[1],
        EditOp::ReplaceDefinition { id, definition: Some(Definition::Formula { source }) }
            if *id == lamp.dim_by_tilt && source == "Lean / 90 deg // Tilt is the lean angle"
    ));
    assert_eq!(
        plan_rename(&snap2, EntityRef::Concept(lamp.tilt), "Brightness"),
        Err(RenameError::Duplicate {
            kind: EntityKind::Concept,
            name: "Brightness".into()
        })
    );
}

// ---- invalidation preview ------------------------------------------------------

#[test]
fn invalidation_preview_names_what_reopens_and_what_stays() {
    let lamp = lamp();
    let s = edit(&lamp.snapshot, formula(lamp.dim_by_tilt, "Tilt / 90 deg"));
    let mut host = IdeHost::new(s);
    let snap = host.snapshot();
    let p = preview_change(
        &snap,
        &EditOp::SetConceptRepresentation {
            id: lamp.tilt,
            representation: Some(Representation::Quantity { dim: Dim::ZERO }),
        },
    );
    assert_eq!(p.kind, Some(EditKind::Edit));
    assert!(p.categories.contains(&Fact::Semantic));
    assert!(p.categories.contains(&Fact::Realization));
    assert!(p.categories.contains(&Fact::SimulationResults));
    assert!(p.preserves.contains(&Fact::Identity));
    assert!(p
        .invalidates
        .iter()
        .any(|i| i.entity == EntityRef::Mapping(lamp.dim_by_tilt)));
    assert_eq!(p.status_changes.len(), 1);
    assert_eq!(
        p.status_changes[0].after,
        bdl_compiler::MappingStatus::Invalid
    );

    let r = preview_change(
        &snap,
        &EditOp::RenameConcept {
            id: lamp.tilt,
            name: "Lean".into(),
        },
    );
    // A bare rename is a refinement (names are not identity) — and yet the
    // formula that says `Tilt` would stop resolving (ADR-0013).  The
    // preview reports both, which is why the rename *plan* rewrites the
    // formula rather than leaving the designer to find out.
    assert!(r.is_refinement());
    assert!(r.invalidates.is_empty());
    assert_eq!(r.status_changes.len(), 1);
    assert_eq!(
        r.status_changes[0].after,
        bdl_compiler::MappingStatus::Invalid
    );

    let bad = preview_change(
        &snap,
        &EditOp::RenameConcept {
            id: lamp.tilt,
            name: "Brightness".into(),
        },
    );
    assert!(bad.error.is_some());
}

// ---- completion --------------------------------------------------------------------

#[test]
fn completion_ranks_by_expected_dimension_and_units_after_numbers() {
    let lamp = lamp();
    let s = edit(
        &lamp.snapshot,
        concept("Delay", Some(Representation::Quantity { dim: Dim::TIME })),
    );
    let delay = s
        .design
        .concepts
        .values()
        .find(|c| c.name == "Delay")
        .map(|c| c.id)
        .expect("delay");
    let s = edit(
        &s,
        EditOp::SetMappingSignature {
            id: lamp.dim_by_tilt,
            signature: bdl_model::surface::Signature {
                inputs: vec![delay, lamp.tilt],
                output: lamp.brightness,
            },
        },
    );
    let mut host = IdeHost::new(s);
    host.set_definition_draft(lamp.dim_by_tilt, "90 d");
    let snap = host.snapshot();
    let items = completion(
        &snap,
        &CompletionContext::Formula {
            mapping: lamp.dim_by_tilt,
            offset: 4,
        },
    );
    // After a number only units make sense; `deg` (an angle, dimension
    // of an input) ranks with the other units above the inputs.
    assert!(items
        .iter()
        .any(|i| i.label == "deg" && i.kind == CompletionKind::Unit));
    assert!(items[0].kind == CompletionKind::Unit, "{items:?}");
    host.set_definition_draft(lamp.dim_by_tilt, "");
    let snap = host.snapshot();
    let items = completion(
        &snap,
        &CompletionContext::Formula {
            mapping: lamp.dim_by_tilt,
            offset: 0,
        },
    );
    let inputs: Vec<&str> = items
        .iter()
        .filter(|i| i.kind == CompletionKind::Input)
        .map(|i| i.label.as_str())
        .collect();
    assert_eq!(
        inputs,
        vec!["Delay", "Tilt"],
        "same rank (expected dimensionless), label order"
    );
    assert!(items.iter().all(|i| i.replace == TextRange::new(0, 0)));
}

#[test]
fn equations_of_the_library_complete_by_prefix_in_the_designers_words() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    host.set_definition_draft(lamp.dim_by_tilt, "cl");
    let snap = host.snapshot();
    let items = completion(
        &snap,
        &CompletionContext::Formula {
            mapping: lamp.dim_by_tilt,
            offset: 2,
        },
    );
    let clamp = items
        .iter()
        .find(|i| i.kind == CompletionKind::Equation && i.label == "clamp(x, low, high)")
        .expect("clamp");
    assert_eq!(clamp.insert, "clamp(");
    let doc = clamp.documentation.as_deref().unwrap_or("");
    assert!(doc.contains("held between"), "{doc}");
    assert!(doc.contains("ordered kind"), "{doc}");
    // the designer's own relationship shadows a library name
    let s = edit(
        &lamp.snapshot,
        EditOp::CreateMapping {
            name: "clamp".into(),
            description: String::new(),
            signature: bdl_model::surface::Signature {
                inputs: vec![],
                output: lamp.brightness,
            },
            definition: None,
            clock: None,
        },
    );
    let mut host = IdeHost::new(s);
    host.set_definition_draft(lamp.dim_by_tilt, "cl");
    let items = completion(
        &host.snapshot(),
        &CompletionContext::Formula {
            mapping: lamp.dim_by_tilt,
            offset: 2,
        },
    );
    assert!(items
        .iter()
        .any(|i| i.kind == CompletionKind::Mapping && i.label == "clamp"));
    assert!(!items.iter().any(|i| i.kind == CompletionKind::Equation));
}

// ---- robustness -------------------------------------------------------------------

#[test]
fn queries_never_panic_on_missing_or_malformed_input() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let ghost = EntityRef::Mapping(bdl_model::DeclId::from_raw(4242));
    host.set_definition_draft(bdl_model::DeclId::from_raw(4242), "(((");
    host.set_definition_draft(lamp.dim_by_tilt, "§§ /* never closed");
    let uri = DocumentUri::new("file:///broken.bdl");
    let (doc, _) = host.set_text_document(&uri, "mapping : -> \n concept\n enum E { §");
    let snap = host.snapshot();
    assert!(hover(&snap, ghost).is_none());
    assert!(explain(&snap, ghost).is_none());
    assert!(references(&snap, ghost).references.is_empty());
    assert_eq!(
        plan_rename(&snap, ghost, "x"),
        Err(RenameError::UnknownEntity { entity: ghost })
    );
    assert!(matches!(
        draft_verdict(&snap, bdl_model::DeclId::from_raw(4242)),
        Err(QueryError::UnknownEntity { .. })
    ));
    let _ = diagnostics(&snap, DiagnosticScope::Project);
    let _ = diagnostics(&snap, DiagnosticScope::Document(doc));
    let _ = completion(
        &snap,
        &CompletionContext::Document {
            document: doc,
            offset: 999,
        },
    );
    let _ = completion(
        &snap,
        &CompletionContext::Document {
            document: DocumentId(77),
            offset: 0,
        },
    );
    let _ = semantic_tokens(&snap, doc);
    let _ = document_symbols(&snap, DocumentId(77));
    let _ = entity_at(&snap, doc, 10_000);
    let _ = actions_at(&snap, ghost);
    let v = draft_verdict(&snap, lamp.dim_by_tilt).expect("a malformed draft still gets a verdict");
    assert!(!v.parse_ok);
}

// ---- standard concept library --------------------------------------------------

#[test]
fn every_library_template_is_a_textual_completion_from_the_same_data() {
    let mut host = IdeHost::empty("lamp");
    let uri = DocumentUri::new("file:///new.bdl");
    let (doc, _) = host.set_text_document(&uri, "concept A");
    let snap = host.snapshot();
    let items = completion(
        &snap,
        &CompletionContext::Document {
            document: doc,
            offset: 9,
        },
    );
    // `concept A|`: every template whose name or keyword starts with `a`,
    // as ordinary declarations.
    let lib = bdl_library::Library::standard();
    let offered: Vec<&str> = items.iter().filter_map(|i| i.template.as_deref()).collect();
    for t in lib.templates() {
        let by_name = t.default_name.to_lowercase().starts_with('a')
            || t.display_name.to_lowercase().starts_with('a');
        let by_keyword = t.keywords.iter().any(|k| k.to_lowercase().starts_with('a'));
        assert_eq!(
            offered.contains(&t.id.as_str()),
            by_name || by_keyword,
            "{}",
            t.id
        );
    }
    let light = items
        .iter()
        .find(|i| i.template.as_deref() == Some("std.environment.ambient_light"))
        .expect("ambient light offered");
    assert_eq!(light.kind, CompletionKind::Template);
    assert_eq!(light.insert, "AmbientLight : Illuminance");
    assert_eq!(light.replace, TextRange::new(8, 9));
    assert!(light
        .documentation
        .as_deref()
        .is_some_and(|d| d.contains("Ambient Light")));

    // With no prefix, everything is offered — the whole library is one
    // list on both surfaces.
    let (doc2, _) = host.set_text_document(&uri, "concept ");
    let snap = host.snapshot();
    let all = completion(
        &snap,
        &CompletionContext::Document {
            document: doc2,
            offset: 8,
        },
    );
    let offered: std::collections::BTreeSet<&str> =
        all.iter().filter_map(|i| i.template.as_deref()).collect();
    let expected: std::collections::BTreeSet<&str> =
        lib.templates().iter().map(|t| t.id.as_str()).collect();
    assert_eq!(offered, expected);
}

// ---- P11: binder locals are the formula's own --------------------------------------

/// `all tilt in readings: tilt < Tilt` — the local `tilt` is not the
/// concept `Tilt` (the elaborator resolves names case-insensitively when
/// unique), so a rename of the concept leaves it alone; references never
/// count it; the tokens say parameter for it, keyword for the word and
/// operator for `..`.
#[test]
fn a_binder_local_is_never_a_reference_to_the_design() {
    let lamp = lamp();
    let s = edit(
        &lamp.snapshot,
        formula(
            lamp.dim_by_tilt,
            "if all tilt in [Tilt, Tilt]: tilt in 0 deg .. 90 deg then 1 else 0",
        ),
    );
    let mut host = IdeHost::new(s.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    let text = render_module(&s.design);
    host.set_text_document(&uri, text.clone());
    let snap = host.snapshot();
    let doc = snap.document_by_uri(&uri).expect("document");

    let plan = plan_rename(&snap, EntityRef::Concept(lamp.tilt), "Lean").expect("plan");
    let edits = plan.text_edits(doc);
    let replaced: Vec<&str> = edits
        .iter()
        .map(|e| &text[e.range.start as usize..e.range.end as usize])
        .collect();
    // `concept Tilt`, `: Tilt ->`, and the two `Tilt` in the collection —
    // never the local's declaration or its uses
    assert_eq!(
        replaced,
        vec!["Tilt", "Tilt", "Tilt", "Tilt"],
        "{replaced:?}"
    );
    let new_text = TextEdit::apply_all(
        &text,
        &edits.iter().map(|e| (*e).clone()).collect::<Vec<_>>(),
    )
    .expect("apply");
    assert!(
        new_text.contains("all tilt in [Lean, Lean]: tilt in 0 deg .. 90 deg"),
        "{new_text}"
    );
    // references to the concept: the mapping reads it (its input), and the
    // text sites are the two spellings in the collection, not the local
    let refs = references(&snap, EntityRef::Concept(lamp.tilt));
    let body_sites: Vec<&str> = refs
        .anchors
        .iter()
        .filter_map(|a| a.text_range())
        .map(|r| &text[r.start as usize..r.end as usize])
        .collect();
    assert!(body_sites.iter().all(|s| *s == "Tilt"), "{body_sites:?}");

    let tokens = semantic_tokens(&snap, doc);
    let classes: Vec<(&str, TokenType, bool)> = tokens
        .iter()
        .map(|t| {
            (
                &text[t.range.start as usize..t.range.end as usize],
                t.ty,
                t.modifiers.declaration,
            )
        })
        .collect();
    assert!(
        classes.contains(&("all", TokenType::Keyword, false)),
        "{classes:?}"
    );
    assert!(
        classes.contains(&("tilt", TokenType::Parameter, true)),
        "{classes:?}"
    );
    assert!(
        classes.contains(&("tilt", TokenType::Parameter, false)),
        "{classes:?}"
    );
    assert!(
        classes.contains(&("..", TokenType::Operator, false)),
        "{classes:?}"
    );
    assert!(
        classes
            .iter()
            .filter(|c| c.0 == "tilt" && c.1 == TokenType::Parameter)
            .count()
            == 2,
        "{classes:?}"
    );
    // `in` is a keyword in both places; `Tilt` in the collection stays the concept
    assert!(
        classes
            .iter()
            .filter(|c| c.0 == "in" && c.1 == TokenType::Keyword)
            .count()
            == 2
    );
    assert!(
        classes
            .iter()
            .filter(|c| c.0 == "Tilt" && c.1 == TokenType::Type)
            .count()
            >= 3
    );

    // the word is an ordinary name when it is one: a mapping called `all`
    let s2 = edit(&lamp.snapshot, mapping("all", vec![], lamp.brightness));
    let all_id = s2
        .design
        .mappings
        .values()
        .find(|m| m.name == "all")
        .map(|m| m.id)
        .expect("all");
    let s2 = edit(&s2, formula(all_id, "1"));
    let s2 = edit(&s2, formula(lamp.dim_by_tilt, "all + 1"));
    let mut host2 = IdeHost::new(s2.clone());
    let text2 = render_module(&s2.design);
    host2.set_text_document(&uri, text2.clone());
    let snap2 = host2.snapshot();
    let doc2 = snap2.document_by_uri(&uri).expect("document");
    let tokens2 = semantic_tokens(&snap2, doc2);
    let all_tokens: Vec<TokenType> = tokens2
        .iter()
        .filter(|t| &text2[t.range.start as usize..t.range.end as usize] == "all")
        .map(|t| t.ty)
        .collect();
    assert!(
        all_tokens.iter().all(|k| *k == TokenType::Variable),
        "{all_tokens:?}"
    );
}

// ---- the unit domain: `mapping f : B` is `mapping f : () -> B` --------------------

/// A relationship without inputs has the canonical type `() -> B`: hover
/// and Explain say so in the designer's spelling — `()`, never the word
/// "unit" — while the surface line keeps the shorthand, the Composer offers
/// the relationship as a plain reference (no synthetic argument), and its
/// definition draft accepts `f`, `f()` and `f(())` alike.
#[test]
fn a_relationship_without_inputs_has_the_unit_domain_in_hover_explain_and_the_composer() {
    let lamp = lamp();
    let s = edit(&lamp.snapshot, mapping("ambient", vec![], lamp.tilt));
    let ambient = s
        .design
        .mappings
        .values()
        .find(|m| m.name == "ambient")
        .map(|m| m.id)
        .expect("ambient");
    let mut host = IdeHost::new(s.clone());
    let snap = host.snapshot();
    let h = hover(&snap, EntityRef::Mapping(ambient)).expect("hover");
    assert_eq!(h.signature.as_deref(), Some("mapping ambient : () -> Tilt"));
    let ty = h
        .details
        .iter()
        .find(|d| d.label == "type")
        .map(|d| d.value.as_str());
    assert_eq!(ty, Some("() -> Tilt"));
    assert!(h
        .details
        .iter()
        .all(|d| !d.value.to_lowercase().contains("unit")));
    let h2 = hover(&snap, EntityRef::Mapping(lamp.dim_by_tilt)).expect("hover");
    let ty2 = h2
        .details
        .iter()
        .find(|d| d.label == "type")
        .map(|d| d.value.as_str());
    assert_eq!(ty2, Some("Tilt -> Brightness"));
    let x = explain(&snap, EntityRef::Mapping(ambient)).expect("explain");
    let sem = x
        .sections
        .iter()
        .find(|s| s.heading == "Semantics")
        .expect("semantics");
    let line = |label: &str| {
        sem.lines
            .iter()
            .find(|(l, _)| l == label)
            .map(|(_, v)| v.as_str())
    };
    assert_eq!(line("canonical type"), Some("() -> Tilt"));
    assert!(line("domain").is_some_and(|d| d.contains("empty product")));
    // the kernel interface stays the value's type: `() -> B` encoded as `B`
    assert_eq!(line("interface"), Some(&*format!("{}", lamp.tilt)));
    let x2 = explain(&snap, EntityRef::Mapping(lamp.dim_by_tilt)).expect("explain");
    let sem2 = x2
        .sections
        .iter()
        .find(|s| s.heading == "Semantics")
        .expect("semantics");
    assert!(sem2.lines.iter().all(|(l, _)| l != "domain"));

    // the Composer: `ambient` is a reference candidate as itself
    host.set_definition_draft(lamp.dim_by_tilt, "? / 90 deg");
    let snap = host.snapshot();
    let slot = formula_slot(&snap, lamp.dim_by_tilt, "r.0").expect("slot");
    let amb = slot
        .references
        .iter()
        .find(|r| r.label == "ambient")
        .expect("ambient offered");
    assert_eq!(amb.insert, "ambient");
    // the three spellings of the one application are one draft verdict
    for src in [
        "ambient / 90 deg",
        "ambient() / 90 deg",
        "ambient(()) / 90 deg",
    ] {
        host.set_definition_draft(lamp.dim_by_tilt, src);
        let v = draft_verdict(&host.snapshot(), lamp.dim_by_tilt).expect("verdict");
        assert!(v.parse_ok, "{src}");
        assert!(
            v.diagnostics.iter().all(|d| !d.is_error()),
            "{src}: {:?}",
            v.diagnostics
        );
    }
    host.set_definition_draft(lamp.dim_by_tilt, "ambient(Tilt) / 90 deg");
    let v = draft_verdict(&host.snapshot(), lamp.dim_by_tilt).expect("verdict");
    assert!(
        v.diagnostics
            .iter()
            .any(|d| d.code == "formula.call.arity"
                && d.message.contains("its only argument is `()`"))
    );
}

// ---- the preferred spelling `() -> A` and the legacy shorthand ---------------------

/// `mapping f : () -> A` is the spelling the language prefers: it gets no
/// diagnostic.  The legacy `mapping f : A` still parses to the same
/// declaration and the same canonical type, carries a hint — never an
/// error — with the quick fix *Make empty domain explicit*, one insertion
/// that leaves everything else byte for byte; hover and Explain show the
/// declared spelling beside the canonical type; a bare type anywhere else
/// (a concept's value form) is never flagged; and the rendered projection
/// of the model writes the explicit form.
#[test]
fn the_legacy_output_only_shorthand_is_a_hint_with_a_quick_fix_and_the_explicit_form_is_clean() {
    let lamp = lamp();
    let mut host = IdeHost::new(lamp.snapshot.clone());
    let uri = DocumentUri::new("file:///lamp.bdl");
    let text = "concept Tilt : Angle\nconcept Brightness : Scalar\n\n// the sensor\nmapping tilt : Tilt  // legacy\nmapping level : () -> Brightness\nmapping dimByTilt : Tilt -> Brightness\n";
    let (doc, _) = host.set_text_document(&uri, text);
    let snap = host.snapshot();
    let set = diagnostics(&snap, DiagnosticScope::Document(doc));
    let hints: Vec<&SemanticDiagnostic> = set
        .items
        .iter()
        .filter(|d| d.code == "text.legacy_unit_domain")
        .collect();
    assert_eq!(hints.len(), 1, "{:?}", set.items);
    let h = hints[0];
    assert_eq!(h.severity, SemanticSeverity::Hint);
    assert!(!h.is_error());
    assert_eq!(
        h.message,
        "A relationship with no inputs is written explicitly as `() -> Tilt`. The output-only shorthand is deprecated."
    );
    let range = h.primary.source.expect("placed").range;
    assert_eq!(&text[range.start as usize..range.end as usize], "Tilt");
    let tilt_decl = snap
        .effective()
        .design
        .mappings
        .values()
        .find(|m| m.name == "tilt")
        .expect("tilt bound");
    assert_eq!(h.primary.entity, EntityRef::Mapping(tilt_decl.id));
    // the same canonical type as the explicit form
    let level = snap
        .effective()
        .design
        .mappings
        .values()
        .find(|m| m.name == "level")
        .expect("level");
    assert!(tilt_decl.signature.is_unit_domain() && level.signature.is_unit_domain());
    // the quick fix: one insertion
    let actions = actions_for(&snap, h);
    let fix = actions
        .iter()
        .find(|a| a.title == "Make empty domain explicit")
        .expect("quick fix");
    assert!(fix.is_ready());
    let plan = fix.plan.as_ref().expect("plan");
    let edits: Vec<TextEdit> = plan.text_edits(doc).into_iter().cloned().collect();
    assert_eq!(edits.len(), 1);
    let fixed = TextEdit::apply_all(text, &edits).expect("apply");
    assert_eq!(
        fixed,
        "concept Tilt : Angle\nconcept Brightness : Scalar\n\n// the sensor\nmapping tilt : () -> Tilt  // legacy\nmapping level : () -> Brightness\nmapping dimByTilt : Tilt -> Brightness\n"
    );
    // hover and Explain: the declared spelling beside the canonical type
    let hov = hover(&snap, EntityRef::Mapping(tilt_decl.id)).expect("hover");
    assert_eq!(hov.signature.as_deref(), Some("mapping tilt : () -> Tilt"));
    let detail = |label: &str| {
        hov.details
            .iter()
            .find(|d| d.label == label)
            .map(|d| d.value.as_str())
    };
    assert_eq!(detail("declared spelling"), Some("Tilt"));
    assert_eq!(detail("type"), Some("() -> Tilt"));
    let x = explain(&snap, EntityRef::Mapping(tilt_decl.id)).expect("explain");
    let sem = x
        .sections
        .iter()
        .find(|s| s.heading == "Semantics")
        .expect("semantics");
    let line = |l: &str| {
        sem.lines
            .iter()
            .find(|(k, _)| k == l)
            .map(|(_, v)| v.as_str())
    };
    assert_eq!(line("declared spelling"), Some("Tilt"));
    assert_eq!(line("canonical type"), Some("() -> Tilt"));
    assert!(line("domain").is_some_and(|d| d.contains("omitted domain is the empty product")));
    let hov2 = hover(&snap, EntityRef::Mapping(level.id)).expect("hover");
    assert!(hov2.details.iter().all(|d| d.label != "declared spelling"));
    // after the fix: clean
    let (doc2, _) = host.set_text_document(&uri, fixed.clone());
    let snap2 = host.snapshot();
    let set2 = diagnostics(&snap2, DiagnosticScope::Document(doc2));
    assert!(
        set2.items
            .iter()
            .all(|d| d.code != "text.legacy_unit_domain"),
        "{:?}",
        set2.items
    );
    // the formatter keeps both spellings as authored
    assert_eq!(
        bdl_syntax::format::format_module(text).as_deref(),
        Some(text)
    );
    assert_eq!(
        bdl_syntax::format::format_module(&fixed).as_deref(),
        Some(fixed.as_str())
    );
    // what the model renders is the explicit form
    let rendered = render_module(&snap2.effective().design);
    assert!(rendered.contains("mapping tilt : () -> Tilt"), "{rendered}");
    assert!(!rendered.contains("mapping tilt : Tilt"), "{rendered}");
}
