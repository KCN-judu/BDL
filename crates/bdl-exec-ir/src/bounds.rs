//! How large the lists a program carries can get — a static, sound upper
//! bound per declaration and per state cell, for deployment validation
//! and the manifest's memory estimate (docs/spec/deployment-capacity.md).
//!
//! The kernel's lists are unbounded (FV Phase 9a, D-83); a deployed core
//! has finite memory, so whether a design's lists fit is a validation
//! obligation, never a change to the list semantics.  This module answers
//! it by abstract interpretation over the executable IR: every list
//! position of every value gets a [`Bound`] — a number of elements, *as
//! large as an input* (the host decides), or *unbounded* (the design grows
//! it on its own, as the Phase-9a log does with `cons x (delay [] log)`).
//!
//! Soundness: every bound is an upper bound of the lengths the reference
//! evaluator can produce (tested against runs in `crates/bdl-compiler`);
//! it is not tight — `take k` is the one operator that narrows a bound, so
//! a design bounds its state by writing the bound.  A state cell's bound
//! is the fixpoint of its operand, computed by iteration with a widening
//! to *unbounded* and one narrowing pass (so `take cap (cons x (delay []
//! log))` is bounded by `cap` whatever `cap`, and `cons x (delay [] log)`
//! is unbounded).

use crate::{DeclKind, ExecExpr, ExecIr, LocalId, PrimOp, StateSlot};
use bdl_ir::Ty;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// An upper bound on a list's length.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Bound {
    /// At most this many elements, whatever the inputs.
    Finite { elements: u64 },
    /// As large as a list the host supplies (an unresolved declaration of
    /// list type, or something built from one without narrowing).
    Input,
    /// The design itself makes it grow without bound over time.
    Unbounded,
}

impl Bound {
    fn join(self, other: Bound) -> Bound {
        self.max(other)
    }
    fn add(self, other: Bound) -> Bound {
        match (self, other) {
            (Bound::Finite { elements: a }, Bound::Finite { elements: b }) => Bound::Finite {
                elements: a.saturating_add(b),
            },
            (a, b) => a.max(b),
        }
    }
    fn plus(self, k: u64) -> Bound {
        self.add(Bound::Finite { elements: k })
    }
    /// Narrow to at most `k` elements.
    fn at_most(self, k: u64) -> Bound {
        match self {
            Bound::Finite { elements } => Bound::Finite {
                elements: elements.min(k),
            },
            _ => Bound::Finite { elements: k },
        }
    }
    /// `n` steps each adding at most `delta` elements.
    fn times(self, delta: u64) -> Bound {
        if delta == 0 {
            return Bound::Finite { elements: 0 };
        }
        match self {
            Bound::Finite { elements } => Bound::Finite {
                elements: elements.saturating_mul(delta),
            },
            b => b,
        }
    }
}

/// The abstract value: the shape of a value with a bound at every list
/// position.  Scalars carry nothing.  `Bottom` is "no value yet" (an
/// empty list's elements, a never-written cell).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum Shape {
    Bottom,
    Scalar,
    List { bound: Bound, elem: Box<Shape> },
    Pair { fst: Box<Shape>, snd: Box<Shape> },
    Opt { inner: Box<Shape> },
}

impl Shape {
    fn list(bound: Bound, elem: Shape) -> Shape {
        Shape::List {
            bound,
            elem: Box::new(elem),
        }
    }
    fn opt(inner: Shape) -> Shape {
        Shape::Opt {
            inner: Box::new(inner),
        }
    }
    fn pair(fst: Shape, snd: Shape) -> Shape {
        Shape::Pair {
            fst: Box::new(fst),
            snd: Box::new(snd),
        }
    }

    /// The least upper bound; shapes of one type always align.
    pub fn join(&self, other: &Shape) -> Shape {
        match (self, other) {
            (Shape::Bottom, s) | (s, Shape::Bottom) => s.clone(),
            (Shape::Scalar, Shape::Scalar) => Shape::Scalar,
            (Shape::List { bound: a, elem: x }, Shape::List { bound: b, elem: y }) => {
                Shape::list(a.join(*b), x.join(y))
            }
            (Shape::Pair { fst: a, snd: b }, Shape::Pair { fst: c, snd: d }) => {
                Shape::pair(a.join(c), b.join(d))
            }
            (Shape::Opt { inner: a }, Shape::Opt { inner: b }) => Shape::opt(a.join(b)),
            // a type mismatch cannot happen for a checked program; be safe
            _ => Shape::Scalar,
        }
    }

    /// The largest shape of a type: every list as large as an input.
    pub fn top(t: &Ty) -> Shape {
        match t {
            Ty::Bool | Ty::Nat | Ty::Unit | Ty::Q { .. } | Ty::Sem { .. } | Ty::Arr { .. } => {
                Shape::Scalar
            }
            Ty::Opt { inner } => Shape::opt(Shape::top(inner)),
            Ty::List { elem } => Shape::list(Bound::Input, Shape::top(elem)),
            Ty::Prod { fst, snd } => Shape::pair(Shape::top(fst), Shape::top(snd)),
        }
    }

    fn list_parts(&self) -> (Bound, Shape) {
        match self {
            Shape::List { bound, elem } => (*bound, (**elem).clone()),
            _ => (Bound::Finite { elements: 0 }, Shape::Bottom),
        }
    }

    /// Whether any list position is unbounded.
    pub fn is_unbounded(&self) -> bool {
        self.any_bound(|b| b == Bound::Unbounded)
    }
    /// Whether any list position is as large as an input.
    pub fn depends_on_input(&self) -> bool {
        self.any_bound(|b| b == Bound::Input)
    }
    fn any_bound(&self, p: impl Fn(Bound) -> bool + Copy) -> bool {
        match self {
            Shape::Bottom | Shape::Scalar => false,
            Shape::List { bound, elem } => p(*bound) || elem.any_bound(p),
            Shape::Pair { fst, snd } => fst.any_bound(p) || snd.any_bound(p),
            Shape::Opt { inner } => inner.any_bound(p),
        }
    }

    /// Every list bound in the shape, outermost first.
    fn bounds(&self, out: &mut Vec<Bound>) {
        match self {
            Shape::Bottom | Shape::Scalar => {}
            Shape::List { bound, elem } => {
                out.push(*bound);
                elem.bounds(out);
            }
            Shape::Pair { fst, snd } => {
                fst.bounds(out);
                snd.bounds(out);
            }
            Shape::Opt { inner } => inner.bounds(out),
        }
    }

    /// Replace every list bound in the shape by the corresponding entry.
    fn with_bounds(&self, bs: &mut impl Iterator<Item = Bound>) -> Shape {
        match self {
            Shape::Bottom => Shape::Bottom,
            Shape::Scalar => Shape::Scalar,
            Shape::List { bound, elem } => {
                let b = bs.next().unwrap_or(*bound);
                Shape::list(b, elem.with_bounds(bs))
            }
            Shape::Pair { fst, snd } => Shape::pair(fst.with_bounds(bs), snd.with_bounds(bs)),
            Shape::Opt { inner } => Shape::opt(inner.with_bounds(bs)),
        }
    }

    /// Every list bound set to *unbounded* (the widening).
    fn widened(&self) -> Shape {
        let n = {
            let mut v = Vec::new();
            self.bounds(&mut v);
            v.len()
        };
        self.with_bounds(&mut std::iter::repeat_n(Bound::Unbounded, n))
    }

    /// An upper bound in bytes of a value of this shape as the generated
    /// core stores it (`Vec` header 24 bytes, `f64`/`u64` 8, `bool` 1,
    /// `Option` one word more, a tuple the sum); `None` when unbounded.
    pub fn bytes(&self, t: &Ty) -> Option<u64> {
        Some(match (self, t) {
            (_, Ty::Bool) => 1,
            (_, Ty::Nat | Ty::Q { .. }) => 8,
            (_, Ty::Sem { .. } | Ty::Arr { .. }) => 8,
            (Shape::Opt { inner }, Ty::Opt { inner: t }) => 8 + inner.bytes(t)?,
            (Shape::Pair { fst, snd }, Ty::Prod { fst: a, snd: b }) => {
                fst.bytes(a)? + snd.bytes(b)?
            }
            (Shape::List { bound, elem }, Ty::List { elem: t }) => match bound {
                Bound::Finite { elements } => 24 + elements.saturating_mul(elem.bytes(t)?),
                _ => return None,
            },
            (Shape::Bottom, Ty::Opt { .. }) => 8,
            (Shape::Bottom, Ty::List { .. }) => 24,
            (Shape::Bottom, Ty::Prod { fst, snd }) => {
                Shape::Bottom.bytes(fst)? + Shape::Bottom.bytes(snd)?
            }
            _ => 8,
        })
    }
}

/// The result: one shape per declaration (plan order) and per cell.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bounds {
    pub decls: Vec<Shape>,
    pub cells: Vec<Shape>,
}

impl Bounds {
    pub fn cell(&self, slot: StateSlot) -> Option<&Shape> {
        self.cells.get(slot.0 as usize)
    }
}

/// Rounds of plain iteration before widening, and of narrowing after.
const ROUNDS: usize = 8;
const NARROWING: usize = 3;

/// The bounds of every declaration and cell of the program.
pub fn analyse(ir: &ExecIr) -> Bounds {
    let mut cells: Vec<Shape> = vec![Shape::Bottom; ir.cells.len()];
    let mut widened = false;
    let mut rounds = 0;
    loop {
        let decls = decl_shapes(ir, &cells);
        let mut next = cells.clone();
        for (i, c) in ir.cells.iter().enumerate() {
            let s = Cx {
                ir,
                decls: &decls,
                cells: &cells,
                locals: BTreeMap::new(),
            }
            .shape(&c.operand);
            next[i] = next[i].join(&s);
        }
        rounds += 1;
        if next == cells {
            return Bounds { decls, cells };
        }
        cells = next;
        if !widened && rounds >= ROUNDS {
            // still moving: whatever changed is growing — widen it, then
            // let the operands narrow what they bound explicitly
            let decls = decl_shapes(ir, &cells);
            let mut w = cells.clone();
            for (i, c) in ir.cells.iter().enumerate() {
                let again = Cx {
                    ir,
                    decls: &decls,
                    cells: &cells,
                    locals: BTreeMap::new(),
                }
                .shape(&c.operand);
                if cells[i].join(&again) != cells[i] {
                    w[i] = cells[i].widened();
                }
            }
            cells = w;
            widened = true;
            rounds = 0;
        } else if widened && rounds >= NARROWING {
            let decls = decl_shapes(ir, &cells);
            return Bounds { decls, cells };
        }
    }
}

fn decl_shapes(ir: &ExecIr, cells: &[Shape]) -> Vec<Shape> {
    let mut decls: Vec<Shape> = Vec::with_capacity(ir.decls.len());
    for d in &ir.decls {
        let s = match &d.kind {
            DeclKind::Input { .. } => Shape::top(&d.ty),
            DeclKind::Computed { body } => Cx {
                ir,
                decls: &decls,
                cells,
                locals: BTreeMap::new(),
            }
            .shape(body),
        };
        decls.push(s);
    }
    decls
}

struct Cx<'a> {
    ir: &'a ExecIr,
    decls: &'a [Shape],
    cells: &'a [Shape],
    locals: BTreeMap<LocalId, Shape>,
}

impl Cx<'_> {
    fn shape(&mut self, e: &ExecExpr) -> Shape {
        match e {
            ExecExpr::Bool { .. } | ExecExpr::Nat { .. } | ExecExpr::Quantity { .. } => {
                Shape::Scalar
            }
            ExecExpr::Local { id } => self.locals.get(id).cloned().unwrap_or(Shape::Bottom),
            ExecExpr::Let { local, value, body } => {
                let v = self.shape(value);
                self.locals.insert(*local, v);
                self.shape(body)
            }
            ExecExpr::ReadDecl { decl } => {
                self.decls.get(decl.0 as usize).cloned().unwrap_or_else(|| {
                    self.ir
                        .decl(*decl)
                        .map(|d| Shape::top(&d.ty))
                        .unwrap_or(Shape::Scalar)
                })
            }
            ExecExpr::Wrap { e, .. } | ExecExpr::Unwrap { e } => self.shape(e),
            ExecExpr::ReadCell { slot, init } => {
                let i = self.shape(init);
                self.cells
                    .get(slot.0 as usize)
                    .map(|c| c.join(&i))
                    .unwrap_or(i)
            }
            ExecExpr::Prim { op, args } => self.prim(op, args),
            ExecExpr::Fold {
                elem,
                acc,
                step,
                init,
                list,
            } => {
                let (n, elem_shape) = self.shape(list).list_parts();
                let s0 = self.shape(init);
                // two steps tell a stable accumulator from one that grows
                // by a constant per element; anything else is unbounded
                let s1 = self.step(*elem, *acc, step, &elem_shape, &s0).join(&s0);
                let s2 = self.step(*elem, *acc, step, &elem_shape, &s1).join(&s1);
                let (mut b0, mut b1, mut b2) = (Vec::new(), Vec::new(), Vec::new());
                s0.bounds(&mut b0);
                s1.bounds(&mut b1);
                s2.bounds(&mut b2);
                if b1.len() != b2.len() {
                    return s2.widened();
                }
                let after: Vec<Bound> = b1
                    .iter()
                    .zip(&b2)
                    .enumerate()
                    .map(|(i, (l1, l2))| {
                        let l0 = b0.get(i).copied().unwrap_or(Bound::Finite { elements: 0 });
                        match (l0, *l1, *l2) {
                            (a, b, c) if a == b && b == c => a,
                            (
                                Bound::Finite { elements: a },
                                Bound::Finite { elements: b },
                                Bound::Finite { elements: c },
                            ) if b >= a && c >= b && b - a == c - b => {
                                // grows by `c - b` per element over `n` elements
                                Bound::Finite { elements: a }.add(n.times(c - b))
                            }
                            (_, Bound::Input, Bound::Input) => Bound::Input.join(n),
                            _ => Bound::Unbounded,
                        }
                    })
                    .collect();
                s2.with_bounds(&mut after.into_iter())
            }
        }
    }

    fn step(
        &mut self,
        elem: LocalId,
        acc: LocalId,
        step: &ExecExpr,
        elem_shape: &Shape,
        acc_shape: &Shape,
    ) -> Shape {
        self.locals.insert(elem, elem_shape.clone());
        self.locals.insert(acc, acc_shape.clone());
        self.shape(step)
    }

    fn prim(&mut self, op: &PrimOp, args: &[ExecExpr]) -> Shape {
        let a: Vec<Shape> = args.iter().map(|x| self.shape(x)).collect();
        let arg = |i: usize| a.get(i).cloned().unwrap_or(Shape::Bottom);
        match op {
            PrimOp::Add { .. }
            | PrimOp::Sub { .. }
            | PrimOp::Mul { .. }
            | PrimOp::Div { .. }
            | PrimOp::Lt
            | PrimOp::Eq
            | PrimOp::Not
            | PrimOp::And
            | PrimOp::Or
            | PrimOp::IsSome { .. }
            | PrimOp::Length { .. } => Shape::Scalar,
            PrimOp::Ite { .. } => arg(1).join(&arg(2)),
            PrimOp::None { .. } => Shape::opt(Shape::Bottom),
            PrimOp::Some { .. } => Shape::opt(arg(0)),
            PrimOp::GetD { .. } => match arg(0) {
                Shape::Opt { inner } => inner.join(&arg(1)),
                _ => arg(1),
            },
            PrimOp::Nil { .. } => Shape::list(Bound::Finite { elements: 0 }, Shape::Bottom),
            PrimOp::Cons { .. } => {
                let (n, elem) = arg(1).list_parts();
                Shape::list(n.plus(1), elem.join(&arg(0)))
            }
            PrimOp::Take { .. } => {
                let (n, elem) = arg(1).list_parts();
                match &args[0] {
                    ExecExpr::Quantity { value, .. } => {
                        Shape::list(n.at_most(bdl_reactive::eval::count(value.0) as u64), elem)
                    }
                    _ => Shape::list(n, elem),
                }
            }
            PrimOp::Drop { .. } => {
                let (n, elem) = arg(1).list_parts();
                Shape::list(n, elem)
            }
            PrimOp::Reverse { .. } => arg(0),
            PrimOp::Head { .. } => Shape::opt(arg(0).list_parts().1),
            PrimOp::ToList { .. } => match arg(0) {
                Shape::Opt { inner } => Shape::list(Bound::Finite { elements: 1 }, *inner),
                _ => Shape::list(Bound::Finite { elements: 1 }, Shape::Bottom),
            },
            PrimOp::Pair { .. } => Shape::pair(arg(0), arg(1)),
            PrimOp::Fst { .. } => match arg(0) {
                Shape::Pair { fst, .. } => *fst,
                _ => Shape::Bottom,
            },
            PrimOp::Snd { .. } => match arg(0) {
                Shape::Pair { snd, .. } => *snd,
                _ => Shape::Bottom,
            },
        }
    }
}
