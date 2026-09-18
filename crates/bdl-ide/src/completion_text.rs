//! Completion in a text workspace (ADR-0020), where the scope of a
//! position is decided by the authored system: inside a component body
//! the names are the body's, at module level the system's, and every
//! project item has a shape whose next slot is known from the line.
//!
//! This is deliberately line-shaped rather than parser-driven: the item
//! grammar is flat (`keyword name : type @clock`, `bind a.b = c`), the
//! prefix before the cursor says which slot is being filled, and the
//! candidates come from the model, never from a token list.

use crate::completion::{CompletionKind, ExpectedType, SemanticCompletion};
use bdl_ide_db::{DocumentId, EntityRef, TextRange, TextWorld};
use bdl_model::surface::{Design, DeviceKind};
use bdl_system::{BehaviorSystem, ComponentId, PortKind};

const MODULE_ITEMS: &[&str] = &[
    "concept",
    "mapping",
    "enum",
    "clock",
    "output",
    "drive",
    "device",
    "component",
    "instance",
    "bind",
    "export",
];
const COMPONENT_ITEMS: &[&str] = &[
    "use", "param", "clock", "requires", "provides", "mapping", "enum", "output", "drive", "device",
];
const DEVICE_KINDS: &[DeviceKind] = &[
    DeviceKind::PwmChannel,
    DeviceKind::DigitalOutput,
    DeviceKind::HBridgeChannel,
    DeviceKind::I2cSensor,
    DeviceKind::QuadratureEncoder,
    DeviceKind::Uart,
];

/// The slot being filled, read off the line before the cursor.
struct Line<'a> {
    /// The line up to the completion prefix, trimmed.
    before: &'a str,
    /// Its words.
    words: Vec<&'a str>,
}

impl Line<'_> {
    fn head(&self) -> &str {
        self.words.first().copied().unwrap_or("")
    }
    fn ends_with_word(&self, w: &str) -> bool {
        self.words.last().copied() == Some(w)
    }
}

struct Out {
    prefix: String,
    replace: TextRange,
    items: Vec<SemanticCompletion>,
}

impl Out {
    fn matches(&self, label: &str) -> bool {
        self.prefix.is_empty()
            || label
                .to_ascii_lowercase()
                .starts_with(&self.prefix.to_ascii_lowercase())
    }
    fn push(
        &mut self,
        label: &str,
        kind: CompletionKind,
        entity: Option<EntityRef>,
        insert: String,
        relevance: u8,
        doc: Option<String>,
    ) {
        if !self.matches(label) {
            return;
        }
        if self.items.iter().any(|i| i.label == label) {
            return;
        }
        self.items.push(SemanticCompletion {
            label: label.to_owned(),
            kind,
            entity,
            resulting_type: None,
            replace: self.replace,
            insert,
            relevance,
            documentation: doc,
            template: None,
        });
    }
    fn keyword(&mut self, k: &str) {
        self.push(k, CompletionKind::Keyword, None, format!("{k} "), 50, None);
    }
    fn concept(
        &mut self,
        name: &str,
        entity: Option<EntityRef>,
        rep: Option<bdl_model::surface::Representation>,
    ) {
        let ty = ExpectedType::of(rep.as_ref()).describe();
        self.push(
            name,
            CompletionKind::Concept,
            entity,
            name.to_owned(),
            50,
            Some(ty),
        );
    }
    fn concepts_of(&mut self, design: &Design) {
        for c in design.concepts.values() {
            self.concept(&c.name, None, c.representation.clone());
        }
    }
    fn mappings_of(&mut self, design: &Design, entity: bool) {
        for m in design.mappings.values() {
            let e = entity.then_some(EntityRef::Mapping(m.id));
            let out = design
                .concepts
                .get(&m.signature.output)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            self.push(
                &m.name,
                CompletionKind::Mapping,
                e,
                m.name.clone(),
                50,
                Some(format!("relationship → {out}")),
            );
        }
    }
    fn clocks_of(&mut self, design: &Design, entity: bool) {
        for k in design.clocks.values() {
            let e = entity.then_some(EntityRef::Clock(k.id));
            self.push(
                &k.name,
                CompletionKind::Keyword,
                e,
                k.name.clone(),
                50,
                Some("clock domain".into()),
            );
        }
    }
    fn outputs_of(&mut self, design: &Design, entity: bool) {
        for o in design.outputs.values() {
            let e = entity.then_some(EntityRef::Output(o.id));
            let accepts = design
                .concepts
                .get(&o.accepts)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            self.push(
                &o.name,
                CompletionKind::Mapping,
                e,
                o.name.clone(),
                50,
                Some(format!("output of {accepts}")),
            );
        }
    }
    fn representations(&mut self) {
        for name in bdl_ide_db::textual::representation_names() {
            let ty = bdl_ide_db::textual::representation_named(name)
                .map(|r| ExpectedType::of(Some(&r)).describe());
            self.push(
                name,
                CompletionKind::Representation,
                None,
                name.to_owned(),
                50,
                ty,
            );
        }
    }
    fn instances(&mut self, system: &BehaviorSystem, dotted: bool) {
        for i in system.instances.values() {
            let comp = system
                .components
                .get(&i.component)
                .map(|c| c.name.clone())
                .unwrap_or_default();
            let insert = if dotted {
                format!("{}.", i.name)
            } else {
                i.name.clone()
            };
            self.push(
                &i.name,
                CompletionKind::Mapping,
                Some(EntityRef::Instance(i.id.raw())),
                insert,
                55,
                Some(format!("instance of {comp}")),
            );
        }
    }
    fn ports_of_instance(&mut self, system: &BehaviorSystem, instance: &str, kinds: &[PortKind]) {
        let Some(i) = system.instances.values().find(|i| i.name == instance) else {
            return;
        };
        let Some(c) = system.components.get(&i.component) else {
            return;
        };
        for p in c.interface.ports.values() {
            if !kinds.contains(&p.kind) {
                continue;
            }
            let word = match p.kind {
                PortKind::Required => "requires",
                PortKind::Provided => "provides",
                PortKind::Parameter => "param",
            };
            self.push(
                &p.name,
                CompletionKind::Mapping,
                Some(EntityRef::Port {
                    component: c.id.raw(),
                    port: p.id.raw(),
                }),
                p.name.clone(),
                55,
                Some(format!("{word} port of {}", c.name)),
            );
        }
    }
    fn components(&mut self, system: &BehaviorSystem) {
        for c in system.components.values() {
            let ports = c.interface.ports.len();
            self.push(
                &c.name,
                CompletionKind::Concept,
                Some(EntityRef::Component(c.id.raw())),
                c.name.clone(),
                55,
                Some(format!("component with {ports} port(s)")),
            );
        }
    }
    fn device_kinds(&mut self) {
        for k in DEVICE_KINDS {
            let name = bdl_text::build::device_kind_name(*k);
            self.push(
                name,
                CompletionKind::Representation,
                None,
                name.to_owned(),
                50,
                Some("device kind".into()),
            );
        }
    }
}

/// Candidates at `offset` of a workspace document outside formula
/// bodies: the slot the line is filling, in the scope the position is in.
pub fn text_completions(
    world: &TextWorld,
    document: DocumentId,
    source: &str,
    prefix: String,
    replace: TextRange,
) -> Vec<SemanticCompletion> {
    let system = &world.system;
    let scope = world.component_at(document, replace.start);
    let line_start = source[..replace.start as usize]
        .rfind('\n')
        .map(|i| i + 1)
        .unwrap_or(0);
    let raw = &source[line_start..replace.start as usize];
    let before = raw.trim();
    let line = Line {
        before,
        words: before.split_whitespace().collect(),
    };
    let after_at = raw.ends_with('@');
    let after_dot = raw.ends_with('.');
    let design_in_scope: &Design = match scope {
        Some(c) => system
            .components
            .get(&c)
            .map(|c| &c.body)
            .unwrap_or(&system.base),
        None => &system.base,
    };
    let mut out = Out {
        prefix,
        replace,
        items: Vec::new(),
    };

    // `@|` → clocks in scope.
    if after_at {
        out.clocks_of(design_in_scope, scope.is_none());
        return out.items;
    }

    // A component body that is not instantiated has no flat formula to
    // delegate to: a definition line still gets the body's names.
    if let Some(c) = scope {
        if line.before.contains('=')
            && !line.before.contains(':')
            && !matches!(line.head(), "bind" | "drive" | "instance")
        {
            body_formula_names(&mut out, system, c);
            return out.items;
        }
    }

    // Item start.
    if line.words.is_empty() {
        let items = if scope.is_some() {
            COMPONENT_ITEMS
        } else {
            MODULE_ITEMS
        };
        for k in items {
            out.keyword(k);
        }
        return out.items;
    }

    match line.head() {
        "use" if scope.is_some() => match line.words.get(1) {
            None => {
                out.keyword("concept");
                out.keyword("output");
            }
            Some(&"concept") => out.concepts_of_base(system, scope),
            Some(&"output") => out.outputs_of(&system.base, true),
            _ => {}
        },
        "param" if scope.is_some() => {
            if line.words.len() == 1 {
                out.keyword("clock");
            } else if line.before.contains(':') {
                out.concepts_of(design_in_scope);
            }
        }
        "requires" | "provides" if scope.is_some() => {
            if line.before.contains(':') {
                out.concepts_of(design_in_scope);
            }
        }
        "concept" => {
            if line.before.contains(':') {
                out.representations();
            }
        }
        "mapping" | "output" => {
            if line.before.contains(':') || line.ends_with_word("->") {
                out.concepts_of(design_in_scope);
            }
        }
        "drive" => {
            if line.before.contains('=') {
                out.mappings_of(design_in_scope, scope.is_none());
            } else {
                out.outputs_of(design_in_scope, scope.is_none());
            }
        }
        "device" => {
            if line.ends_with_word("for") {
                out.outputs_of(design_in_scope, scope.is_none());
            } else if line.before.contains(':') && !line.words.contains(&"for") {
                out.device_kinds();
            } else if line.words.len() >= 4 && !line.before.contains('{') {
                out.keyword("for");
            }
        }
        "instance" if scope.is_none() => {
            if line.before.contains('{') {
                instance_arguments(&mut out, system, &line);
            } else if line.before.contains(':') {
                out.components(system);
            }
        }
        "bind" if scope.is_none() => {
            if after_dot {
                if let Some(inst) = line.words.last().and_then(|w| w.strip_suffix('.')) {
                    let kinds: &[PortKind] = if line.before.contains('=') {
                        &[PortKind::Provided, PortKind::Required]
                    } else {
                        &[PortKind::Required, PortKind::Provided]
                    };
                    out.ports_of_instance(system, inst, kinds);
                }
            } else if line.words.len() >= 3
                && line.before.contains('=')
                && !line.ends_with_word("=")
            {
                out.keyword("init");
            } else {
                out.instances(system, true);
                out.mappings_of(&system.base, true);
            }
        }
        "export" if scope.is_none() => {
            if after_dot {
                if let Some(inst) = line.words.last().and_then(|w| w.strip_suffix('.')) {
                    out.ports_of_instance(system, inst, &[PortKind::Required, PortKind::Provided]);
                }
            } else if line.words.len() == 1 {
                out.instances(system, true);
            } else if line.words.len() == 2 {
                out.keyword("as");
            }
        }
        _ => {}
    }
    out.items
}

impl Out {
    /// System concepts a component may `use`, minus the ones it already
    /// shares.
    fn concepts_of_base(&mut self, system: &BehaviorSystem, scope: Option<ComponentId>) {
        let shared: Vec<bdl_model::SemanticId> = scope
            .and_then(|c| system.components.get(&c))
            .map(|c| c.shared_concepts.values().copied().collect())
            .unwrap_or_default();
        for c in system.base.concepts.values() {
            if shared.contains(&c.id) {
                continue;
            }
            self.concept(
                &c.name,
                Some(EntityRef::Concept(c.id)),
                c.representation.clone(),
            );
        }
    }
}

/// Inside `instance n : C { … }`: the arguments the interface takes
/// (clock parameters, parameter ports) that are not given yet, or after
/// `= ` the system clocks for a clock parameter.
fn instance_arguments(out: &mut Out, system: &BehaviorSystem, line: &Line<'_>) {
    let comp_name = line
        .words
        .iter()
        .position(|w| *w == ":")
        .and_then(|i| line.words.get(i + 1))
        .map(|w| w.trim_end_matches('{'))
        .unwrap_or("");
    let Some(c) = system.components.values().find(|c| c.name == comp_name) else {
        return;
    };
    let brace = line
        .before
        .find('{')
        .map(|i| i + 1)
        .unwrap_or(line.before.len());
    let args = &line.before[brace..];
    let last = args.rsplit(',').next().unwrap_or("");
    if last.contains('=') {
        // The value: a system clock for a clock parameter, nothing we can
        // guess for a constant.
        let name = last.split('=').next().unwrap_or("").trim();
        let is_clock = c
            .interface
            .clock_params
            .iter()
            .any(|k| c.body.clocks.get(k).is_some_and(|k| k.name == name));
        if is_clock {
            out.clocks_of(&system.base, true);
        }
        return;
    }
    let given: Vec<&str> = args
        .split(',')
        .filter_map(|a| a.split('=').next())
        .map(str::trim)
        .collect();
    for k in &c.interface.clock_params {
        if let Some(clock) = c.body.clocks.get(k) {
            if !given.contains(&clock.name.as_str()) {
                let insert = format!("{} = ", clock.name);
                out.push(
                    &clock.name,
                    CompletionKind::Keyword,
                    None,
                    insert,
                    60,
                    Some("clock parameter".into()),
                );
            }
        }
    }
    for p in c.interface.ports.values() {
        if p.kind == PortKind::Parameter && !given.contains(&p.name.as_str()) {
            let insert = format!("{} = ", p.name);
            out.push(
                &p.name,
                CompletionKind::Mapping,
                Some(EntityRef::Port {
                    component: c.id.raw(),
                    port: p.id.raw(),
                }),
                insert,
                60,
                Some("parameter".into()),
            );
        }
    }
}

/// The names a formula in `component`'s body may use: its relationships
/// and its concepts, plus the formula keywords.
fn body_formula_names(out: &mut Out, system: &BehaviorSystem, component: ComponentId) {
    let Some(c) = system.components.get(&component) else {
        return;
    };
    out.mappings_of(&c.body, false);
    for k in ["if", "then", "else", "true", "false", "delay", "sync"] {
        out.push(k, CompletionKind::Keyword, None, k.to_owned(), 20, None);
    }
    for u in bdl_elab::units::UNITS {
        out.push(
            u.symbol,
            CompletionKind::Unit,
            None,
            u.symbol.to_owned(),
            10,
            None,
        );
    }
}
