//! A BDL project on disk (ADR-0023 §1; the mechanisms of ADR-0020).
//!
//! ```text
//! project/
//! ├── bdl.toml                 name, manifest schema 2 — no kind
//! ├── src/**/*.bdl             the canonical semantic source, read in sorted path order
//! ├── .bdl/identities.json     source key → stable id, allocators, flat ids   (tool-owned)
//! ├── .bdl/authoring.json      behavior groups                                 (tool-owned)
//! └── ui/layout.json           canvas layout                                    (presentation)
//! ```
//!
//! A manifest of schema 1 names a legacy JSON project; [`load_project`]
//! migrates it in place first (`migrate`).

use crate::build::{build_system, BuildResult};
use crate::identity::{IdentityTable, IDENTITIES_SCHEMA_VERSION};
use crate::splice::{write_back, WriteBack};
use bdl_model::layout::Layout;
use bdl_model::persist::{self, Manifest, PersistError, ProjectKind};
use bdl_system::{BehaviorGroup, BehaviorGroupId, BehaviorSystem};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub const SOURCE_DIR: &str = "src";
pub const IDENTITIES_FILE: &str = ".bdl/identities.json";
pub const AUTHORING_FILE: &str = ".bdl/authoring.json";
pub const AUTHORING_SCHEMA_VERSION: u32 = 1;

/// One source file: its path relative to the project root, with `/`
/// separators, and its text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFile {
    pub path: String,
    pub text: String,
}

#[derive(Debug, thiserror::Error)]
pub enum TextError {
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error("{path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("{path}: a legacy {kind:?} project could not be migrated: {message}")]
    Migration {
        path: PathBuf,
        kind: ProjectKind,
        message: String,
    },
    #[error(
        "the sources declare something the text cannot mean; {count} fault(s), first: {first}"
    )]
    Faults { count: usize, first: String },
    /// A source rewrite that would not be lossless was refused.
    #[error("{path}: the source rewrite was refused: {message}")]
    Rewrite { path: PathBuf, message: String },
}

/// `.bdl/authoring.json`: what is authored but not semantic — groups, and
/// the unfinished edits a save keeps exactly as the designer left them.
/// Saving never requires a file to build or a formula to check: the
/// project is what was being worked on, the compiler says what it means.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoringFile {
    pub schema_version: u32,
    #[serde(default)]
    pub groups: BTreeMap<BehaviorGroupId, BehaviorGroup>,
    /// Source files whose text on disk does not build, by path: the text
    /// under `src/` is what the designer typed; `last_good` is the last
    /// text of that file that built, which the graph and the identity
    /// table describe until the typed text builds again.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub source_drafts: BTreeMap<String, SourceDraftFile>,
    /// Definition text the designer typed and has not committed, by
    /// relationship (component-local ids inside a component body).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub definition_drafts: Vec<DefinitionDraftFile>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceDraftFile {
    pub last_good: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DefinitionDraftFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<u64>,
    pub mapping: u64,
    pub source: String,
}

/// The unfinished edits a project carries beside its sources.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Drafts {
    /// Path → the text typed for that file (what `src/` holds on disk).
    pub sources: BTreeMap<String, String>,
    pub definitions: Vec<DefinitionDraftFile>,
}

/// A project as loaded: sources, the build, and the sidecars.
#[derive(Clone, Debug)]
pub struct LoadedWorkspace {
    pub root: PathBuf,
    pub manifest: Manifest,
    /// The files the build read: a file whose typed text does not build
    /// carries its last good text here, the typed text in `drafts`.
    pub files: Vec<SourceFile>,
    pub build: BuildResult,
    pub layout: Layout,
    /// The unfinished edits found beside the sources.
    pub drafts: Drafts,
    /// Modification times of every file read, to notice external edits.
    pub stamps: Vec<(String, Option<std::time::SystemTime>)>,
    /// What opening did to a legacy project, when it migrated one.
    pub migrated: Option<crate::migrate::MigrationReport>,
}

impl LoadedWorkspace {
    /// The system with the authoring sidecar's groups merged in.
    pub fn system(&self) -> &BehaviorSystem {
        &self.build.system
    }
}

/// The source files under `src/`, recursively, sorted by relative path.
pub fn discover_sources(root: &Path) -> Result<Vec<SourceFile>, TextError> {
    let dir = root.join(SOURCE_DIR);
    let mut paths = Vec::new();
    if dir.is_dir() {
        walk(&dir, &mut paths)?;
    }
    paths.sort();
    let mut files = Vec::new();
    for p in paths {
        let text = std::fs::read_to_string(&p).map_err(|source| TextError::Io {
            path: p.clone(),
            source,
        })?;
        let rel = p
            .strip_prefix(root)
            .unwrap_or(&p)
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/");
        files.push(SourceFile { path: rel, text });
    }
    Ok(files)
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), TextError> {
    let entries = std::fs::read_dir(dir).map_err(|source| TextError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| TextError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("bdl") {
            out.push(path);
        }
    }
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<Option<T>, TextError> {
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(path).map_err(|source| TextError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|source| TextError::Json {
            path: path.to_path_buf(),
            source,
        })
}

/// Write the authoring sidecar (groups) of a system.
pub fn write_authoring(root: &Path, system: &BehaviorSystem) -> Result<(), TextError> {
    write_authoring_with(root, system, &BTreeMap::new(), &[])
}

/// The authoring sidecar: the system's groups, the last good text of
/// every file whose typed text does not build, the definition drafts.
pub fn write_authoring_with(
    root: &Path,
    system: &BehaviorSystem,
    last_good: &BTreeMap<String, String>,
    definition_drafts: &[DefinitionDraftFile],
) -> Result<(), TextError> {
    let mut definitions = definition_drafts.to_vec();
    definitions.sort();
    write_json(
        &root.join(AUTHORING_FILE),
        &AuthoringFile {
            schema_version: AUTHORING_SCHEMA_VERSION,
            groups: system.groups.clone(),
            source_drafts: last_good
                .iter()
                .map(|(path, text)| {
                    (
                        path.clone(),
                        SourceDraftFile {
                            last_good: text.clone(),
                        },
                    )
                })
                .collect(),
            definition_drafts: definitions,
        },
    )
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), TextError> {
    let text = serde_json::to_string_pretty(value).map_err(|source| TextError::Json {
        path: path.to_path_buf(),
        source,
    })?;
    persist::write_atomic(path, text.as_bytes())?;
    Ok(())
}

fn stamp(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

/// Read the identity table, or an empty one for a project that has none
/// yet.
pub fn load_identities(root: &Path) -> Result<IdentityTable, TextError> {
    let path = root.join(IDENTITIES_FILE);
    match read_json::<IdentityTable>(&path)? {
        Some(t) => {
            persist::schema_supported(&path, t.schema_version, IDENTITIES_SCHEMA_VERSION)?;
            Ok(t)
        }
        None => Ok(IdentityTable::default()),
    }
}

pub fn load_authoring(root: &Path) -> Result<AuthoringFile, TextError> {
    let path = root.join(AUTHORING_FILE);
    match read_json::<AuthoringFile>(&path)? {
        Some(a) => {
            persist::schema_supported(&path, a.schema_version, AUTHORING_SCHEMA_VERSION)?;
            Ok(a)
        }
        None => Ok(AuthoringFile {
            schema_version: AUTHORING_SCHEMA_VERSION,
            ..AuthoringFile::default()
        }),
    }
}

/// Build the sources of a project in memory against `table`, without
/// touching the disk: the loader every tool shares (ADR-0020 §2).
pub fn load_workspace(name: &str, files: &[SourceFile], table: &IdentityTable) -> BuildResult {
    let sources: Vec<(String, String)> = files
        .iter()
        .map(|f| (f.path.clone(), f.text.clone()))
        .collect();
    build_system(name, &sources, table)
}

/// Write the identity table sidecar.
pub fn write_identities(root: &Path, table: &IdentityTable) -> Result<(), TextError> {
    write_json(&root.join(IDENTITIES_FILE), table)
}

/// Open a text project from disk.  Faults in the sources are returned
/// inside the build, not as an error: an unfinished project opens.
///
/// Identities the sources needed and the sidecar did not have (a project
/// written by hand, an item added by another editor) are allocated by the
/// build and written back at once, so the ids an open session shows are
/// the ids every later open — by any tool — shows, whether or not this
/// session saves (ADR-0020 §3).
pub fn load_project(root: &Path) -> Result<LoadedWorkspace, TextError> {
    load_project_with(root, "")
}

/// [`load_project`] naming the compiler that performs a migration, for
/// the manifest it writes.
pub fn load_project_with(
    root: &Path,
    compiler_version: &str,
) -> Result<LoadedWorkspace, TextError> {
    let migrated = crate::migrate::migrate_legacy(root, compiler_version)?;
    let manifest = persist::read_manifest(root)?;
    let on_disk = discover_sources(root)?;
    let table = load_identities(root)?;
    let authoring = load_authoring(root)?;
    // A file saved while it did not build: the typed text stays a draft
    // and the last good text is what the build reads — unless the typed
    // text builds now (fixed in an editor), which ends the draft.
    let mut files = on_disk.clone();
    let mut drafts = Drafts {
        sources: BTreeMap::new(),
        definitions: authoring.definition_drafts.clone(),
    };
    if !authoring.source_drafts.is_empty() {
        let probe = load_workspace(&manifest.name, &on_disk, &table);
        for (i, f) in on_disk.iter().enumerate() {
            let Some(saved) = authoring.source_drafts.get(&f.path) else {
                continue;
            };
            let broken = probe
                .faults
                .iter()
                .any(|fault| fault.file() == i && !fault.is_open());
            if broken {
                drafts.sources.insert(f.path.clone(), f.text.clone());
                files[i].text = saved.last_good.clone();
            }
        }
    }
    let mut build = load_workspace(&manifest.name, &files, &table);
    // Groups are authoring metadata: merged in, never read by the build.
    let known: std::collections::BTreeSet<bdl_model::DeclId> =
        build.system.base.mappings.keys().copied().collect();
    for (id, mut g) in authoring.groups {
        g.members
            .retain(|m| known.contains(m) || g.scope != bdl_system::GroupScope::SystemBase);
        build.system.groups.insert(id, g);
    }
    bdl_system::prune_groups(&mut build.system);
    if build.table != table {
        write_identities(root, &build.table)?;
    }
    let layout = persist::load_layout(root)?;
    let mut stamps: Vec<(String, Option<std::time::SystemTime>)> = files
        .iter()
        .map(|f| (f.path.clone(), stamp(&root.join(&f.path))))
        .collect();
    stamps.push((IDENTITIES_FILE.into(), stamp(&root.join(IDENTITIES_FILE))));
    Ok(LoadedWorkspace {
        root: root.to_path_buf(),
        manifest,
        files,
        build,
        layout,
        drafts,
        stamps,
        migrated,
    })
}

/// The name ADR-0020 gave [`load_project`].
pub fn load_text_project(root: &Path) -> Result<LoadedWorkspace, TextError> {
    load_project(root)
}

/// Files that changed on disk since `loaded` read them (a new source
/// file counts too).
pub fn changed_on_disk(loaded: &LoadedWorkspace) -> Result<Vec<String>, TextError> {
    let mut changed = Vec::new();
    for (path, then) in &loaded.stamps {
        let now = stamp(&loaded.root.join(path));
        if now != *then {
            changed.push(path.clone());
        }
    }
    let current = discover_sources(&loaded.root)?;
    for f in current {
        if !loaded.files.iter().any(|x| x.path == f.path) {
            changed.push(f.path);
        }
    }
    Ok(changed)
}

/// Write a system back as text: the surgical edits of [`write_back`] on the
/// files of `previous`, the identity table that describes the result, the
/// authoring sidecar, the layout and the manifest.  Returns what was
/// written so the caller can reload anchors from it.
pub fn save_project(
    root: &Path,
    previous: &LoadedWorkspace,
    system: &BehaviorSystem,
    layout: &Layout,
    compiler_version: &str,
) -> Result<WriteBack, TextError> {
    save_project_with(
        root,
        previous,
        system,
        layout,
        &Drafts::default(),
        &[],
        compiler_version,
    )
}

/// [`save_project`] with the unfinished edits: a file with a source draft
/// is written as typed (its last good text goes to the authoring sidecar);
/// `rewrite` names files whose text must reach disk even when the splice
/// changed nothing (text accepted from the Code view).  Every file is
/// written before the sidecars, and the manifest last, so an interrupted
/// save never leaves a manifest pointing at a half-written project.
pub fn save_project_with(
    root: &Path,
    previous: &LoadedWorkspace,
    system: &BehaviorSystem,
    layout: &Layout,
    drafts: &Drafts,
    rewrite: &[String],
    compiler_version: &str,
) -> Result<WriteBack, TextError> {
    let wb = write_back(&previous.build, &previous.files, system);
    let mut last_good = BTreeMap::new();
    for (i, f) in wb.files.iter().enumerate() {
        let path = root.join(&f.path);
        if let Some(typed) = drafts.sources.get(&f.path) {
            last_good.insert(f.path.clone(), f.text.clone());
            let on_disk = std::fs::read_to_string(&path).ok();
            if on_disk.as_deref() != Some(typed.as_str()) {
                persist::write_atomic(&path, typed.as_bytes())?;
            }
        } else if wb.changed.contains(&i) || rewrite.contains(&f.path) {
            persist::write_atomic(&path, f.text.as_bytes())?;
        }
    }
    write_json(&root.join(IDENTITIES_FILE), &wb.table)?;
    write_authoring_with(root, system, &last_good, &drafts.definitions)?;
    persist::save_layout(root, layout)?;
    persist::write_manifest(
        root,
        &Manifest::unified(&system.base.name, compiler_version),
    )?;
    Ok(wb)
}

/// The name ADR-0020 gave [`save_project`].
pub fn save_text_project(
    root: &Path,
    previous: &LoadedWorkspace,
    system: &BehaviorSystem,
    layout: &Layout,
    compiler_version: &str,
) -> Result<WriteBack, TextError> {
    save_project(root, previous, system, layout, compiler_version)
}

/// Create an empty project: a manifest, an empty `src/main.bdl`, an
/// empty identity table.
pub fn init_project(
    root: &Path,
    name: &str,
    compiler_version: &str,
) -> Result<LoadedWorkspace, TextError> {
    persist::write_atomic(
        &root.join(SOURCE_DIR).join("main.bdl"),
        format!("// {name}\n").as_bytes(),
    )?;
    write_json(&root.join(IDENTITIES_FILE), &IdentityTable::default())?;
    write_json(
        &root.join(AUTHORING_FILE),
        &AuthoringFile {
            schema_version: AUTHORING_SCHEMA_VERSION,
            ..AuthoringFile::default()
        },
    )?;
    persist::save_layout(root, &Layout::default())?;
    persist::write_manifest(root, &Manifest::unified(name, compiler_version))?;
    load_project(root)
}

/// The name ADR-0020 gave [`init_project`].
pub fn init_text_project(
    root: &Path,
    name: &str,
    compiler_version: &str,
) -> Result<LoadedWorkspace, TextError> {
    init_project(root, name, compiler_version)
}
