//! Hover: the short, structured card for one entity.
//!
//! Structured sections, not Markdown: the LSP adapter renders them to
//! `MarkupContent`, Studio to its inspector rows, both from the same
//! fields.  Hover is the *short* answer; [`crate::explain`] is the long
//! one.

use bdl_check::pretty;
use bdl_compiler::MappingStatus;
use bdl_ide_db::{AnalysisSnapshot, EntityKind, EntityRef};
use bdl_model::surface::{Definition, Representation};
use bdl_output::OutputState;
use serde::{Deserialize, Serialize};

/// Where an entity stands.  The mapping ladder, the concept binding, the
/// sink state, or nothing to say.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityStatus {
    // mappings
    Declared,
    Open,
    Invalid,
    TypeValid,
    TemporallyValid,
    ClockConsistent,
    // concepts
    Unbound,
    Bound,
    // outputs
    OutputOpen,
    Undriven,
    Driven,
    IllFormed,
    Conflict,
    // clocks, devices, project
    Plain,
}

impl EntityStatus {
    fn of_mapping(s: MappingStatus) -> EntityStatus {
        match s {
            MappingStatus::Declared => EntityStatus::Declared,
            MappingStatus::Open => EntityStatus::Open,
            MappingStatus::Invalid => EntityStatus::Invalid,
            MappingStatus::TypeValid => EntityStatus::TypeValid,
            MappingStatus::TemporallyValid => EntityStatus::TemporallyValid,
            MappingStatus::ClockConsistent => EntityStatus::ClockConsistent,
        }
    }

    /// Product wording.
    pub fn label(self) -> &'static str {
        match self {
            EntityStatus::Declared => "declared, no definition yet",
            EntityStatus::Open => "defined, waiting on an open concept",
            EntityStatus::Invalid => "the definition does not check",
            EntityStatus::TypeValid => "type-valid",
            EntityStatus::TemporallyValid => "temporally valid",
            EntityStatus::ClockConsistent => "clock-consistent",
            EntityStatus::Unbound => "no representation chosen yet",
            EntityStatus::Bound => "representation bound",
            EntityStatus::OutputOpen => "no timing domain yet",
            EntityStatus::Undriven => "undriven",
            EntityStatus::Driven => "driven",
            EntityStatus::IllFormed => "driven by an ill-formed edge",
            EntityStatus::Conflict => "contested by several drivers",
            EntityStatus::Plain => "",
        }
    }

    /// Whether this is a legal, unfinished state rather than an error.
    pub fn is_open(self) -> bool {
        matches!(
            self,
            EntityStatus::Declared
                | EntityStatus::Open
                | EntityStatus::Unbound
                | EntityStatus::OutputOpen
                | EntityStatus::Undriven
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoverDetail {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticHover {
    pub entity: EntityRef,
    pub kind: EntityKind,
    /// The display name.
    pub title: String,
    /// The surface signature (`dimByTilt : Tilt -> Brightness`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// The kernel type view (`sem Tilt → sem Brightness`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_type: Option<String>,
    /// The representation (a concept's, or what a mapping produces).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub representation: Option<String>,
    pub status: EntityStatus,
    pub details: Vec<HoverDetail>,
    /// The description the designer wrote, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
}

fn detail(label: &str, value: impl Into<String>) -> HoverDetail {
    HoverDetail {
        label: label.into(),
        value: value.into(),
    }
}

fn rep_name(r: Representation) -> String {
    match r {
        Representation::Quantity { dim } => pretty::describe_dim(dim),
        Representation::Boolean => "true or false".into(),
        Representation::Count => "a count".into(),
    }
}

/// The hover card for an entity; `None` when the entity does not exist
/// in this snapshot.
pub fn hover(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Option<SemanticHover> {
    let design = &snapshot.effective().design;
    let analysis = snapshot.analysis();
    let ir = &analysis.ir;
    let cname = |id| {
        design
            .concepts
            .get(&id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("{id}"))
    };
    let optional = |s: &str| {
        if s.is_empty() {
            None
        } else {
            Some(s.to_owned())
        }
    };
    Some(match entity {
        EntityRef::Concept(id) => {
            let c = design.concepts.get(&id)?;
            let users = snapshot.index().references_to(entity).len();
            let mut details = vec![detail("used by", format!("{users} reference(s)"))];
            if let Some(r) = c.representation {
                details.push(detail(
                    "kernel",
                    format!(
                        "Θ {id} = {}",
                        pretty::kernel(&bdl_elab::representation_ty(r))
                    ),
                ));
            }
            SemanticHover {
                entity,
                kind: EntityKind::Concept,
                title: c.name.clone(),
                signature: Some(match c.representation {
                    Some(r) => format!(
                        "concept {} : {}",
                        c.name,
                        bdl_ide_db::textual::representation_name(r)
                    ),
                    None => format!("concept {}", c.name),
                }),
                semantic_type: Some(format!("sem {}", c.name)),
                representation: c.representation.map(rep_name),
                status: if c.representation.is_some() {
                    EntityStatus::Bound
                } else {
                    EntityStatus::Unbound
                },
                details,
                explanation: optional(&c.description),
            }
        }
        EntityRef::Mapping(id) => {
            let m = design.mappings.get(&id)?;
            let a = analysis.mappings.get(&id);
            let inputs: Vec<String> = m.signature.inputs.iter().map(|c| cname(*c)).collect();
            let mut sig = inputs.clone();
            sig.push(cname(m.signature.output));
            let mut details = Vec::new();
            match &m.definition {
                Some(Definition::Formula { source })
                | Some(Definition::ScopedFormula { source, .. }) => {
                    details.push(detail(
                        if snapshot.is_drafted(id) {
                            "definition (draft)"
                        } else {
                            "definition"
                        },
                        source.trim(),
                    ));
                }
                Some(Definition::Reference { target, transport }) => {
                    let name = design
                        .mappings
                        .get(target)
                        .map(|t| t.name.clone())
                        .unwrap_or_else(|| target.to_string());
                    details.push(detail(
                        "definition",
                        match transport {
                            None => format!("bound to {name}"),
                            Some(t) => format!(
                                "bound to {name}, carried across domains (initially {})",
                                t.init.trim()
                            ),
                        },
                    ));
                }
                None => details.push(detail("definition", "none — an open declaration")),
            }
            if let Some(a) = a {
                if let Some(t) = &a.inferred_type {
                    details.push(detail("inferred", pretty::describe(ir, t)));
                }
                let errors = a.diagnostics.iter().filter(|d| d.is_error()).count();
                if errors > 0 {
                    details.push(detail("diagnostics", format!("{errors} error(s)")));
                }
            }
            if let Some(c) = m.clock {
                details.push(detail(
                    "domain",
                    design
                        .clocks
                        .get(&c)
                        .map(|x| x.name.clone())
                        .unwrap_or_else(|| c.to_string()),
                ));
            }
            if let Some(o) = m.drives {
                details.push(detail(
                    "drives",
                    design
                        .outputs
                        .get(&o)
                        .map(|x| x.name.clone())
                        .unwrap_or_else(|| o.to_string()),
                ));
            }
            SemanticHover {
                entity,
                kind: EntityKind::Mapping,
                title: m.name.clone(),
                signature: Some(format!("mapping {} : {}", m.name, sig.join(" -> "))),
                semantic_type: a.map(|a| pretty::kernel(&a.interface.expected_type)),
                representation: design
                    .concepts
                    .get(&m.signature.output)
                    .and_then(|c| c.representation)
                    .map(|r| format!("produces {}", rep_name(r))),
                status: a
                    .map(|a| EntityStatus::of_mapping(a.status))
                    .unwrap_or(EntityStatus::Declared),
                details,
                explanation: optional(&m.description),
            }
        }
        EntityRef::Output(id) => {
            let o = design.outputs.get(&id)?;
            let state = analysis.outputs.states.get(&id);
            let status = match (o.clock, state) {
                (None, _) => EntityStatus::OutputOpen,
                (_, Some(OutputState::Driven)) => EntityStatus::Driven,
                (_, Some(OutputState::IllFormed)) => EntityStatus::IllFormed,
                (_, Some(OutputState::Conflict)) => EntityStatus::Conflict,
                _ => EntityStatus::Undriven,
            };
            let mut details = vec![detail("accepts", cname(o.accepts))];
            if let Some(c) = o.clock {
                details.push(detail(
                    "domain",
                    design
                        .clocks
                        .get(&c)
                        .map(|x| x.name.clone())
                        .unwrap_or_else(|| c.to_string()),
                ));
            }
            details.push(detail("required", if o.required { "yes" } else { "no" }));
            let drivers: Vec<String> = design.drivers_of(id).map(|m| m.name.clone()).collect();
            if !drivers.is_empty() {
                details.push(detail("driver(s)", drivers.join(", ")));
            }
            SemanticHover {
                entity,
                kind: EntityKind::Output,
                title: o.name.clone(),
                signature: Some(format!("output {} : {}", o.name, cname(o.accepts))),
                semantic_type: Some(format!("Ω {id} accepts sem {}", cname(o.accepts))),
                representation: None,
                status,
                details,
                explanation: optional(&o.description),
            }
        }
        EntityRef::Clock(id) => {
            let c = design.clocks.get(&id)?;
            let members: Vec<String> = design
                .mappings
                .values()
                .filter(|m| m.clock == Some(id))
                .map(|m| m.name.clone())
                .collect();
            let sinks: Vec<String> = design
                .outputs
                .values()
                .filter(|o| o.clock == Some(id))
                .map(|o| o.name.clone())
                .collect();
            SemanticHover {
                entity,
                kind: EntityKind::Clock,
                title: c.name.clone(),
                signature: Some(format!("clock {}", c.name)),
                semantic_type: None,
                representation: None,
                status: EntityStatus::Plain,
                details: vec![
                    detail("mappings", members.join(", ")),
                    detail("outputs", sinks.join(", ")),
                ],
                explanation: Some(
                    "A timing domain says which values update together; never a rate.".into(),
                ),
            }
        }
        EntityRef::Device(id) => {
            let d = design.devices.get(&id)?;
            let mut details = vec![detail("kind", format!("{:?}", d.kind))];
            if let Some(o) = d.output {
                details.push(detail(
                    "realises",
                    design
                        .outputs
                        .get(&o)
                        .map(|x| x.name.clone())
                        .unwrap_or_else(|| o.to_string()),
                ));
            }
            SemanticHover {
                entity,
                kind: EntityKind::Device,
                title: d.name.clone(),
                signature: None,
                semantic_type: None,
                representation: None,
                status: EntityStatus::Plain,
                details,
                explanation: None,
            }
        }
        EntityRef::Requirement { device, index } => {
            let d = design.devices.get(&device)?;
            SemanticHover {
                entity,
                kind: EntityKind::Requirement,
                title: format!("{} / requirement {index}", d.name),
                signature: None,
                semantic_type: None,
                representation: None,
                status: EntityStatus::Plain,
                details: d
                    .fixed_pins
                    .get(&index)
                    .map(|p| vec![detail("fixed pin", p.clone())])
                    .unwrap_or_default(),
                explanation: None,
            }
        }
        EntityRef::Project => SemanticHover {
            entity,
            kind: EntityKind::Project,
            title: design.name.clone(),
            signature: None,
            semantic_type: None,
            representation: None,
            status: EntityStatus::Plain,
            details: vec![
                detail("concepts", design.concepts.len().to_string()),
                detail("mappings", design.mappings.len().to_string()),
                detail(
                    "outputs complete",
                    if analysis.output_complete {
                        "yes"
                    } else {
                        "no"
                    },
                ),
            ],
            explanation: None,
        },
        EntityRef::Component(_)
        | EntityRef::Port { .. }
        | EntityRef::Instance(_)
        | EntityRef::Binding(_)
        | EntityRef::Export(_) => system_hover(snapshot, entity)?,
    })
}

/// A card for a system's own entities (a text workspace): the component's
/// promise, an instance's component and arguments, a port's contract.
fn system_hover(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Option<SemanticHover> {
    let world = snapshot.text()?;
    let system = &world.system;
    let detail = |label: &str, value: String| HoverDetail {
        label: label.to_owned(),
        value,
    };
    Some(match entity {
        EntityRef::Component(raw) => {
            let c = system
                .components
                .get(&bdl_system::ComponentId::from_raw(raw))?;
            let instances = system
                .instances
                .values()
                .filter(|i| i.component == c.id)
                .count();
            let mut details = vec![detail("instances", instances.to_string())];
            for p in c.interface.ports.values() {
                let word = match p.kind {
                    bdl_system::PortKind::Required => "requires",
                    bdl_system::PortKind::Provided => "provides",
                    bdl_system::PortKind::Parameter => "param",
                };
                let concept = c
                    .body
                    .concepts
                    .get(&p.contract.signature.output)
                    .map(|x| x.name.clone())
                    .unwrap_or_default();
                details.push(detail(word, format!("{} : {concept}", p.name)));
            }
            for k in &c.interface.clock_params {
                if let Some(clock) = c.body.clocks.get(k) {
                    details.push(detail("timing parameter", clock.name.clone()));
                }
            }
            SemanticHover {
                entity,
                kind: EntityKind::Component,
                title: c.name.clone(),
                signature: None,
                semantic_type: None,
                representation: None,
                status: EntityStatus::Plain,
                details,
                explanation: if c.description.is_empty() {
                    None
                } else {
                    Some(c.description.clone())
                },
            }
        }
        EntityRef::Port { component, port } => {
            let c = system
                .components
                .get(&bdl_system::ComponentId::from_raw(component))?;
            let p = c.interface.ports.get(&bdl_system::PortId::from_raw(port))?;
            let word = match p.kind {
                bdl_system::PortKind::Required => "required port",
                bdl_system::PortKind::Provided => "provided port",
                bdl_system::PortKind::Parameter => "parameter port",
            };
            let concept = c
                .body
                .concepts
                .get(&p.contract.signature.output)
                .map(|x| x.name.clone())
                .unwrap_or_default();
            let timing = match p.contract.clock {
                bdl_system::ClockContract::Agnostic => "any timing domain".to_owned(),
                bdl_system::ClockContract::Parameter { clock } => format!(
                    "updates in {} (a timing parameter)",
                    c.body
                        .clocks
                        .get(&clock)
                        .map(|k| k.name.clone())
                        .unwrap_or_default()
                ),
                bdl_system::ClockContract::Private { clock } => format!(
                    "updates in its own {}",
                    c.body
                        .clocks
                        .get(&clock)
                        .map(|k| k.name.clone())
                        .unwrap_or_default()
                ),
            };
            SemanticHover {
                entity,
                kind: EntityKind::Port,
                title: format!("{}.{}", c.name, p.name),
                signature: Some(format!("{word} of {}", c.name)),
                semantic_type: Some(concept),
                representation: None,
                status: EntityStatus::Plain,
                details: vec![detail("timing", timing)],
                explanation: if p.description.is_empty() {
                    None
                } else {
                    Some(p.description.clone())
                },
            }
        }
        EntityRef::Instance(raw) => {
            let i = system
                .instances
                .get(&bdl_system::ComponentInstanceId::from_raw(raw))?;
            let c = system.components.get(&i.component)?;
            let mut details = vec![detail("component", c.name.clone())];
            for (local, sys) in &i.clock_bindings {
                details.push(detail(
                    &c.body
                        .clocks
                        .get(local)
                        .map(|k| k.name.clone())
                        .unwrap_or_default(),
                    system
                        .base
                        .clocks
                        .get(sys)
                        .map(|k| k.name.clone())
                        .unwrap_or_default(),
                ));
            }
            for (port, value) in &i.parameter_bindings {
                details.push(detail(
                    &c.interface
                        .ports
                        .get(port)
                        .map(|p| p.name.clone())
                        .unwrap_or_default(),
                    value.source.clone(),
                ));
            }
            let bound = system
                .bindings
                .values()
                .filter(|b| b.destination.instance() == Some(i.id))
                .count();
            details.push(detail("bound ports", bound.to_string()));
            SemanticHover {
                entity,
                kind: EntityKind::Instance,
                title: i.name.clone(),
                signature: Some(format!("instance {} : {}", i.name, c.name)),
                semantic_type: None,
                representation: None,
                status: EntityStatus::Plain,
                details,
                explanation: None,
            }
        }
        EntityRef::Binding(raw) => {
            let b = system.bindings.get(&bdl_system::BindingId::from_raw(raw))?;
            let end = |e: bdl_system::BindingEnd| bdl_text::print::binding_end(system, e);
            let mut details = vec![
                detail("from", end(b.source)),
                detail("to", end(b.destination)),
            ];
            match &b.transport {
                Some(t) => {
                    details.push(detail("carried across domains, starts at", t.init.clone()))
                }
                None => details.push(detail("transport", "direct".into())),
            }
            SemanticHover {
                entity,
                kind: EntityKind::Binding,
                title: format!("bind {} = {}", end(b.destination), end(b.source)),
                signature: None,
                semantic_type: None,
                representation: None,
                status: EntityStatus::Plain,
                details,
                explanation: Some(
                    "A binding converts nothing: both ends carry the same concept, by identity."
                        .into(),
                ),
            }
        }
        EntityRef::Export(raw) => {
            let e = system.exports.get(&bdl_system::ExportId::from_raw(raw))?;
            SemanticHover {
                entity,
                kind: EntityKind::Export,
                title: e.name.clone(),
                signature: Some(format!(
                    "exported {}",
                    bdl_text::print::binding_end(system, bdl_system::BindingEnd::Port(e.port))
                )),
                semantic_type: None,
                representation: None,
                status: EntityStatus::Plain,
                details: Vec::new(),
                explanation: None,
            }
        }
        _ => return None,
    })
}
