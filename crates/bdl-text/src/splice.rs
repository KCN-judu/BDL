//! Writing a changed model back as text (ADR-0020 §5).
//!
//! The previous load knows where every entity was declared (its item
//! anchor).  Given the system as it now stands, write-back computes, per
//! file, the smallest set of item-level edits that makes the text declare
//! the new model: a changed item is re-rendered over its old span, a new
//! item is appended (to the file that owns its scope), a deleted item's
//! span is removed.  Everything between items — comments, blank lines —
//! and every untouched item's spelling survive; a changed item's own
//! comments do not.  Formula text is never reprinted.

use crate::build::{Anchor, AnchorRole, BuildResult, TextEntity};
use crate::identity::{IdentityTable, KeyEntry, KeyKind, SourceKey};
use crate::print;
use crate::workspace::SourceFile;
use bdl_diagnostics::Span;
use bdl_system::{BehaviorSystem, BindingEnd, ComponentId, PortId};
use std::collections::BTreeMap;

/// The files after write-back and the identity table that describes them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WriteBack {
    pub files: Vec<SourceFile>,
    pub table: IdentityTable,
    /// Files whose text changed, by index into `files`.
    pub changed: Vec<usize>,
}

/// One edit of one file.
#[derive(Clone, Debug)]
struct Edit {
    span: Span,
    text: String,
}

/// The file new top-level items go to: the first source file, or a fresh
/// `main.bdl` when the project has none.
pub const DEFAULT_FILE: &str = "src/main.bdl";

/// Compute the text of every source file that declares `system`, starting
/// from `previous` (the last load: its files and anchors) — see the module
/// documentation.
pub fn write_back(
    previous: &BuildResult,
    files: &[SourceFile],
    system: &BehaviorSystem,
) -> WriteBack {
    let mut w = Writer {
        previous,
        old: &previous.system,
        new: system,
        edits: vec![Vec::new(); files.len()],
        appends: vec![Vec::new(); files.len()],
        body_appends: BTreeMap::new(),
        files: files.to_vec(),
        default_file: None,
    };
    if w.files.is_empty() {
        w.files.push(SourceFile {
            path: DEFAULT_FILE.into(),
            text: String::new(),
        });
        w.edits.push(Vec::new());
        w.appends.push(Vec::new());
    }
    w.default_file = Some(0);
    w.top_level();
    w.components();
    w.instances_bindings_exports();
    w.finish()
}

struct Writer<'a> {
    previous: &'a BuildResult,
    old: &'a BehaviorSystem,
    new: &'a BehaviorSystem,
    edits: Vec<Vec<Edit>>,
    appends: Vec<Vec<String>>,
    /// New body items per component, appended before its closing brace.
    body_appends: BTreeMap<ComponentId, Vec<String>>,
    files: Vec<SourceFile>,
    default_file: Option<usize>,
}

impl Writer<'_> {
    fn anchor(&self, entity: TextEntity, role: AnchorRole) -> Option<&Anchor> {
        self.previous
            .anchors
            .iter()
            .find(|a| a.entity == entity && a.role == role)
    }

    /// Replace an item's span; continuation lines take the item's own
    /// indentation (a body item sits two spaces in).
    fn replace(&mut self, a: &Anchor, text: String) {
        let src = &self.files[a.file].text;
        let line_start = src[..a.span.start as usize]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let indent = &src[line_start..a.span.start as usize];
        let indent = if indent.trim().is_empty() {
            indent.to_owned()
        } else {
            String::new()
        };
        let mut lines = text.lines();
        let mut out = lines.next().unwrap_or_default().to_owned();
        for l in lines {
            out.push('\n');
            out.push_str(&indent);
            out.push_str(l);
        }
        self.edits[a.file].push(Edit {
            span: a.span,
            text: out,
        });
    }

    /// Remove an item and the line break after it.
    fn remove(&mut self, a: &Anchor) {
        let src = &self.files[a.file].text;
        let mut end = a.span.end as usize;
        if src[end..].starts_with("\r\n") {
            end += 2;
        } else if src[end..].starts_with('\n') {
            end += 1;
        }
        // Also eat one blank line before, so items stay one-blank-apart.
        let mut start = a.span.start as usize;
        if src[..start].ends_with("\n\n") {
            start -= 1;
        }
        self.edits[a.file].push(Edit {
            span: Span::new(start as u32, end as u32),
            text: String::new(),
        });
    }

    fn append_top(&mut self, text: String) {
        let f = self.default_file.unwrap_or(0);
        self.appends[f].push(text);
    }

    fn append_body(&mut self, component: ComponentId, text: String) {
        self.body_appends.entry(component).or_default().push(text);
    }

    /// One top-level or body entity by its *rendering* before and after:
    /// unchanged, changed, new or gone.  Comparing spellings (not model
    /// structs) is what makes a renamed concept re-render every signature
    /// that names it.
    fn item(
        &mut self,
        entity: TextEntity,
        old: Option<String>,
        new: Option<String>,
        body_of: Option<ComponentId>,
    ) {
        match (old, new, self.anchor(entity, AnchorRole::Item).copied()) {
            // Declared before and still here: re-render only when changed.
            (Some(o), Some(n), Some(a)) => {
                if o != n {
                    self.replace(&a, n);
                }
            }
            // Anchored but the old model did not know it (cannot happen for
            // a consistent load); treat as changed.
            (None, Some(n), Some(a)) => self.replace(&a, n),
            // New, or known before but never declared in text: append.
            (o, Some(n), None) => {
                if o.as_ref() != Some(&n) {
                    match body_of {
                        Some(c) => self.append_body(c, n),
                        None => self.append_top(n),
                    }
                }
            }
            (Some(_), None, Some(a)) => self.remove(&a),
            (None, None, _) | (Some(_), None, None) => {}
        }
    }

    /// A mapping's own item (without its drive), then its drive item.
    fn mapping_item(
        &mut self,
        entity: TextEntity,
        design_old: &bdl_model::surface::Design,
        design_new: &bdl_model::surface::Design,
        id: bdl_model::DeclId,
        body_of: Option<ComponentId>,
    ) {
        let old = design_old
            .mappings
            .get(&id)
            .map(|m| print::mapping(design_old, m));
        let new = design_new
            .mappings
            .get(&id)
            .map(|m| print::mapping(design_new, m));
        self.item(entity, old, new, body_of);
        // The drive edge is its own item.
        let old_drive = design_old
            .mappings
            .get(&id)
            .and_then(|m| print::drive(design_old, m));
        let new_drive = design_new
            .mappings
            .get(&id)
            .and_then(|m| print::drive(design_new, m));
        if old_drive != new_drive {
            match (self.anchor(entity, AnchorRole::Drive).copied(), new_drive) {
                (Some(a), Some(text)) => self.replace(&a, text),
                (Some(a), None) => self.remove(&a),
                (None, Some(text)) => match body_of {
                    Some(c) => self.append_body(c, text),
                    None => self.append_top(text),
                },
                (None, None) => {}
            }
        }
    }

    fn top_level(&mut self) {
        let old = &self.old.base;
        let new = &self.new.base;
        for id in ids(old.concepts.keys(), new.concepts.keys()) {
            self.item(
                TextEntity::Concept(id),
                old.concepts.get(&id).map(print::concept),
                new.concepts.get(&id).map(print::concept),
                None,
            );
        }
        for id in ids(old.clocks.keys(), new.clocks.keys()) {
            self.item(
                TextEntity::Clock(id),
                old.clocks.get(&id).map(print::clock),
                new.clocks.get(&id).map(print::clock),
                None,
            );
        }
        for id in ids(old.outputs.keys(), new.outputs.keys()) {
            self.item(
                TextEntity::Output(id),
                old.outputs.get(&id).map(|o| print::output(old, o)),
                new.outputs.get(&id).map(|o| print::output(new, o)),
                None,
            );
        }
        for id in ids(old.mappings.keys(), new.mappings.keys()) {
            self.mapping_item(TextEntity::Mapping(id), old, new, id, None);
        }
        for id in ids(old.devices.keys(), new.devices.keys()) {
            self.item(
                TextEntity::Device(id),
                old.devices.get(&id).map(|d| print::device(old, d)),
                new.devices.get(&id).map(|d| print::device(new, d)),
                None,
            );
        }
    }

    fn components(&mut self) {
        let old = self.old;
        let new = self.new;
        for cid in ids(old.components.keys(), new.components.keys()) {
            match (old.components.get(&cid), new.components.get(&cid)) {
                (None, Some(c)) => self.append_top(print::component(c, new)),
                (Some(_), None) => {
                    if let Some(a) = self
                        .anchor(TextEntity::Component(cid), AnchorRole::Item)
                        .copied()
                    {
                        self.remove(&a);
                    }
                }
                (Some(o), Some(n)) => {
                    if self
                        .anchor(TextEntity::Component(cid), AnchorRole::ComponentBody)
                        .is_none()
                    {
                        // Known but never anchored: print it whole.
                        if o != n {
                            self.append_top(print::component(n, new));
                        }
                        continue;
                    }
                    if o.name != n.name {
                        if let Some(a) = self
                            .anchor(TextEntity::Component(cid), AnchorRole::Name)
                            .copied()
                        {
                            self.replace(&a, n.name.clone());
                        }
                    }
                    self.component_body(cid, o, n);
                }
                (None, None) => {}
            }
        }
    }

    fn component_body(
        &mut self,
        cid: ComponentId,
        o: &bdl_system::BehaviorComponent,
        n: &bdl_system::BehaviorComponent,
    ) {
        // Concepts: private or shared decides the spelling.
        for id in ids(o.body.concepts.keys(), n.body.concepts.keys()) {
            let spell = |c: &bdl_system::BehaviorComponent, sys_of: &BehaviorSystem| {
                c.body
                    .concepts
                    .get(&id)
                    .map(|x| match c.shared_concepts.get(&id) {
                        Some(sys) => format!("use concept {}", concept_name(&sys_of.base, *sys)),
                        None => print::concept(x),
                    })
            };
            let os = spell(o, self.old);
            let ns = spell(n, self.new);
            self.item(TextEntity::BodyConcept(cid, id), os, ns, Some(cid));
        }
        for id in ids(o.body.clocks.keys(), n.body.clocks.keys()) {
            let spell = |c: &bdl_system::BehaviorComponent| {
                c.body.clocks.get(&id).map(|k| {
                    if c.interface.is_clock_param(id) {
                        format!("param clock {}", k.name)
                    } else {
                        print::clock(k)
                    }
                })
            };
            let os = spell(o);
            let ns = spell(n);
            self.item(TextEntity::BodyClock(cid, id), os, ns, Some(cid));
        }
        // Ports with their backing relationship; plain relationships.
        let port_decls: BTreeMap<bdl_model::DeclId, PortId> = n
            .interface
            .ports
            .values()
            .map(|p| (p.decl, p.id))
            .chain(o.interface.ports.values().map(|p| (p.decl, p.id)))
            .collect();
        for pid in ids(o.interface.ports.keys(), n.interface.ports.keys()) {
            let spell = |c: &bdl_system::BehaviorComponent| {
                c.interface.ports.get(&pid).map(|p| print::port(c, p))
            };
            let os = spell(o);
            let ns = spell(n);
            // The port and its relationship share one item span; the port's
            // anchor is the one that carries the edit.
            let entity = TextEntity::Port(cid, pid);
            let old_backed = o.interface.ports.get(&pid).map(|p| p.decl);
            let new_backed = n.interface.ports.get(&pid).map(|p| p.decl);
            let drive_changed = match (old_backed, new_backed) {
                (Some(od), Some(nd)) => {
                    o.body.mappings.get(&od).and_then(|m| m.drives)
                        != n.body.mappings.get(&nd).and_then(|m| m.drives)
                }
                _ => false,
            };
            self.item(entity, os, ns, Some(cid));
            if drive_changed {
                if let Some(nd) = new_backed {
                    let text = n
                        .body
                        .mappings
                        .get(&nd)
                        .and_then(|m| print::drive(&n.body, m));
                    let old_anchor = self
                        .anchor(TextEntity::BodyMapping(cid, nd), AnchorRole::Drive)
                        .copied();
                    match (old_anchor, text) {
                        (Some(a), Some(t)) => self.replace(&a, t),
                        (Some(a), None) => self.remove(&a),
                        (None, Some(t)) => self.append_body(cid, t),
                        (None, None) => {}
                    }
                }
            }
        }
        for id in ids(o.body.mappings.keys(), n.body.mappings.keys()) {
            if port_decls.contains_key(&id) {
                continue;
            }
            self.mapping_item(
                TextEntity::BodyMapping(cid, id),
                &o.body,
                &n.body,
                id,
                Some(cid),
            );
        }
        for id in ids(o.body.outputs.keys(), n.body.outputs.keys()) {
            let spell = |c: &bdl_system::BehaviorComponent, sys_of: &BehaviorSystem| {
                c.body
                    .outputs
                    .get(&id)
                    .map(|x| match c.external_outputs.get(&id) {
                        Some(sys) => format!("use output {}", output_name(&sys_of.base, *sys)),
                        None => print::output(&c.body, x),
                    })
            };
            let os = spell(o, self.old);
            let ns = spell(n, self.new);
            self.item(TextEntity::BodyOutput(cid, id), os, ns, Some(cid));
        }
        for id in ids(o.body.devices.keys(), n.body.devices.keys()) {
            self.item(
                TextEntity::BodyDevice(cid, id),
                o.body.devices.get(&id).map(|d| print::device(&o.body, d)),
                n.body.devices.get(&id).map(|d| print::device(&n.body, d)),
                Some(cid),
            );
        }
    }

    fn instances_bindings_exports(&mut self) {
        let old = self.old;
        let new = self.new;
        for id in ids(old.instances.keys(), new.instances.keys()) {
            self.item(
                TextEntity::Instance(id),
                old.instances.get(&id).map(|i| print::instance(old, i)),
                new.instances.get(&id).map(|i| print::instance(new, i)),
                None,
            );
        }
        for id in ids(old.bindings.keys(), new.bindings.keys()) {
            // A binding's spelling depends on the names at its ends; compare
            // the rendering so a renamed end re-renders the binding too.
            let os = old.bindings.get(&id).map(|b| print::binding(old, b));
            let ns = new.bindings.get(&id).map(|b| print::binding(new, b));
            self.item(TextEntity::Binding(id), os, ns, None);
        }
        for id in ids(old.exports.keys(), new.exports.keys()) {
            let os = old.exports.get(&id).map(|e| print::export(old, e));
            let ns = new.exports.get(&id).map(|e| print::export(new, e));
            self.item(TextEntity::Export(id), os, ns, None);
        }
    }

    fn finish(mut self) -> WriteBack {
        // Body appends become edits at each component's closing brace.
        let appends = std::mem::take(&mut self.body_appends);
        for (cid, items) in appends {
            let Some(a) = self
                .anchor(TextEntity::Component(cid), AnchorRole::ComponentBody)
                .copied()
            else {
                continue;
            };
            let src = &self.files[a.file].text;
            let close = a.span.end as usize - 1; // the `}`
                                                 // Indent the items two spaces and keep the brace on its own line.
            let mut text = String::new();
            let before = &src[..close];
            if !before.ends_with('\n') {
                text.push('\n');
            }
            if !before.trim_end().is_empty() && !before.ends_with("\n\n") {
                text.push('\n');
            }
            for item in items {
                for line in item.lines() {
                    text.push_str("  ");
                    text.push_str(line);
                    text.push('\n');
                }
            }
            self.edits[a.file].push(Edit {
                span: Span::new(close as u32, close as u32),
                text,
            });
        }
        let mut changed = Vec::new();
        for (i, file) in self.files.iter_mut().enumerate() {
            let mut edits = std::mem::take(&mut self.edits[i]);
            // Identical spans (a port and its relationship) collapse to one.
            edits.sort_by(|x, y| (y.span.start, y.span.end).cmp(&(x.span.start, x.span.end)));
            edits.dedup_by(|x, y| x.span == y.span);
            let mut text = file.text.clone();
            let mut last_start = u32::MAX;
            for e in &edits {
                if e.span.end > last_start {
                    // Overlapping with an edit already applied: skip rather
                    // than corrupt the file.
                    continue;
                }
                text.replace_range(e.span.start as usize..e.span.end as usize, &e.text);
                last_start = e.span.start;
            }
            for item in std::mem::take(&mut self.appends[i]) {
                if !text.is_empty() && !text.ends_with('\n') {
                    text.push('\n');
                }
                if !text.is_empty() && !text.ends_with("\n\n") {
                    text.push('\n');
                }
                text.push_str(&item);
                text.push('\n');
            }
            if text != file.text {
                changed.push(i);
                file.text = text;
            }
        }
        let table = table_of(self.new, self.previous, &self.files);
        WriteBack {
            files: self.files,
            table,
            changed,
        }
    }
}

/// The union of two key sets, sorted.
fn ids<'a, I: Ord + Copy + 'a>(
    a: impl Iterator<Item = &'a I>,
    b: impl Iterator<Item = &'a I>,
) -> Vec<I> {
    let mut v: Vec<I> = a.copied().chain(b.copied()).collect();
    v.sort();
    v.dedup();
    v
}

fn concept_name(design: &bdl_model::surface::Design, id: bdl_model::SemanticId) -> String {
    design
        .concepts
        .get(&id)
        .map(|c| c.name.clone())
        .unwrap_or_default()
}

fn output_name(design: &bdl_model::surface::Design, id: bdl_model::OutputId) -> String {
    design
        .outputs
        .get(&id)
        .map(|o| o.name.clone())
        .unwrap_or_default()
}

/// The identity table that describes `system` as written: one key per
/// entity, its id, the file it was (or will be) declared in.
pub fn table_of(
    system: &BehaviorSystem,
    previous: &BuildResult,
    files: &[SourceFile],
) -> IdentityTable {
    let file_of = |entity: TextEntity| -> String {
        previous
            .anchors
            .iter()
            .find(|a| a.entity == entity && a.role == AnchorRole::Item)
            .and_then(|a| files.get(a.file))
            .map(|f| f.path.clone())
            .unwrap_or_else(|| files.first().map(|f| f.path.clone()).unwrap_or_default())
    };
    let mut keys = BTreeMap::new();
    let mut put = |key: SourceKey, id: u64, entity: TextEntity, shape: String| {
        keys.insert(
            key.text(),
            KeyEntry {
                id,
                file: file_of(entity),
                shape,
            },
        );
    };
    let base = &system.base;
    for c in base.concepts.values() {
        put(
            SourceKey::top(KeyKind::Concept, &c.name),
            c.id.raw(),
            TextEntity::Concept(c.id),
            String::new(),
        );
    }
    for c in base.clocks.values() {
        put(
            SourceKey::top(KeyKind::Clock, &c.name),
            c.id.raw(),
            TextEntity::Clock(c.id),
            String::new(),
        );
    }
    for o in base.outputs.values() {
        put(
            SourceKey::top(KeyKind::Output, &o.name),
            o.id.raw(),
            TextEntity::Output(o.id),
            String::new(),
        );
    }
    for m in base.mappings.values() {
        put(
            SourceKey::top(KeyKind::Mapping, &m.name),
            m.id.raw(),
            TextEntity::Mapping(m.id),
            m.signature.inputs.len().to_string(),
        );
    }
    for d in base.devices.values() {
        put(
            SourceKey::top(KeyKind::Device, &d.name),
            d.id.raw(),
            TextEntity::Device(d.id),
            String::new(),
        );
    }
    for c in system.components.values() {
        put(
            SourceKey::top(KeyKind::Component, &c.name),
            c.id.raw(),
            TextEntity::Component(c.id),
            String::new(),
        );
        let scope = c.name.as_str();
        for x in c.body.concepts.values() {
            put(
                SourceKey::in_component(scope, KeyKind::Concept, &x.name),
                x.id.raw(),
                TextEntity::BodyConcept(c.id, x.id),
                String::new(),
            );
        }
        for k in c.body.clocks.values() {
            put(
                SourceKey::in_component(scope, KeyKind::Clock, &k.name),
                k.id.raw(),
                TextEntity::BodyClock(c.id, k.id),
                String::new(),
            );
        }
        for m in c.body.mappings.values() {
            put(
                SourceKey::in_component(scope, KeyKind::Mapping, &m.name),
                m.id.raw(),
                TextEntity::BodyMapping(c.id, m.id),
                m.signature.inputs.len().to_string(),
            );
        }
        for p in c.interface.ports.values() {
            put(
                SourceKey::in_component(scope, KeyKind::Port, &p.name),
                p.id.raw(),
                TextEntity::Port(c.id, p.id),
                p.contract.signature.inputs.len().to_string(),
            );
        }
        for o in c.body.outputs.values() {
            put(
                SourceKey::in_component(scope, KeyKind::Output, &o.name),
                o.id.raw(),
                TextEntity::BodyOutput(c.id, o.id),
                String::new(),
            );
        }
        for d in c.body.devices.values() {
            put(
                SourceKey::in_component(scope, KeyKind::Device, &d.name),
                d.id.raw(),
                TextEntity::BodyDevice(c.id, d.id),
                String::new(),
            );
        }
    }
    for i in system.instances.values() {
        put(
            SourceKey::top(KeyKind::Instance, &i.name),
            i.id.raw(),
            TextEntity::Instance(i.id),
            String::new(),
        );
    }
    for b in system.bindings.values() {
        put(
            SourceKey::top(
                KeyKind::Binding,
                format!(
                    "{}<-{}",
                    print::binding_end(system, b.destination),
                    print::binding_end(system, b.source)
                ),
            ),
            b.id.raw(),
            TextEntity::Binding(b.id),
            String::new(),
        );
    }
    for e in system.exports.values() {
        put(
            SourceKey::top(KeyKind::Export, &e.name),
            e.id.raw(),
            TextEntity::Export(e.id),
            String::new(),
        );
    }
    let mut component_ids = BTreeMap::new();
    for c in system.components.values() {
        component_ids.insert(format!("component:{}", c.name), c.body.ids.clone());
    }
    IdentityTable {
        schema_version: crate::identity::IDENTITIES_SCHEMA_VERSION,
        keys,
        base_ids: base.ids.clone(),
        system_ids: system.ids.clone(),
        component_ids,
        flat_ids: system.flat_ids.clone(),
    }
}

#[allow(dead_code)]
fn _binding_end_is_used(system: &BehaviorSystem, e: BindingEnd) -> String {
    print::binding_end(system, e)
}
