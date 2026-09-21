//! The Formula Composer's protobuf adapter: `bdl-ide::formula` results
//! placed on the wire, and a wire action read back.  Rendering only —
//! nothing here decides what a formula means.

use bdl_ide::{
    ComposeOp, FormulaNode, FormulaProjection, FormulaRender, Motion, NodeKind, SemanticDiagnostic,
    SemanticSeverity, Side, SignatureHelp, SlotInfo, TypeView,
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
        element: t.element.as_deref().map(|e| Box::new(type_view_to_pb(e))),
    }
}

fn node_to_pb(n: &FormulaNode) -> pb::FormulaNode {
    let (local, param, param_type) = match &n.kind {
        NodeKind::Reference { local, .. } => (*local, String::new(), None),
        NodeKind::Binder {
            param, param_type, ..
        } => (
            false,
            param.clone(),
            param_type.as_ref().map(type_view_to_pb),
        ),
        _ => (false, String::new(), None),
    };
    let mut out = pb::FormulaNode {
        id: n.id.clone(),
        range: Some(pb::SourceSpan {
            start: n.range.start,
            end: n.range.end,
        }),
        text: n.text.clone(),
        actual: n.actual.as_ref().map(type_view_to_pb),
        expected: n.expected.as_ref().map(type_view_to_pb),
        because: n.because.clone(),
        diagnostics: n
            .diagnostics
            .iter()
            .map(semantic_diagnostic_to_pb)
            .collect(),
        children: n.children.iter().map(node_to_pb).collect(),
        local,
        param,
        param_type,
        role: n.role.clone(),
        locals: n.locals.clone(),
        append_at: n.append_at,
        ..Default::default()
    };
    match &n.kind {
        NodeKind::Reference { name, entity, .. } => {
            out.kind = "reference".into();
            out.name = name.clone();
            out.entity = entity.map(entity_to_pb);
        }
        NodeKind::Binder { form, .. } => {
            out.kind = "binder".into();
            out.name = form.clone();
        }
        NodeKind::Range => out.kind = "range".into(),
        NodeKind::Number { text } => {
            out.kind = "number".into();
            out.coordinate = text.clone();
        }
        NodeKind::Quantity {
            coordinate,
            unit,
            unit_id,
            unit_source,
            unit_display,
        } => {
            out.kind = "quantity".into();
            out.coordinate = coordinate.clone();
            out.unit = unit.clone();
            out.unit_id = unit_id.clone().unwrap_or_default();
            out.unit_source = unit_source.clone().unwrap_or_default();
            out.unit_display = unit_display.clone().unwrap_or_default();
        }
        NodeKind::Bool { value } => {
            out.kind = "bool".into();
            out.name = value.to_string();
        }
        NodeKind::Unary { op } => {
            out.kind = "unary".into();
            out.name = op.clone();
        }
        NodeKind::Binary { op } => {
            out.kind = "binary".into();
            out.name = op.clone();
        }
        NodeKind::Compare { op } => {
            out.kind = "compare".into();
            out.name = op.clone();
        }
        NodeKind::Call {
            name,
            equation,
            entity,
        } => {
            out.kind = "call".into();
            out.name = name.clone();
            out.equation = *equation;
            out.entity = entity.map(entity_to_pb);
        }
        NodeKind::Slot => out.kind = "slot".into(),
        NodeKind::If => out.kind = "if".into(),
        NodeKind::Match => out.kind = "match".into(),
        NodeKind::Arm { pattern, binds } => {
            out.kind = "arm".into();
            out.name = pattern.clone();
            out.binds = binds.clone();
        }
        NodeKind::Block => out.kind = "block".into(),
        NodeKind::Let { pattern, binds } => {
            out.kind = "let".into();
            out.name = pattern.clone();
            out.binds = binds.clone();
        }
        NodeKind::Rule { params } => {
            out.kind = "rule".into();
            out.binds = params.clone();
        }
        NodeKind::List => out.kind = "list".into(),
        NodeKind::Tuple => out.kind = "tuple".into(),
        NodeKind::Delay => out.kind = "delay".into(),
        NodeKind::Sync => out.kind = "sync".into(),
        NodeKind::Opaque { what } => {
            out.kind = "opaque".into();
            out.name = what.clone();
        }
    }
    out
}

use crate::server::entity_to_pb;

/// A semantic diagnostic as the draft's `Diagnostic`: the same code,
/// words and span the text projection carries.
fn semantic_diagnostic_to_pb(d: &SemanticDiagnostic) -> pb::Diagnostic {
    let severity = match d.severity {
        SemanticSeverity::Error => pb::DiagnosticSeverity::Error,
        SemanticSeverity::Warning => pb::DiagnosticSeverity::Warning,
        SemanticSeverity::Open | SemanticSeverity::Hint => pb::DiagnosticSeverity::Info,
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
                display: u.display.clone(),
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
        booleans: s.booleans.clone(),
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
        Action::Binder(b) => ComposeOp::Binder {
            node,
            form: b.form.clone(),
        },
        Action::Range(_) => ComposeOp::Range { node },
        Action::Choose(_) => ComposeOp::Choose { node },
        Action::Insert(i) => ComposeOp::Insert {
            node,
            side: side_from_pb(i.side())?,
            text: i.text.clone(),
        },
        Action::Apply(_) => ComposeOp::Apply { node },
        Action::Unreference(decl) => ComposeOp::Unreference {
            decl: bdl_model::DeclId::from_raw(*decl),
        },
    })
}

pub fn side_from_pb(s: pb::CaretSide) -> Option<Side> {
    match s {
        pb::CaretSide::Before => Some(Side::Before),
        pb::CaretSide::After => Some(Side::After),
        pb::CaretSide::Unspecified => None,
    }
}

pub fn side_to_pb(s: Side) -> pb::CaretSide {
    match s {
        Side::Before => pb::CaretSide::Before,
        Side::After => pb::CaretSide::After,
    }
}

pub fn motion_from_pb(m: pb::CaretMotion) -> Option<Motion> {
    Some(match m {
        pb::CaretMotion::Left => Motion::Left,
        pb::CaretMotion::Right => Motion::Right,
        pb::CaretMotion::Up => Motion::Up,
        pb::CaretMotion::Down => Motion::Down,
        pb::CaretMotion::Exit => Motion::Exit,
        pb::CaretMotion::NextSlot => Motion::NextSlot,
        pb::CaretMotion::PreviousSlot => Motion::PreviousSlot,
        pb::CaretMotion::Unspecified => return None,
    })
}

pub fn signature_to_pb(
    s: Option<&SignatureHelp>,
    revision: u64,
    mapping_id: u64,
) -> pb::FormulaSignatureResponse {
    match s {
        None => pb::FormulaSignatureResponse {
            revision,
            mapping_id,
            found: false,
            ..Default::default()
        },
        Some(s) => pb::FormulaSignatureResponse {
            revision,
            mapping_id,
            found: true,
            node_id: s.node.clone(),
            name: s.name.clone(),
            shape: s.shape.clone(),
            parameters: s
                .parameters
                .iter()
                .map(|p| pb::ParameterHelp {
                    name: p.name.clone(),
                    expected: p.expected.clone(),
                })
                .collect(),
            result: s.result.clone(),
            active: s.active,
            summary: s.summary.clone(),
        },
    }
}

pub fn render_to_pb(
    r: &FormulaRender,
    revision: u64,
    mapping_id: u64,
) -> pb::FormulaRenderResponse {
    pb::FormulaRenderResponse {
        revision,
        mapping_id,
        fragments: r
            .fragments
            .iter()
            .map(|f| pb::FormulaFragment {
                text: f.text.clone(),
                kind: f.kind.clone(),
                node_id: f.node.clone(),
                range: Some(pb::SourceSpan {
                    start: f.range.start,
                    end: f.range.end,
                }),
            })
            .collect(),
        compact: r.compact.clone(),
        result: r.result.clone(),
        error_count: r.error_count,
        warning_count: r.warning_count,
        references: r.references.clone(),
    }
}
