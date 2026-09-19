//! The Formula Composer's view of a formula: a structured projection of
//! the draft's surface tree with the elaborator's types, the expected
//! type of every position (local dimension inference, FV Phase 10), what
//! fits a slot, and the text edits a structured action makes.
//!
//! ```text
//!   draft text ──bdl-syntax──▶ SurfaceExpr ──bdl-elab (trace)──▶ types per span
//!                                   │                                 │
//!                                   └────────── FormulaNode tree ◀────┘
//!                                                     │
//!                          solve (expected dims, top-down) · slot candidates · compose edits
//! ```
//!
//! The formula text is the only source: the projection is a *view* over
//! the syntax and the elaboration of the effective definition (a draft
//! overlay, else the committed formula), and a structured edit is a byte-
//! range edit of that text.  A slot is the surface's `?` — an expression
//! not yet written, refused by elaboration with `formula.slot.empty` and
//! never a kernel term.  Node identity is authoring identity: the tree
//! path, stable within one draft generation, never persisted.
//!
//! Local inference (`solve`, `Composer.lean`): `a + b`, `a − b` give both
//! sides the result's dimension; `a × b` gives the unknown side
//! `result − known`; `a ÷ b` gives the numerator `result + denominator`
//! and the denominator `numerator − result`.  A product of two unknowns
//! is reported as *insufficient information*, never searched
//! (`solve_sound`, `solve_complete` are the formal warrant; the tests here
//! are the production evidence).

use crate::completion::describe_representation;
use crate::diagnostics::{lift_for_mapping, SemanticDiagnostic};
use crate::QueryError;
use bdl_check::pretty;
use bdl_elab::units::{self, UnitDef};
use bdl_elab::{names::InputEnv, FormulaTrace, TypeTrace};
use bdl_equations::{self as equations, Cap, PTy};
use bdl_ide_db::{
    AnalysisSnapshot, EntityRef, OverlayGeneration, OverlayKey, SnapshotStamp, TextEdit, TextRange,
};
use bdl_ir::{DesignIr, Ty};
use bdl_model::surface::{Definition, Design, MappingBlock, Representation};
use bdl_model::{DeclId, Dim, SemanticId};
use bdl_syntax::{BinaryOp, ExprKind, SurfaceExpr, UnaryOp};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ---- the projection ----------------------------------------------------------

/// A type in the designer's words, with what the Composer needs of it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TypeView {
    /// "an angle", "a Brightness", "true or false", …
    pub description: String,
    pub kind: TypeKindView,
    /// The dimension, for a quantity or a concept represented by one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dim: Option<Dim>,
    /// The concept, when the value is a concept value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub concept: Option<SemanticId>,
    /// Only a value of *this* concept fits the position — a relationship's
    /// or an equation's argument bound to it — as opposed to the formula's
    /// result, where any value of the representation is observed and
    /// wrapped (ADR-0013).  Dimension equality never admits another
    /// concept here.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub nominal: bool,
    /// For a collection: what one element is (a binder's local, a `map`
    /// body's result).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<Box<TypeView>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeKindView {
    Quantity,
    Boolean,
    Count,
    Concept,
    Structured,
    /// The empty product `()`: a relationship's domain when it has no
    /// inputs.  Never a physical measurement unit.
    Unit,
    Unknown,
}

impl TypeView {
    fn quantity(dim: Dim) -> TypeView {
        TypeView {
            description: pretty::describe_dim(dim),
            kind: TypeKindView::Quantity,
            dim: Some(dim),
            concept: None,
            nominal: false,
            element: None,
        }
    }
    fn boolean() -> TypeView {
        TypeView {
            description: "true or false".into(),
            kind: TypeKindView::Boolean,
            dim: None,
            concept: None,
            nominal: false,
            element: None,
        }
    }
    fn count() -> TypeView {
        TypeView {
            description: "a count".into(),
            kind: TypeKindView::Count,
            dim: None,
            concept: None,
            nominal: false,
            element: None,
        }
    }
    fn of_ty(ir: &DesignIr, design: &Design, t: &Ty) -> TypeView {
        match t {
            Ty::Q { dim } => TypeView::quantity(*dim),
            Ty::Bool => TypeView::boolean(),
            Ty::Nat => TypeView::count(),
            // the empty product: a relationship's domain, never a slot's
            // expectation
            Ty::Unit => TypeView {
                description: "()".into(),
                kind: TypeKindView::Unit,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
            Ty::Sem { id } => {
                let name = design
                    .concepts
                    .get(id)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| id.to_string());
                TypeView {
                    description: format!("a {name}"),
                    kind: TypeKindView::Concept,
                    dim: match ir.representation_of(*id) {
                        Some(Ty::Q { dim }) => Some(*dim),
                        _ => None,
                    },
                    concept: Some(*id),
                    // a closed concept type is a concept value: nominal
                    nominal: true,
                    // a concept over a collection: its elements are the
                    // representation's, plain (a binder's local)
                    element: match ir.representation_of(*id) {
                        Some(Ty::List { elem }) => {
                            Some(Box::new(TypeView::of_ty(ir, design, elem)))
                        }
                        _ => None,
                    },
                }
            }
            Ty::List { elem } => TypeView {
                description: format!(
                    "a collection of {}",
                    plural_of(&TypeView::of_ty(ir, design, elem))
                ),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: Some(Box::new(TypeView::of_ty(ir, design, elem))),
            },
            other => TypeView {
                description: pretty::kernel(other),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
        }
    }
    /// A plain representation (no concept) as a view.
    fn of_representation_value(r: &Representation) -> TypeView {
        match r {
            Representation::Quantity { dim } => TypeView::quantity(*dim),
            Representation::Boolean => TypeView::boolean(),
            Representation::Count => TypeView::count(),
            Representation::List { element } => TypeView {
                description: describe_representation(r),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: Some(Box::new(TypeView::of_representation_value(element))),
            },
            other => TypeView {
                description: describe_representation(other),
                kind: TypeKindView::Structured,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
        }
    }
    fn of_representation(ir: &DesignIr, design: &Design, concept: SemanticId) -> Option<TypeView> {
        let c = design.concepts.get(&concept)?;
        let rep = c.representation.as_ref()?;
        Some(match rep {
            Representation::Quantity { dim } => TypeView {
                description: format!("a {} ({})", c.name, pretty::describe_dim(*dim)),
                kind: TypeKindView::Concept,
                dim: Some(*dim),
                concept: Some(concept),
                nominal: false,
                element: None,
            },
            Representation::Boolean => TypeView {
                description: format!("a {} (true or false)", c.name),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: None,
            },
            Representation::Count => TypeView {
                description: format!("a {} (a count)", c.name),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: None,
            },
            Representation::List { element } => TypeView {
                description: format!("a {} ({})", c.name, describe_representation(rep)),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: Some(Box::new(TypeView::of_representation_value(element))),
            },
            other => TypeView {
                description: format!("a {} ({})", c.name, describe_representation(other)),
                kind: TypeKindView::Concept,
                dim: None,
                concept: Some(concept),
                nominal: false,
                element: None,
            },
        })
        .inspect(|_| {
            let _ = ir;
        })
    }
}

/// What a node is, in the Composer's vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    /// A name: an input, a relationship, a bound variable.
    Reference {
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<EntityRef>,
        /// Bound by an enclosing binder (`all x in xs: …`) or rule: a
        /// local of the formula, not an entity of the design.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        local: bool,
    },
    /// `all x in xs: body` and its siblings; children: the collection,
    /// the body.  `param` is the local the body reads.
    Binder {
        form: String,
        param: String,
        /// What the local is: one element of the collection.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        param_type: Option<TypeView>,
    },
    /// `lo .. hi`; children: the two ends.  Meaningful after `in`.
    Range,
    /// A number with no unit, as spelled.
    Number {
        text: String,
    },
    /// `90 deg`: a coordinate and its unit.
    Quantity {
        coordinate: String,
        unit: String,
        /// The registry id of the unit, when it is one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit_id: Option<String>,
    },
    Bool {
        value: bool,
    },
    Unary {
        op: String,
    },
    /// `+`, `-`, `*`, `/`.
    Binary {
        op: String,
    },
    /// `<`, `<=`, `>`, `>=`, `==`, `!=`, `&&`, `||`, `in`.
    Compare {
        op: String,
    },
    /// `name(args…)`: an equation of the library, or a relationship.
    Call {
        name: String,
        equation: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        entity: Option<EntityRef>,
    },
    /// `?` — an expression not yet written.
    Slot,
    /// `if c then a else b` — a choice; children: the condition, the
    /// outcome when it holds, the outcome otherwise.
    If,
    /// A form the Composer shows as text: `match`, a block, a rule, a
    /// collection or grouped literal, `delay`/`sync`.
    Opaque {
        what: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormulaNode {
    /// The tree path (`r`, `r.0`, `r.1.0`): stable within one generation.
    pub id: String,
    /// Byte range in the source, parentheses included.
    pub range: TextRange,
    pub kind: NodeKind,
    /// The source text of the node.
    pub text: String,
    /// What the elaborator found the node to be.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actual: Option<TypeView>,
    /// What the position asks for, from the context (local inference).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<TypeView>,
    /// Why the position expects that, in the designer's words.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub because: String,
    /// The diagnostics whose primary span is this node's and no child's.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<SemanticDiagnostic>,
    pub children: Vec<FormulaNode>,
}

impl FormulaNode {
    pub fn find(&self, id: &str) -> Option<&FormulaNode> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter().find_map(|c| c.find(id))
    }
    fn parent_of<'a>(&'a self, id: &str) -> Option<(&'a FormulaNode, usize)> {
        for (i, c) in self.children.iter().enumerate() {
            if c.id == id {
                return Some((self, i));
            }
            if let Some(p) = c.parent_of(id) {
                return Some(p);
            }
        }
        None
    }
    /// Every slot, in source order.
    pub fn slots(&self, out: &mut Vec<String>) {
        if matches!(self.kind, NodeKind::Slot) {
            out.push(self.id.clone());
        }
        for c in &self.children {
            c.slots(out);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormulaProjection {
    pub stamp: SnapshotStamp,
    pub mapping: DeclId,
    /// The draft's generation when the projection is of a draft overlay.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_generation: Option<OverlayGeneration>,
    /// The source the ranges index.
    pub source: String,
    /// False when the source did not parse: no tree, the diagnostics say
    /// where.
    pub parse_ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root: Option<FormulaNode>,
    /// What the whole formula must produce: the output concept.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<TypeView>,
    /// Whether the formula elaborates (no slot, no error).
    pub complete: bool,
    /// The slots in source order — the Tab order.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub slots: Vec<String>,
    /// Diagnostics no node owns (the whole formula's, or spanless).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unplaced: Vec<SemanticDiagnostic>,
}

/// The effective definition source of `mapping`: its draft overlay, else
/// its committed formula.
fn effective_source(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
) -> Result<(String, Option<OverlayGeneration>), QueryError> {
    let design = &snapshot.effective().design;
    let Some(block) = design.mappings.get(&mapping) else {
        return Err(QueryError::UnknownEntity {
            entity: EntityRef::Mapping(mapping),
        });
    };
    let generation = snapshot
        .overlay(OverlayKey::MappingDefinition { mapping })
        .filter(|a| a.fault.is_none())
        .map(|a| a.generation);
    match &block.definition {
        Some(Definition::Formula { source }) | Some(Definition::ScopedFormula { source, .. }) => {
            Ok((source.clone(), generation))
        }
        Some(Definition::Reference { .. }) => Err(QueryError::NotApplicable {
            reason: format!("{} is defined by reference, not by a formula", block.name),
        }),
        None => Ok((String::new(), generation)),
    }
}

fn trace(snapshot: &AnalysisSnapshot, block: &MappingBlock, source: &str) -> FormulaTrace {
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let env = match &block.definition {
        Some(Definition::ScopedFormula { scope, .. }) => {
            InputEnv::scoped(&block.signature.inputs, scope)
        }
        _ => InputEnv::for_mapping(design, block),
    };
    bdl_elab::trace_formula_in(design, ir, block, source, env)
}

/// The projection of `mapping`'s effective definition.
pub fn formula_projection(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
) -> Result<FormulaProjection, QueryError> {
    let (source, draft_generation) = effective_source(snapshot, mapping)?;
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let block = &design.mappings[&mapping];
    let result = TypeView::of_representation(ir, design, block.signature.output);
    if source.trim().is_empty() {
        return Ok(FormulaProjection {
            stamp: snapshot.stamp(),
            mapping,
            draft_generation,
            source,
            parse_ok: true,
            root: None,
            result,
            complete: false,
            slots: Vec::new(),
            unplaced: Vec::new(),
        });
    }
    let t = trace(snapshot, block, &source);
    let diagnostics = lift_for_mapping(snapshot, mapping);
    let mut root = t.surface.as_ref().map(|surface| {
        let types: BTreeMap<(u32, u32), &TypeTrace> = t
            .types
            .iter()
            .map(|e| ((e.span.start, e.span.end), e))
            .collect();
        let mut b = Builder {
            design,
            ir,
            block,
            source: &source,
            types: &types,
            locals: Vec::new(),
        };
        let mut node = b.node(surface, "r".into());
        b.solve(&mut node, result.as_ref(), "".into());
        node
    });
    let mut unplaced = Vec::new();
    if let Some(root) = root.as_mut() {
        for d in diagnostics {
            if !place(root, &d) {
                unplaced.push(d);
            }
        }
    } else {
        unplaced = diagnostics;
    }
    let mut slots = Vec::new();
    if let Some(r) = &root {
        r.slots(&mut slots);
    }
    Ok(FormulaProjection {
        stamp: snapshot.stamp(),
        mapping,
        draft_generation,
        parse_ok: t.surface.is_some(),
        complete: t.ok && slots.is_empty(),
        root,
        result,
        slots,
        unplaced,
        source,
    })
}

/// Attach a diagnostic to the innermost node whose range contains its
/// primary span; false when no node does.
fn place(node: &mut FormulaNode, d: &SemanticDiagnostic) -> bool {
    let Some(span) = d.primary.source.as_ref().map(|s| s.range) else {
        return false;
    };
    if span.start < node.range.start || span.end > node.range.end {
        return false;
    }
    for c in node.children.iter_mut() {
        if place(c, d) {
            return true;
        }
    }
    node.diagnostics.push(d.clone());
    true
}

struct Builder<'a> {
    design: &'a Design,
    ir: &'a DesignIr,
    block: &'a MappingBlock,
    source: &'a str,
    types: &'a BTreeMap<(u32, u32), &'a TypeTrace>,
    /// The binder locals in scope while building, innermost last.
    locals: Vec<String>,
}

fn binary_symbol(op: BinaryOp) -> (&'static str, bool) {
    match op {
        BinaryOp::Add => ("+", true),
        BinaryOp::Sub => ("-", true),
        BinaryOp::Mul => ("*", true),
        BinaryOp::Div => ("/", true),
        BinaryOp::Lt => ("<", false),
        BinaryOp::Le => ("<=", false),
        BinaryOp::Gt => (">", false),
        BinaryOp::Ge => (">=", false),
        BinaryOp::Eq => ("==", false),
        BinaryOp::Ne => ("!=", false),
        BinaryOp::And => ("&&", false),
        BinaryOp::Or => ("||", false),
        BinaryOp::In => ("in", false),
        BinaryOp::Coalesce => ("??", false),
    }
}

impl Builder<'_> {
    fn text(&self, span: bdl_diagnostics::Span) -> String {
        self.source
            .get(span.start as usize..span.end as usize)
            .unwrap_or("")
            .to_owned()
    }

    fn actual(&self, e: &SurfaceExpr) -> Option<TypeView> {
        let t = self.types.get(&(e.span.start, e.span.end))?;
        Some(match &t.ty {
            Some(ty) => TypeView::of_ty(self.ir, self.design, ty),
            None => TypeView {
                description: t.description.clone(),
                kind: TypeKindView::Unknown,
                dim: None,
                concept: None,
                nominal: false,
                element: None,
            },
        })
    }

    fn reference_entity(&self, name: &str) -> Option<EntityRef> {
        let scope = match &self.block.definition {
            Some(Definition::ScopedFormula { scope, .. }) => Some(scope),
            _ => None,
        };
        for (i, c) in self.block.signature.inputs.iter().enumerate() {
            let bound = scope
                .and_then(|s| s.inputs.get(i).cloned())
                .or_else(|| {
                    self.block
                        .parameters
                        .get(i)
                        .filter(|p| !p.is_empty())
                        .cloned()
                })
                .or_else(|| self.design.concepts.get(c).map(|c| c.name.clone()));
            if bound.as_deref() == Some(name) {
                return Some(EntityRef::Concept(*c));
            }
        }
        match scope {
            Some(s) => s.mappings.get(name).map(|id| EntityRef::Mapping(*id)),
            None => self
                .design
                .mappings
                .values()
                .find(|m| m.name == name)
                .map(|m| EntityRef::Mapping(m.id)),
        }
    }

    fn node(&mut self, e: &SurfaceExpr, id: String) -> FormulaNode {
        let child = |b: &mut Self, c: &SurfaceExpr, i: usize| b.node(c, format!("{id}.{i}"));
        let (kind, children) = match &e.kind {
            ExprKind::Hole => (NodeKind::Slot, Vec::new()),
            ExprKind::Name(name) => {
                let local = self.locals.iter().any(|l| l == name);
                (
                    NodeKind::Reference {
                        name: name.clone(),
                        entity: if local {
                            None
                        } else {
                            self.reference_entity(name)
                        },
                        local,
                    },
                    Vec::new(),
                )
            }
            ExprKind::Binder {
                form,
                param,
                collection,
                body,
            } => {
                let coll = child(self, collection, 0);
                let param_type = coll
                    .actual
                    .as_ref()
                    .and_then(|t| t.element.as_deref().cloned());
                self.locals.push(param.name.clone());
                let body_node = child(self, body, 1);
                self.locals.pop();
                (
                    NodeKind::Binder {
                        form: form.word().into(),
                        param: param.name.clone(),
                        param_type,
                    },
                    vec![coll, body_node],
                )
            }
            ExprKind::Range { lo, hi } => (
                NodeKind::Range,
                vec![child(self, lo, 0), child(self, hi, 1)],
            ),
            ExprKind::Number { literal, unit } => (
                match unit {
                    None => NodeKind::Number {
                        text: literal.as_str().to_owned(),
                    },
                    Some(u) => NodeKind::Quantity {
                        coordinate: literal.as_str().to_owned(),
                        unit: u.name.clone(),
                        unit_id: units::lookup(&u.name).map(|d| d.id.to_owned()),
                    },
                },
                Vec::new(),
            ),
            ExprKind::Bool(v) => (NodeKind::Bool { value: *v }, Vec::new()),
            ExprKind::Unary { op, expr } => (
                NodeKind::Unary {
                    op: match op {
                        UnaryOp::Neg => "-".into(),
                        UnaryOp::Not => "!".into(),
                    },
                },
                vec![child(self, expr, 0)],
            ),
            ExprKind::Binary { op, lhs, rhs } => {
                let (sym, arithmetic) = binary_symbol(*op);
                (
                    if arithmetic {
                        NodeKind::Binary { op: sym.into() }
                    } else {
                        NodeKind::Compare { op: sym.into() }
                    },
                    vec![child(self, lhs, 0), child(self, rhs, 1)],
                )
            }
            ExprKind::Call { callee, args } => {
                let name = match &callee.kind {
                    ExprKind::Name(n) => n.clone(),
                    _ => self.text(callee.span),
                };
                let temporal = matches!(name.as_str(), "delay" | "sync");
                if temporal {
                    (
                        NodeKind::Opaque {
                            what: if name == "delay" {
                                "a remembered value".into()
                            } else {
                                "a value from another timing domain".into()
                            },
                        },
                        Vec::new(),
                    )
                } else {
                    let entity = self.reference_entity(&name);
                    let equation = entity.is_none() && equations::lookup(&name).is_some();
                    (
                        NodeKind::Call {
                            name,
                            equation,
                            entity,
                        },
                        args.iter()
                            .enumerate()
                            .map(|(i, a)| child(self, a, i))
                            .collect(),
                    )
                }
            }
            ExprKind::If { cond, then, els } => (
                NodeKind::If,
                vec![
                    child(self, cond, 0),
                    child(self, then, 1),
                    child(self, els, 2),
                ],
            ),
            ExprKind::Match { .. } => (
                NodeKind::Opaque {
                    what: "a match".into(),
                },
                Vec::new(),
            ),
            ExprKind::Block { .. } => (
                NodeKind::Opaque {
                    what: "a block with let".into(),
                },
                Vec::new(),
            ),
            ExprKind::List(_) => (
                NodeKind::Opaque {
                    what: "a collection".into(),
                },
                Vec::new(),
            ),
            ExprKind::Tuple(_) => (
                NodeKind::Opaque {
                    what: "a grouped value".into(),
                },
                Vec::new(),
            ),
            // `f(())`: the explicit application to the unique argument; the
            // Composer offers `f` and never writes this form itself
            ExprKind::Unit => (
                NodeKind::Opaque {
                    what: "the empty product ()".into(),
                },
                Vec::new(),
            ),
            ExprKind::Lambda { .. } => (
                NodeKind::Opaque {
                    what: "a rule".into(),
                },
                Vec::new(),
            ),
        };
        FormulaNode {
            id,
            range: TextRange::new(e.span.start, e.span.end),
            text: self.text(e.span),
            actual: self.actual(e),
            expected: None,
            because: String::new(),
            diagnostics: Vec::new(),
            children,
            kind,
        }
    }

    /// Top-down: what each position expects, from what the parent expects
    /// and what the siblings are (FV `solve`).
    fn solve(&self, node: &mut FormulaNode, expected: Option<&TypeView>, because: String) {
        node.expected = expected.cloned();
        node.because = because;
        let r = expected.and_then(|t| t.dim);
        let dim_of = |n: &FormulaNode| n.actual.as_ref().and_then(|t| t.dim);
        match node.kind.clone() {
            NodeKind::Binary { op } => {
                let (a, b) = (dim_of(&node.children[0]), dim_of(&node.children[1]));
                let (ea, eb, why) = match (op.as_str(), r) {
                    ("+", Some(r)) | ("-", Some(r)) => (
                        Some(r),
                        Some(r),
                        format!(
                            "both sides of {} must be {}, the result",
                            if op == "+" { "a sum" } else { "a difference" },
                            pretty::describe_dim(r)
                        ),
                    ),
                    ("*", Some(r)) => match (a, b) {
                        (Some(a), _) => (
                            Some(a),
                            Some(r - a),
                            format!(
                                "{} × {} = {}",
                                pretty::describe_dim(a),
                                pretty::describe_dim(r - a),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, Some(b)) => (
                            Some(r - b),
                            Some(b),
                            format!(
                                "{} × {} = {}",
                                pretty::describe_dim(r - b),
                                pretty::describe_dim(b),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, None) => (
                            None,
                            None,
                            "a product of two unknown values: not enough is known to say what each must be".into(),
                        ),
                    },
                    ("/", Some(r)) => match (a, b) {
                        (Some(a), _) => (
                            Some(a),
                            Some(a - r),
                            format!(
                                "{} ÷ {} = {}",
                                pretty::describe_dim(a),
                                pretty::describe_dim(a - r),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, Some(b)) => (
                            Some(r + b),
                            Some(b),
                            format!(
                                "{} ÷ {} = {}",
                                pretty::describe_dim(r + b),
                                pretty::describe_dim(b),
                                pretty::describe_dim(r)
                            ),
                        ),
                        (None, None) => (
                            None,
                            None,
                            "a quotient of two unknown values: not enough is known to say what each must be".into(),
                        ),
                    },
                    // no expected result: an operand still constrains the other
                    ("+", None) | ("-", None) => match (a, b) {
                        (Some(a), _) => (Some(a), Some(a), format!("both sides must be {}", pretty::describe_dim(a))),
                        (None, Some(b)) => (Some(b), Some(b), format!("both sides must be {}", pretty::describe_dim(b))),
                        _ => (None, None, String::new()),
                    },
                    _ => (None, None, String::new()),
                };
                let (l, rr) = node.children.split_at_mut(1);
                self.solve(&mut l[0], ea.map(TypeView::quantity).as_ref(), why.clone());
                self.solve(&mut rr[0], eb.map(TypeView::quantity).as_ref(), why);
            }
            NodeKind::Compare { op } => {
                let (a, b) = (dim_of(&node.children[0]), dim_of(&node.children[1]));
                let (ea, eb, why) = if matches!(op.as_str(), "&&" | "||") {
                    (
                        Some(TypeView::boolean()),
                        Some(TypeView::boolean()),
                        "both sides of a logical operator are true or false".to_string(),
                    )
                } else if op == "??" {
                    let why = match expected {
                        Some(t) => format!(
                            "`??` gives {} or, when it is absent, the default",
                            t.description
                        ),
                        None => String::new(),
                    };
                    (None, expected.cloned(), why)
                } else if op == "in" {
                    // `x in lo .. hi`: both ends must be comparable with x —
                    // the same concept when x is one, else its dimension
                    if matches!(node.children[1].kind, NodeKind::Range) {
                        let subject = node.children[0].actual.clone().map(|mut t| {
                            t.nominal = t.kind == TypeKindView::Concept;
                            t
                        });
                        let why = match &subject {
                            Some(t) => format!(
                                "both ends of the range must be comparable with {} ({})",
                                node.children[0].text.trim(),
                                t.description
                            ),
                            None => String::new(),
                        };
                        (None, subject, why)
                    } else {
                        (None, None, String::new())
                    }
                } else {
                    match (a, b) {
                        (Some(a), _) => (
                            Some(TypeView::quantity(a)),
                            Some(TypeView::quantity(a)),
                            format!("a comparison puts two {} side by side", plural(a)),
                        ),
                        (None, Some(b)) => (
                            Some(TypeView::quantity(b)),
                            Some(TypeView::quantity(b)),
                            format!("a comparison puts two {} side by side", plural(b)),
                        ),
                        _ => (
                            None,
                            None,
                            "both sides of a comparison must be the same kind of value".into(),
                        ),
                    }
                };
                let (l, rr) = node.children.split_at_mut(1);
                self.solve(&mut l[0], ea.as_ref(), why.clone());
                self.solve(&mut rr[0], eb.as_ref(), why);
            }
            NodeKind::Range => {
                // what the test expects, passed to both ends
                let why = node.because.clone();
                for c in node.children.iter_mut() {
                    self.solve(c, expected, why.clone());
                }
            }
            NodeKind::Binder { form, param, .. } => {
                let (form, param) = (form.clone(), param.clone());
                let coll_why = format!("{form} reads every element of a collection");
                let (body_expected, body_why) = match form.as_str() {
                    "map" => (
                        expected.and_then(|t| t.element.as_deref().cloned()),
                        format!("map makes a collection of what the body gives for each {param}"),
                    ),
                    _ => (
                        Some(TypeView::boolean()),
                        format!(
                            "{form} asks a question of every {param}: the body is true or false"
                        ),
                    ),
                };
                let (c, b) = node.children.split_at_mut(1);
                self.solve(&mut c[0], None, coll_why);
                self.solve(&mut b[0], body_expected.as_ref(), body_why);
            }
            NodeKind::If => {
                // the condition is a question; both outcomes give what the
                // choice gives — the position's expectation, else what the
                // other outcome already is
                let known =
                    |n: &FormulaNode| n.actual.clone().filter(|t| t.kind != TypeKindView::Unknown);
                let (then_known, else_known) = (known(&node.children[1]), known(&node.children[2]));
                let (outcome, why) = match (expected, then_known, else_known) {
                    (Some(t), _, _) => (
                        Some(t.clone()),
                        format!(
                            "both outcomes of a choice give what the choice gives: {}",
                            t.description
                        ),
                    ),
                    (None, Some(t), _) | (None, None, Some(t)) => (
                        Some(t.clone()),
                        format!(
                            "both outcomes of a choice are the same kind of value: {}",
                            t.description
                        ),
                    ),
                    (None, None, None) => (None, String::new()),
                };
                let (c, rest) = node.children.split_at_mut(1);
                self.solve(
                    &mut c[0],
                    Some(&TypeView::boolean()),
                    "a choice asks a question: the condition is true or false".into(),
                );
                for o in rest.iter_mut() {
                    self.solve(o, outcome.as_ref(), why.clone());
                }
            }
            NodeKind::Unary { op } => {
                let (e, why) = if op == "!" {
                    (
                        Some(TypeView::boolean()),
                        "negation applies to true or false".to_string(),
                    )
                } else {
                    (
                        expected.cloned(),
                        "a negated value has the kind of its result".to_string(),
                    )
                };
                self.solve(&mut node.children[0], e.as_ref(), why);
            }
            NodeKind::Call {
                name,
                equation,
                entity,
            } => {
                if equation {
                    let params = self.equation_params(&name, node, expected);
                    for (i, c) in node.children.iter_mut().enumerate() {
                        let (e, why) = params.get(i).cloned().unwrap_or((None, String::new()));
                        self.solve(c, e.as_ref(), why);
                    }
                } else if let Some(EntityRef::Mapping(id)) = entity {
                    let inputs = self
                        .design
                        .mappings
                        .get(&id)
                        .map(|m| m.signature.inputs.clone())
                        .unwrap_or_default();
                    for (i, c) in node.children.iter_mut().enumerate() {
                        // a relationship reads its input *concepts*: only a
                        // value of that concept fits (DI-17), never another
                        // concept of the same representation
                        let e = inputs.get(i).and_then(|s| {
                            TypeView::of_representation(self.ir, self.design, *s)
                                .map(|v| TypeView { nominal: true, ..v })
                        });
                        self.solve(
                            c,
                            e.as_ref(),
                            format!(
                                "{name} reads {} here",
                                e.as_ref()
                                    .map(|t| t.description.clone())
                                    .unwrap_or_else(|| "a value".into())
                            ),
                        );
                    }
                } else {
                    for c in node.children.iter_mut() {
                        self.solve(c, None, String::new());
                    }
                }
            }
            _ => {
                for c in node.children.iter_mut() {
                    self.solve(c, None, String::new());
                }
            }
        }
    }

    /// The expected type of each argument of an equation call, from its
    /// scheme instantiated by the result and the arguments already known
    /// (P9's matcher; nothing is guessed).
    fn equation_params(
        &self,
        name: &str,
        node: &FormulaNode,
        expected: Option<&TypeView>,
    ) -> Vec<(Option<TypeView>, String)> {
        let Some(entry) = equations::lookup(name) else {
            return Vec::new();
        };
        // the arguments already written bind the variables first — a
        // concept value binds its concept, as the elaborator's instance
        // resolution keeps it; the expected result seeds only what they
        // leave open (its concept is observed there, ADR-0013)
        let mut subst = equations::Subst::default();
        for (c, pat) in node.children.iter().zip(&entry.scheme.params) {
            let known = c.actual.as_ref().and_then(|t| match t.kind {
                TypeKindView::Concept => t.concept.map(|id| Ty::Sem { id }),
                _ => closed_ty(t),
            });
            if let Some(t) = known {
                let mut s = subst.clone();
                if equations::match_ty(pat, &t, &mut s).is_ok() {
                    subst = s;
                }
            }
        }
        if let Some(t) = expected.and_then(closed_ty) {
            let mut s = subst.clone();
            if equations::match_ty(&entry.scheme.result, &t, &mut s).is_ok() {
                subst = s;
            }
        }
        entry
            .scheme
            .params
            .iter()
            .enumerate()
            .map(|(i, pat)| {
                let p = entry.params.get(i).copied().unwrap_or("value");
                match equations::instantiate(pat, &subst) {
                    Some(t) if !matches!(t, Ty::Arr { .. }) => (
                        Some(TypeView::of_ty(self.ir, self.design, &t)),
                        format!("{} takes {p} here", entry.shape()),
                    ),
                    Some(_) => (None, format!("{} takes a rule here", entry.shape())),
                    None => (None, format!("{} takes {p} here", entry.shape())),
                }
            })
            .collect()
    }
}

fn plural_of(t: &TypeView) -> String {
    match (t.kind, t.dim) {
        (TypeKindView::Quantity, Some(d)) => plural(d),
        (TypeKindView::Concept, _) => format!(
            "{} values",
            t.description
                .trim_start_matches("a ")
                .split(" (")
                .next()
                .unwrap_or("")
        ),
        (TypeKindView::Boolean, _) => "truth values".into(),
        (TypeKindView::Count, _) => "counts".into(),
        _ => format!("values ({})", t.description),
    }
}

fn plural(d: Dim) -> String {
    let one = pretty::describe_dim(d);
    let noun = one
        .strip_prefix("an ")
        .or_else(|| one.strip_prefix("a "))
        .unwrap_or(&one);
    if noun.starts_with("dimensionless") {
        "dimensionless quantities".into()
    } else if noun.starts_with("quantity of") {
        format!("quantities {}", &noun["quantity ".len()..])
    } else {
        format!("{noun}s")
    }
}

/// The closed kernel type a view stands for: a quantity, a truth value or
/// a count; a concept as its representation (observation).
fn closed_ty(v: &TypeView) -> Option<Ty> {
    match v.kind {
        TypeKindView::Quantity | TypeKindView::Concept => v.dim.map(|dim| Ty::Q { dim }),
        TypeKindView::Boolean => Some(Ty::Bool),
        TypeKindView::Count => Some(Ty::Nat),
        _ => None,
    }
}

// ---- slots ---------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitCandidate {
    pub id: String,
    pub symbol: String,
    /// "an angle"
    pub measures: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReferenceCandidate {
    pub label: String,
    /// The text that fills the slot (`tilt`, `dimByTilt(?)`).
    pub insert: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entity: Option<EntityRef>,
    /// What the reference is, in the designer's words.
    pub produces: String,
    /// Higher first: the expected concept itself, then the same kind of
    /// value, then the rest.
    pub relevance: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquationCandidate {
    pub name: String,
    /// `min(a, b)`
    pub shape: String,
    /// The text that fills the slot, with a slot per argument.
    pub insert: String,
    pub summary: String,
}

/// What a slot (or any node's position) expects and what fits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotInfo {
    pub node: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected: Option<TypeView>,
    /// "Expected: an angle, because an angle ÷ an angle = a dimensionless
    /// quantity."
    pub explanation: String,
    /// The same in the kernel's terms (exponent vectors), for Explain.
    pub technical: String,
    /// True when the position is not locally determined (two unknown
    /// operands): nothing is offered by dimension.
    pub insufficient: bool,
    pub units: Vec<UnitCandidate>,
    pub references: Vec<ReferenceCandidate>,
    pub equations: Vec<EquationCandidate>,
    /// The truth values, as text to fill the slot with (`true`, `false`),
    /// when the position expects true or false.
    pub booleans: Vec<String>,
}

/// What fits at `node` of `mapping`'s projection.
pub fn formula_slot(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    node: &str,
) -> Result<SlotInfo, QueryError> {
    let projection = formula_projection(snapshot, mapping)?;
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let block = &design.mappings[&mapping];
    let (expected, because, insufficient) = match &projection.root {
        Some(root) if root.id == node => (projection.result.clone(), String::new(), false),
        Some(root) => match root.find(node) {
            Some(n) => (
                n.expected.clone(),
                n.because.clone(),
                n.expected.is_none() && n.because.contains("not enough"),
            ),
            None => {
                return Err(QueryError::NotApplicable {
                    reason: format!("no node {node} in the projection"),
                })
            }
        },
        None if node == "r" => (projection.result.clone(), String::new(), false),
        None => {
            return Err(QueryError::NotApplicable {
                reason: "the formula does not parse".into(),
            })
        }
    };
    let explanation = match (&expected, insufficient) {
        (Some(t), _) if because.is_empty() => format!("Expected: {}.", t.description),
        (Some(t), _) => format!("Expected: {}, because {}.", t.description, because),
        (None, true) => format!("Not enough is known yet: {because}."),
        (None, false) => "Any value fits here; what it produces decides the rest.".into(),
    };
    let technical = match &expected {
        Some(TypeView { dim: Some(d), .. }) => {
            format!("expected {}", pretty::kernel(&Ty::Q { dim: *d }))
        }
        Some(t) => format!("expected {}", t.description),
        None => "expected: undetermined".into(),
    };
    // a literal's own unit pop-up switches the unit while keeping the
    // quantity (18D): its candidates are the units of the literal's own
    // dimension; an empty slot's are those of the expected dimension (18C)
    let literal_dim = projection
        .root
        .as_ref()
        .and_then(|r| r.find(node))
        .and_then(|n| match &n.kind {
            NodeKind::Quantity {
                unit_id: Some(_), ..
            } => n.actual.as_ref().and_then(|t| t.dim),
            _ => None,
        });
    let dim = literal_dim.or(expected.as_ref().and_then(|t| t.dim));
    let units: Vec<UnitCandidate> = match dim {
        Some(d) => units::units_for(d)
            .into_iter()
            .map(|u| UnitCandidate {
                id: u.id.to_owned(),
                symbol: u.symbol.to_owned(),
                measures: pretty::describe_dim(u.dim),
            })
            .collect(),
        None => Vec::new(),
    };
    let references = reference_candidates(design, ir, block, expected.as_ref());
    let equations = equation_candidates(ir, expected.as_ref());
    let booleans = if expects_truth_value(design, expected.as_ref()) {
        vec!["true".to_owned(), "false".to_owned()]
    } else {
        Vec::new()
    };
    Ok(SlotInfo {
        node: node.to_owned(),
        expected,
        explanation,
        technical,
        insufficient,
        units,
        references,
        equations,
        booleans,
    })
}

/// Whether the position takes a truth value: true or false itself, or a
/// concept represented by one (a literal is observed with it, as a
/// quantity literal is with a quantity concept).
fn expects_truth_value(design: &Design, expected: Option<&TypeView>) -> bool {
    match expected {
        Some(TypeView {
            kind: TypeKindView::Boolean,
            ..
        }) => true,
        Some(TypeView {
            kind: TypeKindView::Concept,
            concept: Some(id),
            ..
        }) => matches!(
            design
                .concepts
                .get(id)
                .and_then(|c| c.representation.as_ref()),
            Some(Representation::Boolean)
        ),
        _ => false,
    }
}

/// Whether a value of representation `rep` (or concept `concept`) fits
/// an expected view: the expected concept itself ranks 100, its
/// representation (a concept is observed where a representation is
/// needed) 80, a quantity of another dimension 0 (unfit), anything when
/// nothing is expected 50.
fn fit(
    expected: Option<&TypeView>,
    concept: Option<SemanticId>,
    rep: Option<&Representation>,
) -> Option<u8> {
    let Some(e) = expected else {
        return Some(50);
    };
    if e.concept.is_some() && e.concept == concept {
        return Some(100);
    }
    if e.nominal && e.concept.is_some() {
        // another concept, or a plain value where a concept value is read:
        // nominally wrong however the representations compare
        return None;
    }
    match (e.kind, rep) {
        (
            TypeKindView::Quantity | TypeKindView::Concept,
            Some(Representation::Quantity { dim }),
        ) => (Some(*dim) == e.dim).then_some(80),
        (TypeKindView::Boolean, Some(Representation::Boolean)) => Some(80),
        (TypeKindView::Concept, Some(Representation::Boolean)) if e.dim.is_none() => Some(80),
        (TypeKindView::Count, Some(Representation::Count)) => Some(80),
        (TypeKindView::Unknown, _) => Some(50),
        _ => None,
    }
}

fn reference_candidates(
    design: &Design,
    ir: &DesignIr,
    block: &MappingBlock,
    expected: Option<&TypeView>,
) -> Vec<ReferenceCandidate> {
    let _ = ir;
    let scope = match &block.definition {
        Some(Definition::ScopedFormula { scope, .. }) => Some(scope),
        _ => None,
    };
    let mut out = Vec::new();
    for (i, c) in block.signature.inputs.iter().enumerate() {
        let Some(concept) = design.concepts.get(c) else {
            continue;
        };
        let name = scope
            .and_then(|s| s.inputs.get(i).cloned())
            .or_else(|| block.parameters.get(i).filter(|p| !p.is_empty()).cloned())
            .unwrap_or_else(|| concept.name.clone());
        let Some(relevance) = fit(expected, Some(*c), concept.representation.as_ref()) else {
            continue;
        };
        out.push(ReferenceCandidate {
            label: name.clone(),
            insert: name,
            entity: Some(EntityRef::Concept(*c)),
            produces: format!(
                "{} ({})",
                concept.name,
                concept
                    .representation
                    .as_ref()
                    .map(describe_representation)
                    .unwrap_or_else(|| "no value form yet".into())
            ),
            relevance: relevance + 10,
        });
    }
    let visible: Vec<(String, &MappingBlock)> = match scope {
        Some(s) => s
            .mappings
            .iter()
            .filter_map(|(name, id)| design.mappings.get(id).map(|m| (name.clone(), m)))
            .collect(),
        None => design
            .mappings
            .values()
            .map(|m| (m.name.clone(), m))
            .collect(),
    };
    for (name, m) in visible {
        if m.id == block.id {
            continue;
        }
        let out_concept = design.concepts.get(&m.signature.output);
        let Some(relevance) = fit(
            expected,
            Some(m.signature.output),
            out_concept.and_then(|c| c.representation.as_ref()),
        ) else {
            continue;
        };
        // a relationship with the unit domain is read as a value: its
        // unique argument is nothing to fill
        let callable = !m.signature.is_unit_domain();
        let slots = vec!["?"; m.signature.inputs.len()].join(", ");
        out.push(ReferenceCandidate {
            label: if callable {
                format!("{name}(…)")
            } else {
                name.clone()
            },
            insert: if callable {
                format!("{name}({slots})")
            } else {
                name
            },
            entity: Some(EntityRef::Mapping(m.id)),
            produces: out_concept
                .map(|c| {
                    format!(
                        "{} ({})",
                        c.name,
                        c.representation
                            .as_ref()
                            .map(describe_representation)
                            .unwrap_or_else(|| "no value form yet".into())
                    )
                })
                .unwrap_or_default(),
            relevance: relevance.saturating_sub(if callable { 10 } else { 5 }),
        });
    }
    out.sort_by(|a, b| {
        b.relevance
            .cmp(&a.relevance)
            .then_with(|| a.label.cmp(&b.label))
    });
    out
}

/// The equations whose result can be the expected value and whose
/// capabilities the instance satisfies (P9's scheme matcher): with an
/// angle expected, `min`, `max`, `clamp`, `sum`, … but not `any`.
fn equation_candidates(ir: &DesignIr, expected: Option<&TypeView>) -> Vec<EquationCandidate> {
    let Some(want) = expected.and_then(closed_ty) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for e in equations::entries() {
        let mut subst = equations::Subst::default();
        if equations::match_ty(&e.scheme.result, &want, &mut subst).is_err() {
            continue;
        }
        // a capability on a variable the result binds must hold there
        let satisfiable = e.scheme.caps.iter().all(|(v, cap)| match subst.tys.get(v) {
            Some(t) => match cap {
                Cap::Ord => ir.is_ordered(t),
                Cap::Eq | Cap::Data => !matches!(t, Ty::Arr { .. }),
            },
            None => true,
        });
        if !satisfiable {
            continue;
        }
        // the equation must take at least one value (a rule-only shape
        // is not a value to fill)
        if e.scheme.params.iter().all(|p| matches!(p, PTy::Arr(..))) {
            continue;
        }
        let args: Vec<&str> = e
            .scheme
            .params
            .iter()
            .map(|p| match p {
                PTy::Arr(a, _) => match a.as_ref() {
                    PTy::Prod(..) => "(x, y) => ?",
                    _ => "x => ?",
                },
                _ => "?",
            })
            .collect();
        out.push(EquationCandidate {
            name: e.name.to_owned(),
            shape: e.shape(),
            insert: format!("{}({})", e.name, args.join(", ")),
            summary: e.summary.to_owned(),
        });
    }
    out
}

// ---- composing edits ----------------------------------------------------------

/// A structured action on the formula, answered with the text it makes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ComposeOp {
    /// Put `text` where the node is (`?` → `90 deg`, `tilt`, `min(?, ?)`).
    Fill { node: String, text: String },
    /// `node` becomes `node op ?` (or `? op node` with `before`); `!` is
    /// the prefix form, `!node`, and takes no slot.
    Operator {
        node: String,
        op: String,
        #[serde(default)]
        before: bool,
    },
    /// `node` becomes `name(node, ?, …)` with `arity` arguments.
    Call {
        node: String,
        name: String,
        arity: u32,
    },
    /// A literal's unit: with `preserve_value` the coordinate is
    /// converted so the physical quantity is unchanged (`180 deg` →
    /// `3.14… rad`); without it the coordinate stays (`180 deg` →
    /// `180 rad`, a different quantity).
    SetUnit {
        node: String,
        unit_id: String,
        preserve_value: bool,
    },
    /// A literal's coordinate: the same unit, another quantity.
    SetCoordinate { node: String, text: String },
    /// The node becomes a slot again; a slot that is an operand of `+ - *
    /// /` removes the operator with it.
    Remove { node: String },
    /// `node` becomes `form item in node: ?` with a fresh local name
    /// (`all`, `any`, `map` or `filter`).
    Binder { node: String, form: String },
    /// `node` becomes `node in ? .. ?`.
    Range { node: String },
    /// A choice: a slot becomes `if ? then ? else ?`; any other node
    /// becomes one outcome of it, `if ? then node else ?`.
    Choose { node: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComposeResult {
    pub source: String,
    pub edits: Vec<TextEdit>,
    /// The node to select afterwards (the first new slot, else the
    /// edited node), by its path in the *new* source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub select: Option<String>,
}

/// Whether removing an empty operand of `parent` removes the operator
/// with it: arithmetic, a comparison, `&&`, `||` — every two-sided
/// operator whose other side then stands alone (not `in`, whose range
/// is not a value on its own).
fn removes_operator(parent: &FormulaNode) -> bool {
    match &parent.kind {
        NodeKind::Binary { .. } => true,
        NodeKind::Compare { op } => op != "in",
        _ => false,
    }
}

/// Precedence of a node as an operand: what needs parentheses under what.
/// The ladder is the parser's (`bdl-syntax::parser::expr::infix`), one
/// step per level.
fn precedence(kind: &NodeKind) -> u8 {
    match kind {
        // a binder extends as far right as its body, a choice as far as
        // its `else` does: an operand only in parentheses
        NodeKind::Binder { .. } | NodeKind::If => 0,
        NodeKind::Compare { op } | NodeKind::Binary { op } => op_precedence(op),
        NodeKind::Range => op_precedence(".."),
        NodeKind::Unary { .. } => UNARY,
        _ => ATOM,
    }
}

const UNARY: u8 = 9;
const ATOM: u8 = 10;
/// What a comparison's operand must bind at: comparisons do not chain
/// (`a < b == c` is refused), so one under another is parenthesised on
/// either side, and a range (`..`) stands unparenthesised after `in`.
const COMPARISON_OPERAND: u8 = 5;

fn op_precedence(op: &str) -> u8 {
    match op {
        "||" => 1,
        "&&" => 2,
        "==" | "!=" => 3,
        "<" | "<=" | ">" | ">=" | "in" => 4,
        ".." => 5,
        "??" => 6,
        "+" | "-" => 7,
        "*" | "/" => 8,
        _ => 4,
    }
}

/// `<`, `<=`, `>`, `>=`, `==`, `!=`, `in`: the operators that do not chain.
fn is_comparison(op: &str) -> bool {
    matches!(op, "<" | "<=" | ">" | ">=" | "==" | "!=" | "in")
}

/// Whether one pair of parentheses encloses the whole text (`(a + b)`,
/// not `(a) + b`).
fn wholly_parenthesised(text: &str) -> bool {
    let text = text.trim();
    if !text.starts_with('(') || !text.ends_with(')') {
        return false;
    }
    let mut depth = 0usize;
    for (i, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return i == text.len() - 1;
                }
            }
            _ => {}
        }
    }
    false
}

/// The precedence of a node as an operand: its own, unless it ends in a
/// form that extends as far right as it can (a choice, a binder) — then
/// anything written after it would be swallowed, and it binds like that
/// form: `-if c then a else b` before `+ ?` needs parentheses as much as
/// the choice alone does.
fn node_precedence(node: &FormulaNode) -> u8 {
    let own = precedence(&node.kind);
    let open_right = match &node.kind {
        NodeKind::Unary { .. }
        | NodeKind::Binary { .. }
        | NodeKind::Compare { .. }
        | NodeKind::Range => node
            .children
            .last()
            .is_some_and(|c| node_precedence(c) == 0),
        _ => false,
    };
    if open_right {
        0
    } else {
        own
    }
}

/// The node's text, parenthesised when it would bind weaker than `under`.
fn operand(node: &FormulaNode, under: u8) -> String {
    let text = node.text.trim();
    if node_precedence(node) < under && !wholly_parenthesised(text) {
        format!("({text})")
    } else {
        text.to_owned()
    }
}

/// The precedence a piece of text would have as an operand, for
/// parenthesising it (`node_precedence` over its syntax).
fn precedence_of_text(text: &str) -> u8 {
    fn of(e: &SurfaceExpr) -> u8 {
        let own = match &e.kind {
            ExprKind::Binary { op, .. } => op_precedence(binary_symbol(*op).0),
            ExprKind::Unary { .. } => UNARY,
            ExprKind::If { .. } | ExprKind::Binder { .. } | ExprKind::Lambda { .. } => 0,
            ExprKind::Range { .. } => op_precedence(".."),
            _ => ATOM,
        };
        let open_right = match &e.kind {
            ExprKind::Binary { rhs: last, .. }
            | ExprKind::Unary { expr: last, .. }
            | ExprKind::Range { hi: last, .. } => of(last) == 0,
            _ => false,
        };
        if open_right {
            0
        } else {
            own
        }
    }
    match bdl_syntax::formula(text) {
        Ok(e) => of(&e),
        Err(_) => ATOM,
    }
}

/// Apply a structured action to `source`; the projection it acts on is
/// rebuilt here so the action and the text never disagree.
pub fn compose(
    snapshot: &AnalysisSnapshot,
    mapping: DeclId,
    source: &str,
    op: &ComposeOp,
) -> Result<ComposeResult, QueryError> {
    let design = &snapshot.effective().design;
    let ir = &snapshot.analysis().ir;
    let Some(block) = design.mappings.get(&mapping) else {
        return Err(QueryError::UnknownEntity {
            entity: EntityRef::Mapping(mapping),
        });
    };
    // An empty formula is one slot: every action starts from `?`.
    let empty = source.trim().is_empty();
    let base = if empty {
        "?".to_owned()
    } else {
        source.to_owned()
    };
    let t = trace(snapshot, block, &base);
    let Some(surface) = &t.surface else {
        return Err(QueryError::NotApplicable {
            reason: "the formula does not parse; edit it as text".into(),
        });
    };
    let types: BTreeMap<(u32, u32), &TypeTrace> = t
        .types
        .iter()
        .map(|e| ((e.span.start, e.span.end), e))
        .collect();
    let mut b = Builder {
        design,
        ir,
        block,
        source: &base,
        types: &types,
        locals: Vec::new(),
    };
    let root = b.node(surface, "r".into());
    let node_id = match op {
        ComposeOp::Fill { node, .. }
        | ComposeOp::Operator { node, .. }
        | ComposeOp::Call { node, .. }
        | ComposeOp::SetUnit { node, .. }
        | ComposeOp::SetCoordinate { node, .. }
        | ComposeOp::Remove { node }
        | ComposeOp::Binder { node, .. }
        | ComposeOp::Range { node }
        | ComposeOp::Choose { node } => node.as_str(),
    };
    let Some(node) = root.find(node_id) else {
        return Err(QueryError::NotApplicable {
            reason: format!("no node {node_id} in the formula"),
        });
    };
    let parent = root.parent_of(node_id);
    // What the position of `node` demands of whatever stands there: the
    // precedence its parent binds with (a right operand of `-` or `/` may
    // not bind equally: `a - (b - c)`), everything under a unary minus, and
    // nothing at the root or as a call's argument.
    let context = |parent: Option<(&FormulaNode, usize)>| -> u8 {
        parent
            .map(|(p, i)| match &p.kind {
                NodeKind::Compare { op } if is_comparison(op) => COMPARISON_OPERAND,
                NodeKind::Binary { op } | NodeKind::Compare { op } => {
                    op_precedence(op) + if i == 1 { 1 } else { 0 }
                }
                // a range's ends: arithmetic and `??` bind tighter than
                // `..`, a comparison or a logical operator does not
                NodeKind::Range => op_precedence("??"),
                NodeKind::Unary { .. } => UNARY,
                // `if`, `then` and `else` delimit a choice's parts
                NodeKind::If => 0,
                // a binder's collection stops at the colon: a comparison
                // or another binder there needs parentheses (a range does
                // not); its body extends to the end and needs none
                NodeKind::Binder { .. } if i == 0 => COMPARISON_OPERAND,
                _ => 0,
            })
            .unwrap_or(0)
    };
    let under = context(parent);
    let grouped = |text: String, prec: u8| -> String {
        if prec < under && !wholly_parenthesised(&text) {
            format!("({text})")
        } else {
            text
        }
    };
    let (range, new_text): (TextRange, String) = match op {
        ComposeOp::Fill { text, .. } => {
            let text = text.trim();
            (
                node.range,
                grouped(text.to_owned(), precedence_of_text(text)),
            )
        }
        ComposeOp::Operator { op, .. } if op == "!" => {
            // negation binds tighter than any operator: its operand is
            // parenthesised unless it is atomic or a negation itself
            let operand = operand(node, UNARY);
            (node.range, grouped(format!("!{operand}"), UNARY))
        }
        ComposeOp::Operator { op, before, .. } => {
            let p = op_precedence(op);
            // as the left operand the node may bind equally (`a - b - ?` is
            // `(a - b) - ?`); as the right one it may not (`? / (a / b)`);
            // under a comparison it may be no comparison at all
            let under = if is_comparison(op) {
                COMPARISON_OPERAND
            } else if *before {
                p + 1
            } else {
                p
            };
            let operand = operand(node, under);
            let text = if *before {
                format!("? {op} {operand}")
            } else {
                format!("{operand} {op} ?")
            };
            (node.range, grouped(text, p))
        }
        ComposeOp::Binder { form, .. } => {
            if bdl_syntax::BinderForm::from_word(form).is_none() {
                return Err(QueryError::NotApplicable {
                    reason: format!("`{form}` is not a binder form"),
                });
            }
            let local = fresh_local(design, block, &root, &node.text);
            // the collection is parsed up to the colon: a comparison there
            // needs parentheses
            let coll = operand(node, COMPARISON_OPERAND);
            (
                node.range,
                grouped(format!("{form} {local} in {coll}: ?"), 0),
            )
        }
        ComposeOp::Range { .. } => {
            let subject = operand(node, COMPARISON_OPERAND);
            (
                node.range,
                grouped(format!("{subject} in ? .. ?"), op_precedence("in")),
            )
        }
        ComposeOp::Choose { .. } => {
            // a choice's parts are delimited by its words: nothing inside
            // needs parentheses; the choice itself extends to the end
            let outcome = if matches!(node.kind, NodeKind::Slot) {
                "?".to_owned()
            } else {
                node.text.trim().to_owned()
            };
            (
                node.range,
                grouped(format!("if ? then {outcome} else ?"), 0),
            )
        }
        ComposeOp::Call { name, arity, .. } => {
            let mut args = vec![node.text.trim().to_owned()];
            args.extend(std::iter::repeat_n(
                "?".to_owned(),
                (*arity as usize).saturating_sub(1),
            ));
            (node.range, format!("{name}({})", args.join(", ")))
        }
        ComposeOp::SetUnit {
            unit_id,
            preserve_value,
            ..
        } => {
            let Some(to) = units::by_id(unit_id) else {
                return Err(QueryError::NotApplicable {
                    reason: format!("no unit {unit_id}"),
                });
            };
            let (coordinate, from): (String, Option<&UnitDef>) = match &node.kind {
                NodeKind::Quantity {
                    coordinate, unit, ..
                } => (coordinate.clone(), units::lookup(unit)),
                NodeKind::Number { text } => (text.clone(), None),
                _ => {
                    return Err(QueryError::NotApplicable {
                        reason: "only a number can take a unit".into(),
                    })
                }
            };
            let coordinate = match (from, *preserve_value) {
                (Some(from), true) if from.id != to.id => {
                    let x: f64 = coordinate.parse().map_err(|_| QueryError::NotApplicable {
                        reason: format!("`{coordinate}` is not a number"),
                    })?;
                    match units::convert(x, from, to) {
                        Some(y) => format_coordinate(y),
                        None => {
                            return Err(QueryError::NotApplicable {
                                reason: format!(
                                    "{} measures {}, {} measures {}: the value cannot be kept",
                                    from.symbol,
                                    pretty::describe_dim(from.dim),
                                    to.symbol,
                                    pretty::describe_dim(to.dim)
                                ),
                            })
                        }
                    }
                }
                _ => coordinate,
            };
            (node.range, format!("{coordinate} {}", to.symbol))
        }
        ComposeOp::SetCoordinate { text, .. } => {
            let text = text.trim();
            if text.parse::<f64>().is_err() {
                return Err(QueryError::NotApplicable {
                    reason: format!("`{text}` is not a number"),
                });
            }
            match &node.kind {
                NodeKind::Quantity { unit, .. } => (node.range, format!("{text} {unit}")),
                NodeKind::Number { .. } | NodeKind::Slot => (node.range, text.to_owned()),
                _ => {
                    return Err(QueryError::NotApplicable {
                        reason: "only a number has a coordinate".into(),
                    })
                }
            }
        }
        ComposeOp::Remove { .. } => match parent {
            Some((p, i)) if matches!(node.kind, NodeKind::Slot) && removes_operator(p) => {
                // removing an empty operand removes the operator: the
                // other operand stands alone, grouped as *its* new
                // position demands
                let other = &p.children[1 - i];
                (p.range, operand(other, context(root.parent_of(&p.id))))
            }
            Some((p, _))
                if matches!(node.kind, NodeKind::Slot)
                    && matches!(p.kind, NodeKind::Unary { .. }) =>
            {
                // an empty negation is nothing: the slot stands alone
                (p.range, "?".to_owned())
            }
            _ => (node.range, "?".to_owned()),
        },
    };
    let edit = TextEdit::replace(range, new_text.clone());
    let mut out = base.clone();
    out.replace_range(range.start as usize..range.end as usize, &new_text);
    // the node to select next: what the action made — the new slot of an
    // operator or call, the first slot inside a filled text, the edited
    // node otherwise (never a slot that was already there elsewhere)
    let select = bdl_syntax::formula(&out).ok().map(|e| {
        let mut b = Builder {
            design,
            ir,
            block,
            source: &out,
            types: &BTreeMap::new(),
            locals: Vec::new(),
        };
        let root = b.node(&e, "r".into());
        let first_slot_in = |id: &str| -> Option<String> {
            let n = root.find(id)?;
            let mut slots = Vec::new();
            n.slots(&mut slots);
            slots.into_iter().next()
        };
        match op {
            ComposeOp::Operator { op, .. } if op == "!" => Some(node_id.to_owned()),
            ComposeOp::Operator { before, .. } => {
                Some(format!("{node_id}.{}", if *before { 0 } else { 1 }))
            }
            ComposeOp::Call { arity, .. } if *arity > 1 => Some(format!("{node_id}.1")),
            ComposeOp::Binder { .. } => Some(format!("{node_id}.1")),
            ComposeOp::Range { .. } => Some(format!("{node_id}.1.0")),
            ComposeOp::Choose { .. } => Some(format!("{node_id}.0")),
            ComposeOp::Fill { .. } => first_slot_in(node_id).or_else(|| {
                // no slot in what was written: the next slot after it, the
                // Tab order, else the node itself
                let mut slots = Vec::new();
                root.slots(&mut slots);
                slots
                    .into_iter()
                    .find(|s| root.find(s).is_some_and(|n| n.range.start >= range.end))
                    .or_else(|| Some(node_id.to_owned()))
            }),
            ComposeOp::Remove { .. } => match parent {
                Some((p, _))
                    if matches!(node.kind, NodeKind::Slot)
                        && (removes_operator(p) || matches!(p.kind, NodeKind::Unary { .. })) =>
                {
                    Some(p.id.clone())
                }
                _ => Some(node_id.to_owned()),
            },
            _ => Some(node_id.to_owned()),
        }
        .filter(|id| root.find(id).is_some())
        .unwrap_or_else(|| node_id.to_owned())
    });
    Ok(ComposeResult {
        source: out,
        edits: if empty {
            vec![TextEdit::replace(
                TextRange::new(0, source.len() as u32),
                new_text,
            )]
        } else {
            vec![edit]
        },
        select,
    })
}

/// A readable fresh name for a binder's local: the collection's name
/// without its plural `s` when that is a free identifier (`readings` →
/// `reading`), else `item`, `item2`, … — never a name in scope (an input,
/// a relationship, a concept, an enclosing local, a word of the language).
fn fresh_local(
    design: &Design,
    block: &MappingBlock,
    root: &FormulaNode,
    collection: &str,
) -> String {
    let mut taken: Vec<String> = Vec::new();
    taken.extend(design.mappings.values().map(|m| m.name.clone()));
    taken.extend(design.concepts.values().map(|c| c.name.clone()));
    taken.extend(block.parameters.iter().cloned());
    fn locals(n: &FormulaNode, out: &mut Vec<String>) {
        if let NodeKind::Binder { param, .. } = &n.kind {
            out.push(param.clone());
        }
        n.children.iter().for_each(|c| locals(c, out));
    }
    locals(root, &mut taken);
    fresh_local_name(collection, &taken)
}

/// The fresh name itself: `readings` → `reading` when that is free, else
/// `item`, `item2`, …  `taken` lists the names of the design and the
/// formula; the words of the language and the equations are always taken.
pub(crate) fn fresh_local_name(collection: &str, taken: &[String]) -> String {
    let mut taken: std::collections::BTreeSet<String> = taken.iter().cloned().collect();
    taken.extend(
        [
            "all", "any", "map", "filter", "in", "if", "then", "else", "match", "let", "true",
            "false", "delay", "sync", "None", "Some", "ordered", "concept", "mapping",
        ]
        .map(String::from),
    );
    taken.extend(equations::names().into_iter().map(String::from));
    let free = |n: &str| !taken.contains(n) && bdl_syntax::formula(n).is_ok();
    let name = collection.trim();
    if name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        if let Some(singular) = name.strip_suffix('s') {
            // a local reads as a value, so `Readings` gives `reading`
            let mut chars = singular.chars();
            let singular: String = match chars.next() {
                Some(c) => c.to_lowercase().chain(chars).collect(),
                None => String::new(),
            };
            if singular.len() >= 2 && !singular.ends_with('s') && free(&singular) {
                return singular;
            }
        }
    }
    if free("item") {
        return "item".into();
    }
    (2..)
        .map(|i| format!("item{i}"))
        .find(|n| free(n))
        .unwrap_or_else(|| "item".into())
}

/// A converted coordinate as source text: shortest round-trip form,
/// never exponent notation for ordinary magnitudes.
fn format_coordinate(x: f64) -> String {
    if x == x.trunc() && x.abs() < 1e15 {
        format!("{}", x as i64)
    } else {
        let s = format!("{x}");
        if s.contains('e') {
            format!("{x:.12}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_owned()
        } else {
            s
        }
    }
}
