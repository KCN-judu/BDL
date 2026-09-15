//! The visual projection: where every entity is on the Studio canvas and
//! in its inspector, derived from the model alone.
//!
//! Studio identifies nodes, ports and edges by the model's ids already, so
//! the visual projection needs no layout data: a mapping *is* a node, its
//! `i`-th signature input *is* a port, its drive `β d = o` *is* an edge.
//! Canvas geometry (positions, zoom) is layout, never semantics (ADR-0003),
//! and never appears here.

use crate::entity::{EntityRef, EntityRole};
use crate::projection::{ProjectionAnchor, ProjectionMap, VisualElementRef as V};
use bdl_model::surface::Design;

/// Anchors for every entity of `design` on the visual surface.
pub fn visual_projection(design: &Design) -> ProjectionMap {
    let mut map = ProjectionMap::default();
    let mut add = |e: EntityRef, role: EntityRole, v: V| {
        map.insert(ProjectionAnchor::visual(e, role, v));
    };
    for c in design.concepts.values() {
        let e = EntityRef::Concept(c.id);
        add(e, EntityRole::Declaration, V::ConceptNode(c.id));
        add(e, EntityRole::Name, V::NameField(e));
        add(e, EntityRole::Representation, V::RepresentationField(c.id));
    }
    for m in design.mappings.values() {
        let e = EntityRef::Mapping(m.id);
        add(e, EntityRole::Declaration, V::MappingNode(m.id));
        add(e, EntityRole::Name, V::NameField(e));
        add(e, EntityRole::Signature, V::SignatureField(m.id));
        add(e, EntityRole::Definition, V::DefinitionField(m.id));
        add(e, EntityRole::ClockBinding, V::ClockBadge(m.id));
        for (i, c) in m.signature.inputs.iter().enumerate() {
            let port = V::InputPort {
                mapping: m.id,
                index: i as u16,
            };
            add(e, EntityRole::Input { index: i as u16 }, port);
            add(EntityRef::Concept(*c), EntityRole::Reference, port);
        }
        add(e, EntityRole::Output, V::OutputPort(m.id));
        add(
            EntityRef::Concept(m.signature.output),
            EntityRole::Reference,
            V::OutputPort(m.id),
        );
        if let Some(o) = m.drives {
            let edge = V::DriveEdge {
                mapping: m.id,
                output: o,
            };
            add(e, EntityRole::DriveEdge, edge);
            add(EntityRef::Output(o), EntityRole::Reference, edge);
        }
    }
    for o in design.outputs.values() {
        let e = EntityRef::Output(o.id);
        add(e, EntityRole::Declaration, V::OutputTerminal(o.id));
        add(e, EntityRole::Name, V::NameField(e));
        add(e, EntityRole::Output, V::OutputTerminal(o.id));
        add(e, EntityRole::ClockBinding, V::OutputClockBadge(o.id));
        add(
            EntityRef::Concept(o.accepts),
            EntityRole::Reference,
            V::OutputTerminal(o.id),
        );
        if let Some(c) = o.clock {
            add(
                EntityRef::Clock(c),
                EntityRole::Reference,
                V::OutputClockBadge(o.id),
            );
        }
    }
    for m in design.mappings.values() {
        if let Some(c) = m.clock {
            add(
                EntityRef::Clock(c),
                EntityRole::Reference,
                V::ClockBadge(m.id),
            );
        }
    }
    for c in design.clocks.values() {
        let e = EntityRef::Clock(c.id);
        add(e, EntityRole::Name, V::NameField(e));
    }
    for d in design.devices.values() {
        let e = EntityRef::Device(d.id);
        add(e, EntityRole::Declaration, V::DeviceNode(d.id));
        add(e, EntityRole::Name, V::NameField(e));
        if let Some(o) = d.output {
            add(e, EntityRole::DeviceBinding, V::OutputTerminal(o));
            add(
                EntityRef::Output(o),
                EntityRole::DeviceBinding,
                V::DeviceNode(d.id),
            );
        }
    }
    map.finish();
    map
}
