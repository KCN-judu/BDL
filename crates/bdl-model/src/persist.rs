//! Crash-safe project persistence.
//!
//! ```text
//! project/
//! ├── bdl.toml                 project manifest (name, schema versions)
//! ├── design/project.bdl.json  semantic content   — owned by the compiler
//! ├── ui/layout.json           canvas positions   — owned by the editor
//! └── components/              supplied components (later)
//! ```
//!
//! Every file carries `schema_version`.  Writes go to a temporary file in the
//! same directory, are flushed, and are atomically renamed over the target,
//! so a crash mid-write leaves the previous file intact.

use crate::ids::Revision;
use crate::layout::Layout;
use crate::surface::{Design, ProjectSnapshot};
use crate::{LAYOUT_SCHEMA_VERSION, PROJECT_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const MANIFEST_FILE: &str = "bdl.toml";
pub const DESIGN_FILE: &str = "design/project.bdl.json";
pub const LAYOUT_FILE: &str = "ui/layout.json";

#[derive(Debug, thiserror::Error)]
pub enum PersistError {
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
    #[error("{path}: {message}")]
    Toml { path: PathBuf, message: String },
    #[error("{path}: schema_version {found} is newer than supported {supported}")]
    UnsupportedSchema {
        path: PathBuf,
        found: u32,
        supported: u32,
    },
    #[error("{path}: not a BDL project (missing {MANIFEST_FILE})")]
    NotAProject { path: PathBuf },
    #[error("{path}: a {kind:?} project, not a flat design (open it as a system)")]
    NotFlat { path: PathBuf, kind: ProjectKind },
}

/// What kind of authored truth a project holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectKind {
    /// `design/project.bdl.json`: a flat design is the authored truth.
    #[default]
    Flat,
    /// `design/system.bdl.json`: a behaviour system is the authored truth;
    /// the flat design is derived (`bdl-system`).
    System,
    /// `src/**/*.bdl`: the source tree is the authored truth (ADR-0020);
    /// identities live in `.bdl/identities.json`, groups in
    /// `.bdl/authoring.json`, and the system and flat design are derived
    /// by the textual loader (`bdl-text`).
    Text,
}

/// `bdl.toml`
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub schema_version: u32,
    pub name: String,
    /// Compiler that last wrote the project (informational).
    #[serde(default)]
    pub compiler_version: String,
    /// Absent in every project written before behaviour systems: flat.
    #[serde(default, skip_serializing_if = "is_flat")]
    pub kind: ProjectKind,
}

fn is_flat(k: &ProjectKind) -> bool {
    *k == ProjectKind::Flat
}

/// Read only the manifest — to learn a project's kind before loading it.
pub fn read_manifest(root: &Path) -> Result<Manifest, PersistError> {
    let manifest_path = root.join(MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Err(PersistError::NotAProject {
            path: root.to_path_buf(),
        });
    }
    let manifest_text = read(&manifest_path)?;
    let manifest: Manifest = toml::from_str(&manifest_text).map_err(|e| PersistError::Toml {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;
    check_schema(
        &manifest_path,
        manifest.schema_version,
        PROJECT_SCHEMA_VERSION,
    )?;
    Ok(manifest)
}

/// Write a manifest.  Exposed for the system layer, which owns the other
/// files of a system project.
pub fn write_manifest(root: &Path, manifest: &Manifest) -> Result<(), PersistError> {
    let manifest_path = root.join(MANIFEST_FILE);
    let text = toml::to_string_pretty(manifest).map_err(|e| PersistError::Toml {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;
    write_atomic(&manifest_path, text.as_bytes())
}

pub fn read_text(path: &Path) -> Result<String, PersistError> {
    read(path)
}

pub fn schema_supported(path: &Path, found: u32, supported: u32) -> Result<(), PersistError> {
    check_schema(path, found, supported)
}

pub fn load_layout(root: &Path) -> Result<Layout, PersistError> {
    let layout_path = root.join(LAYOUT_FILE);
    if layout_path.is_file() {
        let file: LayoutFile = from_json(&layout_path, &read(&layout_path)?)?;
        check_schema(&layout_path, file.schema_version, LAYOUT_SCHEMA_VERSION)?;
        Ok(file.layout)
    } else {
        Ok(Layout::default())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct DesignFile {
    schema_version: u32,
    design: Design,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct LayoutFile {
    schema_version: u32,
    layout: Layout,
}

/// Everything read back from disk.
#[derive(Clone, Debug, PartialEq)]
pub struct LoadedProject {
    pub manifest: Manifest,
    pub snapshot: ProjectSnapshot,
    pub layout: Layout,
}

/// Create a new project directory with an empty design.
pub fn init_project(
    root: &Path,
    name: &str,
    compiler_version: &str,
) -> Result<LoadedProject, PersistError> {
    let snapshot = ProjectSnapshot::new(Design::empty(name));
    let layout = Layout::default();
    save_project(root, &snapshot, &layout, compiler_version)?;
    Ok(LoadedProject {
        manifest: Manifest {
            schema_version: PROJECT_SCHEMA_VERSION,
            name: name.to_owned(),
            compiler_version: compiler_version.to_owned(),
            kind: ProjectKind::Flat,
        },
        snapshot,
        layout,
    })
}

/// Write manifest, design and layout atomically (each file individually).
pub fn save_project(
    root: &Path,
    snapshot: &ProjectSnapshot,
    layout: &Layout,
    compiler_version: &str,
) -> Result<(), PersistError> {
    save_design(root, snapshot, compiler_version)?;
    save_layout(root, layout)?;
    Ok(())
}

pub fn save_design(
    root: &Path,
    snapshot: &ProjectSnapshot,
    compiler_version: &str,
) -> Result<(), PersistError> {
    let manifest = Manifest {
        schema_version: PROJECT_SCHEMA_VERSION,
        name: snapshot.design.name.clone(),
        compiler_version: compiler_version.to_owned(),
        kind: ProjectKind::Flat,
    };
    let manifest_path = root.join(MANIFEST_FILE);
    let manifest_text = toml::to_string_pretty(&manifest).map_err(|e| PersistError::Toml {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;
    let design_path = root.join(DESIGN_FILE);
    let design_text = to_json(
        &design_path,
        &DesignFile {
            schema_version: PROJECT_SCHEMA_VERSION,
            design: snapshot.design.clone(),
        },
    )?;
    // Design first, manifest last: a manifest is the marker that a project
    // exists, so it must never point at a design that failed to write.
    write_atomic(&design_path, design_text.as_bytes())?;
    write_atomic(&manifest_path, manifest_text.as_bytes())?;
    Ok(())
}

pub fn save_layout(root: &Path, layout: &Layout) -> Result<(), PersistError> {
    let path = root.join(LAYOUT_FILE);
    let text = to_json(
        &path,
        &LayoutFile {
            schema_version: LAYOUT_SCHEMA_VERSION,
            layout: layout.clone(),
        },
    )?;
    write_atomic(&path, text.as_bytes())
}

/// Load a project.  A missing layout file is not an error (a design can exist
/// without any canvas); a missing design is.
pub fn load_project(root: &Path) -> Result<LoadedProject, PersistError> {
    let manifest_path = root.join(MANIFEST_FILE);
    if !manifest_path.is_file() {
        return Err(PersistError::NotAProject {
            path: root.to_path_buf(),
        });
    }
    let manifest_text = read(&manifest_path)?;
    let manifest: Manifest = toml::from_str(&manifest_text).map_err(|e| PersistError::Toml {
        path: manifest_path.clone(),
        message: e.to_string(),
    })?;
    check_schema(
        &manifest_path,
        manifest.schema_version,
        PROJECT_SCHEMA_VERSION,
    )?;
    if manifest.kind != ProjectKind::Flat {
        return Err(PersistError::NotFlat {
            path: root.to_path_buf(),
            kind: manifest.kind,
        });
    }

    let design_path = root.join(DESIGN_FILE);
    let design_file: DesignFile = from_json(&design_path, &read(&design_path)?)?;
    check_schema(
        &design_path,
        design_file.schema_version,
        PROJECT_SCHEMA_VERSION,
    )?;

    let layout_path = root.join(LAYOUT_FILE);
    let layout = if layout_path.is_file() {
        let file: LayoutFile = from_json(&layout_path, &read(&layout_path)?)?;
        check_schema(&layout_path, file.schema_version, LAYOUT_SCHEMA_VERSION)?;
        file.layout
    } else {
        Layout::default()
    };

    Ok(LoadedProject {
        manifest,
        snapshot: ProjectSnapshot {
            revision: Revision::INITIAL,
            design: design_file.design,
        },
        layout,
    })
}

fn check_schema(path: &Path, found: u32, supported: u32) -> Result<(), PersistError> {
    // Migration strategy: older versions are migrated forward here (none
    // exist yet); newer versions are refused rather than misread.
    if found > supported {
        Err(PersistError::UnsupportedSchema {
            path: path.to_path_buf(),
            found,
            supported,
        })
    } else {
        Ok(())
    }
}

fn read(path: &Path) -> Result<String, PersistError> {
    fs::read_to_string(path).map_err(|source| PersistError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn to_json<T: Serialize>(path: &Path, value: &T) -> Result<String, PersistError> {
    serde_json::to_string_pretty(value).map_err(|source| PersistError::Json {
        path: path.to_path_buf(),
        source,
    })
}

fn from_json<T: for<'de> Deserialize<'de>>(path: &Path, text: &str) -> Result<T, PersistError> {
    serde_json::from_str(text).map_err(|source| PersistError::Json {
        path: path.to_path_buf(),
        source,
    })
}

/// Write `bytes` to `path` via a sibling temporary file + fsync + rename.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), PersistError> {
    let io = |source: std::io::Error| PersistError::Io {
        path: path.to_path_buf(),
        source,
    };
    let dir = path
        .parent()
        .ok_or_else(|| io(std::io::Error::other("path has no parent directory")))?;
    fs::create_dir_all(dir).map_err(io)?;
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("file");
    let tmp = dir.join(format!(".{file_name}.tmp-{}", std::process::id()));
    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        drop(f);
        fs::rename(&tmp, path)?;
        // Make the rename durable on filesystems that need a directory sync.
        if let Ok(d) = fs::File::open(dir) {
            let _ = d.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result.map_err(io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::{apply_edit, EditOp};
    use crate::surface::Signature;

    #[test]
    fn unresolved_mapping_survives_save_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let loaded = init_project(root, "lamp", "test").unwrap();
        let s = loaded.snapshot;
        let a = apply_edit(
            &s,
            &EditOp::CreateConcept {
                name: "Tilt".into(),
                description: String::new(),
                representation: None,
            },
        )
        .unwrap();
        let tilt = a.outcome.created_concept.unwrap();
        let a = apply_edit(
            &a.snapshot,
            &EditOp::CreateConcept {
                name: "Brightness".into(),
                description: String::new(),
                representation: None,
            },
        )
        .unwrap();
        let bright = a.outcome.created_concept.unwrap();
        let a = apply_edit(
            &a.snapshot,
            &EditOp::CreateMapping {
                name: "dimByTilt".into(),
                description: String::new(),
                signature: Signature {
                    inputs: vec![tilt],
                    output: bright,
                },
            },
        )
        .unwrap();
        let id = a.outcome.created_mapping.unwrap();
        let mut layout = Layout::default();
        layout
            .mappings
            .insert(id, crate::layout::Point { x: 10.0, y: 20.0 });

        save_project(root, &a.snapshot, &layout, "test").unwrap();
        let back = load_project(root).unwrap();
        assert_eq!(back.snapshot.design, a.snapshot.design);
        assert!(back.snapshot.design.mappings[&id].is_unresolved());
        assert_eq!(back.layout, layout);
        assert_eq!(back.manifest.name, "lamp");
        assert!(root.join(MANIFEST_FILE).is_file());
        assert!(root.join(DESIGN_FILE).is_file());
        assert!(root.join(LAYOUT_FILE).is_file());
        // no temp files left behind
        let leftovers: Vec<_> = fs::read_dir(root.join("design"))
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn atomic_write_replaces_whole_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/x.json");
        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "second");
    }

    #[test]
    fn newer_schema_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        init_project(root, "p", "test").unwrap();
        let design_path = root.join(DESIGN_FILE);
        let text = fs::read_to_string(&design_path)
            .unwrap()
            .replace("\"schema_version\": 1", "\"schema_version\": 999");
        write_atomic(&design_path, text.as_bytes()).unwrap();
        let err = load_project(root).unwrap_err();
        assert!(matches!(
            err,
            PersistError::UnsupportedSchema { found: 999, .. }
        ));
    }

    #[test]
    fn missing_manifest_is_not_a_project() {
        let dir = tempfile::tempdir().unwrap();
        assert!(matches!(
            load_project(dir.path()),
            Err(PersistError::NotAProject { .. })
        ));
    }
}
