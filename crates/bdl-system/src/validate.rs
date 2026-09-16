//! Composition validation on **interfaces** (FV `Realizes`, `BindingWF`,
//! `ComposeWF`; D-68): every component realizes its public contract, and
//! every binding is compatible on the two ports' contracts alone.  Nothing
//! here re-derives typing, causality or clocks and nothing inspects a
//! flattened declaration — the flat analysis remains the final semantic
//! authority, and these are the composition-level facts it cannot phrase
//! (a port that is not open, a body that no longer keeps its promise) in
//! the instance/port vocabulary.

use crate::contract::{binding_compatibility, realizes, Incompatibility};
use crate::flatten::FlattenedSystem;
use crate::model::*;
use bdl_diagnostics::{sort_diagnostics, Diagnostic, Entity};
use bdl_model::DeclId;

pub fn validate_composition(s: &BehaviorSystem, flat: &FlattenedSystem) -> Vec<Diagnostic> {
    let mut out = Vec::new();

    // Realizes, once per component (evidence is equivariant: the verdict
    // holds for every instance).
    for c in s.components.values() {
        out.extend(realizes(c));
    }

    // BindingWF on contracts.
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
        let dst = s
            .flat_ids
            .get(b.destination.instance, LocalEntity::Decl(dp.decl))
            .map(DeclId::from_raw);
        let entity = dst
            .map(|id| Entity::Mapping { id })
            .unwrap_or(Entity::Project);
        let who = format!("{}.{}", di.name, dp.name);
        let from = format!("{}.{}", si.name, sp.name);
        let flat_name = |r: PortRef, p: &Port| -> String {
            let d = s
                .flat_ids
                .get(r.instance, LocalEntity::Decl(p.decl))
                .map(DeclId::from_raw);
            d.and_then(|d| flat.snapshot.design.mappings.get(&d))
                .map(|m| {
                    let c = |id| {
                        flat.snapshot
                            .design
                            .concepts
                            .get(&id)
                            .map(|x| x.name.clone())
                            .unwrap_or_else(|| format!("{id}"))
                    };
                    c(m.signature.output)
                })
                .unwrap_or_default()
        };
        for inc in binding_compatibility(s, b) {
            let d = match inc {
                Incompatibility::Signature => Diagnostic::error("system.binding_type_mismatch", entity, format!("{who} expects {}, but {from} provides {}.", flat_name(b.destination, dp), flat_name(b.source, sp)))
                    .explain("A binding converts nothing: the provided port must promise exactly the shape the required port expects, input for input, over the same concepts. Two concepts with the same representation are still two concepts.")
                    .technical(format!("binding {}: contracts differ after resolution: {:?} vs {:?}", b.id, sp.contract.signature, dp.contract.signature)),
                Incompatibility::Commitments => Diagnostic::error("system.binding_commitments", entity, format!("{who} relies on properties {from} does not promise."))
                    .technical(format!("binding {}: dst {:?} ⊄ src {:?}", b.id, dp.contract.commitments, sp.contract.commitments)),
                Incompatibility::NeedsTransport => Diagnostic::error("system.binding_needs_transport", entity, format!("{who} and {from} update in different timing domains."))
                    .explain(format!("Carry the value across with a stated initial value, or give {} the domain {} runs in.", di.name, si.name))
                    .technical(format!("binding {}: contract clocks {:?} vs {:?}, no transport", b.id, sp.contract.clock, dp.contract.clock)),
                Incompatibility::TransportWithoutSourceDomain => Diagnostic::error("system.transport_without_source_domain", entity, format!("{who} is carried across domains, but {from} promises no timing domain."))
                    .explain("Transport reads the source's last activation strictly before now; a source with no domain has no activations. Bind it directly instead.")
                    .technical(format!("binding {}: transport with an agnostic source contract", b.id)),
                Incompatibility::TransportWithoutDestinationDomain => Diagnostic::error("system.transport_without_destination_domain", entity, format!("{who} is carried across domains but promises no timing domain of its own."))
                    .explain("A transported value is read at the destination's activations; give the destination a domain.")
                    .technical(format!("binding {}: transport with an agnostic destination contract", b.id)),
                Incompatibility::TransportOfRelationship => Diagnostic::error("system.transport_of_relationship", entity, format!("{who} has inputs, so its value cannot be carried across timing domains."))
                    .technical(format!("binding {}: transport of an arrow-typed port", b.id)),
            };
            out.push(d);
        }
    }
    sort_diagnostics(&mut out);
    out
}
