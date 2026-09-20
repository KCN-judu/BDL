//! The Arduino family over `avr-hal` (docs/architecture/embedded-adapter.md
//! § Arduino): the base entry every Arduino board shares — the pin naming
//! (`D3` ⇒ `pins.d3`, `A4` ⇒ `pins.a4`), the PWM timers behind the pins,
//! the blocking tick loop, the halt — and one [`Board`] per board that
//! says which pins exist, which of them a timer drives, and which
//! `arduino-hal` feature and CPU the board is.  The Nano is the first
//! board; another Arduino adds a [`Board`] and nothing else.
//!
//! No Embassy here: `avr-hal` is a synchronous HAL and the AVR has no
//! Embassy time driver, so the firmware's tick is a blocking wait after
//! each step (`bdl_runtime_arduino::tick_wait`).  The behavior semantics
//! are the same one global `step` (ADR-0004); only the timer is coarser.

use crate::adapter::{AdapterPlan, SinkBinding, SinkKind};
use crate::ast::*;
use crate::emit::EmitError;
use crate::names;
use crate::targets::Peripheral;
use crate::CodegenOptions;
use bdl_exec_ir::ExecIr;

pub const FAMILY: &str = "avr";
/// `avr-none` with `-C target-cpu=<mcu>` (rustc's tier-3 AVR target).
pub const TRIPLE: &str = "avr-none";
/// The nightly `avr-hal` pins (`rust-toolchain.toml` of Rahix/avr-hal at
/// [`AVR_HAL_REV`]); AVR code generation is nightly-only.
pub const TOOLCHAIN: &str = "nightly-2025-04-27";
/// That nightly is a 1.88 pre-release: the crate declares 1.87 so the
/// resolver's fallback keeps every dependency buildable on it.
pub const RUST_VERSION: &str = "1.87";
/// `arduino-hal` is not on crates.io; the firmware depends on this commit.
pub const AVR_HAL_REV: &str = "e0b0105b11a7c4209fb1704276a7921c3139d5cb";

/// The timer/counter behind a PWM pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Timer {
    Tc0,
    Tc1,
    Tc2,
}

impl Timer {
    fn index(self) -> u32 {
        match self {
            Timer::Tc0 => 0,
            Timer::Tc1 => 1,
            Timer::Tc2 => 2,
        }
    }
    /// The `avr-hal` PWM driver type and the peripheral it takes.
    fn pwm_type(self) -> &'static str {
        match self {
            Timer::Tc0 => "Timer0Pwm",
            Timer::Tc1 => "Timer1Pwm",
            Timer::Tc2 => "Timer2Pwm",
        }
    }
    fn peripheral(self) -> &'static str {
        match self {
            Timer::Tc0 => "TC0",
            Timer::Tc1 => "TC1",
            Timer::Tc2 => "TC2",
        }
    }
    fn local(self) -> String {
        format!("timer{}", self.index())
    }
}

/// One Arduino board: what the base entry needs to know about it.
#[derive(Debug, PartialEq, Eq)]
pub struct Board {
    /// The board id (`arduino_nano`), also the Cargo feature and the
    /// binary suffix.
    pub id: &'static str,
    /// The `arduino-hal` board feature.
    pub hal_feature: &'static str,
    /// `-C target-cpu`.
    pub cpu: &'static str,
    /// Digital pins `D0` … `D<n>` the HAL exposes as `pins.d<n>`.
    pub digital_pins: u32,
    /// Analog pins `A0` … `A<n>` that are also digital lines
    /// (`pins.a<n>`); the Nano's A6/A7 are analog only.
    pub analog_digital_pins: u32,
    /// The PWM pins and the timer behind each, as the datasheet gives them.
    pub pwm: &'static [(&'static str, Timer)],
}

/// The Arduino Nano (ATmega328P at 16 MHz): D0–D13, A0–A5 as lines, PWM on
/// D3/D11 (timer 2), D5/D6 (timer 0), D9/D10 (timer 1) — the board file's
/// units.
pub static NANO: Board = Board {
    id: "arduino_nano",
    hal_feature: "arduino-nano",
    cpu: "atmega328p",
    digital_pins: 14,
    analog_digital_pins: 6,
    pwm: &[
        ("D3", Timer::Tc2),
        ("D5", Timer::Tc0),
        ("D6", Timer::Tc0),
        ("D9", Timer::Tc1),
        ("D10", Timer::Tc1),
        ("D11", Timer::Tc2),
    ],
};

/// The board of the family by id.
pub fn board(id: &str) -> Option<&'static Board> {
    [&NANO].into_iter().find(|b| b.id == id)
}

/// `D3` ⇒ `d3`, `A4` ⇒ `a4`: the HAL's field of `arduino_hal::pins!`.
fn pin_field(board: &Board, resource: &str) -> Result<String, EmitError> {
    let digital = resource
        .strip_prefix('D')
        .and_then(|n| n.parse::<u32>().ok())
        .filter(|n| *n < board.digital_pins)
        .map(|n| format!("d{n}"));
    let analog = resource
        .strip_prefix('A')
        .and_then(|n| n.parse::<u32>().ok())
        .filter(|n| *n < board.analog_digital_pins)
        .map(|n| format!("a{n}"));
    digital.or(analog).ok_or_else(|| {
        EmitError(format!(
            "{}: `{resource}` is not a line of this board (expected `D0` … `D{}` or `A0` … `A{}`)",
            board.id,
            board.digital_pins - 1,
            board.analog_digital_pins - 1
        ))
    })
}

/// The peripheral for one binding: the pin's PWM channel on its timer, or
/// the pin as an output line.  The board file's unit must be the timer the
/// board table gives the pin.
pub fn peripheral(board: &Board, b: &SinkBinding) -> Result<Peripheral, EmitError> {
    let field = pin_field(board, &b.resource)?;
    match b.kind {
        SinkKind::PwmDuty8 => {
            let timer = board
                .pwm
                .iter()
                .find(|(pin, _)| *pin == b.resource)
                .map(|(_, t)| *t)
                .ok_or_else(|| {
                    EmitError(format!(
                        "{}: `{}` has no PWM timer on this board",
                        board.id, b.resource
                    ))
                })?;
            if let Some(unit) = b.unit {
                if unit != timer.index() {
                    return Err(EmitError(format!(
                        "{}: the board file puts `{}` on timer {unit} but the pad is on timer {}",
                        board.id,
                        b.resource,
                        timer.index()
                    )));
                }
            }
            Ok(Peripheral {
                // pins.d3.into_output().into_pwm(&timer2)
                construct: Expr::method(
                    Expr::method(
                        Expr::field(Expr::path("pins"), field.clone()),
                        "into_output",
                        [],
                    ),
                    "into_pwm",
                    [Expr::Ref {
                        mutable: false,
                        e: Box::new(Expr::path(timer.local())),
                    }],
                ),
                describe: format!(
                    "pins.{field} on {} (OC{}x)",
                    timer.pwm_type(),
                    timer.index()
                ),
            })
        }
        SinkKind::Level => Ok(Peripheral {
            construct: Expr::method(
                Expr::field(Expr::path("pins"), field.clone()),
                "into_output",
                [],
            ),
            describe: format!("pins.{field} as a digital output"),
        }),
    }
}

/// `src/bin/<board>.rs`: the blocking tick loop over `avr-hal`.
pub fn firmware_module(
    board: &Board,
    ir: &ExecIr,
    package: &str,
    plan: &AdapterPlan,
    generator: &str,
) -> Result<Module, EmitError> {
    if plan.family != FAMILY || plan.board != board.id {
        return Err(EmitError(format!(
            "{}: the plan is for `{}` of family `{}`",
            board.id, plan.board, plan.family
        )));
    }
    if plan.arena_bytes.is_some() {
        return Err(EmitError(format!(
            "{}: an AVR board carries no collection arena",
            board.id
        )));
    }
    let mut items = vec![
        Item::Use(format!("{package} as design")),
        Item::Use("bdl_runtime_adapter::schedule".into()),
        Item::Use("bdl_runtime_arduino::halt".into()),
        Item::Use("bdl_runtime_arduino::tick_wait".into()),
        Item::Use("bdl_runtime_arduino::PwmLine".into()),
        Item::Use("bdl_runtime_arduino::Line".into()),
        Item::Use("arduino_hal::simple_pwm::Prescaler".into()),
        Item::Use("arduino_hal::simple_pwm::IntoPwmPin".into()),
        Item::Use("panic_halt as _".into()),
    ];
    // the timers the plan's PWM sinks use, in timer order
    let mut timers: Vec<Timer> = Vec::new();
    for b in &plan.sinks {
        if b.kind == SinkKind::PwmDuty8 {
            let t = board
                .pwm
                .iter()
                .find(|(pin, _)| *pin == b.resource)
                .map(|(_, t)| *t)
                .ok_or_else(|| {
                    EmitError(format!("{}: `{}` has no PWM timer", board.id, b.resource))
                })?;
            if !timers.contains(&t) {
                timers.push(t);
            }
        }
    }
    timers.sort();
    for t in &timers {
        items.push(Item::Use(format!(
            "arduino_hal::simple_pwm::{}",
            t.pwm_type()
        )));
    }
    items.push(Item::Const {
        doc: vec![format!(
            "The base tick the loop waits for after each step (`bdld compile --tick-micros`): {} µs; the wait is blocking, so a tick lasts at least this long.",
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

    let mut stmts = Vec::new();
    stmts.push(Stmt::Let {
        name: "dp".into(),
        mutable: false,
        ty: None,
        value: Expr::Match {
            scrutinee: Box::new(Expr::call("arduino_hal::Peripherals::take", [])),
            arms: vec![
                ("Some(dp)".into(), Expr::path("dp")),
                ("None".into(), Expr::call("halt", [])),
            ],
        },
    });
    stmts.push(Stmt::Let {
        name: "pins".into(),
        mutable: false,
        ty: None,
        value: Expr::path("arduino_hal::pins!(dp)"),
    });
    for t in &timers {
        stmts.push(Stmt::Comment(format!(
            "{}: the PWM carrier, F_CPU / (64 · 256) ≈ 977 Hz at 16 MHz — configuration, not a clock domain",
            t.pwm_type()
        )));
        stmts.push(Stmt::Let {
            name: t.local(),
            mutable: false,
            ty: None,
            value: Expr::call(
                format!("{}::new", t.pwm_type()),
                [
                    Expr::field(Expr::path("dp"), t.peripheral()),
                    Expr::path("Prescaler::Prescale64"),
                ],
            ),
        });
    }
    stmts.push(Stmt::Comment(
        "the sinks, on the lines the placement assigned; every line starts low".into(),
    ));
    let mut apply_args = vec![Expr::Ref {
        mutable: false,
        e: Box::new(Expr::path("t")),
    }];
    for b in &plan.sinks {
        let per = peripheral(board, b)?;
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
            b.resource, per.describe
        )));
        let wrapped = match b.kind {
            SinkKind::PwmDuty8 => Expr::call("PwmLine::new", [per.construct]),
            SinkKind::Level => Expr::call("Line::new", [per.construct]),
        };
        stmts.push(Stmt::Let {
            name: sym.clone(),
            mutable: true,
            ty: None,
            value: wrapped,
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
        name: "tick".into(),
        mutable: true,
        ty: Some(Type::path("u64")),
        value: Expr::u64(0),
    });
    let loop_body = Block::new(
        vec![
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
            Stmt::Expr(Expr::call("tick_wait", [Expr::path("TICK_MICROS")])),
        ],
        None,
    );
    stmts.push(Stmt::Expr(Expr::Loop(loop_body)));
    items.push(Item::Fn(Function {
        doc: vec![],
        attrs: vec!["arduino_hal::entry".into()],
        is_async: false,
        public: false,
        name: "main".into(),
        params: vec![],
        ret: Some(Type::path("!")),
        body: Block::new(stmts, None),
    }));
    Ok(Module {
        doc: vec![
            format!(
                "Generated by {generator} for design `{}` on `{}`. Do not edit.",
                ir.name, plan.board
            ),
            String::new(),
            "The Arduino firmware: a blocking loop steps the core, applies each tick's raw commands to".into(),
            "the lines the placement assigned, and waits `TICK_MICROS`; the clock domains the compiled".into(),
            "schedule says are due are activated by tick number.  Nothing here interprets the design".into(),
            "(docs/architecture/embedded-adapter.md § Arduino).".into(),
        ],
        inner_attrs: vec!["no_std".into(), "no_main".into()],
        items,
    })
}

/// `.cargo/config.toml`: the CPU for the `avr-none` target and the
/// MSRV-respecting resolver.  No default target and no `[unstable]` table:
/// the host build is untouched, and `-Zbuild-std=core` is the build
/// command's.
pub const CARGO_CONFIG: &str = "# Generated. Do not edit.\n[resolver]\nincompatible-rust-versions = \"fallback\"\n\n[target.avr-none]\nrustflags = [\"-C\", \"target-cpu=atmega328p\"]\n";

/// The `[features]` line and the `[dependencies]` lines of the board's
/// feature.
pub fn cargo_sections(board: &Board, o: &CodegenOptions) -> (String, String) {
    let feature = format!(
        "# The {} firmware over avr-hal (nightly `{}`, `-Zbuild-std=core`, avr-gcc as the linker).\n{} = [\"adapter\", \"dep:bdl-runtime-arduino\", \"dep:arduino-hal\", \"dep:panic-halt\"]\n",
        board.id, TOOLCHAIN, board.id
    );
    let deps = format!(
        "bdl-runtime-arduino = {{ path = {arduino:?}, optional = true }}\n\
         arduino-hal = {{ git = \"https://github.com/Rahix/avr-hal\", rev = \"{rev}\", features = [\"{hal}\"], optional = true }}\n\
         panic-halt = {{ version = \"1.0.0\", optional = true }}\n",
        arduino = o.runtime_arduino_path,
        rev = AVR_HAL_REV,
        hal = board.hal_feature,
    );
    (feature, deps)
}

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
    fn nano_pins_map_to_hal_fields_and_timers_by_the_board_table() {
        let p = peripheral(&NANO, &binding("D3", SinkKind::PwmDuty8, Some(2))).unwrap();
        assert_eq!(p.describe, "pins.d3 on Timer2Pwm (OC2x)");
        assert_eq!(
            crate::print::expr_str(&p.construct),
            "pins.d3.into_output().into_pwm(&timer2)"
        );
        let p = peripheral(&NANO, &binding("D9", SinkKind::PwmDuty8, None)).unwrap();
        assert_eq!(p.describe, "pins.d9 on Timer1Pwm (OC1x)");
        let p = peripheral(&NANO, &binding("A4", SinkKind::Level, None)).unwrap();
        assert_eq!(
            crate::print::expr_str(&p.construct),
            "pins.a4.into_output()"
        );
        let p = peripheral(&NANO, &binding("D13", SinkKind::Level, None)).unwrap();
        assert_eq!(p.describe, "pins.d13 as a digital output");
    }

    #[test]
    fn a_line_without_a_timer_a_foreign_pin_or_a_disagreeing_unit_is_an_error() {
        assert!(peripheral(&NANO, &binding("D4", SinkKind::PwmDuty8, None)).is_err());
        assert!(peripheral(&NANO, &binding("A6", SinkKind::Level, None)).is_err());
        assert!(peripheral(&NANO, &binding("GP15", SinkKind::Level, None)).is_err());
        assert!(peripheral(&NANO, &binding("D14", SinkKind::Level, None)).is_err());
        let e = peripheral(&NANO, &binding("D3", SinkKind::PwmDuty8, Some(0))).unwrap_err();
        assert!(e.0.contains("timer 0"), "{}", e.0);
        assert!(board("arduino_uno").is_none());
        assert_eq!(board("arduino_nano").map(|b| b.cpu), Some("atmega328p"));
    }
}
