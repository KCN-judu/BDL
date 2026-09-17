//! A text project on disk (ADR-0020 §1–2, §7–8).
//!
//! ```text
//! project/
//! ├── bdl.toml                 kind = "text"
//! ├── src/**/*.bdl             the canonical semantic source, read in sorted path order
//! ├── .bdl/identities.json     source key → stable id, allocators, flat ids   (tool-owned)
//! ├── .bdl/authoring.json      behavior groups                                 (tool-owned)
//! └── ui/layout.json           canvas layout                                    (as before)
//! ```

use crate::build::{build_system, BuildResult};
use crate::identity::{IdentityTable, IDENTITIES_SCHEMA_VERSION};
use crate::splice::{write_back, WriteBack};
use bdl_model::layout::Layout;
use bdl_model::persist::{self, Manifest, PersistError, ProjectKind};
use bdl_model::PROJECT_SCHEMA_VERSION;
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
    #[error("{path}: a {kind:?} project, not a text project")]
    NotText { path: PathBuf, kind: ProjectKind },
    #[error(
        "the sources declare something the text cannot mean; {count} fault(s), first: {first}"
    )]
    Faults { count: usize, first: String },
}

/// `.bdl/authoring.json`: what is authored but not semantic.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthoringFile {
    pub schema_version: u32,
    #[serde(default)]
    pub groups: BTreeMap<BehaviorGroupId, BehaviorGroup>,
}

/// A text project as loaded: sources, the build, and the sidecars.
#[derive(Clone, Debug)]
pub struct LoadedWorkspace {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub files: Vec<SourceFile>,
    pub build: BuildResult,
    pub layout: Layout,
    /// Modification times of every file read, to notice external edits.
    pub stamps: Vec<(String, Option<std::time::SystemTime>)>,
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
            groups: BTreeMap::new(),
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

/// Open a text project from disk.  Faults in the sources are returned
/// inside the build, not as an error: an unfinished project opens.
pub fn load_text_project(root: &Path) -> Result<LoadedWorkspace, TextError> {
    let manifest = persist::read_manifest(root)?;
    if manifest.kind != ProjectKind::Text {
        return Err(TextError::NotText {
            path: root.to_path_buf(),
            kind: manifest.kind,
        });
    }
    let files = discover_sources(root)?;
    let table = load_identities(root)?;
    let authoring = load_authoring(root)?;
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
        stamps,
    })
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
pub fn save_text_project(
    root: &Path,
    previous: &LoadedWorkspace,
    system: &BehaviorSystem,
    layout: &Layout,
    compiler_version: &str,
) -> Result<WriteBack, TextError> {
    let wb = write_back(&previous.build, &previous.files, system);
    for i in &wb.changed {
        let f = &wb.files[*i];
        persist::write_atomic(&root.join(&f.path), f.text.as_bytes())?;
    }
    write_json(&root.join(IDENTITIES_FILE), &wb.table)?;
    write_json(
        &root.join(AUTHORING_FILE),
        &AuthoringFile {
            schema_version: AUTHORING_SCHEMA_VERSION,
            groups: system.groups.clone(),
        },
    )?;
    persist::save_layout(root, layout)?;
    persist::write_manifest(
        root,
        &Manifest {
            schema_version: PROJECT_SCHEMA_VERSION,
            name: system.base.name.clone(),
            compiler_version: compiler_version.to_owned(),
            kind: ProjectKind::Text,
        },
    )?;
    Ok(wb)
}

/// Create an empty text project: a manifest, an empty `src/main.bdl`, an
/// empty identity table.
pub fn init_text_project(
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
            groups: BTreeMap::new(),
        },
    )?;
    persist::save_layout(root, &Layout::default())?;
    persist::write_manifest(
        root,
        &Manifest {
            schema_version: PROJECT_SCHEMA_VERSION,
            name: name.to_owned(),
            compiler_version: compiler_version.to_owned(),
            kind: ProjectKind::Text,
        },
    )?;
    load_text_project(root)
}
