//! Legacy JSON projects migrate on open, once, keeping every identity and
//! the layout (ADR-0023 §6).

use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::layout::{Layout, Point};
use bdl_model::persist::{self, ProjectKind};
use bdl_model::surface::{ProjectSnapshot, Representation};
use bdl_model::{ConceptId, DeclId, Dim, OutputId, MANIFEST_SCHEMA_VERSION};
use bdl_system::{BehaviorGroup, BehaviorSystem, GroupScope, SystemSnapshot};
use bdl_text::{load_project, migrate_legacy, MigrationReport};
use std::path::Path;

/// A legacy flat project the way Studio wrote one: Tilt/Brightness, an
/// input and a relationship, an output with a display name the text
/// cannot spell, and a layout.
fn legacy_flat(root: &Path) -> (ProjectSnapshot, Layout) {
    let created = persist::init_project(root, "lamp", "legacy").unwrap();
    let mut s = created.snapshot;
    let ops = [
        EditOp::CreateConcept {
            name: "Tilt".into(),
            description: "how far the head is tilted".into(),
            representation: Some(Representation::Quantity { dim: Dim::ANGLE }),
        },
        EditOp::CreateConcept {
            name: "Brightness".into(),
            description: String::new(),
            representation: Some(Representation::Quantity { dim: Dim::ZERO }),
        },
        EditOp::CreateMapping {
            name: "tilt".into(),
            description: String::new(),
            signature: bdl_model::surface::Signature {
                inputs: vec![],
                output: ConceptId::from_raw(0),
            },
            definition: None,
            clock: None,
        },
        EditOp::CreateMapping {
            name: "dimByTilt".into(),
            description: String::new(),
            signature: bdl_model::surface::Signature {
                inputs: vec![ConceptId::from_raw(0)],
                output: ConceptId::from_raw(1),
            },
            definition: None,
            clock: None,
        },
        EditOp::AttachDefinition {
            id: DeclId::from_raw(1),
            definition: bdl_model::surface::Definition::Formula {
                source: "Tilt / 90 deg".into(),
            },
        },
        EditOp::CreateClockDomain {
            name: "interaction".into(),
        },
        EditOp::CreateOutput {
            name: "Light Output".into(),
            description: String::new(),
            accepts: ConceptId::from_raw(1),
            clock: Some(bdl_model::ClockId::from_raw(0)),
        },
    ];
    for op in &ops {
        s = apply_edit(&s, op).unwrap().snapshot;
    }
    let mut layout = Layout::default();
    layout
        .concepts
        .insert(ConceptId::from_raw(0), Point { x: 10.0, y: 20.0 });
    layout
        .concepts
        .insert(ConceptId::from_raw(1), Point { x: 400.0, y: 20.0 });
    layout
        .mappings
        .insert(DeclId::from_raw(1), Point { x: 200.0, y: 40.0 });
    layout
        .outputs
        .insert(OutputId::from_raw(0), Point { x: 600.0, y: 40.0 });
    persist::save_project(root, &s, &layout, "legacy").unwrap();
    (s, layout)
}

#[test]
fn a_legacy_flat_project_migrates_once_and_keeps_ids_and_layout() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let (snapshot, layout) = legacy_flat(root);
    assert_eq!(
        persist::read_manifest(root).unwrap().legacy_kind(),
        Some(ProjectKind::Flat)
    );

    let report = migrate_legacy(root, "0.10").unwrap().expect("migrated");
    assert_eq!(report.from, ProjectKind::Flat);
    assert_eq!(report.source.as_deref(), Some("src/main.bdl"));
    assert_eq!(
        report.renamed,
        vec!["design/project.bdl.json.migrated".to_owned()]
    );
    assert_eq!(
        report.renamed_names,
        vec![("Light Output".to_owned(), "Light_Output".to_owned())]
    );
    assert!(root.join("design/project.bdl.json.migrated").is_file());
    assert!(!root.join("design/project.bdl.json").exists());
    let manifest = persist::read_manifest(root).unwrap();
    assert_eq!(manifest.schema_version, MANIFEST_SCHEMA_VERSION);
    assert_eq!(manifest.legacy_kind(), None);

    // the sources read back as the same design, with the same ids
    let loaded = load_project(root).unwrap();
    assert!(
        loaded.migrated.is_none(),
        "a second open is not a migration"
    );
    let base = &loaded.build.system.base;
    assert_eq!(base.concepts.len(), 2);
    assert_eq!(base.concepts[&ConceptId::from_raw(0)].name, "Tilt");
    assert_eq!(
        base.concepts[&ConceptId::from_raw(0)].description,
        "how far the head is tilted"
    );
    assert_eq!(base.mappings[&DeclId::from_raw(1)].name, "dimByTilt");
    assert_eq!(
        base.mappings[&DeclId::from_raw(1)].definition,
        snapshot.design.mappings[&DeclId::from_raw(1)].definition
    );
    assert_eq!(base.outputs[&OutputId::from_raw(0)].name, "Light_Output");
    assert_eq!(base.ids, snapshot.design.ids, "allocators carry over");
    assert_eq!(loaded.layout, layout, "layout is untouched");
    assert!(loaded.build.faults.iter().all(|f| f.is_open()));

    // the identity sidecar is keyed by name and holds the JSON's ids
    let table = bdl_text::workspace::load_identities(root).unwrap();
    assert_eq!(table.keys["concept:Tilt"].id, 0);
    assert_eq!(table.keys["concept:Brightness"].id, 1);
    assert_eq!(table.keys["mapping:dimByTilt"].id, 1);
    assert_eq!(table.keys["output:Light_Output"].id, 0);
    assert_eq!(table.base_ids, snapshot.design.ids);

    // migrating again is a no-op
    assert_eq!(
        migrate_legacy(root, "0.10").unwrap(),
        None::<MigrationReport>
    );
}

#[test]
fn a_legacy_system_project_migrates_with_its_groups() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let (snapshot, _) = legacy_flat(root);
    // rewrite it as a legacy *system* project with a group
    let mut system = BehaviorSystem::from_flat(snapshot.design.clone());
    let gid = system.ids.fresh_group();
    system.groups.insert(
        gid,
        BehaviorGroup {
            id: gid,
            scope: GroupScope::SystemBase,
            name: "Dimming".into(),
            description: String::new(),
            members: vec![DeclId::from_raw(1)],
        },
    );
    std::fs::remove_file(root.join("design/project.bdl.json")).unwrap();
    bdl_system::persist::save_system_project(
        root,
        &SystemSnapshot::new(system.clone()),
        &Layout::default(),
        "legacy",
    )
    .unwrap();
    assert_eq!(
        persist::read_manifest(root).unwrap().legacy_kind(),
        Some(ProjectKind::System)
    );

    let report = migrate_legacy(root, "0.10").unwrap().expect("migrated");
    assert_eq!(report.from, ProjectKind::System);
    assert_eq!(
        report.renamed,
        vec!["design/system.bdl.json.migrated".to_owned()]
    );
    let loaded = load_project(root).unwrap();
    let group = &loaded.build.system.groups[&gid];
    assert_eq!(group.name, "Dimming");
    assert_eq!(group.members, vec![DeclId::from_raw(1)]);
    assert_eq!(loaded.build.system.base.mappings.len(), 2);
}

#[test]
fn a_manifest_of_the_unified_schema_needs_no_migration_and_an_unspellable_name_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    bdl_text::init_project(root, "fresh", "0.10").unwrap();
    assert_eq!(persist::read_manifest(root).unwrap().legacy_kind(), None);
    assert_eq!(migrate_legacy(root, "0.10").unwrap(), None);
    assert!(bdl_text::is_identifier("Tilt"));
    assert!(bdl_text::why_not_identifier("Light Output").is_some());
}
