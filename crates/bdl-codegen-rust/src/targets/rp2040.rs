//! The RP2040 (Raspberry Pi Pico) target entry: `src/bin/rp2040.rs` over
//! Embassy, the linker inputs, and the resource → peripheral derivation
//! (docs/architecture/embedded-adapter.md § RP2040).
//!
//! A board resource `GPn` is the pad `PIN_n`; its PWM channel is slice
//! `(n / 2) % 8`, channel A on even and B on odd pads — the RP2040
//! datasheet's table, checked against the board file's unit when it names
//! one.  Nothing else is derived: a resource that is not `GPn`, or that the
//! plan binds to a capability the target cannot serve on that pad, is an
//! error, never a substitute pin.

use crate::adapter::{AdapterPlan, SinkBinding, SinkKind};
use crate::ast::*;
use crate::emit::EmitError;
use crate::names;
use bdl_exec_ir::ExecIr;

pub const FAMILY: &str = "rp2040";
pub const TRIPLE: &str = "thumbv6m-none-eabi";

/// The Pico's pads.
const MAX_PIN: u32 = 29;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Channel {
    A,
    B,
}

/// The peripheral a sink lands on.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Peripheral {
    Pwm {
        pin: u32,
        slice: u32,
        channel: Channel,
    },
    Gpio {
        pin: u32,
    },
}

impl Peripheral {
    /// The constructor expression over `p` (`embassy_rp::init`'s
    /// `Peripherals`).
    fn construct(&self) -> Expr {
        match self {
            Peripheral::Pwm {
                pin,
                slice,
                channel,
            } => Expr::call(
                match channel {
                    Channel::A => "PwmA::new",
                    Channel::B => "PwmB::new",
                },
                [
                    Expr::field(Expr::path("p"), format!("PWM_SLICE{slice}")),
                    Expr::field(Expr::path("p"), format!("PIN_{pin}")),
                    Expr::path("DEFAULT_PWM_DIVIDER"),
                ],
            ),
            Peripheral::Gpio { pin } => Expr::call(
                "Line::new",
                [Expr::field(Expr::path("p"), format!("PIN_{pin}"))],
            ),
        }
    }
    /// The manifest's word for it.
    pub fn describe(&self) -> String {
        match self {
            Peripheral::Pwm {
                pin,
                slice,
                channel,
            } => format!(
                "PIN_{pin} on PWM_SLICE{slice} channel {}",
                match channel {
                    Channel::A => "A",
                    Channel::B => "B",
                }
            ),
            Peripheral::Gpio { pin } => format!("PIN_{pin} as a digital output"),
        }
    }
}

fn pin_number(resource: &str) -> Result<u32, EmitError> {
    resource
        .strip_prefix("GP")
        .and_then(|n| n.parse::<u32>().ok())
        .filter(|n| *n <= MAX_PIN)
        .ok_or_else(|| {
            EmitError(format!(
                "rp2040: `{resource}` is not a pad of this target (expected `GP0` … `GP{MAX_PIN}`)"
            ))
        })
}

/// The peripheral for one binding; the board's unit, when given, must be
/// the slice the datasheet gives the pad.
pub fn peripheral(b: &SinkBinding) -> Result<Peripheral, EmitError> {
    let pin = pin_number(&b.resource)?;
    Ok(match b.kind {
        SinkKind::PwmDuty8 => {
            let slice = (pin / 2) % 8;
            if let Some(unit) = b.unit {
                if unit != slice {
                    return Err(EmitError(format!(
                        "rp2040: the board file puts `{}` on PWM slice {unit} but the pad is on slice {slice}",
                        b.resource
                    )));
                }
            }
            Peripheral::Pwm {
                pin,
                slice,
                channel: if pin % 2 == 0 { Channel::A } else { Channel::B },
            }
        }
        SinkKind::Level => Peripheral::Gpio { pin },
    })
}

/// `src/bin/rp2040.rs`.
pub fn firmware_module(
    ir: &ExecIr,
    package: &str,
    plan: &AdapterPlan,
    generator: &str,
) -> Result<Module, EmitError> {
    if plan.family != FAMILY {
        return Err(EmitError(format!(
            "rp2040: the plan is for family `{}`",
            plan.family
        )));
    }
    let mut items = vec![
        Item::Use(format!("{package} as design")),
        Item::Use("bdl_runtime_embassy::schedule".into()),
        Item::Use("bdl_runtime_embassy_rp::halt".into()),
        Item::Use("bdl_runtime_embassy_rp::Line".into()),
        Item::Use("bdl_runtime_embassy_rp::PwmA".into()),
        Item::Use("bdl_runtime_embassy_rp::PwmB".into()),
        Item::Use("bdl_runtime_embassy_rp::DEFAULT_PWM_DIVIDER".into()),
        Item::Use("embassy_executor::Spawner".into()),
        Item::Use("embassy_time::Duration".into()),
        Item::Use("embassy_time::Ticker".into()),
        Item::Use("panic_halt as _".into()),
    ];
    if plan.arena_bytes.is_some() {
        items.push(Item::Use("bdl_runtime_embassy_rp::arena::Heap".into()));
        items.push(Item::Use("static_cell::StaticCell".into()));
    }
    items.push(Item::Const {
        doc: vec![format!(
            "The base tick the timer fires at (`bdld compile --tick-micros`): {} µs.",
            plan.tick_micros
        )],
        name: "TICK_MICROS".into(),
        ty: Type::path("u64"),
        value: Expr::u64(plan.tick_micros),
    });
    items.push(Item::Const {
        doc: vec![
            "Activation period of each clock slot in ticks (`bdld compile --period`), in slot order;".into(),
            "the same rule the simulator applies (`Schedule::active_at`).".into(),
        ],
        name: "PERIODS".into(),
        ty: Type::path(format!("[u64; {}]", plan.periods.len())),
        value: Expr::Array(plan.periods.iter().map(|p| Expr::u64(*p)).collect()),
    });
    if let Some(bytes) = plan.arena_bytes {
        items.push(Item::Static {
            doc: vec![
                "The collection arena's allocator (docs/spec/deployment-capacity.md § 6).".into(),
            ],
            attrs: vec!["global_allocator".into()],
            name: "HEAP".into(),
            ty: Type::path("Heap"),
            value: Expr::call("Heap::empty", []),
        });
        items.push(Item::Static {
            doc: vec![format!(
                "The arena: {bytes} bytes, the manifest's `state_bytes_max + tick_bytes_max` rounded up."
            )],
            attrs: vec![],
            name: "ARENA".into(),
            ty: Type::path(format!("StaticCell<[u8; {bytes}]>")),
            value: Expr::call("StaticCell::new", []),
        });
    }

    let mut stmts = Vec::new();
    stmts.push(Stmt::Let {
        name: "p".into(),
        mutable: false,
        ty: None,
        value: Expr::call("embassy_rp::init", [Expr::call("Default::default", [])]),
    });
    if let Some(bytes) = plan.arena_bytes {
        stmts.push(Stmt::Expr(Expr::call(
            "bdl_runtime_embassy_rp::arena::init",
            [
                Expr::Ref {
                    mutable: false,
                    e: Box::new(Expr::path("HEAP")),
                },
                Expr::method(
                    Expr::path("ARENA"),
                    "init",
                    [Expr::path(format!("[0_u8; {bytes}]"))],
                ),
            ],
        )));
    }
    stmts.push(Stmt::Comment(
        "the sinks, on the resources the placement assigned; every line starts low".into(),
    ));
    let mut apply_args = vec![Expr::Ref {
        mutable: false,
        e: Box::new(Expr::path("t")),
    }];
    for b in &plan.sinks {
        let per = peripheral(b)?;
        let sym = names::command(b.device);
        let sink = ir
            .sinks
            .iter()
            .find(|x| x.device == b.device)
            .map(|x| {
                format!(
                    "{} realises {} as `{}`",
                    x.device_name,
                    crate::adapter::output_name(ir, x.output),
                    x.profile
                )
            })
            .unwrap_or_default();
        stmts.push(Stmt::Comment(format!(
            "{sink} on {}: {}",
            b.resource,
            per.describe()
        )));
        stmts.push(Stmt::Let {
            name: sym.clone(),
            mutable: true,
            ty: None,
            value: per.construct(),
        });
        apply_args.push(Expr::Ref {
            mutable: true,
            e: Box::new(Expr::path(sym)),
        });
    }
    stmts.push(Stmt::Let {
        name: "state".into(),
        mutable: true,
        ty: None,
        value: Expr::call("design::init", []),
    });
    stmts.push(Stmt::Comment(
        "no device provides a value yet (ISS-0016): a due input would fault the tick".into(),
    ));
    stmts.push(Stmt::Let {
        name: "inputs".into(),
        mutable: false,
        ty: None,
        value: Expr::call("design::Inputs::default", []),
    });
    stmts.push(Stmt::Let {
        name: "ticker".into(),
        mutable: true,
        ty: None,
        value: Expr::call(
            "Ticker::every",
            [Expr::call(
                "Duration::from_micros",
                [Expr::path("TICK_MICROS")],
            )],
        ),
    });
    stmts.push(Stmt::Let {
        name: "tick".into(),
        mutable: true,
        ty: Some(Type::path("u64")),
        value: Expr::u64(0),
    });
    let loop_body = Block::new(
        vec![
            Stmt::Expr(Expr::Await(Box::new(Expr::method(
                Expr::path("ticker"),
                "next",
                [],
            )))),
            Stmt::Let {
                name: "active".into(),
                mutable: false,
                ty: None,
                value: Expr::call(
                    "schedule::active",
                    [
                        Expr::path("tick"),
                        Expr::Ref {
                            mutable: false,
                            e: Box::new(Expr::path("PERIODS")),
                        },
                    ],
                ),
            },
            Stmt::Comment(
                "step, then apply the commands in sink order; a failed tick latches the fault"
                    .into(),
            ),
            Stmt::Expr(Expr::Match {
                scrutinee: Box::new(Expr::call(
                    "design::step",
                    [
                        Expr::Ref {
                            mutable: true,
                            e: Box::new(Expr::path("state")),
                        },
                        Expr::path("active"),
                        Expr::Ref {
                            mutable: false,
                            e: Box::new(Expr::path("inputs")),
                        },
                    ],
                )),
                arms: vec![
                    (
                        "Ok(t)".into(),
                        Expr::Block(Block::new(
                            vec![Stmt::Let {
                                name: "_".into(),
                                mutable: false,
                                ty: None,
                                value: Expr::call("design::adapter::apply", apply_args),
                            }],
                            None,
                        )),
                    ),
                    ("Err(_)".into(), Expr::call("halt", [])),
                ],
            }),
            Stmt::Expr(Expr::assign(
                Expr::path("tick"),
                Expr::method(Expr::path("tick"), "wrapping_add", [Expr::u64(1)]),
            )),
        ],
        None,
    );
    stmts.push(Stmt::Expr(Expr::Loop(loop_body)));
    items.push(Item::Fn(Function {
        doc: vec![],
        attrs: vec!["embassy_executor::main".into()],
        is_async: true,
        public: false,
        name: "main".into(),
        params: vec![("_spawner".into(), Type::path("Spawner"))],
        ret: None,
        body: Block::new(stmts, None),
    }));
    Ok(Module {
        doc: vec![
            format!(
                "Generated by {generator} for design `{}` on `{}`. Do not edit.",
                ir.name, plan.board
            ),
            String::new(),
            "The RP2040 firmware: one Embassy task ticks the core at `TICK_MICROS`, activates the".into(),
            "clock domains the compiled schedule says are due, and applies each tick's raw commands to".into(),
            "the peripherals the placement assigned.  Nothing here interprets the design".into(),
            "(docs/architecture/embedded-adapter.md).".into(),
        ],
        inner_attrs: vec!["no_std".into(), "no_main".into()],
        items,
    })
}

/// `memory.x` for the Pico's 2 MiB flash and 264 KiB SRAM; `BOOT2` is the
/// second-stage loader `embassy-rp` places.
pub const MEMORY_X: &str = "MEMORY {\n    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100\n    FLASH : ORIGIN = 0x10000100, LENGTH = 2048K - 0x100\n    RAM   : ORIGIN = 0x20000000, LENGTH = 264K\n}\n";

/// `build.rs`: hands `memory.x` to the linker.
pub const BUILD_RS: &str = "// Generated. Hands memory.x to the linker for the embedded target; a no-op for the host.\nuse std::{env, fs, path::PathBuf};\n\nfn main() {\n    let out = PathBuf::from(env::var_os(\"OUT_DIR\").expect(\"OUT_DIR\"));\n    fs::copy(\"memory.x\", out.join(\"memory.x\")).expect(\"memory.x\");\n    println!(\"cargo:rustc-link-search={}\", out.display());\n    println!(\"cargo:rerun-if-changed=memory.x\");\n    println!(\"cargo:rerun-if-changed=build.rs\");\n}\n";

/// `.cargo/config.toml`: the linker inputs for the embedded triple and the
/// MSRV-respecting resolver (the HAL's newest dependencies may need a newer
/// compiler than the toolchain pins).  The host build is untouched: no
/// default target is set.
pub const CARGO_CONFIG: &str = "# Generated. Do not edit.\n[resolver]\nincompatible-rust-versions = \"fallback\"\n\n[target.thumbv6m-none-eabi]\nrustflags = [\"-C\", \"link-arg=--nmagic\", \"-C\", \"link-arg=-Tlink.x\", \"-C\", \"link-arg=-Tlink-rp.x\"]\n";

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_exec_ir::SinkSlot;
    use bdl_model::{DeviceId, OutputProfileId};

    fn binding(resource: &str, kind: SinkKind, unit: Option<u32>) -> SinkBinding {
        SinkBinding {
            slot: SinkSlot(0),
            device: DeviceId::from_raw(7),
            profile: OutputProfileId("pwm_duty8".into()),
            kind,
            resource: resource.into(),
            unit,
        }
    }

    #[test]
    fn pads_map_to_slices_and_channels_by_the_datasheet() {
        assert_eq!(
            peripheral(&binding("GP15", SinkKind::PwmDuty8, Some(7))).unwrap(),
            Peripheral::Pwm {
                pin: 15,
                slice: 7,
                channel: Channel::B
            }
        );
        assert_eq!(
            peripheral(&binding("GP16", SinkKind::PwmDuty8, None)).unwrap(),
            Peripheral::Pwm {
                pin: 16,
                slice: 0,
                channel: Channel::A
            }
        );
        assert_eq!(
            peripheral(&binding("GP25", SinkKind::Level, None)).unwrap(),
            Peripheral::Gpio { pin: 25 }
        );
    }

    #[test]
    fn a_foreign_resource_or_a_disagreeing_board_file_is_an_error_not_a_fallback() {
        assert!(peripheral(&binding("D3", SinkKind::PwmDuty8, None)).is_err());
        assert!(peripheral(&binding("GP30", SinkKind::Level, None)).is_err());
        assert!(peripheral(&binding("GP", SinkKind::Level, None)).is_err());
        let e = peripheral(&binding("GP15", SinkKind::PwmDuty8, Some(3))).unwrap_err();
        assert!(e.0.contains("slice 3"), "{}", e.0);
    }
}
