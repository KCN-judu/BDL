//! The text-workspace ground (ADR-0020): the committed state of a text
//! project is its source files, and the effective world is those files
//! with open buffers substituted, built by `bdl-text` and flattened by
//! `bdl-system` into the flat design every query runs on.
//!
//! What this module adds on top of the flat ground is the placement of
//! *every* entity on the authored source — including the declarations of a
//! component's body, which the flat design only knows freshened per
//! instance — so that diagnostics, navigation and rename land on the text
//! a person wrote, never on a generated flat declaration.

use crate::entity::{EntityRef, EntityRole};
use crate::index::EntityIndex;
use crate::projection::{ProjectionAnchor, ProjectionMap};
use crate::text::{DocumentId, DocumentUri, TextRange};
use crate::textual::{BindingFault, TextDocumentState};
use bdl_elab::names::{InputEnv, Lookup};
use bdl_model::surface::{Definition, ProjectSnapshot};
use bdl_model::{ClockId, ConceptId, DeclId, DeviceId, OutputId, Revision};
use bdl_syntax::ast::{self, AstNode};
use bdl_system::{
    flatten, BehaviorSystem, ComponentId, ComponentInstanceId, FlattenedSystem, LocalEntity,
    SystemSnapshot,
};
use bdl_text::build::{AnchorRole, BuildResult, TextEntity, TextFault};
use bdl_text::{IdentityTable, SourceFile};
use std::collections::BTreeMap;

/// The URI scheme under which a workspace's source files are documents:
/// `bdl-file:<relative path>`.  The adapter maps it to and from the
/// client's `file://` URIs.
pub const FILE_SCHEME: &str = "bdl-file";

pub fn file_uri(path: &str) -> DocumentUri {
    DocumentUri::new(format!("{FILE_SCHEME}:{path}"))
}

pub fn file_path(uri: &DocumentUri) -> Option<&str> {
    uri.as_str()
        .strip_prefix(FILE_SCHEME)
        .and_then(|s| s.strip_prefix(':'))
}

/// A text project as the host holds it: what is on disk.
#[derive(Clone, Debug)]
pub struct TextGround {
    pub name: String,
    pub files: Vec<SourceFile>,
    pub table: IdentityTable,
    /// Bumped on every reload from disk, so results stamped before a
    /// reload are recognisably stale.
    pub revision: Revision,
}

/// What a text-workspace snapshot knows beyond the flat design.
#[derive(Clone, Debug)]
pub struct TextWorld {
    pub system: BehaviorSystem,
    pub flattened: FlattenedSystem,
    pub table: IdentityTable,
    pub faults: Vec<TextFault>,
    /// The files as built (buffers substituted), in build order.
    pub files: Vec<SourceFile>,
    /// File index → document.
    pub documents: Vec<DocumentId>,
    /// Where each component's body is written: the scope of the items
    /// inside it.
    pub component_bodies: Vec<(DocumentId, TextRange, ComponentId)>,
}

impl TextWorld {
    /// The component whose body encloses `offset` in `document`.
    pub fn component_at(&self, document: DocumentId, offset: u32) -> Option<ComponentId> {
        self.component_bodies
            .iter()
            .find(|(d, r, _)| *d == document && r.start <= offset && offset <= r.end)
            .map(|(_, _, c)| *c)
    }
}

/// What [`compose_text`] yields: the flat snapshot, the text world, the
/// per-document states, the anchors, and the names of the system's own
/// entities.
pub type ComposedText = (
    ProjectSnapshot,
    TextWorld,
    BTreeMap<DocumentId, TextDocumentState>,
    ProjectionMap,
    Vec<(EntityRef, String)>,
);

/// Build the effective world of a text project: files with `buffers`
/// substituted, the system, its flattening, and the per-document states
/// and anchors keyed by *flat* entities.
pub fn compose_text(
    ground: &TextGround,
    buffers: &BTreeMap<DocumentId, String>,
    documents: &BTreeMap<String, DocumentId>,
) -> ComposedText {
    let mut files = ground.files.clone();
    for f in files.iter_mut() {
        if let Some(text) = documents.get(&f.path).and_then(|d| buffers.get(d)) {
            f.text = text.clone();
        }
    }
    // Buffers for files the disk does not have yet are new files.
    let mut extra: Vec<SourceFile> = documents
        .iter()
        .filter(|(p, _)| !files.iter().any(|f| f.path == **p))
        .filter_map(|(p, d)| {
            buffers.get(d).map(|t| SourceFile {
                path: p.clone(),
                text: t.clone(),
            })
        })
        .collect();
    files.append(&mut extra);
    files.sort_by(|a, b| a.path.cmp(&b.path));

    let build: BuildResult = bdl_text::load_workspace(&ground.name, &files, &ground.table);
    let flattened = flatten(&SystemSnapshot {
        revision: ground.revision,
        system: build.system.clone(),
    });
    let snapshot = flattened.snapshot.clone();
    let doc_of: Vec<DocumentId> = files
        .iter()
        .map(|f| {
            documents
                .get(&f.path)
                .copied()
                .unwrap_or(DocumentId(u32::MAX))
        })
        .collect();

    let system = &build.system;
    let mut projections = ProjectionMap::default();
    let mut component_bodies = Vec::new();
    let mut states: BTreeMap<DocumentId, TextDocumentState> = BTreeMap::new();
    for (i, f) in files.iter().enumerate() {
        states.insert(
            doc_of[i],
            TextDocumentState {
                document: doc_of[i],
                source: f.text.clone(),
                syntax_errors: Vec::new(),
                binding_faults: Vec::new(),
                declared: Vec::new(),
                formula_bodies: BTreeMap::new(),
            },
        );
    }
    for fault in &build.faults {
        let Some(state) = states.get_mut(&doc_of[fault.file()]) else {
            continue;
        };
        match fault {
            TextFault::Syntax { error, .. } => state.syntax_errors.push(error.clone()),
            TextFault::Load(l) => state.binding_faults.push(BindingFault {
                code: l.code.clone(),
                range: l.span.into(),
                message: l.message.clone(),
                entity: None,
                open: l.open,
            }),
        }
    }
    let mut names: Vec<(EntityRef, String)> = Vec::new();
    for c in system.components.values() {
        names.push((EntityRef::Component(c.id.raw()), c.name.clone()));
        for p in c.interface.ports.values() {
            names.push((
                EntityRef::Port {
                    component: c.id.raw(),
                    port: p.id.raw(),
                },
                p.name.clone(),
            ));
        }
    }
    for i in system.instances.values() {
        names.push((EntityRef::Instance(i.id.raw()), i.name.clone()));
    }
    for b in system.bindings.values() {
        names.push((
            EntityRef::Binding(b.id.raw()),
            format!("binding {}", b.id.raw()),
        ));
    }
    for e in system.exports.values() {
        names.push((EntityRef::Export(e.id.raw()), e.name.clone()));
    }

    // Flat entity → the authored entities it stands for, so a name inside
    // a formula body (resolved against the flat design) lands on the
    // source declaration — the port, not one instance's freshened copy.
    let mut authored: BTreeMap<EntityRef, Vec<EntityRef>> = BTreeMap::new();
    let mut bodies: Vec<(DocumentId, TextRange, Vec<EntityRef>)> = Vec::new();
    for a in &build.anchors {
        let entities = flat_entities(system, a.entity);
        if a.role == AnchorRole::Name {
            for e in &entities {
                authored
                    .entry(*e)
                    .or_default()
                    .extend(entities.iter().copied());
            }
        }
        if a.role == AnchorRole::Body {
            bodies.push((doc_of[a.file], a.span.into(), entities));
        }
    }
    for v in authored.values_mut() {
        v.sort();
        v.dedup();
    }
    for a in &build.anchors {
        let doc = doc_of[a.file];
        let range: TextRange = a.span.into();
        let entities = flat_entities(system, a.entity);
        let role = match a.role {
            AnchorRole::Item => EntityRole::Declaration,
            AnchorRole::Name => EntityRole::Name,
            AnchorRole::Body => EntityRole::Definition,
            AnchorRole::Reference => EntityRole::Reference,
            AnchorRole::Drive => EntityRole::DriveEdge,
            AnchorRole::ComponentBody => {
                if let TextEntity::Component(c) = a.entity {
                    component_bodies.push((doc, range, c));
                }
                continue;
            }
        };
        for e in &entities {
            projections.insert(ProjectionAnchor::text(*e, role, doc, range));
            if let Some(state) = states.get_mut(&doc) {
                match a.role {
                    AnchorRole::Item => {
                        if !state.declared.contains(e) {
                            state.declared.push(*e);
                        }
                    }
                    AnchorRole::Body => {
                        if let EntityRef::Mapping(d) = e {
                            state.formula_bodies.insert(*d, range);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    for (doc, body, mappings) in bodies {
        // Every flat copy of a body has the same text; the first resolves
        // for all of them.
        let Some(EntityRef::Mapping(m)) = mappings.first() else {
            continue;
        };
        for (range, target) in formula_occurrences(&snapshot.design, *m) {
            let range = TextRange::new(body.start + range.start, body.start + range.end);
            let targets = authored
                .get(&target)
                .cloned()
                .unwrap_or_else(|| vec![target]);
            for t in targets {
                projections.insert(ProjectionAnchor::text(t, EntityRole::Reference, doc, range));
            }
        }
    }
    projections.finish();
    let world = TextWorld {
        system: build.system,
        flattened,
        table: build.table,
        faults: build.faults,
        files,
        documents: doc_of,
        component_bodies,
    };
    (snapshot, world, states, projections, names)
}

/// The name occurrences of a flat mapping's formula that mean a concept
/// input or another mapping, resolved by the elaborator's own rule
/// (plain or pinned scope), as body-relative ranges.  Textual parameter
/// names are not references to the concept and are left out.
fn formula_occurrences(
    design: &bdl_model::surface::Design,
    mapping: DeclId,
) -> Vec<(TextRange, EntityRef)> {
    let Some(block) = design.mappings.get(&mapping) else {
        return Vec::new();
    };
    let (source, env, by_parameter): (&str, InputEnv, Vec<bool>) = match &block.definition {
        Some(Definition::Formula { source }) => (
            source,
            InputEnv::for_mapping(design, block),
            block
                .signature
                .inputs
                .iter()
                .enumerate()
                .map(|(i, _)| block.parameters.get(i).is_some_and(|p| !p.is_empty()))
                .collect(),
        ),
        Some(Definition::ScopedFormula { source, scope }) => {
            let by_parameter = block
                .signature
                .inputs
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    scope.inputs.get(i).is_some_and(|n| {
                        design
                            .concepts
                            .get(c)
                            .is_none_or(|concept| concept.name != *n)
                    })
                })
                .collect();
            (
                source,
                InputEnv::scoped(&block.signature.inputs, scope),
                by_parameter,
            )
        }
        _ => return Vec::new(),
    };
    let parse = bdl_syntax::parse_formula(source);
    let mut out = Vec::new();
    for node in parse.syntax_node().descendants() {
        let Some(name) = ast::NameExpr::cast(node) else {
            continue;
        };
        let Some(r) = name.name() else { continue };
        if name.local_binding().is_some() {
            continue;
        }
        let target = match env.resolve(design, &r.as_str()) {
            Lookup::Input(i) => {
                if by_parameter.get(i).copied().unwrap_or(false) {
                    continue;
                }
                match block.signature.inputs.get(i) {
                    Some(c) => EntityRef::Concept(*c),
                    None => continue,
                }
            }
            Lookup::Mapping(d) => EntityRef::Mapping(d),
            _ => continue,
        };
        out.push((TextRange::from(r.span()), target));
    }
    out
}

/// The flat entities a source entity stands for: one for a top-level
/// item, one per instance for a private body item, the shared system
/// entity for a `use`d one.
pub fn flat_entities(system: &BehaviorSystem, e: TextEntity) -> Vec<EntityRef> {
    let instances_of = |c: ComponentId| -> Vec<ComponentInstanceId> {
        system
            .instances
            .values()
            .filter(|i| i.component == c)
            .map(|i| i.id)
            .collect()
    };
    let flat = |i: ComponentInstanceId, l: LocalEntity| system.flat_ids.get(i, l);
    match e {
        TextEntity::Concept(s) => vec![EntityRef::Concept(s)],
        TextEntity::Mapping(d) => vec![EntityRef::Mapping(d)],
        TextEntity::Clock(c) => vec![EntityRef::Clock(c)],
        TextEntity::Output(o) => vec![EntityRef::Output(o)],
        TextEntity::Device(d) => vec![EntityRef::Device(d)],
        TextEntity::Component(c) => vec![EntityRef::Component(c.raw())],
        TextEntity::Instance(i) => vec![EntityRef::Instance(i.raw())],
        TextEntity::Binding(b) => vec![EntityRef::Binding(b.raw())],
        TextEntity::Export(x) => vec![EntityRef::Export(x.raw())],
        TextEntity::Port(c, p) => {
            let mut v = vec![EntityRef::Port {
                component: c.raw(),
                port: p.raw(),
            }];
            if let Some(port) = system
                .components
                .get(&c)
                .and_then(|comp| comp.interface.ports.get(&p))
            {
                v.extend(flat_entities(system, TextEntity::BodyMapping(c, port.decl)));
            }
            v
        }
        TextEntity::BodyConcept(c, s) => {
            if let Some(sys) = system
                .components
                .get(&c)
                .and_then(|x| x.shared_concepts.get(&s))
            {
                return vec![EntityRef::Concept(*sys)];
            }
            instances_of(c)
                .into_iter()
                .filter_map(|i| flat(i, LocalEntity::Sem(s)))
                .map(|raw| EntityRef::Concept(ConceptId::from_raw(raw)))
                .collect()
        }
        TextEntity::BodyMapping(c, d) => instances_of(c)
            .into_iter()
            .filter_map(|i| flat(i, LocalEntity::Decl(d)))
            .map(|raw| EntityRef::Mapping(DeclId::from_raw(raw)))
            .collect(),
        TextEntity::BodyClock(c, k) => {
            let comp = system.components.get(&c);
            if comp.is_some_and(|x| x.interface.is_clock_param(k)) {
                return instances_of(c)
                    .into_iter()
                    .filter_map(|i| system.instances.get(&i))
                    .filter_map(|i| i.clock_bindings.get(&k).copied())
                    .map(EntityRef::Clock)
                    .collect();
            }
            instances_of(c)
                .into_iter()
                .filter_map(|i| flat(i, LocalEntity::Clock(k)))
                .map(|raw| EntityRef::Clock(ClockId::from_raw(raw)))
                .collect()
        }
        TextEntity::BodyOutput(c, o) => {
            if let Some(sys) = system
                .components
                .get(&c)
                .and_then(|x| x.external_outputs.get(&o))
            {
                return vec![EntityRef::Output(*sys)];
            }
            instances_of(c)
                .into_iter()
                .filter_map(|i| flat(i, LocalEntity::Output(o)))
                .map(|raw| EntityRef::Output(OutputId::from_raw(raw)))
                .collect()
        }
        TextEntity::BodyDevice(c, d) => instances_of(c)
            .into_iter()
            .filter_map(|i| flat(i, LocalEntity::Device(d)))
            .map(|raw| EntityRef::Device(DeviceId::from_raw(raw)))
            .collect(),
    }
}

/// The names of the system's own entities, for the index.
pub fn register_names(index: &mut EntityIndex, names: &[(EntityRef, String)]) {
    for (e, n) in names {
        index.add_name(*e, n.clone());
    }
}
