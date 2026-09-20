//! What is *at* a position, for every editor at once: the name under a
//! byte offset of a document or of a formula, resolved by identity and
//! never by spelling, and the hover, definition and reference answers
//! built on it.  The language server and the daemon both call these, so
//! a client that hovers a source file, a client that hovers a formula
//! field and an LSP editor read one implementation
//! (`docs/architecture/ide-service.md`, ADR-0035).
//!
//! A name resolves through the projection map (an anchor in the `Name`
//! or `Reference` role) or, inside a formula body, through the
//! elaborator's own input environment; a call to the equation library
//! is a name the design does not own and gets the library's words.
//! Anything else — whitespace, a keyword, a number, a local — has no
//! semantic hover, and the answer is `None`, cleanly.

use crate::completion::equation_documentation;
use crate::hover::{hover, SemanticHover};
use crate::references::references;
use bdl_ide_db::{
    AnalysisSnapshot, DocumentId, EntityRef, EntityRole, ProjectionAnchor, TextRange,
};
use bdl_model::surface::Definition;
use bdl_model::DeclId;
use bdl_syntax::ast::{self, AstNode};
use serde::{Deserialize, Serialize};

/// What a name at a position stands for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NameAt {
    /// An entity of the design, by identity.
    Entity { entity: EntityRef, role: EntityRole },
    /// An equation of the library, applied here.
    Equation { name: String },
}

/// The hover for a position: the range the card belongs to, and its
/// content.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HoverAt {
    pub range: TextRange,
    pub content: HoverContent,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum HoverContent {
    Entity(SemanticHover),
    /// An equation of the library: its shape and the designer's-words
    /// summary (`completion::equation_documentation`).
    Equation {
        name: String,
        shape: String,
        documentation: String,
    },
}

/// The name at a byte offset of a document: an entity anchored in the
/// `Name` or `Reference` role, else an equation callee inside a formula
/// body.  `None` elsewhere.
pub fn name_at(
    snapshot: &AnalysisSnapshot,
    document: DocumentId,
    offset: u32,
) -> Option<(TextRange, NameAt)> {
    let map = snapshot.projections();
    // The smallest name site holding the offset; among the entities a
    // site stands for (a port and the flattened copies of the
    // declaration it backs), the authored one.
    if let Some(a) = map
        .document_anchors(document)
        .iter()
        .filter(|a| matches!(a.role, EntityRole::Name | EntityRole::Reference))
        .filter(|a| a.text_range().is_some_and(|r| r.contains(offset)))
        .min_by_key(|a| {
            (
                a.text_range().map(|r| r.len()).unwrap_or(u32::MAX),
                u8::from(!is_authored(snapshot, a.entity)),
            )
        })
    {
        return Some((
            a.text_range()?,
            NameAt::Entity {
                entity: a.entity,
                role: a.role,
            },
        ));
    }
    // inside a formula body, a call to the library
    let body = map
        .document_anchors(document)
        .iter()
        .filter(|a| a.role == EntityRole::Definition)
        .find(|a| a.text_range().is_some_and(|r| r.contains(offset)))?;
    let range = body.text_range()?;
    let doc = snapshot.document(document)?;
    let text = doc.source.get(range.start as usize..range.end as usize)?;
    let (r, name) = equation_at(text, offset - range.start)?;
    Some((r.offset(range.start), NameAt::Equation { name }))
}

/// Whether an entity is one the designer authored, as opposed to a
/// flattened copy the system makes per instance (`lampA.dimByTilt`).  In
/// a text workspace a copy is a mapping, concept or clock the system's
/// base design does not hold; everything is authored elsewhere.
fn is_authored(snapshot: &AnalysisSnapshot, entity: EntityRef) -> bool {
    let Some(world) = snapshot.text() else {
        return true;
    };
    let base = &world.system.base;
    match entity {
        EntityRef::Mapping(d) => base.mappings.contains_key(&d),
        EntityRef::Concept(c) => base.concepts.contains_key(&c),
        EntityRef::Clock(k) => base.clocks.contains_key(&k),
        _ => true,
    }
}

/// The authored spelling of an entity's name: the text at its
/// declaration's name site, which for a flattened copy is the body's own
/// name (`dimByTilt`), never the copy's (`lampA.dimByTilt`).
fn authored_name(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Option<String> {
    let a = snapshot
        .projections()
        .anchors_for(entity, EntityRole::Name)
        .find(|a| a.text_range().is_some())?;
    let doc = snapshot.document(a.document()?)?;
    let r = a.text_range()?;
    doc.source
        .get(r.start as usize..r.end as usize)
        .map(str::to_owned)
}

/// The name at a body-relative offset of a mapping's formula, by the
/// elaborator's own rule: an input concept (by the concept's name or its
/// parameter name), another relationship, a concept the formula cannot
/// read (still that concept), or an equation of the library.
pub fn formula_name_at(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    offset: u32,
) -> Option<(TextRange, NameAt)> {
    use bdl_elab::names::{InputEnv, Lookup};
    let design = &snapshot.effective().design;
    let block = design.mappings.get(&mapping)?;
    let source = match &block.definition {
        Some(Definition::Formula { source }) | Some(Definition::ScopedFormula { source, .. }) => {
            source.as_str()
        }
        _ => return None,
    };
    let env = match &block.definition {
        Some(Definition::ScopedFormula { scope, .. }) => {
            InputEnv::scoped(&block.signature.inputs, scope)
        }
        _ => InputEnv::for_mapping(design, block),
    };
    let parse = bdl_syntax::parse_formula(source);
    let root = parse.syntax_node();
    for node in root.descendants() {
        let Some(name) = ast::NameExpr::cast(node) else {
            continue;
        };
        let Some(r) = name.name() else { continue };
        let range = TextRange::from(r.span());
        if !range.contains(offset) || name.local_binding().is_some() {
            continue;
        }
        let text = r.as_str();
        let entity = match env.resolve(design, &text) {
            Lookup::Input(i) => EntityRef::Concept(*block.signature.inputs.get(i)?),
            Lookup::Mapping(d) => EntityRef::Mapping(d),
            Lookup::NotAnInput(c, _) => EntityRef::Concept(c),
            Lookup::Ambiguous(_) | Lookup::Unknown => {
                if bdl_equations::lookup(&text).is_some() {
                    return Some((range, NameAt::Equation { name: text }));
                }
                return None;
            }
        };
        return Some((
            range,
            NameAt::Entity {
                entity,
                role: EntityRole::Reference,
            },
        ));
    }
    equation_at(source, offset).map(|(r, name)| (r, NameAt::Equation { name }))
}

/// The equation callee whose name holds `offset` in a formula text.
fn equation_at(source: &str, offset: u32) -> Option<(TextRange, String)> {
    let parse = bdl_syntax::parse_formula(source);
    let root = parse.syntax_node();
    crate::tokens::equation_callees(&root)
        .into_iter()
        .find(|r| r.contains(offset))
        .map(|r| (r, source[r.start as usize..r.end as usize].to_owned()))
}

fn hover_of(snapshot: &AnalysisSnapshot, range: TextRange, name: NameAt) -> Option<HoverAt> {
    let content = match name {
        NameAt::Entity { entity, .. } => {
            let mut h = hover(snapshot, entity)?;
            // A flattened copy is presented by its authored name: the
            // source owns the declaration, the copy is the system's.
            if !is_authored(snapshot, entity) {
                if let Some(authored) = authored_name(snapshot, entity) {
                    let flat = h.title.clone();
                    h.title = authored.clone();
                    if let Some(sig) = h.signature.as_mut() {
                        *sig = sig.replacen(&flat, &authored, 1);
                    }
                }
            }
            HoverContent::Entity(h)
        }
        NameAt::Equation { name } => {
            let e = bdl_equations::lookup(&name)?;
            HoverContent::Equation {
                name,
                shape: e.shape(),
                documentation: equation_documentation(e),
            }
        }
    };
    Some(HoverAt { range, content })
}

/// The hover card at a byte offset of a document.
pub fn hover_at(snapshot: &AnalysisSnapshot, document: DocumentId, offset: u32) -> Option<HoverAt> {
    let (range, name) = name_at(snapshot, document, offset)?;
    hover_of(snapshot, range, name)
}

/// The hover card at a body-relative offset of a mapping's formula.
pub fn formula_hover_at(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    offset: u32,
) -> Option<HoverAt> {
    let (range, name) = formula_name_at(snapshot, mapping, offset)?;
    hover_of(snapshot, range, name)
}

/// Where the entity named at a byte offset is declared: its name sites
/// in text, on every document of the workspace (a body's port, the
/// component's source — never a flattened copy's name).  Empty for a
/// name the design does not own.
pub fn definition_at(
    snapshot: &AnalysisSnapshot,
    document: DocumentId,
    offset: u32,
) -> Vec<ProjectionAnchor> {
    let Some((_, NameAt::Entity { entity, .. })) = name_at(snapshot, document, offset) else {
        return Vec::new();
    };
    definition_sites(snapshot, entity)
}

/// The text name sites of an entity's declaration.
pub fn definition_sites(snapshot: &AnalysisSnapshot, entity: EntityRef) -> Vec<ProjectionAnchor> {
    let mut out: Vec<ProjectionAnchor> = snapshot
        .projections()
        .anchors_for(entity, EntityRole::Name)
        .filter(|a| a.text_range().is_some())
        .cloned()
        .collect();
    out.sort();
    out.dedup();
    out
}

/// Every text site that references the entity named at a byte offset,
/// across the workspace's documents, with its declaration's name sites
/// when asked.  By identity: two concepts with the same representation
/// never share a result.
pub fn references_at(
    snapshot: &AnalysisSnapshot,
    document: DocumentId,
    offset: u32,
    include_declaration: bool,
) -> Vec<ProjectionAnchor> {
    let Some((_, NameAt::Entity { entity, .. })) = name_at(snapshot, document, offset) else {
        return Vec::new();
    };
    reference_sites(snapshot, entity, include_declaration)
}

/// The text reference sites of an entity.
pub fn reference_sites(
    snapshot: &AnalysisSnapshot,
    entity: EntityRef,
    include_declaration: bool,
) -> Vec<ProjectionAnchor> {
    let result = references(snapshot, entity);
    let mut out: Vec<ProjectionAnchor> = result
        .anchors
        .into_iter()
        .filter(|a| a.text_range().is_some())
        .collect();
    if include_declaration {
        out.extend(
            result
                .declaration
                .into_iter()
                .filter(|a| a.role == EntityRole::Name && a.text_range().is_some()),
        );
    }
    out.sort();
    out.dedup();
    out
}
