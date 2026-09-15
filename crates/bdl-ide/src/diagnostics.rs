//! Semantic diagnostics: the compiler's verdict, anchored to entities and
//! roles rather than to positions, then projected onto surfaces.
//!
//! ```text
//!   bdl_diagnostics::Diagnostic  ──lift──▶  SemanticDiagnostic  ──project──▶  TextDiagnostic
//!   (entity, span, code, …)               (primary + related anchors)         VisualDiagnostic
//! ```
//!
//! The lift is the only place that knows which *aspect* of an entity a
//! diagnostic code is about (`output.*` is the drive edge, `clock.*` the
//! clock binding, `formula.*` the definition) and which other entities
//! are involved (the other claimants of a contested sink, the members of
//! an instantaneous cycle).  It runs once per snapshot; every surface
//! reads the same list.
//!
//! Severity keeps the compiler's distinction between *wrong* and *open*:
//! an unresolved mapping, an unbound concept or an undriven sink is
//! [`SemanticSeverity::Open`], never an error, on every surface.

use crate::actions::{actions_for, SemanticActionId};
use bdl_diagnostics::Severity;
use bdl_ide_db::{
    AnalysisSnapshot, DocumentId, EntityRef, EntityRole, SnapshotStamp, TextRange, VisualElementRef,
};
use bdl_model::{DeclId, OutputId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SemanticSeverity {
    Error,
    Warning,
    /// Legal and unfinished: an unresolved mapping, an unbound concept, an
    /// undriven sink.  A text editor shows it as information, never as an
    /// error; Studio shows it as the design's state.
    Open,
}

impl SemanticSeverity {
    fn lift(s: Severity) -> SemanticSeverity {
        match s {
            Severity::Error => SemanticSeverity::Error,
            Severity::Warning => SemanticSeverity::Warning,
            Severity::Info => SemanticSeverity::Open,
        }
    }
    pub fn is_error(self) -> bool {
        self == SemanticSeverity::Error
    }
}

/// Where a source span is relative to.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "origin", rename_all = "snake_case")]
pub enum SourceOrigin {
    /// Relative to the formula source of a mapping (the Studio definition
    /// field, or the body inside whichever document declares it).
    Formula { mapping: DeclId },
    /// Absolute in a text document.
    Document { document: DocumentId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SourceSpan {
    pub origin: SourceOrigin,
    pub range: TextRange,
}

/// What a diagnostic points at: an entity and the aspect of it, with the
/// source span when the compiler had one.  Identity first; the span is
/// placement data.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SemanticAnchor {
    pub entity: EntityRef,
    pub role: EntityRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceSpan>,
}

impl SemanticAnchor {
    pub fn new(entity: EntityRef, role: EntityRole) -> SemanticAnchor {
        SemanticAnchor {
            entity,
            role,
            source: None,
        }
    }
    fn at(mut self, source: Option<SourceSpan>) -> SemanticAnchor {
        self.source = source;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticDiagnostic {
    /// Stable machine code (`output.multiple_drivers`).
    pub code: String,
    pub severity: SemanticSeverity,
    pub primary: SemanticAnchor,
    /// Other places the same fact concerns, each with a short note.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<(SemanticAnchor, String)>,
    pub message: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub explanation: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub technical: String,
    /// The compiler's textual hints, verbatim.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fixes: Vec<String>,
    /// Actions that address this diagnostic (resolved by
    /// [`crate::actions::actions_for`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<SemanticActionId>,
}

impl SemanticDiagnostic {
    pub fn is_error(&self) -> bool {
        self.severity.is_error()
    }
    /// Every anchor, primary first.
    pub fn anchors(&self) -> impl Iterator<Item = &SemanticAnchor> {
        std::iter::once(&self.primary).chain(self.related.iter().map(|(a, _)| a))
    }
    pub fn concerns(&self, entity: EntityRef) -> bool {
        self.anchors().any(|a| a.entity == entity)
    }
}

/// The diagnostics of one snapshot, in a deterministic order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticSet {
    pub stamp: SnapshotStamp,
    pub items: Vec<SemanticDiagnostic>,
}

impl DiagnosticSet {
    pub fn errors(&self) -> impl Iterator<Item = &SemanticDiagnostic> {
        self.items.iter().filter(|d| d.is_error())
    }
    pub fn for_entity(&self, entity: EntityRef) -> impl Iterator<Item = &SemanticDiagnostic> {
        self.items.iter().filter(move |d| d.concerns(entity))
    }
    pub fn codes(&self) -> Vec<&str> {
        self.items.iter().map(|d| d.code.as_str()).collect()
    }
}

/// Which diagnostics to return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticScope {
    /// Everything.
    Project,
    /// Those whose primary anchor is this entity.
    Entity(EntityRef),
    /// Those that concern (primary or related) this entity.
    Concerning(EntityRef),
    /// Those placeable in this document (declared there, or a document
    /// fault).
    Document(DocumentId),
}

/// The semantic diagnostics of a snapshot.
pub fn diagnostics(snapshot: &AnalysisSnapshot, scope: DiagnosticScope) -> DiagnosticSet {
    let all = lift_all(snapshot);
    let items = match scope {
        DiagnosticScope::Project => all,
        DiagnosticScope::Entity(e) => all.into_iter().filter(|d| d.primary.entity == e).collect(),
        DiagnosticScope::Concerning(e) => all.into_iter().filter(|d| d.concerns(e)).collect(),
        DiagnosticScope::Document(doc) => all
            .into_iter()
            .filter(|d| placeable_in(snapshot, d, doc))
            .collect(),
    };
    DiagnosticSet {
        stamp: snapshot.stamp(),
        items,
    }
}

fn placeable_in(snapshot: &AnalysisSnapshot, d: &SemanticDiagnostic, doc: DocumentId) -> bool {
    if let Some(SourceSpan {
        origin: SourceOrigin::Document { document },
        ..
    }) = d.primary.source
    {
        return document == doc;
    }
    snapshot
        .document(doc)
        .is_some_and(|s| s.declares(d.primary.entity))
}

// ---- lifting ---------------------------------------------------------------

fn lift_all(snapshot: &AnalysisSnapshot) -> Vec<SemanticDiagnostic> {
    let analysis = snapshot.analysis();
    let design = &snapshot.effective().design;
    let mut out = Vec::new();
    // One semantic diagnostic per contested sink, whichever claimant the
    // compiler reported it on.
    let mut conflicts_done: BTreeMap<OutputId, ()> = BTreeMap::new();

    for d in &analysis.diagnostics {
        let entity = EntityRef::from_diagnostic(d.entity);
        let code = d.code.as_str();
        let mapping = entity.as_mapping();
        let formula_span = |m: DeclId| {
            d.span.map(|s| SourceSpan {
                origin: SourceOrigin::Formula { mapping: m },
                range: s.into(),
            })
        };
        let mut related: Vec<(SemanticAnchor, String)> = Vec::new();
        let primary = if let Some(m) = mapping {
            let name = snapshot
                .name_of(entity)
                .unwrap_or("this mapping")
                .to_owned();
            if code.starts_with("output.") {
                let output = design.mappings.get(&m).and_then(|b| b.drives);
                if code == "output.multiple_drivers" {
                    let Some(o) = output else { continue };
                    if conflicts_done.contains_key(&o) {
                        continue;
                    }
                    conflicts_done.insert(o, ());
                    if let Some(claimants) = analysis.outputs.conflicts.get(&o) {
                        for c in claimants.iter().filter(|c| **c != m) {
                            let cname = snapshot
                                .name_of(EntityRef::Mapping(*c))
                                .unwrap_or("another mapping");
                            related.push((
                                SemanticAnchor::new(EntityRef::Mapping(*c), EntityRole::DriveEdge),
                                format!("`{cname}` also claims this sink."),
                            ));
                        }
                    }
                    related.push((
                        SemanticAnchor::new(EntityRef::Output(o), EntityRole::Output),
                        "the contested sink".into(),
                    ));
                } else if let Some(o) = output {
                    let role = if code == "output.type_mismatch" {
                        EntityRole::Output
                    } else {
                        EntityRole::ClockBinding
                    };
                    related.push((
                        SemanticAnchor::new(EntityRef::Output(o), role),
                        "the sink".into(),
                    ));
                }
                SemanticAnchor::new(entity, EntityRole::DriveEdge)
            } else if code.starts_with("clock.") {
                if code == "clock.cross_domain_reference" {
                    for x in analysis.clocks.crossings.iter().filter(|x| x.from == m) {
                        let to = snapshot
                            .name_of(EntityRef::Mapping(x.to))
                            .unwrap_or("a mapping");
                        related.push((
                            SemanticAnchor::new(EntityRef::Mapping(x.to), EntityRole::ClockBinding),
                            format!("`{to}` is read from another domain."),
                        ));
                    }
                }
                SemanticAnchor::new(entity, EntityRole::ClockBinding).at(formula_span(m))
            } else if code == "reactive.instantaneous_cycle" {
                for cycle in analysis.causality.cycles.iter().filter(|c| c.contains(&m)) {
                    for other in cycle.iter().filter(|o| **o != m) {
                        related.push((
                            SemanticAnchor::new(EntityRef::Mapping(*other), EntityRole::Definition),
                            format!("on the same instantaneous cycle as `{name}`"),
                        ));
                    }
                }
                SemanticAnchor::new(entity, EntityRole::Definition).at(formula_span(m))
            } else if code == "semantic.unbound_representation" {
                if let Some(block) = design.mappings.get(&m) {
                    let mut concepts = block.signature.inputs.clone();
                    concepts.push(block.signature.output);
                    for c in concepts {
                        let unbound = design
                            .concepts
                            .get(&c)
                            .is_some_and(|x| x.representation.is_none());
                        if unbound {
                            let cname = snapshot
                                .name_of(EntityRef::Concept(c))
                                .unwrap_or("a concept");
                            related.push((
                                SemanticAnchor::new(
                                    EntityRef::Concept(c),
                                    EntityRole::Representation,
                                ),
                                format!("`{cname}` has no representation yet."),
                            ));
                        }
                    }
                }
                SemanticAnchor::new(entity, EntityRole::Definition).at(formula_span(m))
            } else {
                // formula.*, dimension.*, realization.*, type.*, semantic.*
                SemanticAnchor::new(entity, EntityRole::Definition).at(formula_span(m))
            }
        } else if let Some(c) = entity.as_concept() {
            SemanticAnchor::new(EntityRef::Concept(c), EntityRole::Representation)
        } else {
            // Project-level: output notes name the sink in the message.
            // The compiler's `Entity` has no output variant yet (DI-29), so
            // the sink is recovered from the analysis, never from parsing
            // the message beyond a name check.
            match project_output(snapshot, code, &d.message) {
                Some((o, role)) => SemanticAnchor::new(EntityRef::Output(o), role),
                None => SemanticAnchor::new(EntityRef::Project, EntityRole::Declaration),
            }
        };
        out.push(SemanticDiagnostic {
            code: code.to_owned(),
            severity: SemanticSeverity::lift(d.severity),
            primary,
            related,
            message: d.message.clone(),
            explanation: d.explanation.clone(),
            technical: d.technical.clone(),
            fixes: d.fixes.clone(),
            actions: Vec::new(),
        });
    }

    // Text documents: syntax errors and binding faults.
    for doc in snapshot.documents() {
        let at = |range: TextRange| {
            Some(SourceSpan {
                origin: SourceOrigin::Document {
                    document: doc.document,
                },
                range,
            })
        };
        for e in &doc.syntax_errors {
            out.push(SemanticDiagnostic {
                code: e.code.as_str().to_owned(),
                severity: SemanticSeverity::Error,
                primary: SemanticAnchor::new(EntityRef::Project, EntityRole::Declaration)
                    .at(at(e.span.into())),
                related: Vec::new(),
                message: e.message.clone(),
                explanation: String::new(),
                technical: e.technical(),
                fixes: e.hint.iter().cloned().collect(),
                actions: Vec::new(),
            });
        }
        for f in &doc.binding_faults {
            out.push(SemanticDiagnostic {
                code: f.code.clone(),
                severity: if f.open {
                    SemanticSeverity::Open
                } else {
                    SemanticSeverity::Error
                },
                primary: SemanticAnchor::new(
                    f.entity.unwrap_or(EntityRef::Project),
                    EntityRole::Declaration,
                )
                .at(at(f.range)),
                related: Vec::new(),
                message: f.message.clone(),
                explanation: String::new(),
                technical: String::new(),
                fixes: Vec::new(),
                actions: Vec::new(),
            });
        }
    }

    // Overlay faults: a draft that targets nothing.
    for o in snapshot.overlays() {
        if let Some(bdl_ide_db::OverlayFault::UnknownMapping { mapping }) = &o.fault {
            out.push(SemanticDiagnostic {
                code: "overlay.unknown_mapping".into(),
                severity: SemanticSeverity::Error,
                primary: SemanticAnchor::new(EntityRef::Mapping(*mapping), EntityRole::Definition),
                related: Vec::new(),
                message: format!(
                    "the draft targets mapping {mapping}, which is not in the design."
                ),
                explanation:
                    "The mapping was deleted or the draft belongs to another project revision."
                        .into(),
                technical: String::new(),
                fixes: Vec::new(),
                actions: Vec::new(),
            });
        }
    }

    out.sort_by(|a, b| {
        a.primary
            .cmp(&b.primary)
            .then_with(|| a.code.cmp(&b.code))
            .then_with(|| a.message.cmp(&b.message))
    });
    for d in &mut out {
        d.actions = actions_for(snapshot, d).into_iter().map(|a| a.id).collect();
    }
    out
}

/// The sink a project-level output note is about: the one output in the
/// relevant analysis set whose name the message mentions.
fn project_output(
    snapshot: &AnalysisSnapshot,
    code: &str,
    message: &str,
) -> Option<(OutputId, EntityRole)> {
    let analysis = snapshot.analysis();
    let (candidates, role): (Vec<OutputId>, EntityRole) = match code {
        "output.missing_driver" => (
            analysis.outputs.missing_required.iter().copied().collect(),
            EntityRole::DriveEdge,
        ),
        "output.clock_unset" => (
            analysis.open_outputs.iter().copied().collect(),
            EntityRole::ClockBinding,
        ),
        _ => return None,
    };
    let mut hits = candidates.into_iter().filter(|o| {
        snapshot
            .name_of(EntityRef::Output(*o))
            .is_some_and(|n| message.contains(n))
    });
    let first = hits.next()?;
    if hits.next().is_some() {
        return None;
    }
    Some((first, role))
}

// ---- projection ------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextRelated {
    pub document: DocumentId,
    pub range: TextRange,
    pub message: String,
}

/// A diagnostic placed in one document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDiagnostic {
    pub range: TextRange,
    pub severity: SemanticSeverity,
    pub code: String,
    pub message: String,
    pub explanation: String,
    pub related: Vec<TextRelated>,
}

/// A diagnostic placed on the canvas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VisualDiagnostic {
    pub severity: SemanticSeverity,
    pub code: String,
    pub message: String,
    /// Elements to highlight, primary first.
    pub highlights: Vec<VisualElementRef>,
    /// A span in the Studio definition field, when the diagnostic has one.
    pub formula_span: Option<(DeclId, TextRange)>,
}

/// Where an anchor lands in a document, if it lands there at all.
fn place_in_document(
    snapshot: &AnalysisSnapshot,
    anchor: &SemanticAnchor,
    document: DocumentId,
) -> Option<TextRange> {
    let doc = snapshot.document(document)?;
    if let Some(s) = anchor.source {
        match s.origin {
            SourceOrigin::Document { document: d } if d == document => return Some(s.range),
            SourceOrigin::Document { .. } => {}
            SourceOrigin::Formula { mapping } => {
                if let Some(r) = doc.formula_range(mapping, s.range) {
                    return Some(r);
                }
            }
        }
    }
    let map = snapshot.projections();
    let pick = |role: EntityRole| {
        map.text_anchors(anchor.entity, document)
            .find(|a| a.role == role)
            .and_then(|a| a.text_range())
    };
    pick(anchor.role)
        .or_else(|| pick(EntityRole::Name))
        .or_else(|| pick(EntityRole::Declaration))
}

/// Place a diagnostic in a document.  `None` when nothing of it is there
/// (it belongs to another surface or another document).
pub fn project_to_document(
    snapshot: &AnalysisSnapshot,
    d: &SemanticDiagnostic,
    document: DocumentId,
) -> Option<TextDiagnostic> {
    let range = place_in_document(snapshot, &d.primary, document)?;
    let related = d
        .related
        .iter()
        .filter_map(|(a, note)| {
            snapshot.projections().documents().find_map(|doc| {
                place_in_document(snapshot, a, doc).map(|range| TextRelated {
                    document: doc,
                    range,
                    message: note.clone(),
                })
            })
        })
        .collect();
    Some(TextDiagnostic {
        range,
        severity: d.severity,
        code: d.code.clone(),
        message: d.message.clone(),
        explanation: d.explanation.clone(),
        related,
    })
}

/// Place a diagnostic on the canvas.
pub fn project_to_visual(snapshot: &AnalysisSnapshot, d: &SemanticDiagnostic) -> VisualDiagnostic {
    let map = snapshot.projections();
    let mut highlights = Vec::new();
    for a in d.anchors() {
        let element = map
            .anchors_for(a.entity, a.role)
            .chain(map.anchors_for(a.entity, EntityRole::Declaration))
            .find_map(|x| x.visual_element());
        if let Some(e) = element {
            if !highlights.contains(&e) {
                highlights.push(e);
            }
        }
    }
    let formula_span = match d.primary.source {
        Some(SourceSpan {
            origin: SourceOrigin::Formula { mapping },
            range,
        }) => Some((mapping, range)),
        _ => None,
    };
    VisualDiagnostic {
        severity: d.severity,
        code: d.code.clone(),
        message: d.message.clone(),
        highlights,
        formula_span,
    }
}

/// Lift one compiler diagnostic in isolation (for the draft verdict, which
/// reports the mapping's own list).  Same rules as the snapshot-wide lift.
pub(crate) fn lift_for_mapping(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
) -> Vec<SemanticDiagnostic> {
    lift_all(snapshot)
        .into_iter()
        .filter(|d| d.primary.entity == EntityRef::Mapping(mapping))
        .collect()
}
