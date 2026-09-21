//! The public contract of a component and what can be decided from it
//! alone (FV `Realizes`, `BindingWF` on interfaces, `IfaceRefines`,
//! `Substitutable`; D-68): whether a body realizes its interface, whether
//! two ports may be bound, whether one component may stand in for another
//! — all without inspecting any body, and never by name.
//!
//! Concepts in a contract are component-local; their *identity* is
//! nominal: a shared concept is the system concept it stands for, a
//! private one is the component's own (fresh per instance).  Two concepts
//! with equal representations and different identities are different
//! (§13 of the brief).

use crate::ids::{ComponentInstanceId, PortId};
use crate::model::*;
use bdl_diagnostics::{Diagnostic, Entity};
use bdl_model::{ClockId, ConceptId, DeclId};
use serde::{Deserialize, Serialize};

/// A concept of a contract, resolved to what it *is* in the system.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ResolvedConcept {
    /// A system concept (through the component's sharing table).
    Shared(ConceptId),
    /// A concept private to one instance.
    Private {
        instance: ComponentInstanceId,
        local: ConceptId,
    },
}

/// The clock of a contract, resolved through one instance's arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResolvedClock {
    Agnostic,
    System(ClockId),
    Private {
        instance: ComponentInstanceId,
        local: ClockId,
    },
    /// A clock parameter the instance has not assigned.
    Unbound(ClockId),
}

pub fn resolve_concept(
    c: &BehaviorComponent,
    instance: ComponentInstanceId,
    local: ConceptId,
) -> ResolvedConcept {
    match c.shared_concepts.get(&local) {
        Some(g) => ResolvedConcept::Shared(*g),
        None => ResolvedConcept::Private { instance, local },
    }
}

pub fn resolve_signature(
    c: &BehaviorComponent,
    instance: ComponentInstanceId,
    sig: &bdl_model::surface::Signature,
) -> (Vec<ResolvedConcept>, ResolvedConcept) {
    (
        sig.inputs
            .iter()
            .map(|s| resolve_concept(c, instance, *s))
            .collect(),
        resolve_concept(c, instance, sig.output),
    )
}

pub fn resolve_clock(inst: &ComponentInstance, clock: ClockContract) -> ResolvedClock {
    match clock {
        ClockContract::Agnostic => ResolvedClock::Agnostic,
        ClockContract::Parameter { clock } => match inst.clock_bindings.get(&clock) {
            Some(sys) => ResolvedClock::System(*sys),
            None => ResolvedClock::Unbound(clock),
        },
        ClockContract::Private { clock } => ResolvedClock::Private {
            instance: inst.id,
            local: clock,
        },
    }
}

fn concept_name(c: &BehaviorComponent, s: ConceptId) -> String {
    c.body
        .concepts
        .get(&s)
        .map(|x| x.name.clone())
        .unwrap_or_else(|| s.to_string())
}

fn sig_text(c: &BehaviorComponent, sig: &bdl_model::surface::Signature) -> String {
    let ins: Vec<String> = sig.inputs.iter().map(|s| concept_name(c, *s)).collect();
    if ins.is_empty() {
        concept_name(c, sig.output)
    } else {
        format!("({}) -> {}", ins.join(", "), concept_name(c, sig.output))
    }
}

fn clock_text(c: &BehaviorComponent, k: ClockContract) -> String {
    match k {
        ClockContract::Agnostic => "any timing domain".into(),
        ClockContract::Parameter { clock } | ClockContract::Private { clock } => c
            .body
            .clocks
            .get(&clock)
            .map(|x| x.name.clone())
            .unwrap_or_else(|| clock.to_string()),
    }
}

/// **`Realizes`** — the component body realizes its public interface.  The
/// production analogue of FV `BehaviorComponent.Realizes`, decided on the
/// component alone (no instance, no flat design): every diagnostic names
/// the component, the port and the backing declaration.
pub fn realizes(c: &BehaviorComponent) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let d = |code: &'static str, msg: String| Diagnostic::error(code, Entity::Project, msg);
    for p in c.interface.ports.values() {
        let Some(m) = c.body.mappings.get(&p.decl) else {
            out.push(
                d("component.port_declaration_missing", format!("{}'s port {} has no relationship behind it.", c.name, p.name))
                    .explain("A port promises what a relationship of the component provides or needs; point the port at a relationship of the body, or retire it.")
                    .technical(format!("{} port {} decl {} absent from body", c.id, p.id, p.decl)),
            );
            continue;
        };
        let actual = PortContract::of_declaration(m, &c.interface);
        if actual.signature != p.contract.signature {
            out.push(
                d("component.port_signature_mismatch", format!("{}'s port {} promises {}, but {} is {}.", c.name, p.name, sig_text(c, &p.contract.signature), m.name, sig_text(c, &m.signature)))
                    .explain("The public contract is what other components bind against; it does not follow the body. Either restore the relationship's shape, or change the port's contract explicitly — that reopens every binding on it.")
                    .technical(format!("{} port {} contract {:?} ≠ body {} signature {:?}", c.id, p.id, p.contract.signature, p.decl, m.signature)),
            );
        }
        if actual.clock != p.contract.clock {
            out.push(
                d("component.port_clock_mismatch", format!("{}'s port {} promises to update in {}, but {} updates in {}.", c.name, p.name, clock_text(c, p.contract.clock), m.name, clock_text(c, actual.clock)))
                    .explain("Timing is part of the promise: a consumer bound to this port relies on the domain it advertises.")
                    .technical(format!("{} port {} contract clock {:?} ≠ body {} clock {:?}", c.id, p.id, p.contract.clock, p.decl, actual.clock)),
            );
        }
        if !p
            .contract
            .commitments
            .iter()
            .all(|k| actual.commitments.contains(k))
        {
            out.push(
                d(
                    "component.port_commitment_unrealized",
                    format!(
                        "{}'s port {} promises properties {} does not establish.",
                        c.name, p.name, m.name
                    ),
                )
                .technical(format!(
                    "{} port {} commitments {:?}, body {:?}",
                    c.id, p.id, p.contract.commitments, actual.commitments
                )),
            );
        }
        match p.kind {
            PortKind::Required if m.definition.is_some() => out.push(
                d("component.required_port_realized", format!("{}'s required port {} is defined inside the component.", c.name, p.name))
                    .explain("A required port is a hole the composer fills; a relationship with a definition is provided, not required. Detach the definition, or make the port provided.")
                    .technical(format!("{} port {} decl {} has a definition", c.id, p.id, p.decl)),
            ),
            PortKind::Parameter => {
                if m.definition.is_some() {
                    out.push(
                        d("component.parameter_invalid", format!("{}'s parameter {} is defined inside the component.", c.name, p.name))
                            .explain("A parameter is set to a constant by each instance; a relationship with a definition cannot be.")
                            .technical(format!("{} port {} decl {} has a definition", c.id, p.id, p.decl)),
                    );
                }
                if !p.contract.signature.is_unit_domain() {
                    out.push(
                        d("component.parameter_invalid", format!("{}'s parameter {} takes inputs; a parameter is a value (its domain is `()`).", c.name, p.name))
                            .technical(format!("{} port {} has {} inputs", c.id, p.id, p.contract.signature.inputs.len())),
                    );
                }
                if p.contract.clock != ClockContract::Agnostic {
                    out.push(
                        d("component.parameter_invalid", format!("{}'s parameter {} names a timing domain; a parameter is constant.", c.name, p.name))
                            .technical(format!("{} port {} clock {:?}", c.id, p.id, p.contract.clock)),
                    );
                }
            }
            _ => {}
        }
        // The contract's concepts must be concepts of the component.
        for s in p
            .contract
            .signature
            .inputs
            .iter()
            .chain([&p.contract.signature.output])
        {
            if !c.body.concepts.contains_key(s) {
                out.push(
                    d(
                        "component.port_concept_missing",
                        format!(
                            "{}'s port {} promises a concept the component no longer has.",
                            c.name, p.name
                        ),
                    )
                    .technical(format!(
                        "{} port {} mentions {s}, absent from body",
                        c.id, p.id
                    )),
                );
            }
        }
        if let Some(k) = p.contract.clock.local() {
            let is_param = c.interface.is_clock_param(k);
            let says_param = matches!(p.contract.clock, ClockContract::Parameter { .. });
            if !c.body.clocks.contains_key(&k) {
                out.push(
                    d(
                        "component.port_clock_missing",
                        format!(
                            "{}'s port {} promises a timing domain the component no longer has.",
                            c.name, p.name
                        ),
                    )
                    .technical(format!(
                        "{} port {} clock {k}, absent from body",
                        c.id, p.id
                    )),
                );
            } else if is_param != says_param {
                out.push(
                    d("component.port_clock_mismatch", format!("{}'s port {} promises {} as a {}, but it is {}.", c.name, p.name, clock_text(c, p.contract.clock), if says_param { "parameter" } else { "private domain" }, if is_param { "a parameter" } else { "private" }))
                        .technical(format!("{} port {} clock {k}: contract says parameter={says_param}, interface says {is_param}", c.id, p.id)),
                );
            }
        }
    }
    for k in &c.interface.clock_params {
        if !c.body.clocks.contains_key(k) {
            out.push(
                d(
                    "component.clock_parameter_missing",
                    format!(
                        "{} declares a timing-domain parameter it no longer has.",
                        c.name
                    ),
                )
                .technical(format!("{} clock_params ∋ {k}, absent from body", c.id)),
            );
        }
    }
    for local in c.shared_concepts.keys() {
        if !c.body.concepts.contains_key(local) {
            out.push(
                d(
                    "component.shared_concept_missing",
                    format!("{} shares a concept it no longer has.", c.name),
                )
                .technical(format!(
                    "{} shared_concepts ∋ {local}, absent from body",
                    c.id
                )),
            );
        }
    }
    for local in c.external_outputs.keys() {
        if !c.body.outputs.contains_key(local) {
            out.push(
                d(
                    "component.external_output_missing",
                    format!("{} drives an external output it no longer has.", c.name),
                )
                .technical(format!(
                    "{} external_outputs ∋ {local}, absent from body",
                    c.id
                )),
            );
        }
    }
    out
}

/// Why two ports may not be bound, decided on their contracts alone.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Incompatibility {
    /// Different shape, or the same shape over different concepts.
    Signature,
    /// The destination promises properties the source does not establish.
    Commitments,
    /// Different domains with no transport.
    NeedsTransport,
    /// Transport of an agnostic source.
    TransportWithoutSourceDomain,
    /// Transport into an agnostic destination.
    TransportWithoutDestinationDomain,
    /// Transport of a relationship with inputs.
    TransportOfRelationship,
}

/// A binding end resolved to what the flat design will see: its flat
/// declaration, its promise over system-level identities, its clock, and
/// a label for messages.  A port end resolves through its instance; a base
/// end *is* the system's own relationship, so every concept is shared and
/// its domain is the system's.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedEnd {
    pub flat: Option<DeclId>,
    pub label: String,
    pub signature: (Vec<ResolvedConcept>, ResolvedConcept),
    pub clock: ResolvedClock,
    pub commitments: Vec<bdl_ir::PropertyId>,
    /// The name of the concept the end produces, for messages.
    pub output_name: String,
}

impl ResolvedEnd {
    /// The end reads inputs: it is a relationship, not a value that a
    /// transport can carry (a value's domain is `()`).
    pub fn has_inputs(&self) -> bool {
        !self.signature.0.is_empty()
    }
}

pub fn resolve_end(s: &BehaviorSystem, e: BindingEnd) -> Option<ResolvedEnd> {
    match e {
        BindingEnd::Port(r) => {
            let c = s.component_of(r.instance)?;
            let p = c.interface.ports.get(&r.port)?;
            let i = s.instances.get(&r.instance)?;
            Some(ResolvedEnd {
                flat: s
                    .flat_ids
                    .get(r.instance, LocalEntity::Decl(p.decl))
                    .map(DeclId::from_raw),
                label: format!("{}.{}", i.name, p.name),
                signature: resolve_signature(c, i.id, &p.contract.signature),
                clock: resolve_clock(i, p.contract.clock),
                commitments: p.contract.commitments.clone(),
                output_name: concept_name(c, p.contract.signature.output),
            })
        }
        BindingEnd::Base { decl } => {
            let m = s.base.mappings.get(&decl)?;
            Some(ResolvedEnd {
                flat: Some(decl),
                label: m.name.clone(),
                signature: (
                    m.signature
                        .inputs
                        .iter()
                        .map(|c| ResolvedConcept::Shared(*c))
                        .collect(),
                    ResolvedConcept::Shared(m.signature.output),
                ),
                clock: m
                    .clock
                    .map(ResolvedClock::System)
                    .unwrap_or(ResolvedClock::Agnostic),
                commitments: Vec::new(),
                output_name: s
                    .base
                    .concepts
                    .get(&m.signature.output)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| m.signature.output.to_string()),
            })
        }
    }
}

/// FV `BindingWF` on the two contracts: the source's resolved signature
/// equals the destination's, commitments are included, and the clocks
/// agree — equal, agnostic source, or bridged by transport.
pub fn binding_compatibility(s: &BehaviorSystem, b: &Binding) -> Vec<Incompatibility> {
    let mut out = Vec::new();
    let (Some(src), Some(dst)) = (resolve_end(s, b.source), resolve_end(s, b.destination)) else {
        return out;
    };
    if src.signature != dst.signature {
        out.push(Incompatibility::Signature);
    }
    if !dst.commitments.iter().all(|k| src.commitments.contains(k)) {
        out.push(Incompatibility::Commitments);
    }
    match (&b.transport, src.clock, dst.clock) {
        (None, ResolvedClock::Agnostic, _) => {}
        (None, a, d) if a == d => {}
        (None, _, _) => out.push(Incompatibility::NeedsTransport),
        (Some(_), ResolvedClock::Agnostic, _) => {
            out.push(Incompatibility::TransportWithoutSourceDomain)
        }
        (Some(_), _, ResolvedClock::Agnostic) => {
            out.push(Incompatibility::TransportWithoutDestinationDomain)
        }
        (Some(_), _, _) if dst.has_inputs() => out.push(Incompatibility::TransportOfRelationship),
        _ => {}
    }
    out
}

/// Why a component cannot stand in for another at a port.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubstitutionProblem {
    pub port: PortId,
    pub reason: SubstitutionReason,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SubstitutionReason {
    /// The replacement has no port with this identity.
    PortMissing,
    /// A different kind of port.
    Kind,
    /// The replacement's contract does not refine the original's.
    Contract,
    /// A clock parameter in use is not a parameter of the replacement.
    ClockParameterMissing,
    /// The replacement shares a concept differently.
    Sharing,
}

/// **Substitutability** (FV `IfaceRefines` + `Substitutable`, D-68): the
/// replacement has, for every port of the original the system uses, a
/// port with the same identity and kind whose contract refines the
/// original's — over the same concept identities (a shared concept is the
/// same system concept; a private one is the same local concept: the
/// replacement is a *version*, as `DuplicateComponent` produces) — and
/// keeps every clock parameter in use.  Decided on interfaces only; the
/// replacement's body is checked by `realizes`, never here.  No claim
/// about behaviour.
pub fn component_substitutable(
    old: &BehaviorComponent,
    new: &BehaviorComponent,
    used_ports: &[PortId],
    used_clock_params: &[ClockId],
) -> Result<(), Vec<SubstitutionProblem>> {
    let mut problems = Vec::new();
    for pid in used_ports {
        let Some(op) = old.interface.ports.get(pid) else {
            continue;
        };
        let Some(np) = new.interface.ports.get(pid) else {
            problems.push(SubstitutionProblem {
                port: *pid,
                reason: SubstitutionReason::PortMissing,
            });
            continue;
        };
        if op.kind != np.kind {
            problems.push(SubstitutionProblem {
                port: *pid,
                reason: SubstitutionReason::Kind,
            });
            continue;
        }
        let same_concepts = |sig_o: &bdl_model::surface::Signature,
                             sig_n: &bdl_model::surface::Signature| {
            let at = |c: &BehaviorComponent, s: ConceptId| match c.shared_concepts.get(&s) {
                Some(g) => ResolvedConcept::Shared(*g),
                None => ResolvedConcept::Private {
                    instance: ComponentInstanceId::from_raw(0),
                    local: s,
                },
            };
            sig_o.inputs.len() == sig_n.inputs.len()
                && sig_o
                    .inputs
                    .iter()
                    .zip(&sig_n.inputs)
                    .all(|(a, b)| at(old, *a) == at(new, *b))
                && at(old, sig_o.output) == at(new, sig_n.output)
        };
        if !same_concepts(&op.contract.signature, &np.contract.signature) {
            problems.push(SubstitutionProblem {
                port: *pid,
                reason: SubstitutionReason::Sharing,
            });
            continue;
        }
        // Concepts were compared by identity above; the rest of
        // `IfaceRefines`: same clock, commitments in the right direction.
        let commitments_ok = match op.kind {
            PortKind::Provided => op
                .contract
                .commitments
                .iter()
                .all(|c| np.contract.commitments.contains(c)),
            PortKind::Required | PortKind::Parameter => np
                .contract
                .commitments
                .iter()
                .all(|c| op.contract.commitments.contains(c)),
        };
        if op.contract.clock != np.contract.clock || !commitments_ok {
            problems.push(SubstitutionProblem {
                port: *pid,
                reason: SubstitutionReason::Contract,
            });
        }
    }
    for k in used_clock_params {
        if !new.interface.is_clock_param(*k) {
            problems.push(SubstitutionProblem {
                port: PortId::from_raw(u64::MAX),
                reason: SubstitutionReason::ClockParameterMissing,
            });
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems)
    }
}

/// The declaration a port is realized by, for diagnostics.
pub fn backing_declaration(c: &BehaviorComponent, port: PortId) -> Option<DeclId> {
    c.interface.ports.get(&port).map(|p| p.decl)
}
