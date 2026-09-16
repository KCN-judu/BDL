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
    let text = persist::read_text(&path)?;
    let mut raw: serde_json::Value =
        serde_json::from_str(&text).map_err(|source| PersistError::Json {
            path: path.clone(),
            source,
        })?;
    let found = raw
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(0) as u32;
    persist::schema_supported(&path, found, SYSTEM_SCHEMA_VERSION)?;
    if found < 2 {
        migrate_v1_to_v2(&mut raw);
    }
    let file: SystemFile = serde_json::from_value(raw).map_err(|source| PersistError::Json {
        path: path.clone(),
        source,
    })?;
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

/// Schema 1 → 2: ports had no explicit contract (it was read off the body
/// declaration on every use) and one `stamp`.  The contract is derived
/// from the backing declaration **once, here**, then persisted; from then
/// on it is the port's own.  A port whose declaration is gone gets a
/// contract over nothing, which `realizes` reports.
pub fn migrate_v1_to_v2(raw: &mut serde_json::Value) {
    use serde_json::{json, Value};
    let Some(components) = raw
        .get_mut("system")
        .and_then(|s| s.get_mut("components"))
        .and_then(Value::as_object_mut)
    else {
        raw["schema_version"] = json!(2);
        return;
    };
    for comp in components.values_mut() {
        let clock_params: Vec<Value> = comp
            .pointer("/interface/clock_params")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mappings = comp
            .pointer("/body/mappings")
            .cloned()
            .unwrap_or(Value::Null);
        if let Some(stamp) = comp.get("stamp").cloned() {
            comp["body_stamp"] = stamp.clone();
            comp["interface_stamp"] = stamp;
            comp.as_object_mut().map(|o| o.remove("stamp"));
        }
        let Some(ports) = comp
            .pointer_mut("/interface/ports")
            .and_then(Value::as_object_mut)
        else {
            continue;
        };
        for port in ports.values_mut() {
            if port.get("contract").is_some() {
                continue;
            }
            let decl = port.get("decl").cloned().unwrap_or(Value::Null);
            let m = mappings.get(decl.to_string().trim_matches('"'));
            let signature = m
                .and_then(|m| m.get("signature"))
                .cloned()
                .unwrap_or_else(|| json!({ "inputs": [], "output": 0 }));
            let clock = match m.and_then(|m| m.get("clock")).cloned() {
                Some(c) if !c.is_null() => {
                    if clock_params.contains(&c) {
                        json!({ "kind": "parameter", "clock": c })
                    } else {
                        json!({ "kind": "private", "clock": c })
                    }
                }
                _ => json!({ "kind": "agnostic" }),
            };
            port["contract"] = json!({ "signature": signature, "clock": clock });
        }
    }
    raw["schema_version"] = json!(2);
}
