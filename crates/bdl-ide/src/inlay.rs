//! Inlay hints: what the model knows about a line that the text does not
//! say — the concept a definition's parameter stands for, and the
//! transport a binding needs when it crosses timing domains.

use bdl_ide_db::{AnalysisSnapshot, DocumentId, EntityRef, EntityRole, TextRange};
use bdl_syntax::ast::{self, AstNode};
use bdl_system::{BindingEnd, LocalEntity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InlayKind {
    /// After a parameter: the concept it reads.
    Type,
    /// After a binding: how the value is transported.
    Transport,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InlayHint {
    pub offset: u32,
    pub label: String,
    pub kind: InlayKind,
    /// The entity the hint is about, for navigation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityRef>,
}

/// The hints of a document within `range` (the whole document when
/// `None`), in position order.
pub fn inlay_hints(
    snapshot: &AnalysisSnapshot,
    document: DocumentId,
    range: Option<TextRange>,
) -> Vec<InlayHint> {
    let Some(doc) = snapshot.document(document) else {
        return Vec::new();
    };
    let source = doc.source.as_str();
    let design = &snapshot.effective().design;
    let mut out = Vec::new();

    // Parameters: `f(t) = …` where `t` reads a concept named otherwise.
    let parse = bdl_syntax::parse_module(source);
    for node in parse.syntax_node().descendants() {
        let Some(def) = ast::MappingDef::cast(node) else {
            continue;
        };
        let at = def.span().start;
        let mapping = snapshot
            .projections()
            .document_anchors(document)
            .iter()
            .filter(|a| a.role == EntityRole::Declaration)
            .filter(|a| a.text_range().is_some_and(|r| r.contains(at)))
            .find_map(|a| a.entity.as_mapping());
        let Some(m) = mapping.and_then(|m| design.mappings.get(&m)) else {
            continue;
        };
        for (i, p) in def.params().enumerate() {
            let ast::Pattern::Ident(ident) = p else {
                continue;
            };
            let Some(name) = ident.name() else { continue };
            let Some(concept) = m
                .signature
                .inputs
                .get(i)
                .and_then(|c| design.concepts.get(c))
            else {
                continue;
            };
            let spelled = name.text();
            if spelled.eq_ignore_ascii_case(&concept.name) {
                continue;
            }
            out.push(InlayHint {
                offset: name.span().end,
                label: format!(": {}", concept.name),
                kind: InlayKind::Type,
                entity: Some(EntityRef::Concept(concept.id)),
            });
        }
    }

    // Bindings that cross domains.
    if let Some(world) = snapshot.text() {
        let system = &world.system;
        let flat_of = |end: BindingEnd| -> Option<bdl_model::DeclId> {
            match end {
                BindingEnd::Base { decl } => Some(decl),
                BindingEnd::Port(r) => {
                    let i = system.instances.get(&r.instance)?;
                    let c = system.components.get(&i.component)?;
                    let port = c.interface.ports.get(&r.port)?;
                    system
                        .flat_ids
                        .get(r.instance, LocalEntity::Decl(port.decl))
                        .map(bdl_model::DeclId::from_raw)
                }
            }
        };
        let clock_name = |d: bdl_model::DeclId| -> Option<String> {
            let c = design.mappings.get(&d)?.clock?;
            design.clocks.get(&c).map(|k| k.name.clone())
        };
        for b in system.bindings.values() {
            let Some(t) = &b.transport else { continue };
            let entity = EntityRef::Binding(b.id.raw());
            let Some(a) = snapshot
                .projections()
                .anchors_for(entity, EntityRole::Declaration)
                .find(|a| a.document() == Some(document))
            else {
                continue;
            };
            let Some(r) = a.text_range() else { continue };
            let from = flat_of(b.source).and_then(clock_name);
            let to = flat_of(b.destination).and_then(clock_name);
            let label = match (from, to) {
                (Some(f), Some(t2)) => format!("sync {f} → {t2}, init {}", t.init),
                _ => format!("sync, init {}", t.init),
            };
            out.push(InlayHint {
                offset: r.end,
                label,
                kind: InlayKind::Transport,
                entity: Some(entity),
            });
        }
    }

    out.sort_by_key(|h| h.offset);
    match range {
        Some(r) => out.into_iter().filter(|h| r.contains(h.offset)).collect(),
        None => out,
    }
}
