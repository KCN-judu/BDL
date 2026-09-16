//! Composition validation on **interfaces** (FV `Realizes`, `BindingWF`,
//! `ComposeWF`; D-68): what the flat compiler cannot phrase, or phrases
//! without the instance/port vocabulary.  Nothing here re-derives typing,
//! causality or clocks — the flat analysis remains authoritative; these
//! diagnostics are the composition-level wording of the same facts, or
//! conditions the flat design cannot express (a required port that is
//! not open).

use crate::flatten::FlattenedSystem;
use crate::model::*;
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_model::DeclId;

pub fn validate_composition(s: &BehaviorSystem, flat: &FlattenedSystem) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let design = &flat.snapshot.design;
    let concept_name = |id| {
        design
            .concepts
            .get(&id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("{id}"))
    };

    // Realizes: every port is a body declaration of the advertised shape.
    for c in s.components.values() {
        for p in c.interface.ports.values() {
            let Some(m) = c.body.mappings.get(&p.decl) else {
                out.push(
                    Diagnostic::error(
                        "system.port_missing_declaration",
                        Entity::Project,
                        format!(
                            "{}'s port {} no longer has a relationship behind it.",
                            c.name, p.name
                        ),
                    )
                    .technical(format!(
                        "{} port {} decl {} absent from body",
                        c.id, p.id, p.decl
                    )),
                );
                continue;
            };
            match p.kind {
                PortKind::Required | PortKind::Parameter if m.definition.is_some() => {
                    for inst in s.instances_of(c.id) {
                        let flat_decl = s.flat_ids.get(inst.id, LocalEntity::Decl(p.decl)).map(DeclId::from_raw);
                        out.push(
                            Diagnostic::error(
                                "system.port_not_open",
                                flat_decl.map(|id| Entity::Mapping { id }).unwrap_or(Entity::Project),
                                format!("{}.{} already has a definition inside its component.", inst.name, p.name),
                            )
                            .explain("A required port or a parameter is a hole the composer fills; a relationship with a definition is provided, not required.")
                            .technical(format!("port {} kind {:?} decl {} realized in body of {}", p.id, p.kind, p.decl, c.id)),
                        );
                    }
                }
                PortKind::Parameter if !m.signature.inputs.is_empty() => out.push(
                    Diagnostic::error("system.parameter_not_a_value", Entity::Project, format!("{}'s parameter {} is a relationship with inputs, not a value.", c.name, p.name))
                        .explain("A parameter is set to a constant at instantiation; only a relationship without inputs can be.")
                        .technical(format!("{} port {} decl {} has {} inputs", c.id, p.id, p.decl, m.signature.inputs.len())),
                ),
                _ => {}
            }
        }
        for k in &c.interface.clock_params {
            if !c.body.clocks.contains_key(k) {
                out.push(
                    Diagnostic::error(
                        "system.clock_parameter_missing",
                        Entity::Project,
                        format!(
                            "{} declares a timing-domain parameter it no longer has.",
                            c.name
                        ),
                    )
                    .technical(format!("{} clock_params ∋ {k}, absent from body", c.id)),
                );
            }
        }
    }

    // BindingWF: each binding on the two ports' interfaces, after renaming
    // (i.e. on the flattened declarations).
    for b in s.bindings.values() {
        let (Some(sp), Some(dp)) = (s.port(b.source), s.port(b.destination)) else {
            continue;
        };
        let (Some(si), Some(di)) = (
            s.instances.get(&b.source.instance),
            s.instances.get(&b.destination.instance),
        ) else {
            continue;
        };
        let src = s
            .flat_ids
            .get(b.source.instance, LocalEntity::Decl(sp.decl))
            .map(DeclId::from_raw);
        let dst = s
            .flat_ids
            .get(b.destination.instance, LocalEntity::Decl(dp.decl))
            .map(DeclId::from_raw);
        let (Some(sm), Some(dm)) = (
            src.and_then(|d| design.mappings.get(&d)),
            dst.and_then(|d| design.mappings.get(&d)),
        ) else {
            continue;
        };
        let entity = dst
            .map(|id| Entity::Mapping { id })
            .unwrap_or(Entity::Project);
        let who = format!("{}.{}", di.name, dp.name);
        let from = format!("{}.{}", si.name, sp.name);
        if sm.signature != dm.signature {
            out.push(
                Diagnostic::error("system.binding_type_mismatch", entity, format!("{who} expects {}, but {from} provides {}.", concept_name(dm.signature.output), concept_name(sm.signature.output)))
                    .explain("A binding converts nothing: the provided port must have exactly the shape the required port has, input for input, over the same concepts.")
                    .technical(format!("binding {}: {:?} ≠ {:?}", b.id, sm.signature, dm.signature)),
            );
        }
        match (&b.transport, sm.clock, dm.clock) {
            (None, Some(sc), dc) if Some(sc) != dc => out.push(
                Diagnostic::error("system.binding_needs_transport", entity, format!("{who} and {from} update in different timing domains."))
                    .explain(format!("Carry the value across with a stated initial value, or give {} the domain {} runs in.", di.name, si.name))
                    .technical(format!("binding {}: Κ src = {sc}, Κ dst = {dc:?}, no transport", b.id)),
            ),
            (Some(_), None, _) => {
                // reported by flatten as system.transport_without_source_domain
            }
            (Some(_), Some(_), None) => out.push(
                Diagnostic::error("system.transport_without_destination_domain", entity, format!("{who} is carried across domains but has no timing domain of its own."))
                    .explain("A transported value is read at the destination's activations; give the destination a domain.")
                    .technical(format!("binding {}: transport with Κ dst = none", b.id)),
            ),
            (Some(_), Some(_), Some(_)) if !dm.signature.inputs.is_empty() => out.push(
                Diagnostic::error("system.transport_of_relationship", entity, format!("{who} has inputs, so its value cannot be carried across timing domains."))
                    .technical(format!("binding {}: transport of an arrow-typed port", b.id)),
            ),
            _ => {}
        }
    }
    sort_diagnostics(&mut out);
    out
}
