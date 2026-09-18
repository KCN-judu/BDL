//! Saving keeps the whole authoring state: a source file is written as
//! typed even when it does not build (its last good text rides along so
//! the graph and the identities survive), and definition drafts are kept
//! by identity.  Reopening returns exactly that state; a file fixed in an
//! editor ends its draft.

#![allow(clippy::unwrap_used)]

use bdl_model::{DeclId, SemanticId};
use bdl_text::{
    init_project, load_project, save_project_with, DefinitionDraftFile, Drafts, LoadedWorkspace,
};
use std::collections::BTreeMap;
use std::path::Path;

const GOOD: &str = "concept Tilt : Angle\nconcept Brightness : Scalar\n\n/// dims\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(Tilt) =\n  Tilt / 90 deg\n";
const BROKEN: &str = "concept Tilt : Angle\nconcept Brightness : Scalar\n\n/// dims\nmapping dimByTilt : Tilt ->\ndimByTilt(Tilt) =\n  Tilt / 90 deg\n";

fn good_project(root: &Path) -> LoadedWorkspace {
    init_project(root, "lamp", "test").unwrap();
    std::fs::write(root.join("src/main.bdl"), GOOD).unwrap();
    load_project(root).unwrap()
}

#[test]
fn unbuilt_text_is_saved_as_typed_and_the_graph_keeps_its_identities() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let loaded = good_project(root);
    let dim = *loaded.build.system.base.mappings.keys().next().unwrap();
    assert_eq!(loaded.build.system.base.mappings.len(), 1);

    // the designer broke the file in the Code view and saved
    let drafts = Drafts {
        sources: BTreeMap::from([("src/main.bdl".to_owned(), BROKEN.to_owned())]),
        definitions: vec![DefinitionDraftFile {
            component: None,
            mapping: dim.raw(),
            source: "Tilt / 45".into(),
        }],
    };
    save_project_with(
        root,
        &loaded,
        &loaded.build.system,
        &loaded.layout,
        &drafts,
        &[],
        "test",
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("src/main.bdl")).unwrap(),
        BROKEN,
        "src/ is what was typed"
    );
    let sidecar = std::fs::read_to_string(root.join(".bdl/authoring.json")).unwrap();
    assert!(sidecar.contains("last_good"), "{sidecar}");

    // reopen: the same graph, the same ids, the typed text as the draft
    let again = load_project(root).unwrap();
    assert_eq!(again.build.system.base.mappings.len(), 1);
    assert!(again.build.system.base.mappings.contains_key(&dim));
    assert_eq!(
        again.build.system.base.concepts[&SemanticId::from_raw(0)].name,
        "Tilt"
    );
    assert_eq!(again.drafts.sources["src/main.bdl"], BROKEN);
    assert_eq!(
        again.files[0].text, GOOD,
        "the build read the last good text"
    );
    assert_eq!(again.drafts.definitions, drafts.definitions);
    assert!(again.build.faults.iter().all(|f| f.is_open()));

    // saving again without the draft: the typed text is gone, last good is
    // what is written (Studio's "revert" is an explicit act, never a save)
    save_project_with(
        root,
        &again,
        &again.build.system,
        &again.layout,
        &Drafts::default(),
        &["src/main.bdl".to_owned()],
        "test",
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(root.join("src/main.bdl")).unwrap(),
        GOOD
    );
    let clean = load_project(root).unwrap();
    assert!(clean.drafts.sources.is_empty());
    assert!(clean.drafts.definitions.is_empty());
}

#[test]
fn a_draft_fixed_in_an_editor_ends_when_the_project_opens() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    let loaded = good_project(root);
    let dim = DeclId::from_raw(0);
    let drafts = Drafts {
        sources: BTreeMap::from([("src/main.bdl".to_owned(), BROKEN.to_owned())]),
        definitions: vec![],
    };
    save_project_with(
        root,
        &loaded,
        &loaded.build.system,
        &loaded.layout,
        &drafts,
        &[],
        "test",
    )
    .unwrap();
    // an external editor repairs the file, with a new name for the sink
    let fixed = GOOD;
    std::fs::write(root.join("src/main.bdl"), fixed).unwrap();
    let again = load_project(root).unwrap();
    assert!(
        again.drafts.sources.is_empty(),
        "the typed text builds: no draft"
    );
    assert!(again.build.system.base.mappings.contains_key(&dim));
    assert_eq!(again.files[0].text, fixed);
}
