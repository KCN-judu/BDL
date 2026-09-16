//! Composition validation on **interfaces** (FV `Realizes`, `BindingWF`,
//! `ComposeWF`; D-68): every component realizes its public contract, and
//! every binding is compatible on the two ports' contracts alone.  Nothing
//! here re-derives typing, causality or clocks and nothing inspects a
//! flattened declaration — the flat analysis remains the final semantic
//! authority, and these are the composition-level facts it cannot phrase
//! (a port that is not open, a body that no longer keeps its promise) in
//! the instance/port vocabulary.

use crate::contract::{binding_compatibility, realizes, resolve_end, Incompatibility};
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
        let (Some(src), Some(dst)) = (resolve_end(s, b.source), resolve_end(s, b.destination))
        else {
            continue;
        };
        let entity = dst
            .flat
            .map(|id| Entity::Mapping { id })
            .unwrap_or(Entity::Project);
        let who = &dst.label;
        let from = &src.label;
        let flat_name = |d: Option<DeclId>, fallback: &str| -> String {
            d.and_then(|d| flat.snapshot.design.mappings.get(&d))
                .and_then(|m| flat.snapshot.design.concepts.get(&m.signature.output))
                .map(|c| c.name.clone())
                .unwrap_or_else(|| fallback.to_owned())
        };
        let (dst_owner, src_owner) = (owner_name(s, b.destination), owner_name(s, b.source));
        for inc in binding_compatibility(s, b) {
            let d = match inc {
                Incompatibility::Signature => Diagnostic::error("system.binding_type_mismatch", entity, format!("{who} expects {}, but {from} provides {}.", flat_name(dst.flat, &dst.output_name), flat_name(src.flat, &src.output_name)))
                    .explain("A binding converts nothing: the provided port must promise exactly the shape the required port expects, input for input, over the same concepts. Two concepts with the same representation are still two concepts.")
                    .technical(format!("binding {}: contracts differ after resolution: {:?} vs {:?}", b.id, src.signature, dst.signature)),
                Incompatibility::Commitments => Diagnostic::error("system.binding_commitments", entity, format!("{who} relies on properties {from} does not promise."))
                    .technical(format!("binding {}: dst {:?} ⊄ src {:?}", b.id, dst.commitments, src.commitments)),
                Incompatibility::NeedsTransport => Diagnostic::error("system.binding_needs_transport", entity, format!("{who} and {from} update in different timing domains."))
                    .explain(format!("Carry the value across with a stated initial value, or give {dst_owner} the domain {src_owner} runs in."))
                    .technical(format!("binding {}: contract clocks {:?} vs {:?}, no transport", b.id, src.clock, dst.clock)),
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

/// Who owns a binding end, for advice: the instance, or the design itself.
fn owner_name(s: &BehaviorSystem, e: BindingEnd) -> String {
    match e {
        BindingEnd::Port(r) => s
            .instances
            .get(&r.instance)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "the instance".into()),
        BindingEnd::Base { decl } => s
            .base
            .mappings
            .get(&decl)
            .map(|m| m.name.clone())
            .unwrap_or_else(|| "the relationship".into()),
    }
}
