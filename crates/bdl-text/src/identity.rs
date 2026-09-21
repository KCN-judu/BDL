//! Stable source identities (ADR-0020 §3–4).
//!
//! Source has names; the model has ids.  The identity table maps a
//! **source key** — kind plus qualified name, never a file or a position —
//! to the id the model uses for it, and carries every allocator so that an
//! id is never reused.  Reconciliation of the keys a load found against
//! the table is deterministic and reported, never guessed:
//!
//! | keys | result |
//! |---|---|
//! | in both | retained |
//! | new only | allocated |
//! | old only | dropped (retired for good) |
//! | one dropped and one allocated of the same kind, shape and file | renamed — the id carries over |
//! | more than one candidate on either side | ambiguous — fresh ids, reported |

use bdl_model::ids::IdAllocator;
use bdl_system::{FlatIds, SystemIdAllocator};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const IDENTITIES_SCHEMA_VERSION: u32 = 1;

/// What kind of entity a key names — decides which allocator mints its
/// id and which other keys it may be a rename of.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyKind {
    Concept,
    Mapping,
    Clock,
    Output,
    Device,
    Component,
    Port,
    Instance,
    Binding,
    Export,
}

impl KeyKind {
    pub fn word(self) -> &'static str {
        match self {
            KeyKind::Concept => "concept",
            KeyKind::Mapping => "mapping",
            KeyKind::Clock => "clock",
            KeyKind::Output => "output",
            KeyKind::Device => "device",
            KeyKind::Component => "component",
            KeyKind::Port => "port",
            KeyKind::Instance => "instance",
            KeyKind::Binding => "binding",
            KeyKind::Export => "export",
        }
    }
}

/// The key of one source item: `concept:Tilt`, `mapping:dimByTilt`,
/// `component:AdaptiveLamp/port:brightness`, `binding:slow<-lampA.brightness`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceKey {
    pub kind: KeyKind,
    /// The component's key for a body item or a port; `None` at the top
    /// level.
    pub scope: Option<String>,
    pub name: String,
}

impl SourceKey {
    pub fn top(kind: KeyKind, name: impl Into<String>) -> SourceKey {
        SourceKey {
            kind,
            scope: None,
            name: name.into(),
        }
    }

    pub fn in_component(component: &str, kind: KeyKind, name: impl Into<String>) -> SourceKey {
        SourceKey {
            kind,
            scope: Some(format!("component:{component}")),
            name: name.into(),
        }
    }

    /// The table's spelling of the key.
    pub fn text(&self) -> String {
        match &self.scope {
            Some(scope) => format!("{scope}/{}:{}", self.kind.word(), self.name),
            None => format!("{}:{}", self.kind.word(), self.name),
        }
    }
}

/// One entry of the table: the id, and the file and shape at the last
/// save (which the rename rule compares).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyEntry {
    pub id: u64,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub file: String,
    /// A rename candidate must have the same shape: a relationship's or
    /// port's arity, empty for every other kind.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub shape: String,
}

/// `.bdl/identities.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentityTable {
    pub schema_version: u32,
    #[serde(default)]
    pub keys: BTreeMap<String, KeyEntry>,
    /// The system's own design allocator (concepts, relationships,
    /// domains, outputs, devices of the top level, and flat ids).
    #[serde(default)]
    pub base_ids: IdAllocator,
    /// Components, instances, ports, bindings, exports, groups.
    #[serde(default)]
    pub system_ids: SystemIdAllocator,
    /// One design allocator per component body, by component key.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub component_ids: BTreeMap<String, IdAllocator>,
    /// The freshening table of instantiated private entities.
    #[serde(default)]
    pub flat_ids: FlatIds,
}

impl Default for IdentityTable {
    fn default() -> Self {
        IdentityTable {
            schema_version: IDENTITIES_SCHEMA_VERSION,
            keys: BTreeMap::new(),
            base_ids: IdAllocator::default(),
            system_ids: SystemIdAllocator::default(),
            component_ids: BTreeMap::new(),
            flat_ids: FlatIds::default(),
        }
    }
}

/// A key as a load found it, with what the rename rule may compare.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundKey {
    pub key: SourceKey,
    pub file: String,
    pub shape: String,
}

/// What reconciliation decided, key by key (table spellings).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reconciliation {
    pub retained: Vec<String>,
    pub allocated: Vec<String>,
    pub dropped: Vec<String>,
    /// `(old key, new key)` pairs whose id was carried over.
    pub renamed: Vec<(String, String)>,
    /// Keys that could have been renames of more than one dropped key (or
    /// several of them of one): fresh ids were minted and the loader
    /// reports it.
    pub ambiguous: Vec<String>,
}

impl IdentityTable {
    /// The id a key has, if any.
    pub fn id_of(&self, key: &SourceKey) -> Option<u64> {
        self.keys.get(&key.text()).map(|e| e.id)
    }

    /// Mint a fresh id for a key's kind and scope.
    pub fn allocate(&mut self, key: &SourceKey) -> u64 {
        match key.kind {
            KeyKind::Component => self.system_ids.fresh_component().raw(),
            KeyKind::Instance => self.system_ids.fresh_instance().raw(),
            KeyKind::Port => self.system_ids.fresh_port().raw(),
            KeyKind::Binding => self.system_ids.fresh_binding().raw(),
            KeyKind::Export => self.system_ids.fresh_export().raw(),
            KeyKind::Concept
            | KeyKind::Mapping
            | KeyKind::Clock
            | KeyKind::Output
            | KeyKind::Device => {
                let alloc = match &key.scope {
                    Some(scope) => self.component_ids.entry(scope.clone()).or_default(),
                    None => &mut self.base_ids,
                };
                let (raw, next) = match key.kind {
                    KeyKind::Concept => {
                        let (id, a) = alloc.fresh_concept();
                        (id.raw(), a)
                    }
                    KeyKind::Mapping => {
                        let (id, a) = alloc.fresh_decl();
                        (id.raw(), a)
                    }
                    KeyKind::Clock => {
                        let (id, a) = alloc.fresh_clock();
                        (id.raw(), a)
                    }
                    KeyKind::Output => {
                        let (id, a) = alloc.fresh_output();
                        (id.raw(), a)
                    }
                    KeyKind::Device => {
                        let (id, a) = alloc.fresh_device();
                        (id.raw(), a)
                    }
                    _ => unreachable!("system kinds handled above"),
                };
                *alloc = next;
                raw
            }
        }
    }

    /// Reconcile the keys a load found against this table (ADR-0020 §4).
    /// Returns the table as it stands after the load — every found key
    /// with an id, dropped keys gone, allocators advanced — and the
    /// report.  Pure: the receiver is not modified.
    pub fn reconcile(&self, found: &[FoundKey]) -> (IdentityTable, Reconciliation) {
        let mut next = IdentityTable {
            schema_version: IDENTITIES_SCHEMA_VERSION,
            keys: BTreeMap::new(),
            base_ids: self.base_ids.clone(),
            system_ids: self.system_ids.clone(),
            component_ids: self.component_ids.clone(),
            flat_ids: self.flat_ids.clone(),
        };
        let mut report = Reconciliation::default();
        let mut pending: Vec<&FoundKey> = Vec::new();
        let mut seen: BTreeMap<String, ()> = BTreeMap::new();

        for f in found {
            let text = f.key.text();
            if seen.insert(text.clone(), ()).is_some() {
                // A duplicate key: the loader has already reported it and
                // binds the first; nothing to decide here.
                continue;
            }
            match self.keys.get(&text) {
                Some(e) => {
                    next.keys.insert(
                        text.clone(),
                        KeyEntry {
                            id: e.id,
                            file: f.file.clone(),
                            shape: f.shape.clone(),
                        },
                    );
                    report.retained.push(text);
                }
                None => pending.push(f),
            }
        }

        // Dropped: old keys nobody found.
        let mut dropped: Vec<(String, KeyEntry)> = self
            .keys
            .iter()
            .filter(|(k, _)| !seen.contains_key(*k))
            .map(|(k, e)| (k.clone(), e.clone()))
            .collect();

        // The rename rule: within one (scope, kind, file, shape) class,
        // exactly one dropped and exactly one pending key.
        let class_of = |scope: &Option<String>, kind: KeyKind, file: &str, shape: &str| {
            format!(
                "{}|{}|{file}|{shape}",
                scope.clone().unwrap_or_default(),
                kind.word()
            )
        };
        let mut dropped_by_class: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, (k, e)) in dropped.iter().enumerate() {
            if let Some(parsed) = parse_key(k) {
                dropped_by_class
                    .entry(class_of(&parsed.scope, parsed.kind, &e.file, &e.shape))
                    .or_default()
                    .push(i);
            }
        }
        let mut pending_by_class: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        for (i, f) in pending.iter().enumerate() {
            pending_by_class
                .entry(class_of(&f.key.scope, f.key.kind, &f.file, &f.shape))
                .or_default()
                .push(i);
        }
        let mut renamed_dropped: Vec<usize> = Vec::new();
        let mut decided: BTreeMap<usize, u64> = BTreeMap::new();
        for (class, p) in &pending_by_class {
            let Some(d) = dropped_by_class.get(class) else {
                continue;
            };
            if d.len() == 1 && p.len() == 1 {
                let (old_key, entry) = &dropped[d[0]];
                decided.insert(p[0], entry.id);
                renamed_dropped.push(d[0]);
                report
                    .renamed
                    .push((old_key.clone(), pending[p[0]].key.text()));
            } else {
                for i in p {
                    report.ambiguous.push(pending[*i].key.text());
                }
            }
        }
        renamed_dropped.sort_unstable();
        renamed_dropped.dedup();
        for i in renamed_dropped.into_iter().rev() {
            dropped.remove(i);
        }

        for (i, f) in pending.iter().enumerate() {
            let text = f.key.text();
            let id = match decided.get(&i) {
                Some(id) => *id,
                None => {
                    report.allocated.push(text.clone());
                    next.allocate(&f.key)
                }
            };
            next.keys.insert(
                text,
                KeyEntry {
                    id,
                    file: f.file.clone(),
                    shape: f.shape.clone(),
                },
            );
        }
        for (k, _) in dropped {
            report.dropped.push(k);
        }
        (next, report)
    }
}

/// Parse the table spelling of a key back into its parts.
pub fn parse_key(text: &str) -> Option<SourceKey> {
    let (scope, rest) = match text.rsplit_once('/') {
        Some((scope, rest)) if scope.starts_with("component:") => (Some(scope.to_owned()), rest),
        _ => (None, text),
    };
    let (kind, name) = rest.split_once(':')?;
    let kind = match kind {
        "concept" => KeyKind::Concept,
        "mapping" => KeyKind::Mapping,
        "clock" => KeyKind::Clock,
        "output" => KeyKind::Output,
        "device" => KeyKind::Device,
        "component" => KeyKind::Component,
        "port" => KeyKind::Port,
        "instance" => KeyKind::Instance,
        "binding" => KeyKind::Binding,
        "export" => KeyKind::Export,
        _ => return None,
    };
    Some(SourceKey {
        kind,
        scope,
        name: name.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(key: SourceKey, file: &str, shape: &str) -> FoundKey {
        FoundKey {
            key,
            file: file.into(),
            shape: shape.into(),
        }
    }

    #[test]
    fn keys_spell_and_parse() {
        let k = SourceKey::in_component("AdaptiveLamp", KeyKind::Port, "brightness");
        assert_eq!(k.text(), "component:AdaptiveLamp/port:brightness");
        assert_eq!(parse_key(&k.text()), Some(k));
        let b = SourceKey::top(KeyKind::Binding, "slow<-lampA.brightness");
        assert_eq!(parse_key(&b.text()), Some(b));
    }

    #[test]
    fn retained_allocated_dropped_and_never_reused() {
        let t = IdentityTable::default();
        let a = found(SourceKey::top(KeyKind::Concept, "Tilt"), "a.bdl", "");
        let b = found(SourceKey::top(KeyKind::Concept, "Brightness"), "a.bdl", "");
        let (t1, r1) = t.reconcile(&[a.clone(), b.clone()]);
        assert_eq!(r1.allocated.len(), 2);
        let tilt = t1.id_of(&a.key).unwrap();
        // Drop Brightness, keep Tilt, add Held: Tilt keeps its id, Held is fresh
        // and Brightness's id is never handed out again.
        let held = found(SourceKey::top(KeyKind::Concept, "Held"), "b.bdl", "");
        let (t2, r2) = t1.reconcile(&[a.clone(), held.clone()]);
        assert_eq!(t2.id_of(&a.key), Some(tilt));
        assert_eq!(r2.retained, vec!["concept:Tilt"]);
        assert_eq!(r2.dropped, vec!["concept:Brightness"]);
        assert_eq!(r2.allocated, vec!["concept:Held"]);
        assert_eq!(t2.id_of(&held.key), Some(2));
        // Recreating Brightness later is a new identity.
        let (t3, _) = t2.reconcile(&[a, held, b.clone()]);
        assert_eq!(t3.id_of(&b.key), Some(3));
    }

    #[test]
    fn a_unique_same_file_swap_is_a_rename_but_two_are_ambiguous() {
        let t = IdentityTable::default();
        let f = found(SourceKey::top(KeyKind::Mapping, "dimByTilt"), "l.bdl", "1");
        let (t1, _) = t.reconcile(std::slice::from_ref(&f));
        let id = t1.id_of(&f.key).unwrap();
        let g = found(SourceKey::top(KeyKind::Mapping, "dimByAngle"), "l.bdl", "1");
        let (t2, r2) = t1.reconcile(std::slice::from_ref(&g));
        assert_eq!(t2.id_of(&g.key), Some(id));
        assert_eq!(
            r2.renamed,
            vec![(
                "mapping:dimByTilt".to_owned(),
                "mapping:dimByAngle".to_owned()
            )]
        );
        assert!(r2.dropped.is_empty());
        // Two new, one gone: nobody can say which is the rename.
        let h = found(SourceKey::top(KeyKind::Mapping, "other"), "l.bdl", "1");
        let (t3, r3) = t2.reconcile(&[f.clone(), h.clone()]);
        assert_eq!(r3.ambiguous.len(), 2);
        assert_ne!(t3.id_of(&f.key), Some(id));
        assert_eq!(r3.dropped, vec!["mapping:dimByAngle"]);
        // A different file or arity is never a rename.
        let elsewhere = found(SourceKey::top(KeyKind::Mapping, "moved"), "m.bdl", "1");
        let (_, r4) = t2.reconcile(&[elsewhere]);
        assert!(r4.renamed.is_empty());
        assert_eq!(r4.dropped, vec!["mapping:dimByAngle"]);
    }

    #[test]
    fn component_bodies_have_their_own_allocators() {
        let t = IdentityTable::default();
        let top = found(SourceKey::top(KeyKind::Concept, "Tilt"), "a.bdl", "");
        let local = found(
            SourceKey::in_component("Lamp", KeyKind::Concept, "Tilt"),
            "a.bdl",
            "",
        );
        let (t1, _) = t.reconcile(&[top.clone(), local.clone()]);
        assert_eq!(t1.id_of(&top.key), Some(0));
        assert_eq!(t1.id_of(&local.key), Some(0));
        assert!(t1.component_ids.contains_key("component:Lamp"));
    }
}
