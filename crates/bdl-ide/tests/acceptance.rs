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
