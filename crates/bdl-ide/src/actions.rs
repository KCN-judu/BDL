//! Semantic actions: explicit, meaning-changing operations offered to the
//! designer, never applied behind their back.
//!
//! An action is a [`SemanticEditPlan`] with a title and a reason.  Some
//! are ready (detach one of two conflicting drivers); some need a choice
//! the tool must not make (which representation, which domain); some are
//! blocked by what the language can express today and say so.  The LSP
//! adapter renders them as code actions, Studio as *Fix* affordances; both
//! apply the same plan through the same model operations.
//!
//! What is *not* here: silent fixes.  Inserting a sync, arbitrating an
//! output, changing a clock or a representation all change what the
//! design means; they are offered, with the invalidation they would cause,
//! and applied only when chosen.

use crate::diagnostics::{SemanticDiagnostic, SourceOrigin, SourceSpan};
use crate::edit_plan::{SemanticEditPlan, SemanticOperation};
use crate::invalidation::preview_change;
use bdl_ide_db::{AnalysisSnapshot, EntityRef, EntityRole, TextEdit, TextRange};
use bdl_model::surface::Representation;
use bdl_model::{DeclId, Dim, EditOp, OutputId};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Stable within a snapshot: `<kind>:<entity>[:<detail>]`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SemanticActionId(pub String);

impl fmt::Display for SemanticActionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    /// Addresses a diagnostic.
    QuickFix,
    /// Restructures without a diagnostic asking for it.
    Refactor,
}

/// One of the values a choice-taking action can be completed with.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionChoice {
    pub label: String,
    /// The edit chosen; a client completes the action by applying it.
    pub edit: EditOp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "applicability", rename_all = "snake_case")]
pub enum Applicability {
    /// The plan can be applied as is.
    Ready,
    /// The designer must pick one option; the tool will not guess.
    NeedsChoice { options: Vec<ActionChoice> },
    /// The language cannot express the fix yet; the reason is shown.
    Blocked { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticAction {
    pub id: SemanticActionId,
    pub title: String,
    pub kind: ActionKind,
    pub applicability: Applicability,
    /// Present when `Ready`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<SemanticEditPlan>,
    /// Why this is offered and what it changes, product language.
    pub explanation: String,
    /// The diagnostic codes this action addresses (empty for refactors).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<String>,
}

impl SemanticAction {
    pub fn is_ready(&self) -> bool {
        matches!(self.applicability, Applicability::Ready)
    }
}

/// Actions addressing one diagnostic.
pub fn actions_for(snapshot: &AnalysisSnapshot, d: &SemanticDiagnostic) -> Vec<SemanticAction> {
    let mut out = Vec::new();
    let code = d.code.as_str();
    let entity = d.primary.entity;
    match code {
        "output.multiple_drivers" => {
            // Every claimant is offered for detaching; the designer chooses
            // which one keeps the sink.
            let claimants: Vec<DeclId> = d
                .anchors()
                .filter(|a| a.role == EntityRole::DriveEdge)
                .filter_map(|a| a.entity.as_mapping())
                .collect();
            let output = d
                .anchors()
                .find_map(|a| a.entity.as_output())
                .or_else(|| drives_of(snapshot, entity));
            for c in &claimants {
                out.push(detach_driver(snapshot, *c, code));
            }
            if let Some(o) = output {
                out.push(create_combination(snapshot, o, &claimants, code));
            }
        }
        "output.type_mismatch" => {
            if let Some(m) = entity.as_mapping() {
                out.push(detach_driver(snapshot, m, code));
                if let Some(o) = drives_of(snapshot, entity) {
                    out.push(create_combination(snapshot, o, &[m], code));
                }
            }
        }
        "output.clock_mismatch" => {
            if let (Some(m), Some(o)) = (entity.as_mapping(), drives_of(snapshot, entity)) {
                let design = &snapshot.effective().design;
                let out_clock = design.outputs.get(&o).and_then(|x| x.clock);
                let map_clock = design.mappings.get(&m).and_then(|x| x.clock);
                let mname = name(snapshot, entity);
                let oname = name(snapshot, EntityRef::Output(o));
                if let Some(c) = out_clock {
                    let cname = name(snapshot, EntityRef::Clock(c));
                    out.push(ready(
                        snapshot,
                        SemanticActionId(format!("clock.move_mapping:{m}:{c}")),
                        format!("Move `{mname}` to `{cname}`"),
                        EditOp::SetMappingClock {
                            id: m,
                            clock: Some(c),
                        },
                        format!("`{oname}` updates in `{cname}`; its driver must update in the same domain.  Every value `{mname}` reads is re-judged against the new domain."),
                        code,
                    ));
                }
                if let Some(c) = map_clock {
                    let cname = name(snapshot, EntityRef::Clock(c));
                    out.push(ready(
                        snapshot,
                        SemanticActionId(format!("clock.move_output:{o}:{c}")),
                        format!("Update `{oname}` in `{cname}`"),
                        EditOp::SetOutputClock {
                            id: o,
                            clock: Some(c),
                        },
                        format!("Makes the sink update in the driver's domain instead.  Changes when `{oname}` changes physically."),
                        code,
                    ));
                }
                out.push(detach_driver(snapshot, m, code));
            }
        }
        "output.clock_unset" => {
            let o = entity.as_output().or_else(|| drives_of(snapshot, entity));
            if let Some(o) = o {
                out.push(choose_clock(snapshot, o, code));
            }
        }
        "output.missing_driver" => {
            if let Some(o) = entity.as_output() {
                out.push(connect_driver(snapshot, o, code));
            }
        }
        "semantic.unbound_representation" => {
            for c in d
                .anchors()
                .filter(|a| a.role == EntityRole::Representation)
                .filter_map(|a| a.entity.as_concept())
            {
                out.push(choose_representation(snapshot, c, code));
            }
        }
        "formula.name.unknown" | "formula.name.not_an_input" | "formula.name.ambiguous" => {
            if let (Some(m), Some(span)) = (entity.as_mapping(), d.primary.source) {
                out.extend(use_current_name(snapshot, m, span, &d.fixes, code));
            }
        }
        "clock.cross_domain_reference" => {
            out.push(SemanticAction {
                id: SemanticActionId(format!("clock.insert_sync:{entity}")),
                title: "Insert explicit sync".into(),
                kind: ActionKind::QuickFix,
                applicability: Applicability::Blocked {
                    reason: "the surface language has no phrase for `sync` yet; it exists at Core level only (DI-3, DI-17)".into(),
                },
                plan: None,
                explanation: "A value from another timing domain can only be read through an explicit transport.  The transport is a meaning-changing choice; it is never inserted for you.".into(),
                addresses: vec![code.to_owned()],
            });
        }
        _ => {}
    }
    out
}

/// Context actions on an entity, independent of any diagnostic.
pub fn actions_at(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Vec<SemanticAction> {
    let design = &snapshot.effective().design;
    let mut out = Vec::new();
    match entity {
        EntityRef::Mapping(m) => {
            if let Some(block) = design.mappings.get(&m) {
                if block.drives.is_some() {
                    let mut a = detach_driver(snapshot, m, "");
                    a.kind = ActionKind::Refactor;
                    a.addresses.clear();
                    out.push(a);
                }
                if block.definition.is_some() {
                    let mname = name(snapshot, entity);
                    out.push(ready(
                        snapshot,
                        SemanticActionId(format!("definition.detach:{m}")),
                        format!("Detach the definition of `{mname}`"),
                        EditOp::ReplaceDefinition {
                            id: m,
                            definition: None,
                        },
                        "The mapping becomes unresolved again: declared and typed, with no body.  Dependents keep its interface.".into(),
                        "",
                    ));
                }
            }
        }
        EntityRef::Concept(c) => {
            if design
                .concepts
                .get(&c)
                .is_some_and(|x| x.representation.is_none())
            {
                out.push(choose_representation(snapshot, c, ""));
            }
        }
        EntityRef::Output(o) => {
            if design.outputs.get(&o).is_some_and(|x| x.clock.is_none()) {
                out.push(choose_clock(snapshot, o, ""));
            }
        }
        _ => {}
    }
    for a in &mut out {
        a.kind = ActionKind::Refactor;
        a.addresses.clear();
    }
    out
}

// ---- builders --------------------------------------------------------------

fn name(snapshot: &AnalysisSnapshot, e: EntityRef) -> String {
    snapshot.name_of(e).unwrap_or("?").to_owned()
}

fn drives_of(snapshot: &AnalysisSnapshot, e: EntityRef) -> Option<OutputId> {
    let m = e.as_mapping()?;
    snapshot.effective().design.mappings.get(&m)?.drives
}

fn ready(
    snapshot: &AnalysisSnapshot,
    id: SemanticActionId,
    title: String,
    edit: EditOp,
    explanation: String,
    code: &str,
) -> SemanticAction {
    let mut plan = SemanticEditPlan::new(title.clone(), snapshot.stamp());
    plan.invalidation = preview_change(snapshot, &edit);
    plan.affected_entities
        .extend(plan.invalidation.invalidates.iter().map(|i| i.entity));
    plan.operations.push(SemanticOperation::Model { edit });
    SemanticAction {
        id,
        title,
        kind: ActionKind::QuickFix,
        applicability: Applicability::Ready,
        plan: Some(plan),
        explanation,
        addresses: if code.is_empty() {
            Vec::new()
        } else {
            vec![code.to_owned()]
        },
    }
}

fn detach_driver(snapshot: &AnalysisSnapshot, m: DeclId, code: &str) -> SemanticAction {
    let mname = name(snapshot, EntityRef::Mapping(m));
    let oname = drives_of(snapshot, EntityRef::Mapping(m))
        .map(|o| name(snapshot, EntityRef::Output(o)))
        .unwrap_or_else(|| "the sink".into());
    ready(
        snapshot,
        SemanticActionId(format!("output.detach_driver:{m}")),
        format!("Detach `{mname}` from `{oname}`"),
        EditOp::SetMappingDrive { id: m, output: None },
        format!("`{mname}` stops driving `{oname}`.  Nothing else about `{mname}` changes; the sink is undriven until another relationship is connected."),
        code,
    )
}

fn create_combination(
    snapshot: &AnalysisSnapshot,
    o: OutputId,
    claimants: &[DeclId],
    code: &str,
) -> SemanticAction {
    let design = &snapshot.effective().design;
    let oname = name(snapshot, EntityRef::Output(o));
    let Some(out) = design.outputs.get(&o) else {
        return SemanticAction {
            id: SemanticActionId(format!("output.create_combination:{o}")),
            title: "Create upstream combination mapping".into(),
            kind: ActionKind::QuickFix,
            applicability: Applicability::Blocked {
                reason: "the sink no longer exists".into(),
            },
            plan: None,
            explanation: String::new(),
            addresses: vec![code.to_owned()],
        };
    };
    // Inputs: what each claimant produces; output: what the sink accepts.
    let inputs: Vec<_> = claimants
        .iter()
        .filter_map(|c| design.mappings.get(c))
        .map(|m| m.signature.output)
        .collect();
    let mut mapping_name = format!("{}Combined", camel(&oname));
    let mut n = 2;
    while design.mappings.values().any(|m| m.name == mapping_name) {
        mapping_name = format!("{}Combined{n}", camel(&oname));
        n += 1;
    }
    let edit = EditOp::CreateMapping {
        name: mapping_name.clone(),
        description: format!("Combines the values that compete for {oname}."),
        signature: bdl_model::surface::Signature {
            inputs,
            output: out.accepts,
        },
    };
    let mut a = ready(
        snapshot,
        SemanticActionId(format!("output.create_combination:{o}")),
        format!("Create `{mapping_name}` to combine the values for `{oname}`"),
        edit,
        "One sink has one driver; every combination of contributors is ordinary computation upstream of that edge.  The new mapping is declared, unresolved, and not yet connected: write its definition, then make it the driver.".into(),
        code,
    );
    a.kind = ActionKind::Refactor;
    a
}

fn choose_clock(snapshot: &AnalysisSnapshot, o: OutputId, code: &str) -> SemanticAction {
    let design = &snapshot.effective().design;
    let oname = name(snapshot, EntityRef::Output(o));
    let options: Vec<ActionChoice> = design
        .clocks
        .values()
        .map(|c| ActionChoice {
            label: c.name.clone(),
            edit: EditOp::SetOutputClock {
                id: o,
                clock: Some(c.id),
            },
        })
        .collect();
    SemanticAction {
        id: SemanticActionId(format!("output.choose_clock:{o}")),
        title: format!("Choose the timing domain `{oname}` updates in"),
        kind: ActionKind::QuickFix,
        applicability: if options.is_empty() {
            Applicability::Blocked {
                reason: "the design has no timing domains yet; create one first".into(),
            }
        } else {
            Applicability::NeedsChoice { options }
        },
        plan: None,
        explanation: "A sink without a domain is open: neither driven nor missing.  Which domain is a design decision the tool does not make.".into(),
        addresses: addresses(code),
    }
}

fn connect_driver(snapshot: &AnalysisSnapshot, o: OutputId, code: &str) -> SemanticAction {
    let design = &snapshot.effective().design;
    let oname = name(snapshot, EntityRef::Output(o));
    let out = design.outputs.get(&o);
    // Candidates: nullary mappings producing what the sink accepts, in the
    // sink's domain (or agnostic), not already driving something (DI-20).
    let options: Vec<ActionChoice> = design
        .mappings
        .values()
        .filter(|m| {
            out.is_some_and(|x| {
                m.signature.inputs.is_empty()
                    && m.signature.output == x.accepts
                    && m.drives.is_none()
                    && (m.clock.is_none() || m.clock == x.clock)
            })
        })
        .map(|m| ActionChoice {
            label: m.name.clone(),
            edit: EditOp::SetMappingDrive {
                id: m.id,
                output: Some(o),
            },
        })
        .collect();
    SemanticAction {
        id: SemanticActionId(format!("output.connect_driver:{o}")),
        title: format!("Connect a driver to `{oname}`"),
        kind: ActionKind::QuickFix,
        applicability: if options.is_empty() {
            Applicability::Blocked {
                reason: format!("no value relationship produces what `{oname}` accepts in its domain yet"),
            }
        } else {
            Applicability::NeedsChoice { options }
        },
        plan: None,
        explanation: "A required sink must have exactly one final driver for the design to be executable.  A partial design may leave it undriven.".into(),
        addresses: addresses(code),
    }
}

fn choose_representation(
    snapshot: &AnalysisSnapshot,
    c: bdl_model::SemanticId,
    code: &str,
) -> SemanticAction {
    let cname = name(snapshot, EntityRef::Concept(c));
    let options = [
        (
            "a dimensionless level (Scalar)",
            Representation::Quantity { dim: Dim::ZERO },
        ),
        ("an angle", Representation::Quantity { dim: Dim::ANGLE }),
        ("a length", Representation::Quantity { dim: Dim::LENGTH }),
        ("a time", Representation::Quantity { dim: Dim::TIME }),
        ("a mass", Representation::Quantity { dim: Dim::MASS }),
        ("a current", Representation::Quantity { dim: Dim::CURRENT }),
        (
            "a temperature",
            Representation::Quantity {
                dim: Dim::TEMPERATURE,
            },
        ),
        ("true or false", Representation::Boolean),
        ("a count", Representation::Count),
    ]
    .into_iter()
    .map(|(label, r)| ActionChoice {
        label: label.into(),
        edit: EditOp::SetConceptRepresentation {
            id: c,
            representation: Some(r),
        },
    })
    .collect();
    SemanticAction {
        id: SemanticActionId(format!("concept.choose_representation:{c}")),
        title: format!("Choose what `{cname}` is represented by"),
        kind: ActionKind::QuickFix,
        applicability: Applicability::NeedsChoice { options },
        plan: None,
        explanation: format!("`{cname}` is a declared intent without a data representation yet.  Binding one is a refinement; every mapping over `{cname}` can then be checked."),
        addresses: addresses(code),
    }
}

/// `formula.name.*`: replace the misspelt name by each current input name
/// the compiler suggested.  A text operation on the draft or the document
/// that holds the formula, or a model operation on a committed formula.
fn use_current_name(
    snapshot: &AnalysisSnapshot,
    m: DeclId,
    span: SourceSpan,
    fixes: &[String],
    code: &str,
) -> Vec<SemanticAction> {
    let SourceOrigin::Formula { mapping } = span.origin else {
        return Vec::new();
    };
    if mapping != m {
        return Vec::new();
    }
    let design = &snapshot.effective().design;
    let Some(block) = design.mappings.get(&m) else {
        return Vec::new();
    };
    let input_names: Vec<String> = block
        .signature
        .inputs
        .iter()
        .filter_map(|c| design.concepts.get(c))
        .map(|c| c.name.clone())
        .collect();
    let Some(bdl_model::surface::Definition::Formula { source }) = &block.definition else {
        return Vec::new();
    };
    fixes
        .iter()
        .filter(|f| input_names.contains(f))
        .map(|new_name| {
            let edit = TextEdit::replace(span.range, new_name.clone());
            let mut plan = SemanticEditPlan::new(format!("Use `{new_name}`"), snapshot.stamp());
            plan.affected_entities.insert(EntityRef::Mapping(m));
            let op = if let Some(doc) = snapshot.declaring_document(EntityRef::Mapping(m)) {
                let range = snapshot
                    .document(doc)
                    .and_then(|d| d.formula_range(m, span.range))
                    .unwrap_or(TextRange::empty_at(0));
                SemanticOperation::Text {
                    document: doc,
                    edits: vec![TextEdit::replace(range, new_name.clone())],
                }
            } else if snapshot.is_drafted(m) {
                SemanticOperation::DraftText {
                    mapping: m,
                    edits: vec![edit],
                }
            } else {
                match TextEdit::apply_all(source, std::slice::from_ref(&edit)) {
                    Ok(rewritten) => {
                        let e = EditOp::ReplaceDefinition {
                            id: m,
                            definition: Some(bdl_model::surface::Definition::Formula {
                                source: rewritten,
                            }),
                        };
                        plan.invalidation = preview_change(snapshot, &e);
                        SemanticOperation::Model { edit: e }
                    }
                    Err(_) => SemanticOperation::DraftText {
                        mapping: m,
                        edits: vec![edit],
                    },
                }
            };
            plan.operations.push(op);
            SemanticAction {
                id: SemanticActionId(format!("formula.use_name:{m}:{}:{new_name}", span.range)),
                title: format!("Use `{new_name}`"),
                kind: ActionKind::QuickFix,
                applicability: Applicability::Ready,
                plan: Some(plan),
                explanation: "Formula names are the current display names of the signature's inputs; this rewrites the name to the one the signature has now.".into(),
                addresses: addresses(code),
            }
        })
        .collect()
}

fn addresses(code: &str) -> Vec<String> {
    if code.is_empty() {
        Vec::new()
    } else {
        vec![code.to_owned()]
    }
}

fn camel(name: &str) -> String {
    let mut out = String::new();
    let mut up = false;
    for ch in name.chars() {
        if ch.is_alphanumeric() {
            if up {
                out.extend(ch.to_uppercase());
                up = false;
            } else {
                out.push(ch);
            }
        } else {
            up = true;
        }
    }
    if out.is_empty() {
        "value".into()
    } else {
        out
    }
}
