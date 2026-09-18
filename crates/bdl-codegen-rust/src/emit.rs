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
//! | `ReadDecl` | `read_decl(&decl_n, n)?` |
//! | `CellPlan` | `Cells.cell_k: Option<T>`; write `next.cell_k = Some(operand)` when writer active |
//! | `ReadCell` | `match &prev.cell_k { Some(v) => v.clone(), None => init }` |
//! | `Wrap` / `Unwrap` | `SemN(e)` / `e.0` |
//! | `PrimOp` | `num::add(a, b, decl)?` … / `(a < b)` / `prim::ite(c, x, y)` / `list::cons(x, xs)` |
//! | `Fold` | `list::fold(xs, init, \|elem, acc\| Ok(step))?` — one closure per recursor, no closure value escapes |
//! | `Ty::List` / `Ty::Prod` | `Vec<T>` (feature `collections`, `extern crate alloc`) / `(A, B)` |
//! | `OutputPlan` | `Outputs.output_n = decl_driver.clone()` after the write phase |
//!
//! A program without lists derives `Copy` on its records and never
//! allocates; one with lists derives `Clone` only and turns the runtime's
//! `collections` feature on (recorded in the manifest).

use crate::ast::*;
use crate::names;
use bdl_exec_ir::{Activation, DeclKind, ExecExpr, ExecIr, PrimOp};
use bdl_ir::Ty;
use bdl_model::DeclId;

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
    stmts.push(Stmt::Let {
        name: "next".into(),
        mutable: true,
        ty: Some(Type::path("Cells")),
        value: Expr::method(Expr::field(Expr::path("state"), "cells"), "clone", []),
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
        "write phase: into the next state only".into(),
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
        let write = Expr::assign(
            Expr::field(Expr::path("next"), names::cell(c.slot)),
            Expr::some(expr(ir, &c.operand, owner.id)?),
        );
        stmts.push(Stmt::Expr(Expr::if_(
            Expr::method(
                Expr::path("active"),
                "is_active",
                [Expr::path(names::clock(c.writer))],
            ),
            Block::new(vec![Stmt::Expr(write)], None),
            None,
        )));
    }
    stmts.push(Stmt::Comment("commit".into()));
    stmts.push(Stmt::Expr(Expr::assign(
        Expr::field(Expr::path("state"), "cells"),
        Expr::path("next"),
    )));
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
    expr_in(ir, e, owner, false)
}

/// `in_closure`: inside a fold step, where a local of the enclosing scope
/// is captured and must be cloned rather than moved.
fn expr_in(ir: &ExecIr, e: &ExecExpr, owner: DeclId, in_closure: bool) -> Result<Expr, EmitError> {
    let decl = Expr::u64(owner.raw());
    let expr = |e: &ExecExpr| expr_in(ir, e, owner, in_closure);
    Ok(match e {
        ExecExpr::Bool { value } => Expr::bool(*value),
        ExecExpr::Nat { value } => Expr::u64(*value),
        ExecExpr::Quantity { value, .. } => Expr::f64(value.0),
        ExecExpr::Local { id } => {
            if in_closure {
                Expr::method(Expr::path(names::local(*id)), "clone", [])
            } else {
                Expr::path(names::local(*id))
            }
        }
        ExecExpr::Let { local, value, body } => Expr::Block(Block::new(
            vec![Stmt::Let {
                name: names::local(*local),
                mutable: false,
                ty: None,
                value: expr(value)?,
            }],
            Some(expr(body)?),
        )),
        ExecExpr::Fold {
            elem,
            acc,
            step,
            init,
            list,
        } => Expr::try_(Expr::call(
            "list::fold",
            [
                expr(list)?,
                expr(init)?,
                Expr::closure(
                    [names::local(*elem), names::local(*acc)],
                    None,
                    Expr::call("Ok", [expr_in(ir, step, owner, true)?]),
                ),
            ],
        )),
        ExecExpr::ReadDecl { decl: d } => {
            let target = ir
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
        ExecExpr::Wrap { sem, e } => Expr::call(names::concept(*sem), [expr(e)?]),
        ExecExpr::Unwrap { e } => Expr::field(expr(e)?, "0"),
        ExecExpr::ReadCell { slot, init } => Expr::Match {
            scrutinee: Box::new(Expr::Ref {
                mutable: false,
                e: Box::new(Expr::field(Expr::path("prev"), names::cell(*slot))),
            }),
            arms: vec![
                ("Some(v)".into(), Expr::method(Expr::path("v"), "clone", [])),
                ("None".into(), expr(init)?),
            ],
        },
        ExecExpr::Prim { op, args } => {
            let mut a = Vec::with_capacity(args.len());
            for x in args {
                a.push(expr(x)?);
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
                PrimOp::Add { .. } => Expr::try_(Expr::call("num::add", [next()?, next()?, decl])),
                PrimOp::Sub { .. } => Expr::try_(Expr::call("num::sub", [next()?, next()?, decl])),
                PrimOp::Mul { .. } => Expr::try_(Expr::call("num::mul", [next()?, next()?, decl])),
                PrimOp::Div { .. } => Expr::try_(Expr::call("num::div", [next()?, next()?, decl])),
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
