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
                Some(Definition::Formula { source }) => {
                    details.push(detail(
                        if snapshot.is_drafted(id) {
                            "definition (draft)"
                        } else {
                            "definition"
                        },
                        source.trim(),
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
    })
}
