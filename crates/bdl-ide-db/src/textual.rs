//! The textual projection: binding a `.bdl` document to semantic entities.
//!
//! A text document is *not* the model; it is one projection of it.  This
//! module answers two questions for one document at one snapshot:
//!
//! 1. **What does the text declare, in terms of stable identity?**  Each
//!    item is bound to a committed entity by its declared name — that is
//!    the identity-resolution pass (`docs/COMPILER_PIPELINE.md` #2) for the
//!    textual surface — or, when nothing committed has that name, to a
//!    fresh id allocated for the overlay.  From then on everything speaks
//!    `EntityRef`.
//! 2. **Where is each entity in the text?**  A [`ProjectionMap`] of anchors
//!    `(entity, role) → byte range`, so diagnostics, references and rename
//!    can be placed without any text search.
//!
//! The bound items are then *applied* to the effective design: the text
//! is authoritative for the entities it declares while its overlay is
//! open, and silent about everything else (a document is a partial view;
//! the canvas and other documents may hold the rest).
//!
//! What is deliberately not here: parsing (`bdl-syntax`), meaning
//! (`bdl-elab`, `bdl-check`), and any name → text search.  Parameter names
//! in a definition (`f(tilt) = …`) have no semantic layer yet (DI-13):
//! the body is elaborated against the concepts' display names, so a body
//! that says `tilt` for a concept named `Tilt` resolves by the elaborator's
//! case-insensitive rule and a body that says something else reports
//! `formula.name.unknown` as it would in Studio.

use crate::entity::{EntityRef, EntityRole};
use crate::projection::{ProjectionAnchor, ProjectionMap};
use crate::text::{DocumentId, TextRange};
use bdl_model::surface::{Concept, Definition, Design, MappingBlock, Representation, Signature};
use bdl_model::{DeclId, SemanticId};
use bdl_syntax::ast::{self, AstNode};
use bdl_syntax::{parse_module, SyntaxError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A problem binding the text to the model, distinct from a syntax error
/// (the parser's) and from a semantic error (the compiler's).  Reported to
/// the client as a diagnostic on the document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BindingFault {
    /// `binding.unknown_concept`, `binding.unknown_representation`,
    /// `binding.unsupported_item`, `binding.duplicate_item`,
    /// `binding.definition_name_mismatch`.
    pub code: String,
    pub range: TextRange,
    pub message: String,
    /// The entity concerned, when the item did bind.
    pub entity: Option<EntityRef>,
    /// `true` for something the language does not have yet (an `enum`),
    /// which is open rather than wrong.
    pub open: bool,
}

/// The textual state of one document at one snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextDocumentState {
    pub document: DocumentId,
    pub source: String,
    /// The parser's diagnostics, in source order.
    pub syntax_errors: Vec<SyntaxError>,
    pub binding_faults: Vec<BindingFault>,
    /// Entities this document declares, in document order.
    pub declared: Vec<EntityRef>,
    /// Where each mapping's formula body sits in the document, so a
    /// body-relative compiler span becomes a document range by offset.
    pub formula_bodies: BTreeMap<DeclId, TextRange>,
}

impl TextDocumentState {
    /// A body-relative span of `mapping`'s formula as a document range.
    pub fn formula_range(&self, mapping: DeclId, span: TextRange) -> Option<TextRange> {
        let body = self.formula_bodies.get(&mapping)?;
        Some(span.offset(body.start).clamp_to(body.end))
    }

    pub fn declares(&self, entity: EntityRef) -> bool {
        self.declared.contains(&entity)
    }
}

/// Bind one document to `design` and apply what it declares.  `design` is
/// the effective design being composed (committed plus earlier overlays);
/// it is mutated in place.  Returns the document's state and its anchors.
pub fn bind_document(
    design: &mut Design,
    document: DocumentId,
    source: &str,
) -> (TextDocumentState, ProjectionMap) {
    let parse = parse_module(source);
    let mut b = Binder {
        design,
        document,
        anchors: ProjectionMap::default(),
        faults: Vec::new(),
        declared: Vec::new(),
        bodies: BTreeMap::new(),
        seen_names: BTreeMap::new(),
    };
    // Concepts first so a mapping may name a concept declared later in
    // the same file; then mappings in order.
    let items: Vec<ast::Item> = parse.tree().items().collect();
    for item in &items {
        if let ast::Item::Concept(c) = item {
            b.concept(c);
        }
    }
    for item in &items {
        match item {
            ast::Item::Concept(_) => {}
            ast::Item::Mapping(m) => b.mapping(m),
            ast::Item::Enum(e) => b.faults.push(BindingFault {
                code: "binding.unsupported_item".into(),
                range: e.span().into(),
                message:
                    "enums are syntax only in this version; the design model has no sum types yet."
                        .into(),
                entity: None,
                open: true,
            }),
            other => b.faults.push(BindingFault {
                code: "binding.unsupported_item".into(),
                range: other.span().into(),
                message: "this item is authored in a text project (`bdl.toml` kind = \"text\"); a document overlaid on a JSON project can declare concepts and mappings only.".into(),
                entity: None,
                open: true,
            }),
        }
    }
    let mut anchors = b.anchors;
    anchors.finish();
    (
        TextDocumentState {
            document,
            source: source.to_owned(),
            syntax_errors: parse.errors().to_vec(),
            binding_faults: b.faults,
            declared: b.declared,
            formula_bodies: b.bodies,
        },
        anchors,
    )
}

struct Binder<'a> {
    design: &'a mut Design,
    document: DocumentId,
    anchors: ProjectionMap,
    faults: Vec<BindingFault>,
    declared: Vec<EntityRef>,
    bodies: BTreeMap<DeclId, TextRange>,
    /// Names already declared in this document, by kind, to report a
    /// duplicate item rather than silently re-binding.
    seen_names: BTreeMap<(&'static str, String), TextRange>,
}

impl Binder<'_> {
    fn anchor(&mut self, entity: EntityRef, role: EntityRole, range: TextRange) {
        self.anchors
            .insert(ProjectionAnchor::text(entity, role, self.document, range));
    }

    fn duplicate(&mut self, kind: &'static str, name: &str, range: TextRange) -> bool {
        if let Some(first) = self.seen_names.get(&(kind, name.to_owned())) {
            self.faults.push(BindingFault {
                code: "binding.duplicate_item".into(),
                range,
                message: format!(
                    "`{name}` is declared twice in this document (first at {first}); the first declaration is the one bound."
                ),
                entity: None,
                open: false,
            });
            return true;
        }
        self.seen_names.insert((kind, name.to_owned()), range);
        false
    }

    fn concept(&mut self, c: &ast::ConceptDecl) {
        let Some(name) = c.name() else { return };
        let name_text = name.as_str();
        let name_range: TextRange = name.span().into();
        if self.duplicate("concept", &name_text, name_range) {
            return;
        }
        let representation = match c.representation() {
            None => None,
            Some(t) => {
                let range: TextRange = t.span().into();
                match representation_of(&t) {
                    Some(r) => Some((r, range)),
                    None => {
                        self.faults.push(BindingFault {
                            code: "binding.unknown_representation".into(),
                            range,
                            message: format!(
                                "`{}` is not a representation; write Bool, Count or a quantity such as {}.",
                                t.text().trim(),
                                bdl_model::quantity::QUANTITIES
                                    .iter()
                                    .take(4)
                                    .map(|q| q.type_name)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            ),
                            entity: None,
                            open: false,
                        });
                        None
                    }
                }
            }
        };
        let id = match self.design.concepts.values().find(|x| x.name == name_text) {
            Some(existing) => existing.id,
            None => {
                let (id, ids) = self.design.ids.fresh_semantic();
                self.design.ids = ids;
                self.design.concepts.insert(
                    id,
                    Concept {
                        id,
                        name: name_text.clone(),
                        description: String::new(),
                        representation: None,
                    },
                );
                id
            }
        };
        let entity = EntityRef::Concept(id);
        // The text is authoritative for what it says; an absent
        // representation in text leaves the committed binding alone, so a
        // canvas-chosen representation is not silently unbound by a file
        // that never mentioned one.
        if let Some((r, range)) = representation {
            if let Some(c) = self.design.concepts.get_mut(&id) {
                c.representation = Some(r);
            }
            self.anchor(entity, EntityRole::Representation, range);
        }
        self.declared.push(entity);
        self.anchor(entity, EntityRole::Declaration, c.span().into());
        self.anchor(entity, EntityRole::Name, name_range);
    }

    fn mapping(&mut self, m: &ast::MappingDecl) {
        let Some(name) = m.name() else { return };
        let name_text = name.as_str();
        let name_range: TextRange = name.span().into();
        if self.duplicate("mapping", &name_text, name_range) {
            return;
        }
        let existing = self
            .design
            .mappings
            .values()
            .find(|x| x.name == name_text)
            .map(|x| x.id);

        // Signature: every named type must be a concept of the effective
        // design.  Anchors are recorded for every name that binds, even
        // when another does not, so navigation keeps working on a broken
        // signature.
        let mut sig_inputs = Vec::new();
        let mut sig_output = None;
        let mut sig_ok = true;
        let mut sig_range = None;
        let mut input_anchors = Vec::new();
        if let Some(t) = m.signature() {
            sig_range = Some(TextRange::from(t.span()));
            let (inputs, output) = t.uncurry();
            for (i, dom) in inputs.iter().enumerate() {
                let range: TextRange = dom.span().into();
                match self.concept_named(dom) {
                    Some(c) => {
                        sig_inputs.push(c);
                        input_anchors.push((i as u16, c, range));
                    }
                    None => {
                        sig_ok = false;
                        self.unknown_concept(dom);
                    }
                }
            }
            let out_range: TextRange = output.span().into();
            match self.concept_named(&output) {
                Some(c) => sig_output = Some((c, out_range)),
                None => {
                    sig_ok = false;
                    self.unknown_concept(&output);
                }
            }
        } else {
            sig_ok = false;
        }

        let id = match existing {
            Some(id) => id,
            None => {
                // A new mapping needs a complete signature to exist at all.
                let Some((output, _)) = sig_output else {
                    return;
                };
                if !sig_ok {
                    return;
                }
                let (id, ids) = self.design.ids.fresh_decl();
                self.design.ids = ids;
                self.design.mappings.insert(
                    id,
                    MappingBlock {
                        id,
                        name: name_text.clone(),
                        description: String::new(),
                        signature: Signature {
                            inputs: sig_inputs.clone(),
                            output,
                        },
                        definition: None,
                        clock: None,
                        drives: None,
                        parameters: Vec::new(),
                    },
                );
                id
            }
        };
        let entity = EntityRef::Mapping(id);
        self.declared.push(entity);
        self.anchor(entity, EntityRole::Declaration, m.span().into());
        self.anchor(entity, EntityRole::Name, name_range);
        if let Some(r) = sig_range {
            self.anchor(entity, EntityRole::Signature, r);
        }
        for (i, c, range) in input_anchors {
            self.anchor(entity, EntityRole::Input { index: i }, range);
            self.anchor(EntityRef::Concept(c), EntityRole::Reference, range);
        }
        if let Some((c, range)) = sig_output {
            self.anchor(entity, EntityRole::Output, range);
            self.anchor(EntityRef::Concept(c), EntityRole::Reference, range);
        }

        // Apply the signature when it bound completely.
        if sig_ok {
            if let (Some((output, _)), Some(block)) =
                (sig_output, self.design.mappings.get_mut(&id))
            {
                block.signature = Signature {
                    inputs: sig_inputs,
                    output,
                };
            }
        }

        // Definition.
        let definition = m.definition();
        let mut body_source = None;
        if let Some(def) = &definition {
            if let Some(def_name) = def.name() {
                let def_text = def_name.as_str();
                if def_text != name_text {
                    self.faults.push(BindingFault {
                        code: "binding.definition_name_mismatch".into(),
                        range: def_name.span().into(),
                        message: format!(
                            "this definition is named `{def_text}`, but the mapping declared above it is `{name_text}`."
                        ),
                        entity: Some(entity),
                        open: false,
                    });
                }
                self.anchor(entity, EntityRole::Reference, def_name.span().into());
            }
            if let Some(body) = def.body() {
                let range: TextRange = body.span().into();
                self.anchor(entity, EntityRole::Definition, range);
                self.bodies.insert(id, range);
                body_source = Some(body.text());
                self.body_references(id, &body);
            }
        }
        if let Some(block) = self.design.mappings.get_mut(&id) {
            block.definition = body_source.map(|source| Definition::Formula { source });
        }
    }

    /// Names inside a body that resolve to signature inputs are references
    /// to those concepts — resolved by the elaborator's rule (exact, then
    /// unique case-insensitive), never by searching the text.
    fn body_references(&mut self, mapping: DeclId, body: &ast::Expr) {
        let Some(block) = self.design.mappings.get(&mapping) else {
            return;
        };
        let env = bdl_elab::names::InputEnv::for_inputs(self.design, &block.signature.inputs);
        let inputs = block.signature.inputs.clone();
        let mut refs = Vec::new();
        for node in body.syntax().descendants() {
            let Some(name) = ast::NameExpr::cast(node) else {
                continue;
            };
            let Some(name_ref) = name.name() else {
                continue;
            };
            let text = name_ref.as_str();
            if let bdl_elab::names::Lookup::Input(i) = env.resolve(self.design, &text) {
                if let Some(c) = inputs.get(i) {
                    refs.push((*c, TextRange::from(name_ref.span())));
                }
            }
        }
        for (c, range) in refs {
            self.anchor(EntityRef::Concept(c), EntityRole::Reference, range);
        }
    }

    fn concept_named(&self, t: &ast::Type) -> Option<SemanticId> {
        let n = named_type(t)?;
        let name = n.name()?.as_str();
        self.design
            .concepts
            .values()
            .find(|c| c.name == name)
            .map(|c| c.id)
    }

    fn unknown_concept(&mut self, t: &ast::Type) {
        self.faults.push(BindingFault {
            code: "binding.unknown_concept".into(),
            range: t.span().into(),
            message: format!(
                "`{}` is not a concept of this project; declare it with `concept {} : …` or pick an existing one.",
                t.text().trim(),
                t.text().trim()
            ),
            entity: None,
            open: false,
        });
    }
}

/// The representation a type name in a `concept` declaration stands for.
/// The textual vocabulary of representations (`docs/TEXTUAL_SYNTAX.md` §1).
pub fn representation_of(t: &ast::Type) -> Option<Representation> {
    representation_named(&named_type(t)?.name()?.as_str())
}

/// The plain named type inside any parentheses; `None` for a function
/// type or a generic application (neither names a concept).
fn named_type(t: &ast::Type) -> Option<ast::NamedType> {
    match t {
        ast::Type::Named(n) if !n.has_type_args() => Some(n.clone()),
        ast::Type::Paren(p) => named_type(&p.inner()?),
        _ => None,
    }
}

/// The representation a textual type name stands for: `Bool`, `Count`, or
/// a named quantity from the shared vocabulary (`bdl_model::quantity`).
pub fn representation_named(name: &str) -> Option<Representation> {
    Some(match name {
        "Bool" | "Boolean" => Representation::Boolean,
        "Count" | "Nat" => Representation::Count,
        _ => Representation::Quantity {
            dim: bdl_model::quantity::by_type_name(name)?.dim,
        },
    })
}

/// The textual spelling of a representation, inverse of
/// [`representation_named`] for named quantities; a dimension no quantity
/// names renders as `Scalar` with a comment, since the syntax has no
/// dimension literals yet.
pub fn representation_name(r: Representation) -> String {
    match r {
        Representation::Boolean => "Bool".into(),
        Representation::Count => "Count".into(),
        Representation::Quantity { dim } => bdl_model::quantity::by_dim(dim)
            .map(|q| q.type_name.to_owned())
            .unwrap_or_else(|| "Scalar /* unnamed dimension */".into()),
    }
}

/// The type names the textual surface accepts in `concept X : …`.
pub fn representation_names() -> Vec<&'static str> {
    let mut names = vec!["Bool", "Count"];
    names.extend(bdl_model::quantity::QUANTITIES.iter().map(|q| q.type_name));
    names
}

/// Render the concepts and mappings of a design as a canonical `.bdl`
/// module (`docs/TEXTUAL_SYNTAX.md` §10).  The inverse direction of
/// [`bind_document`] for what the model can express today; used for the
/// generated textual projection of a project and in tests that need the
/// same entity on both surfaces.
pub fn render_module(design: &Design) -> String {
    let mut out = String::new();
    let name = |id: SemanticId| {
        design
            .concepts
            .get(&id)
            .map(|c| c.name.clone())
            .unwrap_or_else(|| format!("Concept{}", id.raw()))
    };
    for c in design.concepts.values() {
        match c.representation {
            Some(r) => out.push_str(&format!(
                "concept {} : {}\n",
                c.name,
                representation_name(r)
            )),
            None => out.push_str(&format!("concept {}\n", c.name)),
        }
    }
    for m in design.mappings.values() {
        if !out.is_empty() {
            out.push('\n');
        }
        let mut ty: Vec<String> = m.signature.inputs.iter().map(|i| name(*i)).collect();
        ty.push(name(m.signature.output));
        out.push_str(&format!("mapping {} : {}\n", m.name, ty.join(" -> ")));
        if let Some(Definition::Formula { source }) = &m.definition {
            let params: Vec<String> = m.signature.inputs.iter().map(|i| name(*i)).collect();
            out.push_str(&format!(
                "{}({}) =\n  {}\n",
                m.name,
                params.join(", "),
                source.trim()
            ));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_model::surface::Design;
    use bdl_model::Dim;

    #[test]
    fn binds_by_name_to_committed_ids_and_allocates_fresh_ones() {
        let mut design = Design::empty("lamp");
        let (tilt, ids) = design.ids.fresh_semantic();
        design.ids = ids;
        design.concepts.insert(
            tilt,
            Concept {
                id: tilt,
                name: "Tilt".into(),
                description: "committed".into(),
                representation: Some(Representation::Quantity { dim: Dim::ANGLE }),
            },
        );
        let src = "concept Tilt : Angle\nconcept Brightness : Scalar\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(tilt) =\n  tilt / (90 deg)\n";
        let (state, anchors) = bind_document(&mut design, DocumentId(1), src);
        assert!(state.syntax_errors.is_empty());
        assert!(state.binding_faults.is_empty());
        // Tilt kept its identity and its committed description.
        assert_eq!(design.concepts[&tilt].description, "committed");
        assert_eq!(design.concepts.len(), 2);
        let m = design.mappings.values().next().unwrap();
        assert_eq!(m.name, "dimByTilt");
        assert_eq!(m.signature.inputs, vec![tilt]);
        assert_eq!(
            m.definition,
            Some(Definition::Formula {
                source: "tilt / (90 deg)".into()
            })
        );
        assert_eq!(
            state.declared,
            vec![
                EntityRef::Concept(tilt),
                EntityRef::Concept(SemanticId::from_raw(1)),
                EntityRef::Mapping(m.id)
            ]
        );
        // Anchors: the name, the signature input, and the body use of `tilt`
        // are all references to the same concept.
        let refs: Vec<TextRange> = anchors
            .anchors_for(EntityRef::Concept(tilt), EntityRole::Reference)
            .filter_map(|a| a.text_range())
            .collect();
        assert_eq!(refs.len(), 2, "{refs:?}");
        assert_eq!(&src[refs[0].start as usize..refs[0].end as usize], "Tilt");
        assert_eq!(&src[refs[1].start as usize..refs[1].end as usize], "tilt");
        let body = state.formula_bodies[&m.id];
        assert_eq!(
            &src[body.start as usize..body.end as usize],
            "tilt / (90 deg)"
        );
        assert_eq!(
            state.formula_range(m.id, TextRange::new(0, 4)),
            Some(TextRange::new(body.start, body.start + 4))
        );
    }

    #[test]
    fn unknown_concept_is_a_binding_fault_not_a_panic() {
        let mut design = Design::empty("x");
        let src = "mapping f : Nope -> Nope\n";
        let (state, _) = bind_document(&mut design, DocumentId(0), src);
        assert_eq!(state.binding_faults.len(), 2);
        assert_eq!(state.binding_faults[0].code, "binding.unknown_concept");
        assert!(design.mappings.is_empty());
    }

    #[test]
    fn render_then_bind_round_trips_the_model() {
        let mut design = Design::empty("lamp");
        let src = "concept Tilt : Angle\nconcept Brightness : Scalar\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(Tilt) =\n  Tilt / (90 deg)\n";
        bind_document(&mut design, DocumentId(0), src);
        let rendered = render_module(&design);
        assert_eq!(rendered, src);
        let mut again = design.clone();
        let (state, _) = bind_document(&mut again, DocumentId(1), &rendered);
        assert!(state.binding_faults.is_empty());
        assert_eq!(again, design, "binding the rendering changes nothing");
    }

    #[test]
    fn garbage_binds_to_nothing_without_panicking() {
        let mut design = Design::empty("x");
        for src in [
            "",
            "mapping",
            "concept : Angle",
            "enum E {",
            "§§§ concept T : Angle",
        ] {
            let (state, _) = bind_document(&mut design, DocumentId(0), src);
            assert_eq!(state.source, src);
        }
    }
}
