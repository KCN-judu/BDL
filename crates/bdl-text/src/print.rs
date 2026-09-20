//! The canonical spelling of one item of the model (TEXTUAL_SYNTAX §10,
//! §14.6).  Used by write-back for items that changed or are new, and by
//! `render_system` for a whole project (conversion from JSON, tests).
//! Formula text is the authored text and is never reprinted.

use crate::build::{device_kind_name, representation_name};
use bdl_model::surface::{
    ClockDomain, Concept, Definition, Design, DeviceBinding, MappingBlock, PhysicalOutput,
};
use bdl_system::{
    BehaviorComponent, BehaviorSystem, Binding, BindingEnd, ComponentInstance, Export, PortKind,
};

/// A name that the grammar cannot spell (a keyword): the item is still
/// printed, with the offending name, and the loader will report it.
fn ident(name: &str) -> String {
    name.to_owned()
}

/// A description as the `///` lines above an item.
fn doc(description: &str, item: String) -> String {
    if description.trim().is_empty() {
        return item;
    }
    let mut s = String::new();
    for line in description.lines() {
        s.push_str("/// ");
        s.push_str(line);
        s.push('\n');
    }
    s.push_str(&item);
    s
}

pub fn concept(c: &Concept) -> String {
    let keyword = if c.ordered {
        "ordered concept"
    } else {
        "concept"
    };
    let item = match &c.representation {
        Some(r) => format!("{keyword} {} : {}", ident(&c.name), representation_name(r)),
        None => format!("{keyword} {}", ident(&c.name)),
    };
    doc(&c.description, item)
}

pub fn clock(c: &ClockDomain) -> String {
    format!("clock {}", ident(&c.name))
}

fn concept_name(design: &Design, id: bdl_model::SemanticId) -> String {
    design
        .concepts
        .get(&id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| format!("Concept{}", id.raw()))
}

fn clock_name(design: &Design, id: bdl_model::ClockId) -> String {
    design
        .clocks
        .get(&id)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| format!("clock{}", id.raw()))
}

fn output_name(design: &Design, id: bdl_model::OutputId) -> String {
    design
        .outputs
        .get(&id)
        .map(|o| o.name.clone())
        .unwrap_or_else(|| format!("output{}", id.raw()))
}

/// The signature and the clock tag of a mapping, in the preferred
/// spelling: `A -> B -> C`, and `() -> B` for the unit domain
/// (docs/spec/textual-syntax.md §4.1 — the output-only shorthand is
/// compatibility syntax and is never generated).  A port keeps the bare
/// output (`explicit_unit: false`): its grammar has no domain to spell.
fn signature_and_tag(design: &Design, m: &MappingBlock, explicit_unit: bool) -> String {
    let mut ty: Vec<String> = m
        .signature
        .inputs
        .iter()
        .map(|i| concept_name(design, *i))
        .collect();
    if ty.is_empty() && explicit_unit {
        ty.push("()".to_string());
    }
    ty.push(concept_name(design, m.signature.output));
    let mut s = ty.join(" -> ");
    if let Some(c) = m.clock {
        s.push_str(" @");
        s.push_str(&clock_name(design, c));
    }
    s
}

/// The definition lines of a mapping, if it has a formula.
fn definition(design: &Design, m: &MappingBlock) -> Option<String> {
    let Some(Definition::Formula { source }) = &m.definition else {
        return None;
    };
    let params: Vec<String> = m
        .signature
        .inputs
        .iter()
        .enumerate()
        .map(|(i, c)| {
            m.parameters
                .get(i)
                .filter(|p| !p.is_empty())
                .cloned()
                .unwrap_or_else(|| concept_name(design, *c))
        })
        .collect();
    let body = source.trim();
    if body.contains('\n') {
        let indented: Vec<String> = body.lines().map(|l| format!("  {l}")).collect();
        Some(format!(
            "{}({}) =\n{}",
            ident(&m.name),
            params.join(", "),
            indented.join("\n")
        ))
    } else {
        Some(format!(
            "{}({}) =\n  {}",
            ident(&m.name),
            params.join(", "),
            body
        ))
    }
}

pub fn mapping(design: &Design, m: &MappingBlock) -> String {
    let mut s = format!(
        "mapping {} : {}",
        ident(&m.name),
        signature_and_tag(design, m, true)
    );
    if let Some(d) = definition(design, m) {
        s.push('\n');
        s.push_str(&d);
    }
    doc(&m.description, s)
}

pub fn output(design: &Design, o: &PhysicalOutput) -> String {
    let mut s = format!(
        "output {} : {}",
        ident(&o.name),
        concept_name(design, o.accepts)
    );
    if let Some(c) = o.clock {
        s.push_str(" @");
        s.push_str(&clock_name(design, c));
    }
    if !o.required {
        s.push_str(" optional");
    }
    doc(&o.description, s)
}

/// `drive output = driver` for a mapping that drives.
pub fn drive(design: &Design, m: &MappingBlock) -> Option<String> {
    let o = m.drives?;
    Some(format!(
        "drive {} = {}",
        output_name(design, o),
        ident(&m.name)
    ))
}

pub fn device(design: &Design, d: &DeviceBinding) -> String {
    let mut s = format!("device {} : {}", ident(&d.name), device_kind_name(d.kind));
    if let Some(o) = d.output {
        s.push_str(" for ");
        s.push_str(&output_name(design, o));
    }
    let mut body: Vec<String> = Vec::new();
    if let Some(profile) = &d.realization {
        body.push(format!("realization {profile}"));
    }
    body.extend(d.fixed_pins.iter().map(|(i, p)| format!("pin {i} = {p}")));
    if !body.is_empty() {
        s.push_str(&format!(" {{ {} }}", body.join(", ")));
    }
    s
}

/// A port declaration: `requires n : T @c` plus its definition when the
/// backing relationship has one.
pub fn port(c: &BehaviorComponent, port: &bdl_system::Port) -> String {
    let word = match port.kind {
        PortKind::Required => "requires",
        PortKind::Provided => "provides",
        PortKind::Parameter => "param",
    };
    let Some(m) = c.body.mappings.get(&port.decl) else {
        return format!("{word} {} : Concept?", ident(&port.name));
    };
    let mut s = format!(
        "{word} {} : {}",
        ident(&port.name),
        signature_and_tag(&c.body, m, false)
    );
    if let Some(d) = definition(&c.body, m) {
        s.push('\n');
        s.push_str(&d);
    }
    doc(&port.description, s)
}

/// One body item of a component, by kind, in canonical order helpers.
pub fn body_items(c: &BehaviorComponent, system: &BehaviorSystem) -> Vec<String> {
    let mut out = Vec::new();
    for x in c.body.concepts.values() {
        match c.shared_concepts.get(&x.id) {
            Some(sys) => out.push(format!("use concept {}", concept_name(&system.base, *sys))),
            None => out.push(concept(x)),
        }
    }
    for k in c.body.clocks.values() {
        if c.interface.is_clock_param(k.id) {
            out.push(format!("param clock {}", ident(&k.name)));
        } else {
            out.push(clock(k));
        }
    }
    for p in c.interface.ports.values() {
        out.push(port(c, p));
    }
    for m in c.body.mappings.values() {
        if c.interface.port_for_decl(m.id).is_some() {
            continue;
        }
        out.push(mapping(&c.body, m));
    }
    for o in c.body.outputs.values() {
        match c.external_outputs.get(&o.id) {
            Some(sys) => out.push(format!("use output {}", output_name(&system.base, *sys))),
            None => out.push(output(&c.body, o)),
        }
    }
    for m in c.body.mappings.values() {
        if let Some(d) = drive(&c.body, m) {
            out.push(d);
        }
    }
    for d in c.body.devices.values() {
        out.push(device(&c.body, d));
    }
    out
}

pub fn component(c: &BehaviorComponent, system: &BehaviorSystem) -> String {
    let items = body_items(c, system);
    let mut s = format!("component {} {{", ident(&c.name));
    for item in items {
        s.push('\n');
        for line in item.lines() {
            s.push_str("  ");
            s.push_str(line);
            s.push('\n');
        }
    }
    if s.ends_with('\n') {
        s.pop();
    }
    s.push_str("\n}");
    doc(&c.description, s)
}

pub fn instance(system: &BehaviorSystem, i: &ComponentInstance) -> String {
    let comp_name = system
        .components
        .get(&i.component)
        .map(|c| c.name.clone())
        .unwrap_or_else(|| format!("Component{}", i.component.raw()));
    let mut args = Vec::new();
    if let Some(c) = system.components.get(&i.component) {
        for (local, sys) in &i.clock_bindings {
            args.push(format!(
                "{} = {}",
                clock_name(&c.body, *local),
                clock_name(&system.base, *sys)
            ));
        }
        for (port, value) in &i.parameter_bindings {
            let name = c
                .interface
                .ports
                .get(port)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("port{}", port.raw()));
            args.push(format!("{name} = {}", value.source.trim()));
        }
    }
    let mut s = format!("instance {} : {}", ident(&i.name), comp_name);
    if !args.is_empty() {
        s.push_str(&format!(" {{ {} }}", args.join(", ")));
    }
    s
}

pub fn binding_end(system: &BehaviorSystem, e: BindingEnd) -> String {
    match e {
        BindingEnd::Base { decl } => system
            .base
            .mappings
            .get(&decl)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| format!("decl{}", decl.raw())),
        BindingEnd::Port(r) => {
            let inst = system
                .instances
                .get(&r.instance)
                .map(|i| i.name.clone())
                .unwrap_or_else(|| format!("instance{}", r.instance.raw()));
            let port = system
                .component_of(r.instance)
                .and_then(|c| c.interface.ports.get(&r.port))
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("port{}", r.port.raw()));
            format!("{inst}.{port}")
        }
    }
}

pub fn binding(system: &BehaviorSystem, b: &Binding) -> String {
    let mut s = format!(
        "bind {} = {}",
        binding_end(system, b.destination),
        binding_end(system, b.source)
    );
    if let Some(t) = &b.transport {
        s.push_str(" init ");
        s.push_str(t.init.trim());
    }
    s
}

pub fn export(system: &BehaviorSystem, e: &Export) -> String {
    format!(
        "export {} as {}",
        binding_end(system, BindingEnd::Port(e.port)),
        ident(&e.name)
    )
}

/// Every top-level item of a system as one module, in canonical order —
/// the rendering a conversion from JSON writes and tests compare.
pub fn render_system(system: &BehaviorSystem) -> String {
    let base = &system.base;
    let sections: Vec<Vec<String>> = vec![
        base.concepts.values().map(concept).collect(),
        base.clocks.values().map(clock).collect(),
        base.mappings.values().map(|m| mapping(base, m)).collect(),
        base.outputs.values().map(|o| output(base, o)).collect(),
        base.mappings
            .values()
            .filter_map(|m| drive(base, m))
            .collect(),
        base.devices.values().map(|d| device(base, d)).collect(),
        system
            .components
            .values()
            .map(|c| component(c, system))
            .collect(),
        system
            .instances
            .values()
            .map(|i| instance(system, i))
            .collect(),
        system
            .bindings
            .values()
            .map(|b| binding(system, b))
            .collect(),
        system.exports.values().map(|e| export(system, e)).collect(),
    ];
    let mut out = String::new();
    for section in sections {
        for item in section {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&item);
            out.push('\n');
        }
    }
    out
}
