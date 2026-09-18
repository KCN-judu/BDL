//! Executable IR → the generated semantic core (`src/lib.rs`).
//!
//! Correspondence (docs/architecture/codegen-rust.md):
//!
//! | exec IR | generated Rust |
//! |---|---|
//! | `ClockSlot` | `pub const CLOCK_n: ClockSlot`; `active.is_active(CLOCK_n)` |
//! | `ConceptPlan` | `pub struct SemN(pub Repr)` |
//! | `DeclPlan` (input) | `Inputs.decl_n: Option<T>`; `read_input(…)?` when due |
//! | `DeclPlan` (computed) | `let decl_n: Option<T> = if due { Some(body?) } else { None };` |
//! | `ReadDecl` | `read_decl(&decl_n, n)?`; `read_decl_ref(&decl_n, n)?` where a borrow suffices |
//! | `CellPlan` | `Cells.cell_k: Option<T>`; write `next.cell_k = Some(operand)` when writer active |
//! | `ReadCell` | `match &prev.cell_k { Some(v) => v.clone(), None => init }` |
//! | `Wrap` / `Unwrap` | `SemN(e)` / `e.0` |
//! | `PrimOp` | `num::add(a, b, decl)?` … / `(a < b)` / `prim::ite(c, x, y)` / `list::cons(x, xs)` |
//! | `Length` / `Head` / `Take` / `Eq` / `Lt` | over borrowed operands (`list::length_of(&xs)`, `(&a == &b)`): a list read for its length, first element, prefix or comparison is never copied |
//! | `Ite` with total branches | `if c { x } else { y }` — observationally the strict `ite` (neither branch can fail), without building both |
//! | `Fold` | `list::fold(xs, init, \|elem, acc\| Ok(step))?` — one closure per recursor, no closure value escapes |
//! | `Local` | `l_n` moved when it is the local's only use in its own scope, `l_n.clone()` otherwise (a captured local is always cloned) |
//! | `Ty::List` / `Ty::Prod` | `Vec<T>` (feature `collections`, `extern crate alloc`) / `(A, B)` |
//! | `OutputPlan` | `Outputs.output_n = decl_driver.clone()` after the write phase |
//!
//! A program without lists derives `Copy` on its records and never
//! allocates; one with lists derives `Clone` only and turns the runtime's
//! `collections` feature on (recorded in the manifest).
//!
//! Cost discipline (docs/architecture/codegen-rust.md §Cost): a tick clones
//! no cell it does not read — the write phase evaluates every active
//! writer's operand into a local and the commit moves those into the
//! state — and a fold step moves its element and accumulator through
//! `cons`, so the library's `map`, `filter` and `append` are linear in the
//! generated core as they are in the reference evaluator.

use crate::ast::*;
use crate::names;
use bdl_exec_ir::{Activation, DeclKind, ExecExpr, ExecIr, LocalId, PrimOp};
use bdl_ir::Ty;
use bdl_model::DeclId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitError(pub String);

pub fn rust_type(t: &Ty) -> Result<Type, EmitError> {
    Ok(match t {
        Ty::Bool => Type::path("bool"),
        Ty::Nat => Type::path("u64"),
        Ty::Q { .. } => Type::path("f64"),
        Ty::Sem { id } => Type::path(names::concept(*id)),
        Ty::Opt { inner } => Type::option(rust_type(inner)?),
        Ty::List { elem } => Type::vec(rust_type(elem)?),
        Ty::Prod { fst, snd } => Type::Tuple(vec![rust_type(fst)?, rust_type(snd)?]),
        Ty::Arr { .. } => {
            return Err(EmitError(format!(
                "function type {} has no runtime representation",
                bdl_check::pretty::kernel(t)
            )))
        }
    })
}

const DERIVES_VALUE: &[&str] = &["Clone", "Copy", "Debug", "PartialEq"];
const DERIVES_RECORD: &[&str] = &["Clone", "Copy", "Debug", "PartialEq", "Default"];

/// The derives of a record or concept: `Copy` only when nothing in the
/// program owns a list.
fn derives(d: &[&str], copy: bool) -> Vec<String> {
    d.iter()
        .filter(|s| copy || **s != "Copy")
        .map(|s| s.to_string())
        .collect()
}

/// The core module.
pub fn core_module(ir: &ExecIr, generator: &str) -> Result<Module, EmitError> {
    let copy = !ir.uses_lists();
    let mut items = Vec::new();
    if !copy {
        items.push(Item::ExternCrate("alloc".into()));
        items.push(Item::Use("alloc::vec::Vec".into()));
        items.push(Item::Use("bdl_runtime_core::list".into()));
    }
    items.extend([
        Item::Use("bdl_runtime_core::num".into()),
        Item::Use("bdl_runtime_core::prim".into()),
        Item::Use("bdl_runtime_core::read_decl".into()),
        Item::Use("bdl_runtime_core::read_decl_ref".into()),
        Item::Use("bdl_runtime_core::read_input".into()),
        Item::Use("bdl_runtime_core::ActiveDomains".into()),
        Item::Use("bdl_runtime_core::ClockSlot".into()),
        Item::Use("bdl_runtime_core::RuntimeError".into()),
        Item::Const {
            doc: vec!["The design this core was generated from.".into()],
            name: "DESIGN".into(),
            ty: Type::path("&str"),
            value: Expr::str(&ir.name),
        },
        Item::Const {
            doc: vec!["Whether the design has any clock domain; decides when domain-agnostic declarations run (DI-16).".into()],
            name: "HAS_DOMAINS".into(),
            ty: Type::path("bool"),
            value: Expr::bool(ir.has_domains),
        },
        Item::Const {
            doc: vec![],
            name: "CLOCK_COUNT".into(),
            ty: Type::path("u16"),
            value: Expr::Lit(Lit::U16(ir.clocks.len() as u16)),
        },
    ]);
    for c in &ir.clocks {
        items.push(Item::Const {
            doc: vec![format!("Clock domain `{}` ({}).", c.name, c.id)],
            name: names::clock(c.slot),
            ty: Type::path("ClockSlot"),
            value: Expr::call("ClockSlot", [Expr::Lit(Lit::U16(c.slot.0))]),
        });
    }

    for c in &ir.concepts {
        items.push(Item::Struct {
            doc: vec![format!(
                "Concept `{}` ({}): {}.",
                c.name,
                c.id,
                bdl_check::pretty::kernel(&c.representation)
            )],
            derives: derives(DERIVES_VALUE, copy),
            name: names::concept(c.id),
            fields: Fields::Tuple(vec![rust_type(&c.representation)?]),
        });
    }

    // Records.
    let mut cell_fields = Vec::new();
    for c in &ir.cells {
        cell_fields.push((names::cell(c.slot), Type::option(rust_type(&c.ty)?)));
    }
    items.push(Item::Struct {
        doc: vec![
            "Temporal state: one slot per `delay`/`sync` site, `None` until first written.".into(),
            "Slot ↔ `StateCellId` in `bdl-manifest.json`.".into(),
        ],
        derives: derives(DERIVES_RECORD, copy),
        name: "Cells".into(),
        fields: Fields::Named(cell_fields),
    });
    items.push(Item::Struct {
        doc: vec!["Everything that survives from one tick to the next.".into()],
        derives: derives(DERIVES_RECORD, copy),
        name: "State".into(),
        fields: Fields::Named(vec![("cells".into(), Type::path("Cells"))]),
    });
    let mut input_fields = Vec::new();
    for i in &ir.inputs {
        let d = ir
            .decl(i.decl)
            .ok_or_else(|| EmitError(format!("input {:?} points at no declaration", i.decl)))?;
        input_fields.push((names::decl(d.id), Type::option(rust_type(&d.ty)?)));
    }
    items.push(Item::Struct {
        doc: vec![
            "Values for the unresolved declarations at one tick.".into(),
            "A `None` for a declaration that is due is `RuntimeError::MissingInput`, never a default.".into(),
        ],
        derives: derives(DERIVES_RECORD, copy),
        name: "Inputs".into(),
        fields: Fields::Named(input_fields),
    });
    let mut value_fields = Vec::new();
    for d in &ir.decls {
        value_fields.push((names::decl(d.id), Type::option(rust_type(&d.ty)?)));
    }
    items.push(Item::Struct {
        doc: vec!["Every declaration's value at one tick; `None` when it was not due.".into()],
        derives: derives(DERIVES_RECORD, copy),
        name: "Values".into(),
        fields: Fields::Named(value_fields),
    });
    let mut output_fields = Vec::new();
    for o in &ir.outputs {
        output_fields.push((names::output(o.id), Type::option(rust_type(&o.ty)?)));
    }
    items.push(Item::Struct {
        doc: vec!["Physical outputs committed at one tick: the single driver's value, `None` when the driver was not due.".into()],
        derives: derives(DERIVES_RECORD, copy),
        name: "Outputs".into(),
        fields: Fields::Named(output_fields),
    });
    items.push(Item::Struct {
        doc: vec!["The observable result of one tick.".into()],
        derives: derives(DERIVES_RECORD, copy),
        name: "Tick".into(),
        fields: Fields::Named(vec![
            ("values".into(), Type::path("Values")),
            ("outputs".into(), Type::path("Outputs")),
        ]),
    });

    items.push(Item::Fn(Function {
        doc: vec!["The initial state: no cell has been written.".into()],
        attrs: vec![],
        public: true,
        name: "init".into(),
        params: vec![],
        ret: Some(Type::path("State")),
        body: Block::expr(Expr::call("State::default", [])),
    }));
    items.push(Item::Fn(step_fn(ir)?));

    Ok(Module {
        doc: vec![
            format!("Generated by {generator} for design `{}`. Do not edit.", ir.name),
            String::new(),
            "One global tick = `step`: read phase (every due declaration, in plan order,".into(),
            "reading inputs, declarations evaluated earlier this tick, and *committed* state),".into(),
            "write phase (every cell whose writing domain is active, into the next state),".into(),
            "commit, then output projection.  Nothing is mutated in place; reads never see".into(),
            "writes of the same tick.  Semantics: docs/spec/runtime-semantics.md.".into(),
        ],
        inner_attrs: vec![
            "no_std".into(),
            "forbid(unsafe_code)".into(),
            "allow(unused_imports, unused_variables, unused_mut, unused_parens, dead_code, clippy::all)".into(),
        ],
        items,
    })
}

fn due_expr(ir: &ExecIr, a: Activation) -> Expr {
    match a {
        Activation::Domain { clock } => Expr::method(
            Expr::path("active"),
            "is_active",
            [Expr::path(names::clock(clock))],
        ),
        Activation::Agnostic => {
            let _ = ir;
            Expr::call(
                "prim::or",
                [
                    Expr::method(Expr::path("active"), "any", []),
                    Expr::Not(Box::new(Expr::path("HAS_DOMAINS"))),
                ],
            )
        }
    }
}

fn step_fn(ir: &ExecIr) -> Result<Function, EmitError> {
    let mut stmts = Vec::new();
    stmts.push(Stmt::Let {
        name: "prev".into(),
        mutable: false,
        ty: Some(Type::reference(Type::path("Cells"), false)),
        value: Expr::Ref {
            mutable: false,
            e: Box::new(Expr::field(Expr::path("state"), "cells")),
        },
    });
    stmts.push(Stmt::Comment("read phase".into()));
    for d in &ir.decls {
        let raw = d.id.raw();
        let value = match &d.kind {
            DeclKind::Input { .. } => Expr::try_(Expr::call(
                "read_input",
                [
                    Expr::Ref {
                        mutable: false,
                        e: Box::new(Expr::field(Expr::path("inputs"), names::decl(d.id))),
                    },
                    Expr::u64(raw),
                ],
            )),
            DeclKind::Computed { body } => expr(ir, body, d.id)?,
        };
        stmts.push(Stmt::Comment(format!("{} ({})", d.name, d.id)));
        stmts.push(Stmt::Let {
            name: names::decl(d.id),
            mutable: false,
            ty: Some(Type::option(rust_type(&d.ty)?)),
            value: Expr::if_else(due_expr(ir, d.activation), Expr::some(value), Expr::none()),
        });
    }
    stmts.push(Stmt::Comment(
        "write phase: every active writer's operand, before any commit".into(),
    ));
    for c in &ir.cells {
        let owner = ir
            .decl(c.owner)
            .ok_or_else(|| EmitError(format!("cell {:?} owner missing", c.slot)))?;
        stmts.push(Stmt::Comment(format!(
            "{} at {:?} in {} ({})",
            names::cell(c.slot),
            c.cell.path,
            owner.name,
            owner.id
        )));
        stmts.push(Stmt::Let {
            name: names::write(c.slot),
            mutable: false,
            ty: Some(Type::option(rust_type(&c.ty)?)),
            value: Expr::if_else(
                Expr::method(
                    Expr::path("active"),
                    "is_active",
                    [Expr::path(names::clock(c.writer))],
                ),
                Expr::some(expr(ir, &c.operand, owner.id)?),
                Expr::none(),
            ),
        });
    }
    stmts.push(Stmt::Comment(
        "commit: only the cells written this tick change; nothing else is copied".into(),
    ));
    for c in &ir.cells {
        let write = Expr::assign(
            Expr::field(
                Expr::field(Expr::path("state"), "cells"),
                names::cell(c.slot),
            ),
            Expr::path(names::write(c.slot)),
        );
        stmts.push(Stmt::Expr(Expr::if_(
            Expr::method(Expr::path(names::write(c.slot)), "is_some", []),
            Block::new(vec![Stmt::Expr(write)], None),
            None,
        )));
    }
    let values = Expr::strukt(
        "Values",
        ir.decls
            .iter()
            .map(|d| (names::decl(d.id), Expr::path(names::decl(d.id)))),
    );
    let outputs = Expr::strukt(
        "Outputs",
        ir.outputs.iter().map(|o| {
            let driver = ir
                .decl(o.driver)
                .map(|d| names::decl(d.id))
                .unwrap_or_default();
            (
                names::output(o.id),
                Expr::method(Expr::path(driver), "clone", []),
            )
        }),
    );
    let tail = Expr::call(
        "Ok",
        [Expr::strukt(
            "Tick",
            [
                ("values".to_string(), values),
                ("outputs".to_string(), outputs),
            ],
        )],
    );
    Ok(Function {
        doc: vec![
            "One global tick with the given active domains and inputs.".into(),
            "On `Err` the state is unchanged.".into(),
        ],
        attrs: vec![],
        public: true,
        name: "step".into(),
        params: vec![
            ("state".into(), Type::reference(Type::path("State"), true)),
            ("active".into(), Type::path("ActiveDomains")),
            (
                "inputs".into(),
                Type::reference(Type::path("Inputs"), false),
            ),
        ],
        ret: Some(Type::path("Result<Tick, RuntimeError>")),
        body: Block::new(stmts, Some(tail)),
    })
}

/// An expression of declaration `owner` (for error attribution).
pub fn expr(ir: &ExecIr, e: &ExecExpr, owner: DeclId) -> Result<Expr, EmitError> {
    let mut uses = Uses::default();
    uses.analyse(e, 0);
    let mut cx = Cx {
        ir,
        owner,
        uses: &uses,
        depth: 0,
    };
    cx.expr(e)
}

/// Whether an expression can fail at runtime — whether its generated form
/// carries a `?`.  Arithmetic (non-finite results, division by zero) and
/// reads of declarations can; everything else is total.  A strict `ite`
/// whose branches are total is emitted lazily: the reference evaluates
/// both branches, but with nothing that can fail in either, choosing
/// first is not observable.
fn can_fail(e: &ExecExpr) -> bool {
    match e {
        ExecExpr::Bool { .. }
        | ExecExpr::Nat { .. }
        | ExecExpr::Quantity { .. }
        | ExecExpr::Local { .. } => false,
        ExecExpr::ReadDecl { .. } => true,
        ExecExpr::Let { value, body, .. } => can_fail(value) || can_fail(body),
        ExecExpr::Wrap { e, .. } | ExecExpr::Unwrap { e } => can_fail(e),
        ExecExpr::ReadCell { init, .. } => can_fail(init),
        ExecExpr::Prim { op, args } => {
            matches!(
                op,
                PrimOp::Add { .. } | PrimOp::Sub { .. } | PrimOp::Mul { .. } | PrimOp::Div { .. }
            ) || args.iter().any(can_fail)
        }
        ExecExpr::Fold {
            step, init, list, ..
        } => can_fail(step) || can_fail(init) || can_fail(list),
    }
}

/// A strict `ite` both of whose branches are total: emitted as a Rust
/// `if`, and its branches count as alternatives for the use analysis.
fn lazy_ite(op: &PrimOp, args: &[ExecExpr]) -> bool {
    matches!(op, PrimOp::Ite { .. })
        && args.len() == 3
        && !can_fail(&args[1])
        && !can_fail(&args[2])
}

/// Per local: how many times it is referenced (alternatives of a lazy
/// `if` counting once, as the larger branch) and the closure depth it is
/// bound at.  A local referenced once, in the scope that binds it, is
/// moved at that reference; any other reference clones.
#[derive(Default)]
struct Uses {
    count: BTreeMap<LocalId, usize>,
    depth: BTreeMap<LocalId, u32>,
}

impl Uses {
    fn analyse(&mut self, e: &ExecExpr, depth: u32) {
        match e {
            ExecExpr::Bool { .. }
            | ExecExpr::Nat { .. }
            | ExecExpr::Quantity { .. }
            | ExecExpr::ReadDecl { .. } => {}
            ExecExpr::Local { id } => *self.count.entry(*id).or_default() += 1,
            ExecExpr::Let { local, value, body } => {
                self.depth.insert(*local, depth);
                self.analyse(value, depth);
                self.analyse(body, depth);
            }
            ExecExpr::Wrap { e, .. } | ExecExpr::Unwrap { e } => self.analyse(e, depth),
            ExecExpr::ReadCell { init, .. } => self.analyse(init, depth),
            ExecExpr::Prim { op, args } if lazy_ite(op, args) => {
                self.analyse(&args[0], depth);
                let base = self.count.clone();
                self.analyse(&args[1], depth);
                let then = std::mem::replace(&mut self.count, base);
                self.analyse(&args[2], depth);
                for (id, n) in then {
                    let e = self.count.entry(id).or_default();
                    *e = (*e).max(n);
                }
            }
            ExecExpr::Prim { args, .. } => args.iter().for_each(|a| self.analyse(a, depth)),
            ExecExpr::Fold {
                elem,
                acc,
                step,
                init,
                list,
            } => {
                self.analyse(list, depth);
                self.analyse(init, depth);
                self.depth.insert(*elem, depth + 1);
                self.depth.insert(*acc, depth + 1);
                self.analyse(step, depth + 1);
            }
        }
    }

    fn moved(&self, id: LocalId, depth: u32) -> bool {
        self.count.get(&id).copied().unwrap_or(0) == 1
            && self.depth.get(&id).copied() == Some(depth)
    }
}

struct Cx<'a> {
    ir: &'a ExecIr,
    owner: DeclId,
    uses: &'a Uses,
    /// Closure nesting: 0 outside any fold step.
    depth: u32,
}

impl Cx<'_> {
    /// `e` as a borrow (`&T`): a local by reference, a declaration through
    /// `read_decl_ref`, anything else as a borrowed temporary, which lives
    /// through the call it is an argument of.
    fn borrowed(&mut self, e: &ExecExpr) -> Result<Expr, EmitError> {
        let r = |e: Expr| Expr::Ref {
            mutable: false,
            e: Box::new(e),
        };
        Ok(match e {
            ExecExpr::Local { id } => r(Expr::path(names::local(*id))),
            ExecExpr::ReadDecl { decl: d } => {
                let target = self
                    .ir
                    .decl(*d)
                    .ok_or_else(|| EmitError(format!("read of missing declaration {d:?}")))?;
                Expr::try_(Expr::call(
                    "read_decl_ref",
                    [
                        r(Expr::path(names::decl(target.id))),
                        Expr::u64(target.id.raw()),
                    ],
                ))
            }
            _ => r(self.expr(e)?),
        })
    }

    fn expr(&mut self, e: &ExecExpr) -> Result<Expr, EmitError> {
        Ok(match e {
            ExecExpr::Bool { value } => Expr::bool(*value),
            ExecExpr::Nat { value } => Expr::u64(*value),
            ExecExpr::Quantity { value, .. } => Expr::f64(value.0),
            ExecExpr::Local { id } => {
                if self.uses.moved(*id, self.depth) {
                    Expr::path(names::local(*id))
                } else {
                    Expr::method(Expr::path(names::local(*id)), "clone", [])
                }
            }
            ExecExpr::Let { local, value, body } => Expr::Block(Block::new(
                vec![Stmt::Let {
                    name: names::local(*local),
                    mutable: false,
                    ty: None,
                    value: self.expr(value)?,
                }],
                Some(self.expr(body)?),
            )),
            ExecExpr::Fold {
                elem,
                acc,
                step,
                init,
                list,
            } => {
                let list = self.expr(list)?;
                let init = self.expr(init)?;
                self.depth += 1;
                let step = self.expr(step);
                self.depth -= 1;
                Expr::try_(Expr::call(
                    "list::fold",
                    [
                        list,
                        init,
                        Expr::closure(
                            [names::local(*elem), names::local(*acc)],
                            None,
                            Expr::call("Ok", [step?]),
                        ),
                    ],
                ))
            }
            ExecExpr::ReadDecl { decl: d } => {
                let target = self
                    .ir
                    .decl(*d)
                    .ok_or_else(|| EmitError(format!("read of missing declaration {d:?}")))?;
                Expr::try_(Expr::call(
                    "read_decl",
                    [
                        Expr::Ref {
                            mutable: false,
                            e: Box::new(Expr::path(names::decl(target.id))),
                        },
                        Expr::u64(target.id.raw()),
                    ],
                ))
            }
            ExecExpr::Wrap { sem, e } => Expr::call(names::concept(*sem), [self.expr(e)?]),
            ExecExpr::Unwrap { e } => Expr::field(self.expr(e)?, "0"),
            ExecExpr::ReadCell { slot, init } => Expr::Match {
                scrutinee: Box::new(Expr::Ref {
                    mutable: false,
                    e: Box::new(Expr::field(Expr::path("prev"), names::cell(*slot))),
                }),
                arms: vec![
                    ("Some(v)".into(), Expr::method(Expr::path("v"), "clone", [])),
                    ("None".into(), self.expr(init)?),
                ],
            },
            ExecExpr::Prim { op, args } if lazy_ite(op, args) => Expr::if_else(
                self.expr(&args[0])?,
                self.expr(&args[1])?,
                self.expr(&args[2])?,
            ),
            // operators that only inspect a list (or compare two values):
            // borrowed operands, nothing copied
            ExecExpr::Prim { op, args } if args.len() == op.arity() => match op {
                PrimOp::Length { .. } => Expr::call("list::length_of", [self.borrowed(&args[0])?]),
                PrimOp::Head { .. } => Expr::call("list::head_of", [self.borrowed(&args[0])?]),
                PrimOp::Take { .. } => {
                    let k = self.expr(&args[0])?;
                    Expr::call("list::take_of", [k, self.borrowed(&args[1])?])
                }
                PrimOp::Eq | PrimOp::Lt => Expr::Binary {
                    op: if matches!(op, PrimOp::Eq) {
                        BinOp::Eq
                    } else {
                        BinOp::Lt
                    },
                    l: Box::new(self.borrowed(&args[0])?),
                    r: Box::new(self.borrowed(&args[1])?),
                },
                _ => self.prim(op, args)?,
            },
            ExecExpr::Prim { op, args } => self.prim(op, args)?,
        })
    }

    fn prim(&mut self, op: &PrimOp, args: &[ExecExpr]) -> Result<Expr, EmitError> {
        let decl = Expr::u64(self.owner.raw());
        Ok({
            {
                let mut a = Vec::with_capacity(args.len());
                for x in args {
                    a.push(self.expr(x)?);
                }
                let arity = op.arity();
                if a.len() != arity {
                    return Err(EmitError(format!(
                        "{op:?} applied to {} arguments",
                        a.len()
                    )));
                }
                let mut it = a.into_iter();
                let mut next = || {
                    it.next()
                        .ok_or_else(|| EmitError("missing argument".into()))
                };
                match op {
                    PrimOp::Add { .. } => {
                        Expr::try_(Expr::call("num::add", [next()?, next()?, decl]))
                    }
                    PrimOp::Sub { .. } => {
                        Expr::try_(Expr::call("num::sub", [next()?, next()?, decl]))
                    }
                    PrimOp::Mul { .. } => {
                        Expr::try_(Expr::call("num::mul", [next()?, next()?, decl]))
                    }
                    PrimOp::Div { .. } => {
                        Expr::try_(Expr::call("num::div", [next()?, next()?, decl]))
                    }
                    PrimOp::Lt => Expr::Binary {
                        op: BinOp::Lt,
                        l: Box::new(next()?),
                        r: Box::new(next()?),
                    },
                    PrimOp::Eq => Expr::Binary {
                        op: BinOp::Eq,
                        l: Box::new(next()?),
                        r: Box::new(next()?),
                    },
                    PrimOp::Not => Expr::Not(Box::new(next()?)),
                    PrimOp::And => Expr::call("prim::and", [next()?, next()?]),
                    PrimOp::Or => Expr::call("prim::or", [next()?, next()?]),
                    PrimOp::Ite { .. } => Expr::call("prim::ite", [next()?, next()?, next()?]),
                    PrimOp::None { ty } => Expr::path(format!(
                        "Option::<{}>::None",
                        crate::print::ty_str(&rust_type(ty)?)
                    )),
                    PrimOp::Some { .. } => Expr::some(next()?),
                    PrimOp::IsSome { .. } => Expr::method(next()?, "is_some", []),
                    PrimOp::GetD { .. } => Expr::call("prim::get_d", [next()?, next()?]),
                    PrimOp::Nil { ty } => Expr::call(
                        format!("list::nil::<{}>", crate::print::ty_str(&rust_type(ty)?)),
                        [],
                    ),
                    PrimOp::Cons { .. } => Expr::call("list::cons", [next()?, next()?]),
                    PrimOp::Length { .. } => Expr::call("list::length", [next()?]),
                    PrimOp::Take { .. } => Expr::call("list::take", [next()?, next()?]),
                    PrimOp::Drop { .. } => Expr::call("list::drop", [next()?, next()?]),
                    PrimOp::Reverse { .. } => Expr::call("list::reverse", [next()?]),
                    PrimOp::Head { .. } => Expr::call("list::head", [next()?]),
                    PrimOp::ToList { .. } => Expr::call("list::to_list", [next()?]),
                    PrimOp::Pair { .. } => Expr::Tuple(vec![next()?, next()?]),
                    PrimOp::Fst { .. } => Expr::field(next()?, "0"),
                    PrimOp::Snd { .. } => Expr::field(next()?, "1"),
                }
            }
        })
    }
}
