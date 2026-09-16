//! Persistence of a system project (docs/PROJECT_FORMAT.md, docs/
//! BEHAVIOR_SYSTEM_ARCHITECTURE.md §9):
//!
//! ```text
//! project/
//! ├── bdl.toml                 kind = "system"
//! ├── design/system.bdl.json   the authored system — the ONLY truth
//! └── ui/layout.json           canvas positions (flat ids), not semantics
//! ```
//!
//! The flattened design is derived on open and on every commit and is
//! never written: two files claiming to be the truth of one project is
//! exactly what is forbidden.  A flat project keeps `design/project.bdl.json`
//! and is untouched by this module.

use crate::model::{BehaviorSystem, SystemSnapshot, SYSTEM_SCHEMA_VERSION};
use bdl_model::layout::Layout;
use bdl_model::persist::{self, Manifest, PersistError, ProjectKind, MANIFEST_FILE};
use bdl_model::PROJECT_SCHEMA_VERSION;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SYSTEM_FILE: &str = "design/system.bdl.json";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct SystemFile {
    schema_version: u32,
    system: BehaviorSystem,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LoadedSystem {
    pub manifest: Manifest,
    pub snapshot: SystemSnapshot,
    pub layout: Layout,
}

/// Create a new system project with an empty base design.
pub fn init_system_project(
    root: &Path,
    name: &str,
    compiler_version: &str,
) -> Result<LoadedSystem, PersistError> {
    let snapshot = SystemSnapshot::new(BehaviorSystem::empty(name));
    let layout = Layout::default();
    save_system_project(root, &snapshot, &layout, compiler_version)?;
    Ok(LoadedSystem {
        manifest: manifest_for(&snapshot, compiler_version),
        snapshot,
        layout,
    })
}

fn manifest_for(snapshot: &SystemSnapshot, compiler_version: &str) -> Manifest {
    Manifest {
        schema_version: PROJECT_SCHEMA_VERSION,
        name: snapshot.system.base.name.clone(),
        compiler_version: compiler_version.to_owned(),
        kind: ProjectKind::System,
    }
}

/// Write the system (first) and the manifest (last), each atomically, then
/// the layout.
pub fn save_system_project(
    root: &Path,
    snapshot: &SystemSnapshot,
    layout: &Layout,
    compiler_version: &str,
) -> Result<(), PersistError> {
    let path = root.join(SYSTEM_FILE);
    let text = serde_json::to_string_pretty(&SystemFile {
        schema_version: SYSTEM_SCHEMA_VERSION,
        system: snapshot.system.clone(),
    })
    .map_err(|source| PersistError::Json {
        path: path.clone(),
        source,
    })?;
    persist::write_atomic(&path, text.as_bytes())?;
    persist::write_manifest(root, &manifest_for(snapshot, compiler_version))?;
    persist::save_layout(root, layout)?;
    Ok(())
}

/// Load a system project.  A flat project is refused here (open it as a
/// flat project); a newer schema is refused.
pub fn load_system_project(root: &Path) -> Result<LoadedSystem, PersistError> {
    let manifest = persist::read_manifest(root)?;
    if manifest.kind != ProjectKind::System {
        return Err(PersistError::Json {
            path: root.join(MANIFEST_FILE),
            source: serde::de::Error::custom("not a system project (kind = flat)"),
        });
    }
    let path = root.join(SYSTEM_FILE);
    let file: SystemFile =
        serde_json::from_str(&persist::read_text(&path)?).map_err(|source| PersistError::Json {
            path: path.clone(),
            source,
        })?;
    persist::schema_supported(&path, file.schema_version, SYSTEM_SCHEMA_VERSION)?;
    let layout = persist::load_layout(root)?;
    Ok(LoadedSystem {
        manifest,
        snapshot: SystemSnapshot {
            revision: bdl_model::Revision::INITIAL,
            system: file.system,
        },
        layout,
    })
}
