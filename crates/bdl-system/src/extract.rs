//! "Package as reusable component" — extraction of a group into a
//! component, one instance and the bindings that reconnect it (FV Phase 8b
//! `Extract.lean`, `ExtractPreservation.lean`).
//!
//! The component's body is the members' relationships *unchanged* plus one
//! unresolved copy of each crossing-in relationship (its required ports);
//! its provided ports are the crossing-out members; every domain the
//! members use is a clock parameter; concepts stay shared; sinks stay
//! external unless the designer internalises them, and drive edges stay
//! with the members that drive them.  The base keeps everything else, with
//! an unresolved copy of each crossing-out member in place of the original;
//! two families of ordinary bindings reconnect the sides.  Nothing here is
//! semantic: the flattened design is analysed by the existing compiler,
//! and its causality verdict is the only gate (never a coarse instance
//! graph — a bidirectional boundary is fine as long as the declarations
//! stay acyclic, `flat_causal`).
//!
//! [`preview_extraction`] computes the same decisions and mutates nothing;
//! the four choices the FV cannot infer (open members as inputs, sinks to
//! internalise, the component's name, the instance's name) are
//! [`ExtractionChoices`].

use crate::boundary::{group_boundary, GroupBoundary};
use crate::flatten::flatten;
use crate::ids::{BehaviorGroupId, ComponentId, ComponentInstanceId, PortId};
use crate::model::*;
use bdl_diagnostics::{Diagnostic, Entity};
use bdl_model::surface::Design;
use bdl_model::{ClockId, DeclId, OutputId, SemanticId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ExtractionChoices {
    /// The component's name; the group's when empty.
    #[serde(default)]
    pub name: String,
    /// The instance's name; derived from the component's when empty.
    #[serde(default)]
    pub instance_name: String,
    /// Open members to keep internal (unresolved inside the body) instead
    /// of exposing as required ports.  Default: every open member becomes
    /// an input.
    #[serde(default)]
    pub keep_internal: Vec<DeclId>,
    /// Sinks driven by members to move into the body as private sinks.
    /// Default: every sink stays the system's, driven through the
    /// component's external-output table.
    #[serde(default)]
    pub internalize_sinks: Vec<OutputId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewPort {
    pub decl: DeclId,
    pub name: String,
    pub kind: PortKind,
    /// The concept the port carries (system identity).
    pub concept: SemanticId,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenMemberDecision {
    pub decl: DeclId,
    pub name: String,
    /// True: becomes a required port; false: stays internal and open.
    pub as_input: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SinkDecision {
    pub output: OutputId,
    pub name: String,
    /// The members that drive it.
    pub drivers: Vec<DeclId>,
    /// True: moved into the body; false: stays the system's sink.
    pub internal: bool,
}

/// What an extraction would do.  A view; nothing is changed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractionPreview {
    pub group: BehaviorGroupId,
    pub name: String,
    pub instance_name: String,
    pub boundary: GroupBoundary,
    pub required: Vec<PreviewPort>,
    pub provided: Vec<PreviewPort>,
    pub open_members: Vec<OpenMemberDecision>,
    pub clocks: Vec<ClockId>,
    pub sinks: Vec<SinkDecision>,
    /// Members that end up private to the component.
    pub private: Vec<DeclId>,
    pub warnings: Vec<Diagnostic>,
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error, Serialize, Deserialize)]
#[serde(tag = "refused", rename_all = "snake_case")]
pub enum ExtractError {
    #[error("unknown group {id}")]
    UnknownGroup { id: BehaviorGroupId },
    #[error("group {id} has no relationships to package")]
    EmptyGroup { id: BehaviorGroupId },
    #[error("a name is required")]
    EmptyName,
    #[error("a component named `{name}` already exists")]
    DuplicateComponentName { name: String },
    #[error("an instance named `{name}` already exists")]
    DuplicateInstanceName { name: String },
    #[error("declaration {decl} is not an open member of the group")]
    NotAnOpenMember { decl: DeclId },
    #[error("output {output} is not driven by a member of the group")]
    NotADrivenSink { output: OutputId },
    #[error("group {id} is inside a component; packaging applies to the system's own design")]
    NotABaseGroup { id: BehaviorGroupId },
}

fn name_of(design: &Design, d: DeclId) -> String {
    design
        .mappings
        .get(&d)
        .map(|m| m.name.clone())
        .unwrap_or_else(|| d.to_string())
}

/// Decide everything an extraction needs, without changing anything.
pub fn preview_extraction(
    snapshot: &SystemSnapshot,
    group: BehaviorGroupId,
    choices: &ExtractionChoices,
) -> Result<ExtractionPreview, ExtractError> {
    let s = &snapshot.system;
    let g = s
        .groups
        .get(&group)
        .ok_or(ExtractError::UnknownGroup { id: group })?;
    // Packaging is defined on the system's own design (the FV residual);
    // a component-local group is organisation only for now (ADR-0019
    // amendment, option A).
    if g.scope != GroupScope::SystemBase {
        return Err(ExtractError::NotABaseGroup { id: group });
    }
    let flat = flatten(snapshot);
    let analysis = bdl_compiler::analyze(&flat.snapshot);
    let boundary = group_boundary(&s.base, &analysis, &g.members);
    if boundary.members.is_empty() {
        return Err(ExtractError::EmptyGroup { id: group });
    }
    let name = if choices.name.trim().is_empty() {
        g.name.clone()
    } else {
        choices.name.trim().to_owned()
    };
    if name.is_empty() {
        return Err(ExtractError::EmptyName);
    }
    if s.components.values().any(|c| c.name == name) {
        return Err(ExtractError::DuplicateComponentName { name });
    }
    let instance_name = if choices.instance_name.trim().is_empty() {
        instance_name_for(&name)
    } else {
        choices.instance_name.trim().to_owned()
    };
    if s.instances.values().any(|i| i.name == instance_name) {
        return Err(ExtractError::DuplicateInstanceName {
            name: instance_name,
        });
    }
    for d in &choices.keep_internal {
        if !boundary.open_members.contains(d) {
            return Err(ExtractError::NotAnOpenMember { decl: *d });
        }
    }
    let base = &s.base;
    let mut sinks: Vec<SinkDecision> = Vec::new();
    for m in &boundary.driven_members {
        let Some(o) = base.mappings.get(m).and_then(|x| x.drives) else {
            continue;
        };
        match sinks.iter_mut().find(|d| d.output == o) {
            Some(d) => d.drivers.push(*m),
            None => sinks.push(SinkDecision {
                output: o,
                name: base
                    .outputs
                    .get(&o)
                    .map(|x| x.name.clone())
                    .unwrap_or_else(|| o.to_string()),
                drivers: vec![*m],
                internal: choices.internalize_sinks.contains(&o),
            }),
        }
    }
    for o in &choices.internalize_sinks {
        if !sinks.iter().any(|d| d.output == *o) {
            return Err(ExtractError::NotADrivenSink { output: *o });
        }
    }
    let open_members: Vec<OpenMemberDecision> = boundary
        .open_members
        .iter()
        .map(|d| OpenMemberDecision {
            decl: *d,
            name: name_of(base, *d),
            as_input: !choices.keep_internal.contains(d),
        })
        .collect();
    let port = |d: DeclId, kind: PortKind| PreviewPort {
        decl: d,
        name: name_of(base, d),
        kind,
        concept: base
            .mappings
            .get(&d)
            .map(|m| m.signature.output)
            .unwrap_or(SemanticId::from_raw(0)),
    };
    let mut required: Vec<PreviewPort> = boundary
        .crossing_in
        .iter()
        .map(|d| port(*d, PortKind::Required))
        .collect();
    required.extend(
        open_members
            .iter()
            .filter(|o| o.as_input)
            .map(|o| port(o.decl, PortKind::Required)),
    );
    let provided: Vec<PreviewPort> = boundary
        .crossing_out
        .iter()
        .map(|d| port(*d, PortKind::Provided))
        .collect();
    let mut clocks: BTreeSet<ClockId> = boundary.clocks.iter().copied().collect();
    for d in &sinks {
        if let Some(c) = base.outputs.get(&d.output).and_then(|o| o.clock) {
            clocks.insert(c);
        }
    }
    let mut warnings = Vec::new();
    if provided.is_empty() && sinks.is_empty() {
        warnings.push(
            Diagnostic::warning(
                "extract.nothing_observable",
                Entity::Project,
                format!("Nothing outside the group reads {name}, and it drives no output."),
            )
            .explain("The component would have no provided port and no physical effect. Package it anyway if it is a work in progress."),
        );
    }
    for o in open_members.iter().filter(|o| !o.as_input) {
        warnings.push(
            Diagnostic::warning(
                "extract.open_member_internal",
                Entity::Mapping { id: o.decl },
                format!(
                    "{} stays open inside {name}: no instance can supply it.",
                    o.name
                ),
            )
            .explain("Only editing the component's source can define it later."),
        );
    }
    let mut private: Vec<DeclId> = boundary.private_candidates.clone();
    private.retain(|d| !required.iter().any(|p| p.decl == *d));
    Ok(ExtractionPreview {
        group,
        name,
        instance_name,
        boundary,
        required,
        provided,
        open_members,
        clocks: clocks.into_iter().collect(),
        sinks,
        private,
        warnings,
    })
}

/// `AdaptiveLamp` → `adaptiveLamp`; a name that is already lower-case
/// gets a `1`.
pub fn instance_name_for(component: &str) -> String {
    let mut chars = component.chars();
    match chars.next() {
        Some(c) if c.is_uppercase() => c.to_lowercase().chain(chars).collect(),
        Some(_) => format!("{component}1"),
        None => "instance".into(),
    }
}

/// What the extraction produced, beyond the outcome.
pub struct Extracted {
    pub component: ComponentId,
    pub instance: ComponentInstanceId,
    pub required: BTreeMap<DeclId, PortId>,
    pub provided: BTreeMap<DeclId, PortId>,
    pub preview: ExtractionPreview,
}

/// Perform the extraction on `s` (the edit model calls this and completes
/// the freshening table afterwards).
pub fn extract_group(
    snapshot: &SystemSnapshot,
    s: &mut BehaviorSystem,
    group: BehaviorGroupId,
    choices: &ExtractionChoices,
) -> Result<Extracted, ExtractError> {
    let preview = preview_extraction(snapshot, group, choices)?;
    let b = &preview.boundary;
    let internal_sinks: BTreeSet<OutputId> = preview
        .sinks
        .iter()
        .filter(|d| d.internal)
        .map(|d| d.output)
        .collect();

    // --- the body: a restriction of the base (FV `restrict`) ---
    let mut body = Design::empty(preview.name.clone());
    let base = s.base.clone();
    let mut concepts: BTreeSet<SemanticId> = BTreeSet::new();
    for d in b.members.iter().chain(b.crossing_in.iter()) {
        if let Some(m) = base.mappings.get(d) {
            concepts.extend(m.signature.inputs.iter().copied());
            concepts.insert(m.signature.output);
        }
    }
    for d in &preview.sinks {
        if let Some(o) = base.outputs.get(&d.output) {
            concepts.insert(o.accepts);
        }
    }
    let mut shared_concepts = BTreeMap::new();
    for c in &concepts {
        if let Some(concept) = base.concepts.get(c) {
            body.concepts.insert(*c, concept.clone());
            shared_concepts.insert(*c, *c);
        }
    }
    for c in &preview.clocks {
        if let Some(clock) = base.clocks.get(c) {
            body.clocks.insert(*c, clock.clone());
        }
    }
    for d in &b.members {
        if let Some(m) = base.mappings.get(d) {
            body.mappings.insert(*d, m.clone());
        }
    }
    for d in &b.crossing_in {
        if let Some(m) = base.mappings.get(d) {
            let mut copy = m.clone();
            copy.definition = None;
            copy.drives = None;
            body.mappings.insert(*d, copy);
        }
    }
    let mut external_outputs = BTreeMap::new();
    for d in &preview.sinks {
        if let Some(o) = base.outputs.get(&d.output) {
            body.outputs.insert(d.output, o.clone());
            if d.internal {
                for dev in base.devices.values().filter(|x| x.output == Some(d.output)) {
                    body.devices.insert(dev.id, dev.clone());
                }
            } else {
                external_outputs.insert(d.output, d.output);
            }
        }
    }
    body.reserve_ids();

    // --- the interface ---
    let interface_clocks = preview.clocks.clone();
    let mut interface = BehaviorInterface {
        ports: BTreeMap::new(),
        clock_params: interface_clocks,
    };
    let mut required = BTreeMap::new();
    let mut provided = BTreeMap::new();
    for p in preview.required.iter().chain(preview.provided.iter()) {
        let Some(m) = body.mappings.get(&p.decl) else {
            continue;
        };
        let id = s.ids.fresh_port();
        interface.ports.insert(
            id,
            Port {
                id,
                name: p.name.clone(),
                description: m.description.clone(),
                kind: p.kind,
                decl: p.decl,
                contract: PortContract::of_declaration(m, &interface),
            },
        );
        match p.kind {
            PortKind::Provided => provided.insert(p.decl, id),
            _ => required.insert(p.decl, id),
        };
    }

    // --- the component and its instance ---
    let component = s.ids.fresh_component();
    s.components.insert(
        component,
        BehaviorComponent {
            id: component,
            name: preview.name.clone(),
            description: s
                .groups
                .get(&group)
                .map(|g| g.description.clone())
                .unwrap_or_default(),
            body,
            interface,
            shared_concepts,
            external_outputs,
            body_stamp: 0,
            interface_stamp: 0,
        },
    );
    let instance = s.ids.fresh_instance();
    s.instances.insert(
        instance,
        ComponentInstance {
            id: instance,
            component,
            name: preview.instance_name.clone(),
            clock_bindings: preview.clocks.iter().map(|c| (*c, *c)).collect(),
            parameter_bindings: BTreeMap::new(),
        },
    );

    // --- the residual base (FV `resid`) ---
    for d in &b.members {
        if b.crossing_out.contains(d) {
            if let Some(m) = s.base.mappings.get_mut(d) {
                m.definition = None;
                m.drives = None;
            }
        } else {
            s.base.mappings.remove(d);
        }
    }
    for o in &internal_sinks {
        s.base.outputs.remove(o);
        s.base.devices.retain(|_, dev| dev.output != Some(*o));
    }

    // --- reconnection (FV `bindings`) ---
    for r in &b.crossing_in {
        if let Some(port) = required.get(r) {
            let id = s.ids.fresh_binding();
            s.bindings.insert(
                id,
                Binding {
                    id,
                    source: BindingEnd::Base { decl: *r },
                    destination: BindingEnd::port(instance, *port),
                    transport: None,
                },
            );
        }
    }
    for p in &b.crossing_out {
        if let Some(port) = provided.get(p) {
            let id = s.ids.fresh_binding();
            s.bindings.insert(
                id,
                Binding {
                    id,
                    source: BindingEnd::port(instance, *port),
                    destination: BindingEnd::Base { decl: *p },
                    transport: None,
                },
            );
        }
    }
    s.groups.remove(&group);
    Ok(Extracted {
        component,
        instance,
        required,
        provided,
        preview,
    })
}
