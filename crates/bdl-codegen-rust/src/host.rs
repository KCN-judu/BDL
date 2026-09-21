//! The generated `host` binary: an implementation of
//! `bdl_runtime_host::HostProgram` bridging the core's static types and
//! `DynValue`, plus `main`.  `std` code, never part of the core.

use crate::ast::*;
use crate::emit::EmitError;
use crate::names;
use bdl_exec_ir::ExecIr;
use bdl_ir::Ty;
use bdl_model::ConceptId;
use std::collections::BTreeMap;

/// Concept representations, from the exec IR's concept plans.
type Reps = BTreeMap<ConceptId, Ty>;

fn representation(reps: &Reps, id: ConceptId) -> Result<&Ty, EmitError> {
    reps.get(&id)
        .ok_or_else(|| EmitError(format!("concept {id} is not in the plan")))
}

pub fn host_module(
    ir: &ExecIr,
    package: &str,
    generator: &str,
    plan: Option<&crate::adapter::AdapterPlan>,
) -> Result<Module, EmitError> {
    let reps: Reps = ir
        .concepts
        .iter()
        .map(|c| (c.id, c.representation.clone()))
        .collect();
    let mut items = Vec::new();
    items.push(Item::Use(format!("{package} as design")));
    items.push(Item::Use("bdl_runtime_core::ActiveDomains".into()));
    items.push(Item::Use("bdl_runtime_core::RuntimeError".into()));
    items.push(Item::Use("bdl_runtime_host::AdapterOp".into()));
    items.push(Item::Use("bdl_runtime_host::BridgeError".into()));
    items.push(Item::Use("bdl_runtime_host::DynValue".into()));
    items.push(Item::Use("bdl_runtime_host::HostProgram".into()));
    items.push(Item::Use("bdl_runtime_host::ReadingError".into()));
    items.push(Item::Struct {
        doc: vec![],
        derives: vec![],
        name: "Bridge".into(),
        fields: Fields::Named(vec![]),
    });

    // inputs_from_dyn
    let mut input_fields = Vec::new();
    for i in &ir.inputs {
        let d = ir
            .decl(i.decl)
            .ok_or_else(|| EmitError(format!("input {:?} points at no declaration", i.decl)))?;
        let slot = i.slot.0 as usize;
        let conv = from_dyn(&reps, &d.ty, slot)?;
        input_fields.push((
            names::decl(d.id),
            Expr::Match {
                scrutinee: Box::new(Expr::method(
                    Expr::path("slots"),
                    "get",
                    [Expr::Lit(Lit::Usize(slot))],
                )),
                arms: vec![
                    ("Some(Some(v))".into(), Expr::some(conv)),
                    ("_".into(), Expr::none()),
                ],
            },
        ));
    }
    let inputs_fn = Function {
        doc: vec![],
        attrs: vec![],
        is_async: false,
        public: false,
        name: "inputs_from_dyn".into(),
        params: vec![("slots".into(), Type::path("&[Option<DynValue>]"))],
        ret: Some(Type::path("Result<design::Inputs, BridgeError>")),
        body: Block::expr(Expr::call(
            "Ok",
            [Expr::strukt("design::Inputs", input_fields)],
        )),
    };

    let mut values = Vec::new();
    for d in &ir.decls {
        values.push(Expr::method(
            Expr::method(
                Expr::field(Expr::field(Expr::path("tick"), "values"), names::decl(d.id)),
                "clone",
                [],
            ),
            "map",
            [Expr::closure(
                ["v".to_string()],
                None,
                to_dyn(&reps, &d.ty)?,
            )],
        ));
    }
    let values_fn = Function {
        doc: vec![],
        attrs: vec![],
        is_async: false,
        public: false,
        name: "values_to_dyn".into(),
        params: vec![("tick".into(), Type::path("&design::Tick"))],
        ret: Some(Type::path("Vec<Option<DynValue>>")),
        body: Block::expr(Expr::VecMacro(values)),
    };
    let mut outputs = Vec::new();
    for o in &ir.outputs {
        outputs.push(Expr::method(
            Expr::method(
                Expr::field(
                    Expr::field(Expr::path("tick"), "outputs"),
                    names::output(o.id),
                ),
                "clone",
                [],
            ),
            "map",
            [Expr::closure(
                ["v".to_string()],
                None,
                to_dyn(&reps, &o.ty)?,
            )],
        ));
    }
    let outputs_fn = Function {
        doc: vec![],
        attrs: vec![],
        is_async: false,
        public: false,
        name: "outputs_to_dyn".into(),
        params: vec![("tick".into(), Type::path("&design::Tick"))],
        ret: Some(Type::path("Vec<Option<DynValue>>")),
        body: Block::expr(Expr::VecMacro(outputs)),
    };
    let mut commands = Vec::new();
    for s in &ir.sinks {
        commands.push(Expr::method(
            Expr::method(
                Expr::field(
                    Expr::field(Expr::path("tick"), "commands"),
                    names::command(s.device),
                ),
                "clone",
                [],
            ),
            "map",
            [Expr::closure(
                ["v".to_string()],
                None,
                to_dyn(&reps, &s.raw)?,
            )],
        ));
    }
    let commands_fn = Function {
        doc: vec![],
        attrs: vec![],
        is_async: false,
        public: false,
        name: "commands_to_dyn".into(),
        params: vec![("tick".into(), Type::path("&design::Tick"))],
        ret: Some(Type::path("Vec<Option<DynValue>>")),
        body: Block::expr(Expr::VecMacro(commands)),
    };

    items.push(Item::Impl {
        trait_: Some("HostProgram".into()),
        target: Type::path("Bridge"),
        items: vec![
            ImplItem::Type("State".into(), Type::path("design::State")),
            ImplItem::Type("Inputs".into(), Type::path("design::Inputs")),
            ImplItem::Type("Tick".into(), Type::path("design::Tick")),
            ImplItem::Fn(Function {
                doc: vec![],
                attrs: vec![],
                is_async: false,
                public: false,
                name: "init".into(),
                params: vec![],
                ret: Some(Type::path("design::State")),
                body: Block::expr(Expr::call("design::init", [])),
            }),
            ImplItem::Fn(Function {
                doc: vec![],
                attrs: vec![],
                is_async: false,
                public: false,
                name: "step".into(),
                params: vec![
                    ("state".into(), Type::path("&mut design::State")),
                    ("active".into(), Type::path("ActiveDomains")),
                    ("inputs".into(), Type::path("&design::Inputs")),
                ],
                ret: Some(Type::path("Result<design::Tick, RuntimeError>")),
                body: Block::expr(Expr::call(
                    "design::step",
                    [
                        Expr::path("state"),
                        Expr::path("active"),
                        Expr::path("inputs"),
                    ],
                )),
            }),
            ImplItem::Fn(inputs_fn),
            ImplItem::Fn(crate::adapter::host_inputs_from_readings(ir, plan)?),
            ImplItem::Fn(values_fn),
            ImplItem::Fn(outputs_fn),
            ImplItem::Fn(commands_fn),
            ImplItem::Fn(crate::adapter::host_adapter_ops(plan)),
            ImplItem::Fn(Function {
                doc: vec![],
                attrs: vec![],
                is_async: false,
                public: false,
                name: "state_bytes".into(),
                params: vec![],
                ret: Some(Type::path("usize")),
                body: Block::expr(Expr::call("std::mem::size_of::<design::State>", [])),
            }),
        ],
    });
    items.push(Item::Fn(Function {
        doc: vec![],
        attrs: vec![],
        is_async: false,
        public: false,
        name: "main".into(),
        params: vec![],
        ret: None,
        body: Block::expr(Expr::call("bdl_runtime_host::main_stdio::<Bridge>", [])),
    }));
    Ok(Module {
        doc: vec![format!(
            "Generated by {generator} for design `{}`: host bridge. Do not edit.",
            ir.name
        )],
        inner_attrs: vec![
            "forbid(unsafe_code)".into(),
            "allow(unused_imports, unused_variables, clippy::all)".into(),
        ],
        items,
    })
}

/// `v: &DynValue` → the static type, or a `BridgeError` for `slot`.
fn from_dyn(reps: &Reps, ty: &Ty, slot: usize) -> Result<Expr, EmitError> {
    let s = Expr::Lit(Lit::Usize(slot));
    Ok(match ty {
        Ty::Q { .. } => Expr::try_(Expr::method(Expr::path("v"), "quantity", [s])),
        Ty::Bool => Expr::try_(Expr::method(Expr::path("v"), "boolean", [s])),
        Ty::Nat => Expr::try_(Expr::method(Expr::path("v"), "nat", [s])),
        Ty::Sem { id } => {
            let rep = representation(reps, *id)?;
            let inner = Expr::Block(Block::new(
                vec![Stmt::Let {
                    name: "v".into(),
                    mutable: false,
                    ty: None,
                    value: Expr::try_(Expr::method(
                        Expr::path("v"),
                        "semantic",
                        [Expr::u64(id.raw()), s],
                    )),
                }],
                Some(from_dyn(reps, rep, slot)?),
            ));
            Expr::call(format!("design::{}", names::concept(*id)), [inner])
        }
        Ty::Opt { inner } => Expr::Match {
            scrutinee: Box::new(Expr::try_(Expr::method(Expr::path("v"), "option", [s]))),
            arms: vec![
                ("None".into(), Expr::none()),
                ("Some(v)".into(), Expr::some(from_dyn(reps, inner, slot)?)),
            ],
        },
        // The core stores lists last element first (`bdl_runtime_core::list`):
        // bdl_runtime_core::list::from_ordered(v.list(slot)?.iter().map(|v| -> Result<T, BridgeError> { Ok(…) }).collect::<Result<Vec<_>, BridgeError>>()?)
        Ty::List { elem } => Expr::call(
            "bdl_runtime_core::list::from_ordered",
            [Expr::try_(Expr::method(
                Expr::method(
                    Expr::method(
                        Expr::try_(Expr::method(Expr::path("v"), "list", [s])),
                        "iter",
                        [],
                    ),
                    "map",
                    [Expr::closure(
                        ["v".to_string()],
                        Some(Type::path(format!(
                            "Result<{}, BridgeError>",
                            crate::print::ty_str(&crate::emit::rust_type(elem).map(host_type)?)
                        ))),
                        Expr::call("Ok", [from_dyn(reps, elem, slot)?]),
                    )],
                ),
                "collect::<Result<Vec<_>, BridgeError>>",
                [],
            ))],
        ),
        Ty::Prod { fst, snd } => Expr::Block(Block::new(
            vec![Stmt::Let {
                name: "(a, b)".into(),
                mutable: false,
                ty: None,
                value: Expr::try_(Expr::method(Expr::path("v"), "pair", [s])),
            }],
            Some(Expr::Tuple(vec![
                Expr::Block(Block::new(
                    vec![Stmt::Let {
                        name: "v".into(),
                        mutable: false,
                        ty: None,
                        value: Expr::path("a"),
                    }],
                    Some(from_dyn(reps, fst, slot)?),
                )),
                Expr::Block(Block::new(
                    vec![Stmt::Let {
                        name: "v".into(),
                        mutable: false,
                        ty: None,
                        value: Expr::path("b"),
                    }],
                    Some(from_dyn(reps, snd, slot)?),
                )),
            ])),
        )),
        Ty::Arr { .. } => return Err(EmitError("function-typed input".into())),
        Ty::Unit => {
            return Err(EmitError(
                "the empty product is never a runtime value".into(),
            ))
        }
    })
}

/// The core's types as the host names them: `SemN` is `design::SemN`.
fn host_type(t: Type) -> Type {
    match t {
        Type::Path(p) if p.starts_with("Sem") => Type::Path(format!("design::{p}")),
        Type::Option(i) => Type::Option(Box::new(host_type(*i))),
        Type::Vec(i) => Type::Vec(Box::new(host_type(*i))),
        Type::Tuple(ts) => Type::Tuple(ts.into_iter().map(host_type).collect()),
        other => other,
    }
}

/// `v: T` → `DynValue`.
fn to_dyn(reps: &Reps, ty: &Ty) -> Result<Expr, EmitError> {
    Ok(match ty {
        Ty::Q { .. } => Expr::strukt(
            "DynValue::Quantity",
            [("value".to_string(), Expr::path("v"))],
        ),
        Ty::Bool => Expr::strukt("DynValue::Bool", [("value".to_string(), Expr::path("v"))]),
        Ty::Nat => Expr::strukt("DynValue::Nat", [("value".to_string(), Expr::path("v"))]),
        Ty::Sem { id } => {
            let rep = representation(reps, *id)?;
            let inner = Expr::Block(Block::new(
                vec![Stmt::Let {
                    name: "v".into(),
                    mutable: false,
                    ty: None,
                    value: Expr::field(Expr::path("v"), "0"),
                }],
                Some(to_dyn(reps, rep)?),
            ));
            Expr::call("DynValue::sem", [Expr::u64(id.raw()), inner])
        }
        Ty::Opt { inner } => Expr::Match {
            scrutinee: Box::new(Expr::path("v")),
            arms: vec![
                ("None".into(), Expr::path("DynValue::None")),
                (
                    "Some(v)".into(),
                    Expr::call("DynValue::some", [to_dyn(reps, inner)?]),
                ),
            ],
        },
        Ty::List { elem } => Expr::strukt(
            "DynValue::List",
            [(
                "items".to_string(),
                Expr::method(
                    Expr::method(
                        Expr::method(
                            Expr::call("bdl_runtime_core::list::into_ordered", [Expr::path("v")]),
                            "into_iter",
                            [],
                        ),
                        "map",
                        [Expr::closure(["v".to_string()], None, to_dyn(reps, elem)?)],
                    ),
                    "collect",
                    [],
                ),
            )],
        ),
        Ty::Prod { fst, snd } => Expr::Block(Block::new(
            vec![Stmt::Let {
                name: "(a, b)".into(),
                mutable: false,
                ty: None,
                value: Expr::path("v"),
            }],
            Some(Expr::call(
                "DynValue::pair_of",
                [
                    Expr::Block(Block::new(
                        vec![Stmt::Let {
                            name: "v".into(),
                            mutable: false,
                            ty: None,
                            value: Expr::path("a"),
                        }],
                        Some(to_dyn(reps, fst)?),
                    )),
                    Expr::Block(Block::new(
                        vec![Stmt::Let {
                            name: "v".into(),
                            mutable: false,
                            ty: None,
                            value: Expr::path("b"),
                        }],
                        Some(to_dyn(reps, snd)?),
                    )),
                ],
            )),
        )),
        Ty::Arr { .. } => return Err(EmitError("function-typed value".into())),
        Ty::Unit => {
            return Err(EmitError(
                "the empty product is never a runtime value".into(),
            ))
        }
    })
}
