//! Explain: the long answer about one entity, for an explanation view.
//!
//! Where hover says what a thing is, explain says why it stands where it
//! stands: identity, surface signature, interface type, representation,
//! grant, Core lowering, dependencies, clock, output relation,
//! invalidation state, and the diagnostics that apply.  Sections are
//! structured so a client can show some and fold others; nothing is
//! pre-rendered.

use crate::diagnostics::{diagnostics, DiagnosticScope, SemanticDiagnostic};
use crate::hover::{hover, EntityStatus};
use bdl_check::pretty;
use bdl_ide_db::{AnalysisSnapshot, EntityKind, EntityRef};
use serde::{Deserialize, Serialize};

/// One section of an explanation: a heading and lines of `label: value`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplanationSection {
    pub heading: String,
    pub lines: Vec<(String, String)>,
}

impl ExplanationSection {
    fn new(heading: &str) -> ExplanationSection {
        ExplanationSection {
            heading: heading.into(),
            lines: Vec::new(),
        }
    }
    fn line(mut self, label: &str, value: impl Into<String>) -> ExplanationSection {
        self.lines.push((label.into(), value.into()));
        self
    }
    fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Explanation {
    pub entity: EntityRef,
    pub kind: EntityKind,
    pub title: String,
    pub status: EntityStatus,
    pub sections: Vec<ExplanationSection>,
    pub diagnostics: Vec<SemanticDiagnostic>,
}

/// The explanation of an entity; `None` when it does not exist.
pub fn explain(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Option<Explanation> {
    let h = hover(snapshot, entity)?;
    let design = &snapshot.effective().design;
    let analysis = snapshot.analysis();
    let ir = &analysis.ir;
    let mut sections = Vec::new();

    let mut identity = ExplanationSection::new("Identity")
        .line("entity", entity.to_string())
        .line("display name", h.title.clone());
    if let Some(sig) = &h.signature {
        identity = identity.line("surface", sig.clone());
    }
    if let Some(t) = &h.concept_type {
        identity = identity.line("kernel type", t.clone());
    }
    if let Some(r) = &h.representation {
        identity = identity.line("representation", r.clone());
    }
    if entity.as_mapping().is_some_and(|m| snapshot.is_drafted(m)) {
        identity = identity.line("source", "uncommitted draft (overlay)");
    }
    sections.push(identity);

    if let EntityRef::Mapping(id) = entity {
        if let Some(a) = analysis.mappings.get(&id) {
            let m = design.mappings.get(&id);
            let mut sem = ExplanationSection::new("Semantics").line("status", h.status.label());
            if let Some(m) = m {
                sem = sem.line(
                    "canonical type",
                    pretty::mapping_type(ir, &m.signature.inputs, m.signature.output),
                );
                if m.signature.is_unit_domain() {
                    if let Some(spelling) = crate::hover::declared_spelling(snapshot, id) {
                        sem = sem.line("declared spelling", spelling).line(
                            "domain",
                            "The omitted domain is the empty product (). The output-only shorthand is compatibility syntax; the preferred spelling is `() -> B`.",
                        );
                    } else {
                        sem = sem.line(
                            "domain",
                            "This relationship has no explicit inputs. Its canonical domain is (), the empty product; the kernel encodes `() -> B` as `B` (unit elimination).",
                        );
                    }
                }
            }
            sem = sem.line("interface", pretty::kernel(&a.interface.expected_type));
            if let Some(role) = crate::role::relationship_role(snapshot, id) {
                sem = sem.line("role", role.word());
                if let Some(kind) = crate::role::port_backed(snapshot, id) {
                    sem = sem.line(
                        "port",
                        match kind {
                            bdl_system::PortKind::Required => "required",
                            bdl_system::PortKind::Provided => "provided",
                            bdl_system::PortKind::Parameter => "parameter",
                        },
                    );
                }
                match role {
                    crate::role::RelationshipRole::Source => {
                        let p = crate::role::provider(snapshot, id)
                            .unwrap_or(crate::role::Provider::Environment);
                        sem = sem.line("provision", p.word()).line(
                            "reading",
                            "The declaration is observed once per activation: its value is provided for that tick (FV Phase 12 `source_value`). The unit argument is erased above the kernel; a reference is a reading of the declaration, not an effectful zero-argument call (`refForms_agree`).",
                        );
                    }
                    crate::role::RelationshipRole::Value => {
                        sem = sem.line(
                            "reading",
                            "A realized `() -> B` never consults the environment (FV Phase 12 `resolved_not_source`): the domain shape alone makes nothing a Source. The value is the realization's at every activation of its domain.",
                        );
                    }
                    crate::role::RelationshipRole::Rule => {
                        sem = sem.line(
                            "reading",
                            "A function from what it reads to what it produces: it has no value of its own at a tick; a value's realization applies it (the dependency graph's reverse edges name the appliers).",
                        );
                    }
                }
            }
            let grant: Vec<String> = a
                .interface
                .expected_type
                .grant()
                .into_iter()
                .map(|s| {
                    design
                        .concepts
                        .get(&s)
                        .map(|c| c.name.clone())
                        .unwrap_or_else(|| s.to_string())
                })
                .collect();
            sem = sem.line(
                "grant",
                if grant.is_empty() {
                    "constructs nothing".to_owned()
                } else {
                    format!("may construct {}", grant.join(", "))
                },
            );
            if let Some(t) = &a.inferred_type {
                sem = sem.line("inferred", pretty::kernel(t));
            }
            if let Some(e) = &a.realization {
                sem = sem.line("core", pretty::expr(e));
            }
            sections.push(sem);

            let g = &analysis.dependencies;
            let names = |set: Option<&std::collections::BTreeSet<bdl_model::DeclId>>| -> String {
                set.map(|s| {
                    s.iter()
                        .map(|d| {
                            design
                                .mappings
                                .get(d)
                                .map(|m| m.name.clone())
                                .unwrap_or_else(|| d.to_string())
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "none".into())
            };
            let mut reactive = ExplanationSection::new("Reactive")
                .line("depends on", names(g.all.get(&id)))
                .line("instantaneously on", names(g.instantaneous.get(&id)))
                .line("depended on by", names(g.reverse_all.get(&id)));
            if let Some(r) = analysis.causality.rank.get(&id) {
                reactive = reactive.line("rank", r.to_string());
            }
            if analysis.causality.in_cycle(id) {
                reactive = reactive.line("causality", "on an instantaneous cycle");
            }
            sections.push(reactive);

            let block = design.mappings.get(&id);
            let mut clock = ExplanationSection::new("Timing");
            match block.and_then(|b| b.clock) {
                Some(c) => {
                    clock = clock.line(
                        "domain",
                        design
                            .clocks
                            .get(&c)
                            .map(|x| x.name.clone())
                            .unwrap_or_else(|| c.to_string()),
                    )
                }
                None => clock = clock.line("domain", "agnostic (serves any domain)"),
            }
            if analysis.clocks.ill_clocked.contains(&id) {
                clock = clock.line("judgment", "reads across domains without a transport");
            }
            sections.push(clock);

            let mut output = ExplanationSection::new("Output relation");
            match block.and_then(|b| b.drives) {
                Some(o) => {
                    let oname = design
                        .outputs
                        .get(&o)
                        .map(|x| x.name.clone())
                        .unwrap_or_else(|| o.to_string());
                    output = output.line("drives", oname);
                    if let Some(f) = analysis.outputs.faults.get(&id) {
                        output = output.line("edge", format!("ill-formed: {f:?}"));
                    } else if analysis.outputs.valid_bindings.contains_key(&id) {
                        output = output.line("edge", "well formed (DriveWF)");
                    }
                }
                None => output = output.line("drives", "nothing"),
            }
            sections.push(output);
        }
    }

    if let EntityRef::Concept(id) = entity {
        let users: Vec<String> = design.mappings_using(id).map(|m| m.name.clone()).collect();
        let sinks: Vec<String> = design
            .outputs
            .values()
            .filter(|o| o.accepts == id)
            .map(|o| o.name.clone())
            .collect();
        let mut s = ExplanationSection::new("Uses")
            .line(
                "mappings",
                if users.is_empty() {
                    "none".into()
                } else {
                    users.join(", ")
                },
            )
            .line(
                "outputs",
                if sinks.is_empty() {
                    "none".into()
                } else {
                    sinks.join(", ")
                },
            );
        if let Some(t) = ir.representation_of(id) {
            s = s.line("Θ", pretty::kernel(t));
        }
        sections.push(s);
    }

    let inv = ExplanationSection::new("Invalidation")
        .line(
            "if renamed",
            "nothing (a refinement: names are not identity)",
        )
        .line(
            "if redefined",
            "its realization and reactive facts; dependents re-validate",
        );
    sections.push(inv);

    let diagnostics = diagnostics(snapshot, DiagnosticScope::Concerning(entity)).items;
    sections.retain(|s| !s.is_empty());
    Some(Explanation {
        entity,
        kind: h.kind,
        title: h.title,
        status: h.status,
        sections,
        diagnostics,
    })
}
