//! The Formula Composer's protobuf adapter: `bdl-ide::formula` results
//! placed on the wire, and a wire action read back.  Rendering only —
//! nothing here decides what a formula means.

use bdl_ide::{
    ComposeOp, FormulaNode, FormulaProjection, NodeKind, SemanticDiagnostic, SemanticSeverity,
    SlotInfo, TypeView,
};
use bdl_protocol::convert;
use bdl_protocol::pb;

pub fn projection_to_pb(p: &FormulaProjection) -> pb::FormulaProjection {
    pb::FormulaProjection {
        source: p.source.clone(),
        parse_ok: p.parse_ok,
        complete: p.complete,
        root: p.root.as_ref().map(node_to_pb),
        result: p.result.as_ref().map(type_view_to_pb),
        slots: p.slots.clone(),
        unplaced: p.unplaced.iter().map(semantic_diagnostic_to_pb).collect(),
        draft_generation: p.draft_generation.map(|g| g.0),
    }
}

fn type_view_to_pb(t: &TypeView) -> pb::TypeView {
    pb::TypeView {
        description: t.description.clone(),
        kind: format!("{:?}", t.kind).to_lowercase(),
        dim: t.dim.map(convert::dim_to_pb),
        concept_id: t.concept.map(|c| c.raw()),
    }
}

fn node_to_pb(n: &FormulaNode) -> pb::FormulaNode {
    let (kind, name, coordinate, unit, unit_id, equation, entity) = match &n.kind {
        NodeKind::Reference { name, entity } => (
            "reference",
            name.clone(),
            String::new(),
            String::new(),
            String::new(),
            false,
            *entity,
        ),
        NodeKind::Number { text } => (
            "number",
            String::new(),
            text.clone(),
            String::new(),
            String::new(),
            false,
            None,
        ),
        NodeKind::Quantity {
            coordinate,
            unit,
            unit_id,
        } => (
            "quantity",
            String::new(),
            coordinate.clone(),
            unit.clone(),
            unit_id.clone().unwrap_or_default(),
            false,
            None,
        ),
        NodeKind::Bool { value } => (
            "bool",
            value.to_string(),
            String::new(),
            String::new(),
            String::new(),
            false,
            None,
        ),
        NodeKind::Unary { op } => (
            "unary",
            op.clone(),
            String::new(),
            String::new(),
            String::new(),
            false,
            None,
        ),
        NodeKind::Binary { op } => (
            "binary",
            op.clone(),
            String::new(),
            String::new(),
            String::new(),
            false,
            None,
        ),
        NodeKind::Compare { op } => (
            "compare",
            op.clone(),
            String::new(),
            String::new(),
            String::new(),
            false,
            None,
        ),
        NodeKind::Call {
            name,
            equation,
            entity,
        } => (
            "call",
            name.clone(),
            String::new(),
            String::new(),
            String::new(),
            *equation,
            *entity,
        ),
        NodeKind::Slot => (
            "slot",
            String::new(),
            String::new(),
            String::new(),
            String::new(),
            false,
            None,
        ),
        NodeKind::Opaque { what } => (
            "opaque",
            what.clone(),
            String::new(),
            String::new(),
            String::new(),
            false,
            None,
        ),
    };
    pb::FormulaNode {
        id: n.id.clone(),
        range: Some(pb::SourceSpan {
            start: n.range.start,
            end: n.range.end,
        }),
        kind: kind.into(),
        text: n.text.clone(),
        name,
        coordinate,
        unit,
        unit_id,
        equation,
        entity: entity.map(entity_to_pb),
        actual: n.actual.as_ref().map(type_view_to_pb),
        expected: n.expected.as_ref().map(type_view_to_pb),
        because: n.because.clone(),
        diagnostics: n
            .diagnostics
            .iter()
            .map(semantic_diagnostic_to_pb)
            .collect(),
        children: n.children.iter().map(node_to_pb).collect(),
    }
}

use crate::server::entity_to_pb;

/// A semantic diagnostic as the draft's `Diagnostic`: the same code,
/// words and span the text projection carries.
fn semantic_diagnostic_to_pb(d: &SemanticDiagnostic) -> pb::Diagnostic {
    let severity = match d.severity {
        SemanticSeverity::Error => pb::DiagnosticSeverity::Error,
        SemanticSeverity::Warning => pb::DiagnosticSeverity::Warning,
        SemanticSeverity::Open => pb::DiagnosticSeverity::Info,
    };
    let entity = match d.primary.entity {
        bdl_ide::EntityRef::Concept(id) => pb::diagnostic::Entity::ConceptId(id.raw()),
        bdl_ide::EntityRef::Mapping(id) => pb::diagnostic::Entity::MappingId(id.raw()),
        _ => pb::diagnostic::Entity::Project(pb::Unit {}),
    };
    pb::Diagnostic {
        code: d.code.clone(),
        severity: severity.into(),
        entity: Some(entity),
        span: d.primary.source.as_ref().map(|s| pb::SourceSpan {
            start: s.range.start,
            end: s.range.end,
        }),
        message: d.message.clone(),
        explanation: d.explanation.clone(),
        technical: d.technical.clone(),
        fixes: d.fixes.clone(),
    }
}

pub fn slot_to_pb(s: &SlotInfo, revision: u64, mapping_id: u64) -> pb::FormulaSlotResponse {
    pb::FormulaSlotResponse {
        revision,
        mapping_id,
        node_id: s.node.clone(),
        expected: s.expected.as_ref().map(type_view_to_pb),
        explanation: s.explanation.clone(),
        technical: s.technical.clone(),
        insufficient: s.insufficient,
        units: s
            .units
            .iter()
            .map(|u| pb::UnitCandidate {
                id: u.id.clone(),
                symbol: u.symbol.clone(),
                measures: u.measures.clone(),
            })
            .collect(),
        references: s
            .references
            .iter()
            .map(|r| pb::ReferenceCandidate {
                label: r.label.clone(),
                insert: r.insert.clone(),
                entity: r.entity.map(entity_to_pb),
                produces: r.produces.clone(),
                relevance: u32::from(r.relevance),
            })
            .collect(),
        equations: s
            .equations
            .iter()
            .map(|e| pb::EquationCandidate {
                name: e.name.clone(),
                shape: e.shape.clone(),
                insert: e.insert.clone(),
                summary: e.summary.clone(),
            })
            .collect(),
    }
}

pub fn action_from_pb(a: &pb::ComposeAction) -> Option<ComposeOp> {
    use pb::compose_action::Action;
    let node = a.node_id.clone();
    Some(match a.action.as_ref()? {
        Action::Fill(text) => ComposeOp::Fill {
            node,
            text: text.clone(),
        },
        Action::Operator(o) => ComposeOp::Operator {
            node,
            op: o.op.clone(),
            before: o.before,
        },
        Action::Call(c) => ComposeOp::Call {
            node,
            name: c.name.clone(),
            arity: c.arity,
        },
        Action::SetUnit(u) => ComposeOp::SetUnit {
            node,
            unit_id: u.unit_id.clone(),
            preserve_value: u.preserve_value,
        },
        Action::SetCoordinate(text) => ComposeOp::SetCoordinate {
            node,
            text: text.clone(),
        },
        Action::Remove(_) => ComposeOp::Remove { node },
    })
}
