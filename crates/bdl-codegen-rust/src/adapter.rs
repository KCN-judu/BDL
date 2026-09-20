//! The platform adapter's generated half (docs/architecture/embedded-adapter.md).
//!
//! A core's machine sinks (`SinkPlan`, `Commands`) stop at the raw command
//! boundary.  An [`AdapterPlan`] — built by the compiler from the solved
//! deployment, never here — says which board resource carries each sink
//! and how the firmware ticks; this module turns it into:
//!
//! * `src/adapter.rs`, target-independent glue over `bdl-runtime-adapter`:
//!   `SINKS` (the bindings as data), `Applied` (what became of each
//!   command at one tick) and `apply(tick, sink, …)`, one `&mut dyn`
//!   sink parameter per machine sink in sink order — no map, no name, no
//!   string dispatch;
//! * a target entry (`targets::rp2040`) that constructs the sinks on the
//!   assigned peripherals and drives the tick loop;
//! * the host bridge's `adapter_ops`, which applies the same `apply` to
//!   recording sinks so a host trace carries the firmware's operations.
//!
//! Nothing here reads a declaration, an output or a concept; the plan
//! carries ids and resource names only.

use crate::ast::*;
use crate::emit::{expr, rust_type, EmitError};
use crate::names;
use bdl_exec_ir::{ExecIr, InputSlot, SinkSlot};
use bdl_model::{DeviceId, InputProfileId, OutputProfileId};

/// How a sink's raw command reaches a peripheral: the sink trait it is
/// applied through and the numeric policy on the way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SinkKind {
    /// A `q[1]` raw duty, `bdl_runtime_adapter::duty8`, a `PwmDuty8` sink.
    PwmDuty8,
    /// A `bool` raw command, a `Level` sink.
    Level,
}

impl SinkKind {
    pub fn trait_name(self) -> &'static str {
        match self {
            SinkKind::PwmDuty8 => "PwmDuty8",
            SinkKind::Level => "Level",
        }
    }
    /// The board capability the resource must carry.
    pub fn capability(self) -> &'static str {
        match self {
            SinkKind::PwmDuty8 => "pwm",
            SinkKind::Level => "digital_out",
        }
    }
}

/// How a provider's raw reading is taken from a peripheral: the source
/// trait it is read through and the peripheral configuration the profile
/// prescribes (a pull is configuration, never a value; a polarity is the
/// transducer's, never the reader's).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceKind {
    /// A `bool` raw reading from a line pulled down, a `LevelSource`.
    LevelPullDown,
    /// A `bool` raw reading from a line pulled up, a `LevelSource`.
    LevelPullUp,
}

impl SourceKind {
    pub fn trait_name(self) -> &'static str {
        match self {
            SourceKind::LevelPullDown | SourceKind::LevelPullUp => "LevelSource",
        }
    }
    /// The board capability the resource must carry.
    pub fn capability(self) -> &'static str {
        match self {
            SourceKind::LevelPullDown | SourceKind::LevelPullUp => "digital_in",
        }
    }
}

/// One provider bound to one solver-assigned board resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderBinding {
    pub slot: InputSlot,
    pub device: DeviceId,
    pub profile: InputProfileId,
    pub kind: SourceKind,
    /// The resource as the board names it (`GP2`); identity, not a label.
    pub resource: String,
}

/// One machine sink bound to one solver-assigned board resource.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SinkBinding {
    pub slot: SinkSlot,
    pub device: DeviceId,
    pub profile: OutputProfileId,
    pub kind: SinkKind,
    /// The resource as the board names it (`GP15`); identity, not a label.
    pub resource: String,
    /// The board's unit behind the capability (a PWM slice), when it has
    /// one; a target may check it against its own derivation.
    pub unit: Option<u32>,
}

/// Everything a target entry needs beyond the exec IR.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdapterPlan {
    /// The board id (`rp2040_pico`) and its family (`rp2040`), which
    /// selects the target entry.
    pub board: String,
    pub family: String,
    /// The base tick the firmware's timer fires at.
    pub tick_micros: u64,
    /// Activation period in ticks per clock slot, in slot order.
    pub periods: Vec<u64>,
    /// In sink order.
    pub sinks: Vec<SinkBinding>,
    /// In input-slot order (the exec IR's `providers` order).
    pub providers: Vec<ProviderBinding>,
    /// The collection arena in bytes when the core carries lists.
    pub arena_bytes: Option<u64>,
}

/// `src/adapter.rs`.
pub fn adapter_module(
    ir: &ExecIr,
    plan: &AdapterPlan,
    generator: &str,
) -> Result<Module, EmitError> {
    let mut items = vec![
        Item::Use("bdl_runtime_adapter::apply_duty8".into()),
        Item::Use("bdl_runtime_adapter::apply_level".into()),
        Item::Use("bdl_runtime_adapter::read_level".into()),
        Item::Use("bdl_runtime_adapter::CommandFault".into()),
        Item::Use("bdl_runtime_adapter::Level".into()),
        Item::Use("bdl_runtime_adapter::LevelSource".into()),
        Item::Use("bdl_runtime_adapter::PwmDuty8".into()),
        Item::Use("bdl_runtime_adapter::SinkBinding".into()),
        Item::Use("bdl_runtime_adapter::SourceBinding".into()),
        Item::Use("crate::RuntimeError".into()),
        Item::Use("crate::*".into()),
    ];
    let sources: Vec<Expr> = plan
        .providers
        .iter()
        .map(|s| {
            Expr::strukt(
                "SourceBinding",
                [
                    ("device_id".to_string(), Expr::u64(s.device.raw())),
                    ("symbol".to_string(), Expr::str(names::reading(s.device))),
                    ("profile".to_string(), Expr::str(s.profile.as_str())),
                    ("resource".to_string(), Expr::str(&s.resource)),
                    ("capability".to_string(), Expr::str(s.kind.capability())),
                ],
            )
        })
        .collect();
    items.push(Item::Const {
        doc: vec![format!(
            "The providers and the board resources the placement assigned them on `{}`, in input-slot order.",
            plan.board
        )],
        name: "SOURCES".into(),
        ty: Type::path(format!("[SourceBinding; {}]", plan.providers.len())),
        value: Expr::Array(sources),
    });
    items.push(Item::Fn(provide_fn(ir, plan)?));
    items.push(Item::Fn(read_fn(plan)));
    let bindings: Vec<Expr> = plan
        .sinks
        .iter()
        .map(|s| {
            Expr::strukt(
                "SinkBinding",
                [
                    ("device_id".to_string(), Expr::u64(s.device.raw())),
                    ("symbol".to_string(), Expr::str(names::command(s.device))),
                    ("profile".to_string(), Expr::str(s.profile.as_str())),
                    ("resource".to_string(), Expr::str(&s.resource)),
                    ("capability".to_string(), Expr::str(s.kind.capability())),
                ],
            )
        })
        .collect();
    items.push(Item::Const {
        doc: vec![format!(
            "The machine sinks and the board resources the placement assigned them on `{}`, in sink order.",
            plan.board
        )],
        name: "SINKS".into(),
        ty: Type::path(format!("[SinkBinding; {}]", plan.sinks.len())),
        value: Expr::Array(bindings),
    });
    let applied_fields: Vec<(String, Type)> = plan
        .sinks
        .iter()
        .map(|s| (names::command(s.device), Type::option(applied_ty(s.kind))))
        .collect();
    items.push(Item::Struct {
        doc: vec![
            "What became of each command at one tick: `None` when the driver was not due (the line holds),".into(),
            "an accepted value when applied, a `CommandFault` when the numeric policy refused it (the line holds).".into(),
        ],
        derives: vec!["Clone".into(), "Copy".into(), "Debug".into(), "PartialEq".into()],
        name: "Applied".into(),
        fields: Fields::Named(applied_fields),
    });
    let mut params = vec![(
        "tick".to_string(),
        Type::reference(Type::path("crate::Tick"), false),
    )];
    let mut stmts = Vec::new();
    for s in &plan.sinks {
        let sym = names::command(s.device);
        params.push((
            sym.clone(),
            Type::path(format!("&mut dyn {}", s.kind.trait_name())),
        ));
        let apply = match s.kind {
            SinkKind::PwmDuty8 => "apply_duty8",
            SinkKind::Level => "apply_level",
        };
        let sink = ir
            .sinks
            .iter()
            .find(|x| x.device == s.device)
            .map(|x| {
                format!(
                    "{} realises {} as `{}` on {}",
                    x.device_name,
                    output_name(ir, x.output),
                    x.profile,
                    s.resource
                )
            })
            .unwrap_or_default();
        stmts.push(Stmt::Comment(sink));
        stmts.push(Stmt::Let {
            name: sym.clone(),
            mutable: false,
            ty: None,
            value: Expr::Match {
                scrutinee: Box::new(Expr::field(
                    Expr::field(Expr::path("tick"), "commands"),
                    sym.clone(),
                )),
                arms: vec![
                    (
                        "Some(raw)".into(),
                        Expr::some(Expr::call(apply, [Expr::path(&sym), Expr::path("raw")])),
                    ),
                    ("None".into(), Expr::none()),
                ],
            },
        });
    }
    let tail = Expr::strukt(
        "Applied",
        plan.sinks.iter().map(|s| {
            (
                names::command(s.device),
                Expr::path(names::command(s.device)),
            )
        }),
    );
    items.push(Item::Fn(Function {
        doc: vec![
            "Apply one tick's commands to the sinks, in sink order, after the core has stepped.".into(),
            "A sink whose driver was not due is left as it is; a refused command leaves it as it is.".into(),
        ],
        attrs: vec![],
        is_async: false,
        public: true,
        name: "apply".into(),
        params,
        ret: Some(Type::path("Applied")),
        body: Block::new(stmts, Some(tail)),
    }));
    Ok(Module {
        doc: vec![
            format!(
                "Generated by {generator} for design `{}` on `{}`. Do not edit.",
                ir.name, plan.board
            ),
            String::new(),
            "The platform adapter's target-independent glue: each provider's raw reading turned".into(),
            "into the core's `Inputs` through its transducer (`provide`), and each machine sink's raw".into(),
            "command (`Tick.commands`) applied to one sink through `bdl-runtime-adapter`'s numeric policy.".into(),
            "Observation ends at the reading and behavior begins at the input; behavior ends at the".into(),
            "command and physical effect begins in the sink (docs/architecture/embedded-adapter.md).".into(),
        ],
        inner_attrs: vec![
            "allow(unused_imports, unused_variables, clippy::all)".into(),
        ],
        items,
    })
}

/// `provide(reading_<dev>: Option<Raw>, …) -> Result<crate::Inputs, RuntimeError>`:
/// every provided Source's value from its raw reading, the rest `None`.
/// Each provider's term reads its reading and nothing else, so the
/// function is pure; a reading not taken leaves the slot empty.
fn provide_fn(ir: &ExecIr, plan: &AdapterPlan) -> Result<Function, EmitError> {
    let mut params = Vec::new();
    let mut stmts = Vec::new();
    let mut fields: Vec<(String, Expr)> = Vec::new();
    for b in &plan.providers {
        let p = ir
            .providers
            .iter()
            .find(|p| p.slot == b.slot)
            .ok_or_else(|| EmitError(format!("provider slot {:?} not in the exec IR", b.slot)))?;
        let source = ir
            .decl(p.decl)
            .ok_or_else(|| EmitError(format!("provider {:?} points at no declaration", p.decl)))?;
        let sym = names::reading(b.device);
        params.push((sym.clone(), Type::option(rust_type(&p.raw)?)));
        stmts.push(Stmt::Comment(format!(
            "{} provides {} as `{}` on {}",
            p.device_name, source.name, p.profile, b.resource
        )));
        let value = expr(ir, &p.provide, source.id)?;
        stmts.push(Stmt::Let {
            name: names::decl(source.id),
            mutable: false,
            ty: None,
            value: Expr::Match {
                scrutinee: Box::new(Expr::path(&sym)),
                arms: vec![
                    (
                        format!("Some({})", names::local(p.local)),
                        Expr::some(value),
                    ),
                    ("None".into(), Expr::none()),
                ],
            },
        });
        fields.push((names::decl(source.id), Expr::path(names::decl(source.id))));
    }
    // every input field, the unprovided ones `None`
    let mut all_fields = Vec::new();
    for i in &ir.inputs {
        let d = ir
            .decl(i.decl)
            .ok_or_else(|| EmitError(format!("input {:?} points at no declaration", i.decl)))?;
        let name = names::decl(d.id);
        let value = fields
            .iter()
            .find(|(n, _)| *n == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(Expr::none);
        all_fields.push((name, value));
    }
    let tail = Expr::call("Ok", [Expr::strukt("crate::Inputs", all_fields)]);
    Ok(Function {
        doc: vec![
            "The core's inputs from this tick's raw readings, one per provider in input-slot order, before the core steps.".into(),
            "A Source no provider reads stays `None` (a simulation supplies it; firmware never has one).".into(),
        ],
        attrs: vec![],
        is_async: false,
        public: true,
        name: "provide".into(),
        params,
        ret: Some(Type::path("Result<crate::Inputs, RuntimeError>")),
        body: Block::new(stmts, Some(tail)),
    })
}

/// `read(source, …) -> Result<crate::Inputs, RuntimeError>`: take every
/// reading from its source, in input-slot order, then `provide`.
fn read_fn(plan: &AdapterPlan) -> Function {
    let mut params = Vec::new();
    let mut args = Vec::new();
    for b in &plan.providers {
        let sym = names::reading(b.device);
        params.push((
            sym.clone(),
            Type::path(format!("&mut dyn {}", b.kind.trait_name())),
        ));
        let read = match b.kind {
            SourceKind::LevelPullDown | SourceKind::LevelPullUp => "read_level",
        };
        args.push(Expr::some(Expr::call(read, [Expr::path(sym)])));
    }
    Function {
        doc: vec![
            "Observe every source once, in input-slot order, and provide the core's inputs from the readings.".into(),
        ],
        attrs: vec![],
        is_async: false,
        public: true,
        name: "read".into(),
        params,
        ret: Some(Type::path("Result<crate::Inputs, RuntimeError>")),
        body: Block::expr(Expr::call("provide", args)),
    }
}

/// The host bridge's `inputs_from_readings`: `provide` over the request's
/// raw readings, written over the request's inputs for the provided
/// Sources only.
pub fn host_inputs_from_readings(
    ir: &ExecIr,
    plan: Option<&AdapterPlan>,
) -> Result<Function, EmitError> {
    let body = match plan {
        None => Block::expr(Expr::call("Ok", [Expr::Tuple(vec![])])),
        Some(plan) => {
            let mut stmts = Vec::new();
            let mut args = Vec::new();
            for (i, b) in plan.providers.iter().enumerate() {
                let p = ir
                    .providers
                    .iter()
                    .find(|p| p.slot == b.slot)
                    .ok_or_else(|| {
                        EmitError(format!("provider slot {:?} not in the exec IR", b.slot))
                    })?;
                let conv = match &p.raw {
                    bdl_ir::Ty::Bool => "boolean",
                    bdl_ir::Ty::Nat => "nat",
                    bdl_ir::Ty::Q { .. } => "quantity",
                    other => {
                        return Err(EmitError(format!(
                            "provider {:?} reads {other:?}, which the host cannot pass",
                            b.slot
                        )))
                    }
                };
                args.push(Expr::Match {
                    scrutinee: Box::new(Expr::method(
                        Expr::path("readings"),
                        "get",
                        [Expr::Lit(Lit::Usize(i))],
                    )),
                    arms: vec![
                        (
                            "Some(Some(v))".into(),
                            Expr::some(Expr::try_(Expr::method(
                                Expr::path("v"),
                                conv,
                                [Expr::Lit(Lit::Usize(i))],
                            ))),
                        ),
                        ("_".into(), Expr::none()),
                    ],
                });
            }
            stmts.push(Stmt::Let {
                name: "provided".into(),
                mutable: false,
                ty: None,
                value: Expr::try_(Expr::call("design::adapter::provide", args)),
            });
            for b in &plan.providers {
                let p = ir
                    .providers
                    .iter()
                    .find(|p| p.slot == b.slot)
                    .ok_or_else(|| {
                        EmitError(format!("provider slot {:?} not in the exec IR", b.slot))
                    })?;
                let source = ir.decl(p.decl).ok_or_else(|| {
                    EmitError(format!("provider {:?} points at no declaration", p.decl))
                })?;
                let field = names::decl(source.id);
                stmts.push(Stmt::Expr(Expr::if_(
                    Expr::method(
                        Expr::field(Expr::path("provided"), field.clone()),
                        "is_some",
                        [],
                    ),
                    Block::new(
                        vec![Stmt::Expr(Expr::assign(
                            Expr::field(Expr::path("inputs"), field.clone()),
                            Expr::field(Expr::path("provided"), field),
                        ))],
                        None,
                    ),
                    None,
                )));
            }
            Block::new(stmts, Some(Expr::call("Ok", [Expr::Tuple(vec![])])))
        }
    };
    Ok(Function {
        doc: vec![],
        attrs: vec![],
        is_async: false,
        public: false,
        name: "inputs_from_readings".into(),
        params: vec![
            ("readings".into(), Type::path("&[Option<DynValue>]")),
            ("inputs".into(), Type::path("&mut design::Inputs")),
        ],
        ret: Some(Type::path("Result<(), ReadingError>")),
        body,
    })
}

/// The output's display name for a comment; never a symbol.
pub(crate) fn output_name(ir: &ExecIr, id: bdl_model::OutputId) -> String {
    ir.outputs
        .iter()
        .find(|o| o.id == id)
        .map(|o| o.name.clone())
        .unwrap_or_else(|| id.to_string())
}

fn applied_ty(kind: SinkKind) -> Type {
    match kind {
        SinkKind::PwmDuty8 => Type::path("Result<u8, CommandFault>"),
        SinkKind::Level => Type::path("bool"),
    }
}

/// The host bridge's `adapter_ops`: the same `apply` over recording sinks.
pub fn host_adapter_ops(plan: Option<&AdapterPlan>) -> Function {
    let body = match plan {
        None => Block::expr(Expr::VecMacro(Vec::new())),
        Some(plan) => {
            let mut stmts = Vec::new();
            let mut args = vec![Expr::path("tick")];
            for s in &plan.sinks {
                let sym = names::command(s.device);
                let mock = match s.kind {
                    SinkKind::PwmDuty8 => "bdl_runtime_host::mock::MockPwm::default",
                    SinkKind::Level => "bdl_runtime_host::mock::MockLine::default",
                };
                stmts.push(Stmt::Let {
                    name: sym.clone(),
                    mutable: true,
                    ty: None,
                    value: Expr::call(mock, []),
                });
                args.push(Expr::Ref {
                    mutable: true,
                    e: Box::new(Expr::path(sym)),
                });
            }
            stmts.push(Stmt::Let {
                name: "applied".into(),
                mutable: false,
                ty: None,
                value: Expr::call("design::adapter::apply", args),
            });
            let ops: Vec<Expr> = plan
                .sinks
                .iter()
                .map(|s| {
                    let ctor = match s.kind {
                        SinkKind::PwmDuty8 => "AdapterOp::duty",
                        SinkKind::Level => "AdapterOp::level",
                    };
                    Expr::call(
                        ctor,
                        [
                            Expr::u64(s.device.raw()),
                            Expr::field(Expr::path("applied"), names::command(s.device)),
                        ],
                    )
                })
                .collect();
            Block::new(stmts, Some(Expr::VecMacro(ops)))
        }
    };
    Function {
        doc: vec![],
        attrs: vec![],
        is_async: false,
        public: false,
        name: "adapter_ops".into(),
        params: vec![("tick".into(), Type::path("&design::Tick"))],
        ret: Some(Type::path("Vec<AdapterOp>")),
        body,
    }
}
