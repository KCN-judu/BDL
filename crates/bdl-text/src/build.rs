//! From the lowered items of every source file to one authored
//! `BehaviorSystem` (ADR-0020 §2, §6).
//!
//! Names are resolved project-wide after every file is read, in a fixed
//! order — concepts, clocks, outputs, relationships, drives, devices, then
//! each component's body the same way plus its ports, then instances,
//! bindings, exports — so a file may refer to a declaration in any other
//! file.  Every entity gets its id from the identity table (reconciled
//! first, so ids are stable across loads) and an [`Anchor`] into the
//! source; everything the text cannot mean is a [`LoadFault`] on the item
//! concerned, never a panic and never a silent guess.

use crate::identity::{FoundKey, IdentityTable, KeyKind, Reconciliation, SourceKey};
use bdl_diagnostics::Span;
use bdl_model::surface::{
    ClockDomain, Concept, Definition, Design, DeviceBinding, DeviceKind, MappingBlock,
    PhysicalOutput, Representation, Signature,
};
use bdl_model::{ClockId, DeclId, DeviceId, OutputId, OutputProfileId, SemanticId};
use bdl_syntax::lower::{
    BindEndItem, ComponentBodyItem, ComponentItem, MappingDefinition, PatternKind, PortWord,
    SurfaceItem, SurfaceModule, SurfaceType, TypeKind,
};
use bdl_syntax::{parse_module, SyntaxError};
use bdl_system::{
    BehaviorComponent, BehaviorInterface, BehaviorSystem, Binding, BindingEnd, BindingId,
    BindingTransport, ComponentId, ComponentInstance, ComponentInstanceId, Export, ExportId,
    ParameterValue, Port, PortContract, PortId, PortKind, PortRef,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An entity as the source declares it — including the declarations of a
/// component's body, which the flat design only knows freshened per
/// instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextEntity {
    Concept(SemanticId),
    Mapping(DeclId),
    Clock(ClockId),
    Output(OutputId),
    Device(DeviceId),
    Component(ComponentId),
    BodyConcept(ComponentId, SemanticId),
    BodyMapping(ComponentId, DeclId),
    BodyClock(ComponentId, ClockId),
    BodyOutput(ComponentId, OutputId),
    BodyDevice(ComponentId, DeviceId),
    Port(ComponentId, PortId),
    Instance(ComponentInstanceId),
    Binding(BindingId),
    Export(ExportId),
}

/// What an anchor's range is: the whole item, its name, its formula
/// body, or a mention of the entity somewhere else.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnchorRole {
    /// The whole item, keyword to end (what write-back replaces).
    Item,
    /// The defining name token.
    Name,
    /// A formula body (body-relative compiler spans offset into it).
    Body,
    /// A name that refers to the entity (a signature type, a clock tag, a
    /// binding end, a `drive`, a `use`).
    Reference,
    /// A `drive o = m` item, anchored to the driving relationship `m`.
    Drive,
    /// A component's `{ … }` body (new body items are inserted before its
    /// closing brace).
    ComponentBody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Anchor {
    pub file: usize,
    pub span: Span,
    pub entity: TextEntity,
    pub role: AnchorRole,
}

/// Something the text could not mean.  `open` marks what is a state
/// rather than a mistake (an `enum`, which the model has no type for).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoadFault {
    pub file: usize,
    pub span: Span,
    /// `text.unknown_concept`, `text.unknown_clock`, `text.unknown_output`,
    /// `text.unknown_relationship`, `text.unknown_component`,
    /// `text.unknown_instance`, `text.unknown_port`, `text.unknown_kind`,
    /// `text.duplicate_item`, `text.unsupported_item`, `text.bad_parameter`,
    /// `text.bad_argument`, `text.bad_binding`, `text.port_definition`,
    /// `text.ambiguous_identity`, `text.second_driver`.
    pub code: String,
    pub message: String,
    pub open: bool,
}

/// A fault or a syntax error, on one file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextFault {
    Syntax { file: usize, error: SyntaxError },
    Load(LoadFault),
}

impl TextFault {
    pub fn file(&self) -> usize {
        match self {
            TextFault::Syntax { file, .. } => *file,
            TextFault::Load(f) => f.file,
        }
    }
    pub fn span(&self) -> Span {
        match self {
            TextFault::Syntax { error, .. } => error.span,
            TextFault::Load(f) => f.span,
        }
    }
    pub fn message(&self) -> &str {
        match self {
            TextFault::Syntax { error, .. } => &error.message,
            TextFault::Load(f) => &f.message,
        }
    }
    pub fn code(&self) -> String {
        match self {
            TextFault::Syntax { error, .. } => error.code.as_str().to_owned(),
            TextFault::Load(f) => f.code.clone(),
        }
    }
    pub fn is_open(&self) -> bool {
        matches!(self, TextFault::Load(f) if f.open)
    }
}

/// The result of building: the system, the identities as they now
/// stand, what reconciliation decided, every fault, every anchor.
#[derive(Clone, Debug)]
pub struct BuildResult {
    pub system: BehaviorSystem,
    pub table: IdentityTable,
    pub reconciliation: Reconciliation,
    pub faults: Vec<TextFault>,
    pub anchors: Vec<Anchor>,
    /// Per file: the lowered module (for tools that need spans of items
    /// the model does not keep, such as enums).
    pub modules: Vec<SurfaceModule>,
}

impl BuildResult {
    /// The anchors of one entity, in file order.
    pub fn anchors_of(&self, entity: TextEntity) -> impl Iterator<Item = &Anchor> {
        self.anchors.iter().filter(move |a| a.entity == entity)
    }

    /// The item span an entity was declared at, if it came from source.
    pub fn item_anchor(&self, entity: TextEntity) -> Option<&Anchor> {
        self.anchors
            .iter()
            .find(|a| a.entity == entity && a.role == AnchorRole::Item)
    }

    /// Whether any fault is an error (syntax or load, not merely open).
    pub fn has_errors(&self) -> bool {
        self.faults.iter().any(|f| !f.is_open())
    }
}

/// Parse, lower and build the sources in the order given (the caller
/// sorts them by path), reconciling identities against `previous`.
pub fn build_system(
    name: &str,
    sources: &[(String, String)],
    previous: &IdentityTable,
) -> BuildResult {
    let mut modules = Vec::new();
    let mut faults = Vec::new();
    for (file, (_, text)) in sources.iter().enumerate() {
        let parse = parse_module(text);
        let (module, errors) = bdl_syntax::lower_module(&parse);
        for error in errors {
            faults.push(TextFault::Syntax { file, error });
        }
        modules.push(module);
    }
    let paths: Vec<&str> = sources.iter().map(|(p, _)| p.as_str()).collect();
    let texts: Vec<&str> = sources.iter().map(|(_, t)| t.as_str()).collect();
    let mut b = Builder::new(name, &paths, &texts, &modules, previous, faults);
    b.run();
    let Builder {
        system,
        table,
        reconciliation,
        faults,
        anchors,
        ..
    } = b;
    BuildResult {
        system,
        table,
        reconciliation,
        faults,
        anchors,
        modules,
    }
}

// ---- the builder -----------------------------------------------------------

/// Names of one design (the top level or a body), for resolution.
#[derive(Default)]
struct Names {
    concepts: BTreeMap<String, SemanticId>,
    clocks: BTreeMap<String, ClockId>,
    outputs: BTreeMap<String, OutputId>,
    mappings: BTreeMap<String, DeclId>,
    devices: BTreeMap<String, DeviceId>,
}

struct Builder<'a> {
    paths: &'a [&'a str],
    texts: &'a [&'a str],
    modules: &'a [SurfaceModule],
    table: IdentityTable,
    reconciliation: Reconciliation,
    system: BehaviorSystem,
    faults: Vec<TextFault>,
    anchors: Vec<Anchor>,
    base: Names,
    components: BTreeMap<String, ComponentId>,
    component_names: BTreeMap<ComponentId, Names>,
    ports: BTreeMap<ComponentId, BTreeMap<String, PortId>>,
    instances: BTreeMap<String, ComponentInstanceId>,
    /// Keys already bound in this load, to report duplicates once.
    bound: BTreeMap<String, (usize, Span)>,
}

impl<'a> Builder<'a> {
    fn new(
        name: &str,
        paths: &'a [&'a str],
        texts: &'a [&'a str],
        modules: &'a [SurfaceModule],
        previous: &IdentityTable,
        faults: Vec<TextFault>,
    ) -> Builder<'a> {
        // Reconcile every key first so ids are decided before anything
        // is built.
        let found = collect_keys(paths, modules);
        let (table, reconciliation) = previous.reconcile(&found);
        let mut faults = faults;
        for key in &reconciliation.ambiguous {
            if let Some((file, span)) = find_key_span(paths, modules, key) {
                faults.push(TextFault::Load(LoadFault {
                    file,
                    span,
                    code: "text.ambiguous_identity".into(),
                    message: format!(
                        "`{}` is new and more than one declaration of its kind disappeared from this file, so it could not be matched to an existing identity; it has a fresh one. Use the editor's rename to keep an identity.",
                        key.rsplit(':').next().unwrap_or(key)
                    ),
                    open: false,
                }));
            }
        }
        Builder {
            paths,
            texts,
            modules,
            table,
            reconciliation,
            system: BehaviorSystem::empty(name),
            faults,
            anchors: Vec::new(),
            base: Names::default(),
            components: BTreeMap::new(),
            component_names: BTreeMap::new(),
            ports: BTreeMap::new(),
            instances: BTreeMap::new(),
            bound: BTreeMap::new(),
        }
    }

    fn id(&self, key: &SourceKey) -> u64 {
        self.table
            .id_of(key)
            .expect("every key was reconciled before building")
    }

    fn fault(&mut self, file: usize, span: Span, code: &str, message: String) {
        self.faults.push(TextFault::Load(LoadFault {
            file,
            span,
            code: code.into(),
            message,
            open: false,
        }));
    }

    fn open_fault(&mut self, file: usize, span: Span, code: &str, message: String) {
        self.faults.push(TextFault::Load(LoadFault {
            file,
            span,
            code: code.into(),
            message,
            open: true,
        }));
    }

    fn anchor(&mut self, file: usize, span: Span, entity: TextEntity, role: AnchorRole) {
        // An item's node carries the comments directly above it; the item
        // anchor starts at the keyword so a re-rendered item keeps them.
        let span = match role {
            AnchorRole::Item | AnchorRole::Drive => trim_leading_trivia(self.texts[file], span),
            _ => span,
        };
        self.anchors.push(Anchor {
            file,
            span,
            entity,
            role,
        });
    }

    /// Bind a key once; a second declaration is a fault and is skipped.
    fn claim(&mut self, key: &SourceKey, file: usize, name_span: Span, item_span: Span) -> bool {
        let text = key.text();
        if let Some((first_file, _)) = self.bound.get(&text) {
            let where_ = if *first_file == file {
                "earlier in this file".to_owned()
            } else {
                format!("in `{}`", self.paths[*first_file])
            };
            self.fault(
                file,
                item_span,
                "text.duplicate_item",
                format!(
                    "`{}` is declared twice ({where_} first); the first declaration is the one used.",
                    key.name
                ),
            );
            return false;
        }
        self.bound.insert(text, (file, name_span));
        true
    }

    fn run(&mut self) {
        let modules = self.modules;
        // Top level, in resolution order.
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Concept(c) = item {
                    let key = SourceKey::top(KeyKind::Concept, &c.name.name);
                    if !self.claim(&key, file, c.name.span, c.span) {
                        continue;
                    }
                    let id = SemanticId::from_raw(self.id(&key));
                    let repr = self.representation(file, c.representation.as_ref());
                    let ordered =
                        self.ordered(file, c.span, &c.name.name, c.ordered, repr.as_ref());
                    self.system.base.concepts.insert(
                        id,
                        Concept {
                            id,
                            name: c.name.name.clone(),
                            description: String::new(),
                            representation: repr,
                            ordered,
                        },
                    );
                    self.base.concepts.insert(c.name.name.clone(), id);
                    self.anchor(file, c.span, TextEntity::Concept(id), AnchorRole::Item);
                    self.anchor(file, c.name.span, TextEntity::Concept(id), AnchorRole::Name);
                }
            }
        }
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Clock(c) = item {
                    let key = SourceKey::top(KeyKind::Clock, &c.name.name);
                    if !self.claim(&key, file, c.name.span, c.span) {
                        continue;
                    }
                    let id = ClockId::from_raw(self.id(&key));
                    self.system.base.clocks.insert(
                        id,
                        ClockDomain {
                            id,
                            name: c.name.name.clone(),
                        },
                    );
                    self.base.clocks.insert(c.name.name.clone(), id);
                    self.anchor(file, c.span, TextEntity::Clock(id), AnchorRole::Item);
                    self.anchor(file, c.name.span, TextEntity::Clock(id), AnchorRole::Name);
                }
            }
        }
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Output(o) = item {
                    let key = SourceKey::top(KeyKind::Output, &o.name.name);
                    if !self.claim(&key, file, o.name.span, o.span) {
                        continue;
                    }
                    let id = OutputId::from_raw(self.id(&key));
                    let Some(accepts) = self.top_concept(file, &o.accepts) else {
                        continue;
                    };
                    let clock = match &o.clock {
                        Some(c) => match self.base.clocks.get(&c.name).copied() {
                            Some(id) => {
                                self.anchor(
                                    file,
                                    c.span,
                                    TextEntity::Clock(id),
                                    AnchorRole::Reference,
                                );
                                Some(id)
                            }
                            None => {
                                self.unknown_clock(file, c.span, &c.name);
                                None
                            }
                        },
                        None => None,
                    };
                    self.system.base.outputs.insert(
                        id,
                        PhysicalOutput {
                            id,
                            name: o.name.name.clone(),
                            description: String::new(),
                            accepts,
                            clock,
                            required: o.required,
                        },
                    );
                    self.base.outputs.insert(o.name.name.clone(), id);
                    self.anchor(file, o.span, TextEntity::Output(id), AnchorRole::Item);
                    self.anchor(file, o.name.span, TextEntity::Output(id), AnchorRole::Name);
                    self.anchor(
                        file,
                        o.accepts.span,
                        TextEntity::Concept(accepts),
                        AnchorRole::Reference,
                    );
                }
            }
        }
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Mapping(mp) = item {
                    let key = SourceKey::top(KeyKind::Mapping, &mp.name.name);
                    if !self.claim(&key, file, mp.name.span, mp.span) {
                        continue;
                    }
                    let id = DeclId::from_raw(self.id(&key));
                    let Some(block) = self.mapping_block(
                        file,
                        id,
                        &mp.name.name,
                        &mp.signature,
                        mp.clock.as_ref(),
                        mp.definition.as_ref(),
                        None,
                    ) else {
                        continue;
                    };
                    self.system.base.mappings.insert(id, block);
                    self.base.mappings.insert(mp.name.name.clone(), id);
                    self.anchor(file, mp.span, TextEntity::Mapping(id), AnchorRole::Item);
                    self.anchor(
                        file,
                        mp.name.span,
                        TextEntity::Mapping(id),
                        AnchorRole::Name,
                    );
                    if let Some(d) = &mp.definition {
                        self.anchor(
                            file,
                            d.name.span,
                            TextEntity::Mapping(id),
                            AnchorRole::Reference,
                        );
                        self.anchor(file, d.body.span, TextEntity::Mapping(id), AnchorRole::Body);
                    }
                }
            }
        }
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Drive(d) = item {
                    let Some(output) = self.base.outputs.get(&d.output.name).copied() else {
                        self.fault(
                            file,
                            d.output.span,
                            "text.unknown_output",
                            format!(
                                "`{}` is not a physical output of this project.",
                                d.output.name
                            ),
                        );
                        continue;
                    };
                    let Some(driver) = self.base.mappings.get(&d.driver.name).copied() else {
                        self.unknown_relationship(file, d.driver.span, &d.driver.name);
                        continue;
                    };
                    if self.drive(file, d.span, output, driver, None) {
                        self.anchor(file, d.span, TextEntity::Mapping(driver), AnchorRole::Drive);
                    }
                    self.anchor(
                        file,
                        d.output.span,
                        TextEntity::Output(output),
                        AnchorRole::Reference,
                    );
                    self.anchor(
                        file,
                        d.driver.span,
                        TextEntity::Mapping(driver),
                        AnchorRole::Reference,
                    );
                }
            }
        }
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Device(d) = item {
                    let key = SourceKey::top(KeyKind::Device, &d.name.name);
                    if !self.claim(&key, file, d.name.span, d.span) {
                        continue;
                    }
                    let id = DeviceId::from_raw(self.id(&key));
                    let Some(kind) = self.device_kind(file, d.kind.span, &d.kind.name) else {
                        continue;
                    };
                    let output = match &d.output {
                        Some(o) => match self.base.outputs.get(&o.name).copied() {
                            Some(id) => {
                                self.anchor(
                                    file,
                                    o.span,
                                    TextEntity::Output(id),
                                    AnchorRole::Reference,
                                );
                                Some(id)
                            }
                            None => {
                                self.fault(
                                    file,
                                    o.span,
                                    "text.unknown_output",
                                    format!(
                                        "`{}` is not a physical output of this project.",
                                        o.name
                                    ),
                                );
                                continue;
                            }
                        },
                        None => None,
                    };
                    let fixed_pins = d.pins.iter().map(|(i, p)| (*i, p.name.clone())).collect();
                    self.system.base.devices.insert(
                        id,
                        DeviceBinding {
                            id,
                            name: d.name.name.clone(),
                            kind,
                            output,
                            realization: d
                                .realization
                                .as_ref()
                                .map(|r| OutputProfileId(r.name.clone())),
                            fixed_pins,
                        },
                    );
                    self.base.devices.insert(d.name.name.clone(), id);
                    self.anchor(file, d.span, TextEntity::Device(id), AnchorRole::Item);
                    self.anchor(file, d.name.span, TextEntity::Device(id), AnchorRole::Name);
                }
            }
        }
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Enum(e) = item {
                    self.open_fault(
                        file,
                        e.span,
                        "text.unsupported_item",
                        "enums are syntax only in this version; the design model has no sum types yet.".into(),
                    );
                }
            }
        }

        // Components.
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Component(c) = item {
                    self.component(file, c);
                }
            }
        }
        // Instances, then bindings and exports.
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                if let SurfaceItem::Instance(i) = item {
                    self.instance(file, i);
                }
            }
        }
        for (file, m) in modules.iter().enumerate() {
            for item in &m.items {
                match item {
                    SurfaceItem::Bind(b) => self.bind(file, b),
                    SurfaceItem::Export(e) => self.export(file, e),
                    _ => {}
                }
            }
        }

        // Allocators and the flat freshening table live in the identity
        // table; the system must carry them so later edits continue the
        // same sequences.
        self.system.base.ids = self.table.base_ids.clone();
        self.system.ids = self.table.system_ids.clone();
        self.system.flat_ids = self.table.flat_ids.clone();
        for (cid, comp) in self.system.components.iter_mut() {
            let key = format!("component:{}", comp.name);
            if let Some(ids) = self.table.component_ids.get(&key) {
                comp.body.ids = ids.clone();
            }
            let _ = cid;
        }
        bdl_system::edit::ensure_flat_ids(&mut self.system);
        self.table.base_ids = self.system.base.ids.clone();
        self.table.flat_ids = self.system.flat_ids.clone();
        self.descriptions();
    }

    /// `///` lines directly above an item are its description.
    fn descriptions(&mut self) {
        let docs: Vec<(TextEntity, String)> = self
            .anchors
            .iter()
            .filter(|a| a.role == AnchorRole::Item)
            .map(|a| (a.entity, leading_doc(self.texts[a.file], a.span)))
            .filter(|(_, d)| !d.is_empty())
            .collect();
        let s = &mut self.system;
        for (entity, doc) in docs {
            match entity {
                TextEntity::Concept(id) => {
                    if let Some(c) = s.base.concepts.get_mut(&id) {
                        c.description = doc;
                    }
                }
                TextEntity::Mapping(id) => {
                    if let Some(m) = s.base.mappings.get_mut(&id) {
                        m.description = doc;
                    }
                }
                TextEntity::Output(id) => {
                    if let Some(o) = s.base.outputs.get_mut(&id) {
                        o.description = doc;
                    }
                }
                TextEntity::Component(id) => {
                    if let Some(c) = s.components.get_mut(&id) {
                        c.description = doc;
                    }
                }
                TextEntity::BodyConcept(cid, id) => {
                    if let Some(c) = s
                        .components
                        .get_mut(&cid)
                        .and_then(|c| c.body.concepts.get_mut(&id))
                    {
                        c.description = doc;
                    }
                }
                TextEntity::BodyMapping(cid, id) => {
                    if let Some(m) = s
                        .components
                        .get_mut(&cid)
                        .and_then(|c| c.body.mappings.get_mut(&id))
                    {
                        m.description = doc;
                    }
                }
                TextEntity::BodyOutput(cid, id) => {
                    if let Some(o) = s
                        .components
                        .get_mut(&cid)
                        .and_then(|c| c.body.outputs.get_mut(&id))
                    {
                        o.description = doc;
                    }
                }
                TextEntity::Port(cid, id) => {
                    if let Some(p) = s
                        .components
                        .get_mut(&cid)
                        .and_then(|c| c.interface.ports.get_mut(&id))
                    {
                        p.description = doc;
                    }
                }
                _ => {}
            }
        }
    }

    // ---- pieces --------------------------------------------------------------

    fn representation(&mut self, file: usize, t: Option<&SurfaceType>) -> Option<Representation> {
        let t = t?;
        match representation_of_type(t) {
            Ok(r) => Some(r),
            Err(RepresentationError::UnknownName(name)) => {
                self.fault(
                    file,
                    t.span,
                    "text.unknown_representation",
                    format!(
                        "`{name}` is not a value form; write Bool, Count, a quantity such as {}, or List<…>, Pair<…, …>, Option<…> of those.",
                        bdl_model::quantity::QUANTITIES
                            .iter()
                            .take(4)
                            .map(|q| q.type_name)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                );
                None
            }
            Err(RepresentationError::Shape(what)) => {
                self.fault(file, t.span, "text.unknown_representation", what);
                None
            }
        }
    }

    /// The order invariant of the model (`Concept::order_is_valid`): an
    /// `ordered concept` needs a quantity value form.  Anything else is
    /// reported and loaded unordered, so the model never holds an order
    /// nothing can compare by.
    fn ordered(
        &mut self,
        file: usize,
        span: Span,
        name: &str,
        declared: bool,
        repr: Option<&Representation>,
    ) -> bool {
        if !declared {
            return false;
        }
        if repr.is_some_and(Representation::supports_order) {
            return true;
        }
        self.fault(
            file,
            span,
            "text.order_needs_quantity",
            format!(
                "{name} is declared ordered, but only a quantity value form has an order to declare; it is loaded without one."
            ),
        );
        false
    }

    fn top_concept(&mut self, file: usize, t: &SurfaceType) -> Option<SemanticId> {
        let name = single_name(t);
        match name.and_then(|n| self.base.concepts.get(n).copied()) {
            Some(id) => Some(id),
            None => {
                self.unknown_concept(file, t);
                None
            }
        }
    }

    fn unknown_concept(&mut self, file: usize, t: &SurfaceType) {
        let text = &self.texts[file][t.span.start as usize..t.span.end as usize];
        let message = match &t.kind {
            // `()` is a domain, never a concept: it can only open a signature
            TypeKind::Unit => "`()` is the empty product, the domain of a relationship without inputs: it opens a signature (`mapping f : () -> B`) and is not a concept.".to_string(),
            TypeKind::Tuple(_) => "`(A, B)` spells a relationship's inputs and can only open a signature; each part must be a concept.".to_string(),
            _ => format!("`{text}` is not a concept of this project; declare it with `concept {text} : …` or pick an existing one."),
        };
        self.fault(file, t.span, "text.unknown_concept", message);
    }

    fn unknown_clock(&mut self, file: usize, span: Span, name: &str) {
        self.fault(
            file,
            span,
            "text.unknown_clock",
            format!("`{name}` is not a timing domain here; declare it with `clock {name}`."),
        );
    }

    fn unknown_relationship(&mut self, file: usize, span: Span, name: &str) {
        self.fault(
            file,
            span,
            "text.unknown_relationship",
            format!("`{name}` is not a relationship of this project."),
        );
    }

    fn device_kind(&mut self, file: usize, span: Span, name: &str) -> Option<DeviceKind> {
        match device_kind_named(name) {
            Some(k) => Some(k),
            None => {
                self.fault(
                    file,
                    span,
                    "text.unknown_kind",
                    format!("`{name}` is not a device kind; one of pwm_channel, digital_output, h_bridge_channel, i2c_sensor, quadrature_encoder, uart."),
                );
                None
            }
        }
    }

    /// Record `driver` as the single driver of `output` in `design` (the
    /// base, or a body when `component` is given).
    fn drive(
        &mut self,
        file: usize,
        span: Span,
        output: OutputId,
        driver: DeclId,
        component: Option<ComponentId>,
    ) -> bool {
        let design = match component {
            Some(c) => {
                &mut self
                    .system
                    .components
                    .get_mut(&c)
                    .expect("component exists")
                    .body
            }
            None => &mut self.system.base,
        };
        if let Some(existing) = design.mappings.values().find(|m| m.drives == Some(output)) {
            let existing_name = existing.name.clone();
            self.fault(
                file,
                span,
                "text.second_driver",
                format!("this output is already driven by `{existing_name}`; an output has one driver, so this `drive` is ignored."),
            );
            return false;
        }
        match design.mappings.get_mut(&driver) {
            Some(m) => {
                m.drives = Some(output);
                true
            }
            None => false,
        }
    }

    /// Build a mapping block of a design (`names` = the scope's names).
    #[allow(clippy::too_many_arguments)]
    fn mapping_block(
        &mut self,
        file: usize,
        id: DeclId,
        name: &str,
        signature: &SurfaceType,
        clock: Option<&bdl_syntax::lower::Ident>,
        definition: Option<&MappingDefinition>,
        component: Option<ComponentId>,
    ) -> Option<MappingBlock> {
        let (inputs, output) = signature.uncurry();
        let mut sig_inputs = Vec::new();
        let mut ok = true;
        let mut refs = Vec::new();
        for t in inputs {
            match self.concept_in(component, t) {
                Some(c) => {
                    sig_inputs.push(c);
                    refs.push((t.span, c));
                }
                None => {
                    self.unknown_concept(file, t);
                    ok = false;
                }
            }
        }
        let out = match self.concept_in(component, output) {
            Some(c) => {
                refs.push((output.span, c));
                Some(c)
            }
            None => {
                self.unknown_concept(file, output);
                ok = false;
                None
            }
        };
        for (span, c) in refs {
            let entity = match component {
                Some(cid) => TextEntity::BodyConcept(cid, c),
                None => TextEntity::Concept(c),
            };
            self.anchor(file, span, entity, AnchorRole::Reference);
        }
        let clock_id = match clock {
            Some(c) => match self.clock_in(component, &c.name) {
                Some(id) => {
                    let entity = match component {
                        Some(cid) => TextEntity::BodyClock(cid, id),
                        None => TextEntity::Clock(id),
                    };
                    self.anchor(file, c.span, entity, AnchorRole::Reference);
                    Some(id)
                }
                None => {
                    self.unknown_clock(file, c.span, &c.name);
                    None
                }
            },
            None => None,
        };
        if !ok {
            return None;
        }
        let out = out?;
        let mut parameters = Vec::new();
        let mut def_source = None;
        if let Some(def) = definition {
            for p in &def.params {
                match &p.kind {
                    PatternKind::Ident(n) => parameters.push(n.clone()),
                    PatternKind::Wildcard => parameters.push(String::new()),
                    _ => {
                        self.fault(
                            file,
                            p.span,
                            "text.bad_parameter",
                            "a parameter is a plain name (or `_`); patterns cannot destructure an input.".into(),
                        );
                        parameters.push(String::new());
                    }
                }
            }
            def_source = Some(
                self.texts[file][def.body.span.start as usize..def.body.span.end as usize]
                    .trim()
                    .to_owned(),
            );
        }
        Some(MappingBlock {
            id,
            name: name.to_owned(),
            description: String::new(),
            signature: Signature {
                inputs: sig_inputs,
                output: out,
            },
            definition: def_source.map(|source| Definition::Formula { source }),
            clock: clock_id,
            drives: None,
            parameters,
        })
    }

    fn concept_in(&self, component: Option<ComponentId>, t: &SurfaceType) -> Option<SemanticId> {
        let name = single_name(t)?;
        match component {
            Some(c) => self.component_names.get(&c)?.concepts.get(name).copied(),
            None => self.base.concepts.get(name).copied(),
        }
    }

    fn clock_in(&self, component: Option<ComponentId>, name: &str) -> Option<ClockId> {
        match component {
            Some(c) => self.component_names.get(&c)?.clocks.get(name).copied(),
            None => self.base.clocks.get(name).copied(),
        }
    }

    // ---- components ------------------------------------------------------------

    fn component(&mut self, file: usize, c: &ComponentItem) {
        let key = SourceKey::top(KeyKind::Component, &c.name.name);
        if !self.claim(&key, file, c.name.span, c.span) {
            return;
        }
        let cid = ComponentId::from_raw(self.id(&key));
        let scope = c.name.name.as_str();
        let mut body = Design::empty(&c.name.name);
        let mut names = Names::default();
        let mut shared = BTreeMap::new();
        let mut external = BTreeMap::new();
        let mut clock_params = Vec::new();
        let mut interface = BehaviorInterface::default();

        // Concepts and `use concept`, clocks and `param clock` first.
        for item in &c.items {
            match item {
                ComponentBodyItem::Concept(x) => {
                    let key = SourceKey::in_component(scope, KeyKind::Concept, &x.name.name);
                    if !self.claim(&key, file, x.name.span, x.span) {
                        continue;
                    }
                    let id = SemanticId::from_raw(self.id(&key));
                    let repr = self.representation(file, x.representation.as_ref());
                    let ordered =
                        self.ordered(file, x.span, &x.name.name, x.ordered, repr.as_ref());
                    body.concepts.insert(
                        id,
                        Concept {
                            id,
                            name: x.name.name.clone(),
                            description: String::new(),
                            representation: repr,
                            ordered,
                        },
                    );
                    names.concepts.insert(x.name.name.clone(), id);
                    self.anchor(
                        file,
                        x.span,
                        TextEntity::BodyConcept(cid, id),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        x.name.span,
                        TextEntity::BodyConcept(cid, id),
                        AnchorRole::Name,
                    );
                }
                ComponentBodyItem::Use(u) if u.concept => {
                    let key = SourceKey::in_component(scope, KeyKind::Concept, &u.name.name);
                    if !self.claim(&key, file, u.name.span, u.span) {
                        continue;
                    }
                    let Some(system_id) = self.base.concepts.get(&u.name.name).copied() else {
                        self.fault(
                            file,
                            u.name.span,
                            "text.unknown_concept",
                            format!("`{}` is not a concept of the system, so the component cannot share it.", u.name.name),
                        );
                        continue;
                    };
                    let id = SemanticId::from_raw(self.id(&key));
                    let shared_concept = &self.system.base.concepts[&system_id];
                    let (repr, ordered) = (
                        shared_concept.representation.clone(),
                        shared_concept.ordered,
                    );
                    body.concepts.insert(
                        id,
                        Concept {
                            id,
                            name: u.name.name.clone(),
                            description: String::new(),
                            representation: repr,
                            ordered,
                        },
                    );
                    shared.insert(id, system_id);
                    names.concepts.insert(u.name.name.clone(), id);
                    self.anchor(
                        file,
                        u.span,
                        TextEntity::BodyConcept(cid, id),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        u.name.span,
                        TextEntity::Concept(system_id),
                        AnchorRole::Reference,
                    );
                }
                ComponentBodyItem::Clock(x) | ComponentBodyItem::ParamClock(x) => {
                    let key = SourceKey::in_component(scope, KeyKind::Clock, &x.name.name);
                    if !self.claim(&key, file, x.name.span, x.span) {
                        continue;
                    }
                    let id = ClockId::from_raw(self.id(&key));
                    body.clocks.insert(
                        id,
                        ClockDomain {
                            id,
                            name: x.name.name.clone(),
                        },
                    );
                    names.clocks.insert(x.name.name.clone(), id);
                    if matches!(item, ComponentBodyItem::ParamClock(_)) {
                        clock_params.push(id);
                    }
                    self.anchor(
                        file,
                        x.span,
                        TextEntity::BodyClock(cid, id),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        x.name.span,
                        TextEntity::BodyClock(cid, id),
                        AnchorRole::Name,
                    );
                }
                _ => {}
            }
        }
        interface.clock_params = clock_params;
        // Register the body so resolution helpers can see it.
        self.component_names.insert(cid, names);
        self.components.insert(c.name.name.clone(), cid);
        self.system.components.insert(
            cid,
            BehaviorComponent {
                id: cid,
                name: c.name.name.clone(),
                description: String::new(),
                body,
                interface,
                shared_concepts: shared,
                external_outputs: external.clone(),
                body_stamp: 0,
                interface_stamp: 0,
            },
        );
        self.anchor(file, c.span, TextEntity::Component(cid), AnchorRole::Item);
        self.anchor(
            file,
            c.name.span,
            TextEntity::Component(cid),
            AnchorRole::Name,
        );
        self.anchor(
            file,
            c.body_span,
            TextEntity::Component(cid),
            AnchorRole::ComponentBody,
        );

        // Outputs and `use output`.
        for item in &c.items {
            match item {
                ComponentBodyItem::Output(o) => {
                    let key = SourceKey::in_component(scope, KeyKind::Output, &o.name.name);
                    if !self.claim(&key, file, o.name.span, o.span) {
                        continue;
                    }
                    let id = OutputId::from_raw(self.id(&key));
                    let Some(accepts) = self.concept_in(Some(cid), &o.accepts) else {
                        self.unknown_concept(file, &o.accepts);
                        continue;
                    };
                    let clock = match &o.clock {
                        Some(cl) => match self.clock_in(Some(cid), &cl.name) {
                            Some(id) => Some(id),
                            None => {
                                self.unknown_clock(file, cl.span, &cl.name);
                                None
                            }
                        },
                        None => None,
                    };
                    let comp = self
                        .system
                        .components
                        .get_mut(&cid)
                        .expect("component exists");
                    comp.body.outputs.insert(
                        id,
                        PhysicalOutput {
                            id,
                            name: o.name.name.clone(),
                            description: String::new(),
                            accepts,
                            clock,
                            required: o.required,
                        },
                    );
                    self.component_names
                        .get_mut(&cid)
                        .expect("registered")
                        .outputs
                        .insert(o.name.name.clone(), id);
                    self.anchor(
                        file,
                        o.span,
                        TextEntity::BodyOutput(cid, id),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        o.name.span,
                        TextEntity::BodyOutput(cid, id),
                        AnchorRole::Name,
                    );
                }
                ComponentBodyItem::Use(u) if !u.concept => {
                    let key = SourceKey::in_component(scope, KeyKind::Output, &u.name.name);
                    if !self.claim(&key, file, u.name.span, u.span) {
                        continue;
                    }
                    let Some(system_id) = self.base.outputs.get(&u.name.name).copied() else {
                        self.fault(
                            file,
                            u.name.span,
                            "text.unknown_output",
                            format!("`{}` is not a physical output of the system, so the component cannot drive it.", u.name.name),
                        );
                        continue;
                    };
                    let id = OutputId::from_raw(self.id(&key));
                    let sys = self.system.base.outputs[&system_id].clone();
                    // The body's copy accepts the body's concept that stands
                    // for the system's; without a shared concept the use is
                    // meaningless.
                    let local_concept = self.system.components.get(&cid).and_then(|comp| {
                        comp.shared_concepts
                            .iter()
                            .find(|(_, s)| **s == sys.accepts)
                            .map(|(l, _)| *l)
                    });
                    let Some(accepts) = local_concept else {
                        self.fault(
                            file,
                            u.name.span,
                            "text.unknown_concept",
                            format!("`use output {}` needs `use concept {}` in this component first, so the output's concept is shared.", u.name.name, self.system.base.concepts.get(&sys.accepts).map(|c| c.name.clone()).unwrap_or_default()),
                        );
                        continue;
                    };
                    let comp = self
                        .system
                        .components
                        .get_mut(&cid)
                        .expect("component exists");
                    comp.body.outputs.insert(
                        id,
                        PhysicalOutput {
                            id,
                            name: u.name.name.clone(),
                            description: String::new(),
                            accepts,
                            clock: None,
                            required: sys.required,
                        },
                    );
                    comp.external_outputs.insert(id, system_id);
                    external.insert(id, system_id);
                    self.component_names
                        .get_mut(&cid)
                        .expect("registered")
                        .outputs
                        .insert(u.name.name.clone(), id);
                    self.anchor(
                        file,
                        u.span,
                        TextEntity::BodyOutput(cid, id),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        u.name.span,
                        TextEntity::Output(system_id),
                        AnchorRole::Reference,
                    );
                }
                _ => {}
            }
        }

        // Relationships and ports (each port is also a body relationship).
        for item in &c.items {
            match item {
                ComponentBodyItem::Mapping(mp) => {
                    let key = SourceKey::in_component(scope, KeyKind::Mapping, &mp.name.name);
                    if !self.claim(&key, file, mp.name.span, mp.span) {
                        continue;
                    }
                    let id = DeclId::from_raw(self.id(&key));
                    let Some(block) = self.mapping_block(
                        file,
                        id,
                        &mp.name.name,
                        &mp.signature,
                        mp.clock.as_ref(),
                        mp.definition.as_ref(),
                        Some(cid),
                    ) else {
                        continue;
                    };
                    let comp = self
                        .system
                        .components
                        .get_mut(&cid)
                        .expect("component exists");
                    comp.body.mappings.insert(id, block);
                    self.component_names
                        .get_mut(&cid)
                        .expect("registered")
                        .mappings
                        .insert(mp.name.name.clone(), id);
                    self.anchor(
                        file,
                        mp.span,
                        TextEntity::BodyMapping(cid, id),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        mp.name.span,
                        TextEntity::BodyMapping(cid, id),
                        AnchorRole::Name,
                    );
                    if let Some(d) = &mp.definition {
                        self.anchor(
                            file,
                            d.name.span,
                            TextEntity::BodyMapping(cid, id),
                            AnchorRole::Reference,
                        );
                        self.anchor(
                            file,
                            d.body.span,
                            TextEntity::BodyMapping(cid, id),
                            AnchorRole::Body,
                        );
                    }
                }
                ComponentBodyItem::Port(p) => {
                    let mkey = SourceKey::in_component(scope, KeyKind::Mapping, &p.name.name);
                    let pkey = SourceKey::in_component(scope, KeyKind::Port, &p.name.name);
                    if !self.claim(&mkey, file, p.name.span, p.span) {
                        continue;
                    }
                    self.claim(&pkey, file, p.name.span, p.span);
                    let decl = DeclId::from_raw(self.id(&mkey));
                    let port = PortId::from_raw(self.id(&pkey));
                    let kind = match p.word {
                        PortWord::Requires => PortKind::Required,
                        PortWord::Provides => PortKind::Provided,
                        PortWord::Param => PortKind::Parameter,
                    };
                    match (kind, p.definition.is_some()) {
                        (PortKind::Required, true) => self.fault(
                            file,
                            p.span,
                            "text.port_definition",
                            format!("`{}` is a required port: what it needs comes from outside, so it has no definition here. Use `provides` for a value the component computes.", p.name.name),
                        ),
                        (PortKind::Parameter, true) => self.fault(
                            file,
                            p.span,
                            "text.port_definition",
                            format!("`{}` is a parameter: each instance gives it a value, so it has no definition here.", p.name.name),
                        ),
                        _ => {}
                    }
                    let Some(block) = self.mapping_block(
                        file,
                        decl,
                        &p.name.name,
                        &p.signature,
                        p.clock.as_ref(),
                        p.definition.as_ref(),
                        Some(cid),
                    ) else {
                        continue;
                    };
                    let comp = self
                        .system
                        .components
                        .get_mut(&cid)
                        .expect("component exists");
                    let contract = PortContract::of_declaration(&block, &comp.interface);
                    comp.body.mappings.insert(decl, block);
                    comp.interface.ports.insert(
                        port,
                        Port {
                            id: port,
                            name: p.name.name.clone(),
                            description: String::new(),
                            kind,
                            decl,
                            contract,
                        },
                    );
                    self.component_names
                        .get_mut(&cid)
                        .expect("registered")
                        .mappings
                        .insert(p.name.name.clone(), decl);
                    self.ports
                        .entry(cid)
                        .or_default()
                        .insert(p.name.name.clone(), port);
                    self.anchor(file, p.span, TextEntity::Port(cid, port), AnchorRole::Item);
                    self.anchor(
                        file,
                        p.name.span,
                        TextEntity::Port(cid, port),
                        AnchorRole::Name,
                    );
                    self.anchor(
                        file,
                        p.span,
                        TextEntity::BodyMapping(cid, decl),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        p.name.span,
                        TextEntity::BodyMapping(cid, decl),
                        AnchorRole::Name,
                    );
                    if let Some(d) = &p.definition {
                        self.anchor(
                            file,
                            d.name.span,
                            TextEntity::Port(cid, port),
                            AnchorRole::Reference,
                        );
                        self.anchor(
                            file,
                            d.body.span,
                            TextEntity::BodyMapping(cid, decl),
                            AnchorRole::Body,
                        );
                    }
                }
                _ => {}
            }
        }
        // Drives and devices of the body.
        for item in &c.items {
            match item {
                ComponentBodyItem::Drive(d) => {
                    let names = self.component_names.get(&cid).expect("registered");
                    let Some(output) = names.outputs.get(&d.output.name).copied() else {
                        self.fault(
                            file,
                            d.output.span,
                            "text.unknown_output",
                            format!("`{}` is not an output of this component.", d.output.name),
                        );
                        continue;
                    };
                    let Some(driver) = names.mappings.get(&d.driver.name).copied() else {
                        self.fault(
                            file,
                            d.driver.span,
                            "text.unknown_relationship",
                            format!(
                                "`{}` is not a relationship of this component.",
                                d.driver.name
                            ),
                        );
                        continue;
                    };
                    if self.drive(file, d.span, output, driver, Some(cid)) {
                        self.anchor(
                            file,
                            d.span,
                            TextEntity::BodyMapping(cid, driver),
                            AnchorRole::Drive,
                        );
                    }
                }
                ComponentBodyItem::Device(d) => {
                    let key = SourceKey::in_component(scope, KeyKind::Device, &d.name.name);
                    if !self.claim(&key, file, d.name.span, d.span) {
                        continue;
                    }
                    let id = DeviceId::from_raw(self.id(&key));
                    let Some(kind) = self.device_kind(file, d.kind.span, &d.kind.name) else {
                        continue;
                    };
                    let output = match &d.output {
                        Some(o) => {
                            let names = self.component_names.get(&cid).expect("registered");
                            match names.outputs.get(&o.name).copied() {
                                Some(id) => Some(id),
                                None => {
                                    self.fault(
                                        file,
                                        o.span,
                                        "text.unknown_output",
                                        format!("`{}` is not an output of this component.", o.name),
                                    );
                                    continue;
                                }
                            }
                        }
                        None => None,
                    };
                    let comp = self
                        .system
                        .components
                        .get_mut(&cid)
                        .expect("component exists");
                    comp.body.devices.insert(
                        id,
                        DeviceBinding {
                            id,
                            name: d.name.name.clone(),
                            kind,
                            output,
                            realization: d
                                .realization
                                .as_ref()
                                .map(|r| OutputProfileId(r.name.clone())),
                            fixed_pins: d.pins.iter().map(|(i, p)| (*i, p.name.clone())).collect(),
                        },
                    );
                    self.anchor(
                        file,
                        d.span,
                        TextEntity::BodyDevice(cid, id),
                        AnchorRole::Item,
                    );
                    self.anchor(
                        file,
                        d.name.span,
                        TextEntity::BodyDevice(cid, id),
                        AnchorRole::Name,
                    );
                }
                ComponentBodyItem::Enum(e) => self.open_fault(
                    file,
                    e.span,
                    "text.unsupported_item",
                    "enums are syntax only in this version; the design model has no sum types yet."
                        .into(),
                ),
                _ => {}
            }
        }
    }

    // ---- instances, bindings, exports --------------------------------------------

    fn instance(&mut self, file: usize, i: &bdl_syntax::lower::InstanceItem) {
        let key = SourceKey::top(KeyKind::Instance, &i.name.name);
        if !self.claim(&key, file, i.name.span, i.span) {
            return;
        }
        let Some(component) = self.components.get(&i.component.name).copied() else {
            self.fault(
                file,
                i.component.span,
                "text.unknown_component",
                format!("`{}` is not a component of this project.", i.component.name),
            );
            return;
        };
        let id = ComponentInstanceId::from_raw(self.id(&key));
        let mut clock_bindings = BTreeMap::new();
        let mut parameter_bindings = BTreeMap::new();
        for arg in &i.args {
            let comp = &self.system.components[&component];
            let param_clock = comp
                .body
                .clocks
                .values()
                .find(|c| c.name == arg.name.name && comp.interface.is_clock_param(c.id))
                .map(|c| c.id);
            let param_port = comp
                .interface
                .ports
                .values()
                .find(|p| p.name == arg.name.name && p.kind == PortKind::Parameter)
                .map(|p| p.id);
            match (param_clock, param_port) {
                (Some(local), _) => {
                    let system_clock = match &arg.value.kind {
                        bdl_syntax::ExprKind::Name(n) => self.base.clocks.get(n).copied(),
                        _ => None,
                    };
                    match system_clock {
                        Some(sc) => {
                            clock_bindings.insert(local, sc);
                            self.anchor(file, arg.value.span, TextEntity::Clock(sc), AnchorRole::Reference);
                            self.anchor(file, arg.name.span, TextEntity::BodyClock(component, local), AnchorRole::Reference);
                        }
                        None => self.fault(
                            file,
                            arg.value.span,
                            "text.bad_argument",
                            format!("`{}` is a timing parameter of `{}`; give it a timing domain of the system.", arg.name.name, i.component.name),
                        ),
                    }
                }
                (None, Some(port)) => {
                    parameter_bindings.insert(
                        port,
                        ParameterValue {
                            source: arg.text.clone(),
                        },
                    );
                    self.anchor(
                        file,
                        arg.name.span,
                        TextEntity::Port(component, port),
                        AnchorRole::Reference,
                    );
                }
                (None, None) => self.fault(
                    file,
                    arg.name.span,
                    "text.bad_argument",
                    format!(
                        "`{}` is neither a timing parameter nor a parameter port of `{}`.",
                        arg.name.name, i.component.name
                    ),
                ),
            }
        }
        self.system.instances.insert(
            id,
            ComponentInstance {
                id,
                component,
                name: i.name.name.clone(),
                clock_bindings,
                parameter_bindings,
            },
        );
        self.instances.insert(i.name.name.clone(), id);
        self.anchor(file, i.span, TextEntity::Instance(id), AnchorRole::Item);
        self.anchor(
            file,
            i.name.span,
            TextEntity::Instance(id),
            AnchorRole::Name,
        );
        self.anchor(
            file,
            i.component.span,
            TextEntity::Component(component),
            AnchorRole::Reference,
        );
    }

    /// Resolve `instance.port` or a top-level relationship.
    fn bind_end(&mut self, file: usize, end: &BindEndItem) -> Option<BindingEnd> {
        match &end.second {
            Some(port_name) => {
                let Some(instance) = self.instances.get(&end.first.name).copied() else {
                    self.fault(
                        file,
                        end.first.span,
                        "text.unknown_instance",
                        format!("`{}` is not an instance of this project.", end.first.name),
                    );
                    return None;
                };
                let component = self.system.instances[&instance].component;
                let Some(port) = self
                    .ports
                    .get(&component)
                    .and_then(|p| p.get(&port_name.name))
                    .copied()
                else {
                    self.fault(
                        file,
                        port_name.span,
                        "text.unknown_port",
                        format!("`{}` has no port `{}`.", end.first.name, port_name.name),
                    );
                    return None;
                };
                self.anchor(
                    file,
                    end.first.span,
                    TextEntity::Instance(instance),
                    AnchorRole::Reference,
                );
                self.anchor(
                    file,
                    port_name.span,
                    TextEntity::Port(component, port),
                    AnchorRole::Reference,
                );
                Some(BindingEnd::Port(PortRef { instance, port }))
            }
            None => {
                let Some(decl) = self.base.mappings.get(&end.first.name).copied() else {
                    self.unknown_relationship(file, end.first.span, &end.first.name);
                    return None;
                };
                self.anchor(
                    file,
                    end.first.span,
                    TextEntity::Mapping(decl),
                    AnchorRole::Reference,
                );
                Some(BindingEnd::Base { decl })
            }
        }
    }

    fn bind(&mut self, file: usize, b: &bdl_syntax::lower::BindItem) {
        let key = SourceKey::top(
            KeyKind::Binding,
            format!("{}<-{}", end_text(&b.destination), end_text(&b.source)),
        );
        if !self.claim(&key, file, b.destination.span, b.span) {
            return;
        }
        let id = BindingId::from_raw(self.id(&key));
        let Some(destination) = self.bind_end(file, &b.destination) else {
            return;
        };
        let Some(source) = self.bind_end(file, &b.source) else {
            return;
        };
        if let (BindingEnd::Base { .. }, BindingEnd::Base { .. }) = (destination, source) {
            self.fault(
                file,
                b.span,
                "text.bad_binding",
                "a binding joins an instance's port to something; two top-level relationships are related by a formula, not a binding.".into(),
            );
            return;
        }
        self.system.bindings.insert(
            id,
            Binding {
                id,
                source,
                destination,
                transport: b
                    .init
                    .as_ref()
                    .map(|(_, text)| BindingTransport { init: text.clone() }),
            },
        );
        self.anchor(file, b.span, TextEntity::Binding(id), AnchorRole::Item);
    }

    fn export(&mut self, file: usize, e: &bdl_syntax::lower::ExportItem) {
        let key = SourceKey::top(KeyKind::Export, &e.name.name);
        if !self.claim(&key, file, e.name.span, e.span) {
            return;
        }
        let id = ExportId::from_raw(self.id(&key));
        let Some(BindingEnd::Port(port)) = self.bind_end(file, &e.port) else {
            self.fault(
                file,
                e.port.span,
                "text.bad_binding",
                "only an instance's port can be exported, as `instance.port`.".into(),
            );
            return;
        };
        self.system.exports.insert(
            id,
            Export {
                id,
                port,
                name: e.name.name.clone(),
            },
        );
        self.anchor(file, e.span, TextEntity::Export(id), AnchorRole::Item);
        self.anchor(file, e.name.span, TextEntity::Export(id), AnchorRole::Name);
    }
}

// ---- keys of a lowered tree ----------------------------------------------------

fn end_text(e: &BindEndItem) -> String {
    match &e.second {
        Some(s) => format!("{}.{}", e.first.name, s.name),
        None => e.first.name.clone(),
    }
}

fn arity(t: &SurfaceType) -> String {
    t.uncurry().0.len().to_string()
}

/// Every key the sources declare, with file and shape, in source order.
fn collect_keys(paths: &[&str], modules: &[SurfaceModule]) -> Vec<FoundKey> {
    let mut out = Vec::new();
    let mut push = |key: SourceKey, file: usize, shape: String| {
        out.push(FoundKey {
            key,
            file: paths[file].to_owned(),
            shape,
        })
    };
    for (file, m) in modules.iter().enumerate() {
        for item in &m.items {
            match item {
                SurfaceItem::Concept(c) => push(
                    SourceKey::top(KeyKind::Concept, &c.name.name),
                    file,
                    String::new(),
                ),
                SurfaceItem::Mapping(mp) => push(
                    SourceKey::top(KeyKind::Mapping, &mp.name.name),
                    file,
                    arity(&mp.signature),
                ),
                SurfaceItem::Clock(c) => push(
                    SourceKey::top(KeyKind::Clock, &c.name.name),
                    file,
                    String::new(),
                ),
                SurfaceItem::Output(o) => push(
                    SourceKey::top(KeyKind::Output, &o.name.name),
                    file,
                    String::new(),
                ),
                SurfaceItem::Device(d) => push(
                    SourceKey::top(KeyKind::Device, &d.name.name),
                    file,
                    String::new(),
                ),
                SurfaceItem::Component(c) => {
                    push(
                        SourceKey::top(KeyKind::Component, &c.name.name),
                        file,
                        String::new(),
                    );
                    let scope = c.name.name.as_str();
                    for bi in &c.items {
                        match bi {
                            ComponentBodyItem::Concept(x) => push(
                                SourceKey::in_component(scope, KeyKind::Concept, &x.name.name),
                                file,
                                String::new(),
                            ),
                            ComponentBodyItem::Use(u) => push(
                                SourceKey::in_component(
                                    scope,
                                    if u.concept {
                                        KeyKind::Concept
                                    } else {
                                        KeyKind::Output
                                    },
                                    &u.name.name,
                                ),
                                file,
                                String::new(),
                            ),
                            ComponentBodyItem::Mapping(mp) => push(
                                SourceKey::in_component(scope, KeyKind::Mapping, &mp.name.name),
                                file,
                                arity(&mp.signature),
                            ),
                            ComponentBodyItem::Clock(x) | ComponentBodyItem::ParamClock(x) => push(
                                SourceKey::in_component(scope, KeyKind::Clock, &x.name.name),
                                file,
                                String::new(),
                            ),
                            ComponentBodyItem::Output(o) => push(
                                SourceKey::in_component(scope, KeyKind::Output, &o.name.name),
                                file,
                                String::new(),
                            ),
                            ComponentBodyItem::Device(d) => push(
                                SourceKey::in_component(scope, KeyKind::Device, &d.name.name),
                                file,
                                String::new(),
                            ),
                            ComponentBodyItem::Port(p) => {
                                push(
                                    SourceKey::in_component(scope, KeyKind::Mapping, &p.name.name),
                                    file,
                                    arity(&p.signature),
                                );
                                push(
                                    SourceKey::in_component(scope, KeyKind::Port, &p.name.name),
                                    file,
                                    arity(&p.signature),
                                );
                            }
                            ComponentBodyItem::Drive(_) | ComponentBodyItem::Enum(_) => {}
                        }
                    }
                }
                SurfaceItem::Instance(i) => push(
                    SourceKey::top(KeyKind::Instance, &i.name.name),
                    file,
                    String::new(),
                ),
                SurfaceItem::Bind(b) => push(
                    SourceKey::top(
                        KeyKind::Binding,
                        format!("{}<-{}", end_text(&b.destination), end_text(&b.source)),
                    ),
                    file,
                    String::new(),
                ),
                SurfaceItem::Export(e) => push(
                    SourceKey::top(KeyKind::Export, &e.name.name),
                    file,
                    String::new(),
                ),
                SurfaceItem::Drive(_) | SurfaceItem::Enum(_) => {}
            }
        }
    }
    out
}

/// The item span that declares a key, for faults about identities.
fn find_key_span(paths: &[&str], modules: &[SurfaceModule], key: &str) -> Option<(usize, Span)> {
    let wanted = crate::identity::parse_key(key)?;
    for (file, m) in modules.iter().enumerate() {
        let _ = paths;
        for item in &m.items {
            let hit = match (&wanted.scope, item) {
                (None, SurfaceItem::Concept(c)) => {
                    wanted.kind == KeyKind::Concept && c.name.name == wanted.name
                }
                (None, SurfaceItem::Mapping(mp)) => {
                    wanted.kind == KeyKind::Mapping && mp.name.name == wanted.name
                }
                (None, SurfaceItem::Clock(c)) => {
                    wanted.kind == KeyKind::Clock && c.name.name == wanted.name
                }
                (None, SurfaceItem::Output(o)) => {
                    wanted.kind == KeyKind::Output && o.name.name == wanted.name
                }
                (None, SurfaceItem::Device(d)) => {
                    wanted.kind == KeyKind::Device && d.name.name == wanted.name
                }
                (None, SurfaceItem::Component(c)) => {
                    wanted.kind == KeyKind::Component && c.name.name == wanted.name
                }
                (None, SurfaceItem::Instance(i)) => {
                    wanted.kind == KeyKind::Instance && i.name.name == wanted.name
                }
                (None, SurfaceItem::Export(e)) => {
                    wanted.kind == KeyKind::Export && e.name.name == wanted.name
                }
                (Some(scope), SurfaceItem::Component(c))
                    if *scope == format!("component:{}", c.name.name) =>
                {
                    for bi in &c.items {
                        let name = match bi {
                            ComponentBodyItem::Concept(x) => Some(&x.name.name),
                            ComponentBodyItem::Mapping(mp) => Some(&mp.name.name),
                            ComponentBodyItem::Clock(x) | ComponentBodyItem::ParamClock(x) => {
                                Some(&x.name.name)
                            }
                            ComponentBodyItem::Output(o) => Some(&o.name.name),
                            ComponentBodyItem::Device(d) => Some(&d.name.name),
                            ComponentBodyItem::Port(p) => Some(&p.name.name),
                            ComponentBodyItem::Use(u) => Some(&u.name.name),
                            _ => None,
                        };
                        if name == Some(&wanted.name) {
                            return Some((file, bi.span()));
                        }
                    }
                    false
                }
                _ => false,
            };
            if hit {
                return Some((file, item.span()));
            }
        }
    }
    None
}

/// The span from its first non-trivia character — except that a run of
/// `///` doc-comment lines directly above the item stays inside it: they
/// are the item's description (TEXTUAL_SYNTAX §14.7).  Ordinary comments
/// and blank lines are left to the surrounding text.
pub fn trim_leading_trivia(text: &str, span: Span) -> Span {
    let mut i = span.start as usize;
    let end = span.end as usize;
    let bytes = text.as_bytes();
    let mut doc_start: Option<usize> = None;
    loop {
        while i < end && bytes[i].is_ascii_whitespace() {
            // A blank line breaks a doc run.
            if bytes[i] == b'\n' && doc_start.is_some() {
                let rest = &text[i + 1..end];
                if rest.starts_with('\n') || rest.starts_with("\r\n") {
                    doc_start = None;
                }
            }
            i += 1;
        }
        if i + 2 < end && &bytes[i..i + 3] == b"///" {
            if doc_start.is_none() {
                doc_start = Some(i);
            }
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            doc_start = None;
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if i + 1 < end && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            doc_start = None;
            match text[i..end].find("*/") {
                Some(j) => {
                    i += j + 2;
                    continue;
                }
                None => break,
            }
        }
        break;
    }
    Span::new(doc_start.unwrap_or(i) as u32, span.end)
}

/// The description an item's `///` lines carry (the lines' text, joined
/// with newlines), given its trimmed item span.
pub fn leading_doc(text: &str, span: Span) -> String {
    let mut lines = Vec::new();
    for line in text[span.start as usize..span.end as usize].lines() {
        let t = line.trim_start();
        if let Some(rest) = t.strip_prefix("///") {
            lines.push(rest.strip_prefix(' ').unwrap_or(rest).to_owned());
        } else if t.is_empty() {
            continue;
        } else {
            break;
        }
    }
    lines.join("\n")
}

fn single_name(t: &SurfaceType) -> Option<&str> {
    match &t.kind {
        TypeKind::Named { name, args } if args.is_empty() => Some(name),
        _ => None,
    }
}

/// The value form a type name stands for: `Bool`, `Count`, or a named
/// quantity of the shared vocabulary.
pub fn representation_named(name: &str) -> Option<Representation> {
    Some(match name {
        "Bool" | "Boolean" => Representation::Boolean,
        "Count" | "Nat" => Representation::Count,
        _ => Representation::Quantity {
            dim: bdl_model::quantity::by_type_name(name)?.dim,
        },
    })
}

/// Why a written type is not a value form.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RepresentationError {
    UnknownName(String),
    Shape(String),
}

/// The value form a written type stands for: a plain name, or `List<R>`,
/// `Pair<R₁, R₂>`, `Option<R>` over value forms — never a concept, never a
/// relationship type.
pub fn representation_of_type(t: &SurfaceType) -> Result<Representation, RepresentationError> {
    match &t.kind {
        TypeKind::Named { name, args } if args.is_empty() => {
            representation_named(name).ok_or_else(|| RepresentationError::UnknownName(name.clone()))
        }
        TypeKind::Named { name, args } => match (name.as_str(), args.as_slice()) {
            ("List", [e]) => Ok(Representation::list(representation_of_type(e)?)),
            ("Option", [e]) => Ok(Representation::optional(representation_of_type(e)?)),
            ("Pair", [a, b]) => Ok(Representation::pair(
                representation_of_type(a)?,
                representation_of_type(b)?,
            )),
            ("List" | "Option", _) => Err(RepresentationError::Shape(format!(
                "`{name}` takes one value form: `{name}<Scalar>`."
            ))),
            ("Pair", _) => Err(RepresentationError::Shape(
                "`Pair` takes two value forms: `Pair<Temperature, Scalar>`.".into(),
            )),
            _ => Err(RepresentationError::UnknownName(name.clone())),
        },
        TypeKind::Function { .. } => Err(RepresentationError::Shape(
            "a concept's value form is a value, never a relationship type.".into(),
        )),
        TypeKind::Unit => Err(RepresentationError::Shape(
            "`()` is the empty product — the domain of a relationship without inputs — never a concept's value form.".into(),
        )),
        TypeKind::Tuple(_) => Err(RepresentationError::Shape(
            "a grouped value form is written `Pair<A, B>`; `(A, B)` spells a relationship's inputs.".into(),
        )),
    }
}

/// A device kind by its snake-case name (the serde spelling).
pub fn device_kind_named(name: &str) -> Option<DeviceKind> {
    Some(match name {
        "pwm_channel" => DeviceKind::PwmChannel,
        "digital_output" => DeviceKind::DigitalOutput,
        "h_bridge_channel" => DeviceKind::HBridgeChannel,
        "i2c_sensor" => DeviceKind::I2cSensor,
        "quadrature_encoder" => DeviceKind::QuadratureEncoder,
        "uart" => DeviceKind::Uart,
        _ => return None,
    })
}

/// The snake-case name of a device kind, inverse of [`device_kind_named`].
pub fn device_kind_name(kind: DeviceKind) -> &'static str {
    match kind {
        DeviceKind::PwmChannel => "pwm_channel",
        DeviceKind::DigitalOutput => "digital_output",
        DeviceKind::HBridgeChannel => "h_bridge_channel",
        DeviceKind::I2cSensor => "i2c_sensor",
        DeviceKind::QuadratureEncoder => "quadrature_encoder",
        DeviceKind::Uart => "uart",
    }
}

/// The textual spelling of a value form, inverse of
/// [`representation_of_type`].
pub fn representation_name(r: &Representation) -> String {
    match r {
        Representation::Boolean => "Bool".into(),
        Representation::Count => "Count".into(),
        Representation::Quantity { dim } => bdl_model::quantity::by_dim(*dim)
            .map(|q| q.type_name.to_owned())
            .unwrap_or_else(|| "Scalar".into()),
        Representation::Optional { inner } => format!("Option<{}>", representation_name(inner)),
        Representation::List { element } => format!("List<{}>", representation_name(element)),
        Representation::Pair { first, second } => format!(
            "Pair<{}, {}>",
            representation_name(first),
            representation_name(second)
        ),
    }
}
