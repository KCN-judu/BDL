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
use crate::names;
use bdl_exec_ir::{ExecIr, SinkSlot};
use bdl_model::{DeviceId, OutputProfileId};

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
    /// The collection arena in bytes when the core carries lists.
    pub arena_bytes: Option<u64>,
}

/// `src/adapter.rs`.
pub fn adapter_module(ir: &ExecIr, plan: &AdapterPlan, generator: &str) -> Module {
    let mut items = vec![
        Item::Use("bdl_runtime_adapter::apply_duty8".into()),
        Item::Use("bdl_runtime_adapter::apply_level".into()),
        Item::Use("bdl_runtime_adapter::CommandFault".into()),
        Item::Use("bdl_runtime_adapter::Level".into()),
        Item::Use("bdl_runtime_adapter::PwmDuty8".into()),
        Item::Use("bdl_runtime_adapter::SinkBinding".into()),
    ];
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
    Module {
        doc: vec![
            format!(
                "Generated by {generator} for design `{}` on `{}`. Do not edit.",
                ir.name, plan.board
            ),
            String::new(),
            "The platform adapter's target-independent glue: each machine sink's raw command".into(),
            "(`Tick.commands`) applied to one sink through `bdl-runtime-adapter`'s numeric policy.".into(),
            "Behavior ends at the command; physical effect begins in the sink (docs/architecture/embedded-adapter.md).".into(),
        ],
        inner_attrs: vec![
            "allow(unused_imports, unused_variables, clippy::all)".into(),
        ],
        items,
    }
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
