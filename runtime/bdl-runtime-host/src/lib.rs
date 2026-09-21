//! The host side of a generated program.
//!
//! A generated crate's `host` binary implements [`HostProgram`] for its
//! core (generated code converts between the core's static types and
//! [`DynValue`]) and calls [`main_stdio`]: a [`RunRequest`] comes in as
//! JSON on stdin, a [`RunTrace`] goes out on stdout.  [`harness`] is the
//! other end — it writes a generated crate to disk, `cargo check`s /
//! builds it and runs the binary — for differential tests and tooling.
//!
//! This crate is `std`; nothing in it is linked into a semantic core.

#![forbid(unsafe_code)]

pub mod harness;

use bdl_runtime_core::{ActiveDomains, ClockSlot, RuntimeError};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

/// A runtime value in the reference evaluator's shape (dimensions are
/// static and not carried).  Concept identity is the raw `ConceptId`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DynValue {
    Bool {
        value: bool,
    },
    Nat {
        value: u64,
    },
    Quantity {
        value: f64,
    },
    Semantic {
        id: u64,
        repr: Box<DynValue>,
    },
    None,
    Some {
        value: Box<DynValue>,
    },
    List {
        items: Vec<DynValue>,
    },
    Pair {
        fst: Box<DynValue>,
        snd: Box<DynValue>,
    },
}

/// A value of the wrong shape reached a typed input slot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BridgeError {
    pub slot: usize,
    pub expected: String,
}

impl DynValue {
    pub fn quantity(&self, slot: usize) -> Result<f64, BridgeError> {
        match self {
            DynValue::Quantity { value } => Ok(*value),
            _ => Err(BridgeError {
                slot,
                expected: "quantity".into(),
            }),
        }
    }
    pub fn boolean(&self, slot: usize) -> Result<bool, BridgeError> {
        match self {
            DynValue::Bool { value } => Ok(*value),
            _ => Err(BridgeError {
                slot,
                expected: "bool".into(),
            }),
        }
    }
    pub fn nat(&self, slot: usize) -> Result<u64, BridgeError> {
        match self {
            DynValue::Nat { value } => Ok(*value),
            _ => Err(BridgeError {
                slot,
                expected: "nat".into(),
            }),
        }
    }
    pub fn semantic(&self, id: u64, slot: usize) -> Result<&DynValue, BridgeError> {
        match self {
            DynValue::Semantic { id: i, repr } if *i == id => Ok(repr),
            _ => Err(BridgeError {
                slot,
                expected: format!("sem#{id}"),
            }),
        }
    }
    pub fn option(&self, slot: usize) -> Result<Option<&DynValue>, BridgeError> {
        match self {
            DynValue::None => Ok(None),
            DynValue::Some { value } => Ok(Some(value)),
            _ => Err(BridgeError {
                slot,
                expected: "option".into(),
            }),
        }
    }
    pub fn list(&self, slot: usize) -> Result<&[DynValue], BridgeError> {
        match self {
            DynValue::List { items } => Ok(items),
            _ => Err(BridgeError {
                slot,
                expected: "list".into(),
            }),
        }
    }
    pub fn pair(&self, slot: usize) -> Result<(&DynValue, &DynValue), BridgeError> {
        match self {
            DynValue::Pair { fst, snd } => Ok((fst, snd)),
            _ => Err(BridgeError {
                slot,
                expected: "pair".into(),
            }),
        }
    }
    pub fn pair_of(fst: DynValue, snd: DynValue) -> DynValue {
        DynValue::Pair {
            fst: Box::new(fst),
            snd: Box::new(snd),
        }
    }
    pub fn sem(id: u64, repr: DynValue) -> DynValue {
        DynValue::Semantic {
            id,
            repr: Box::new(repr),
        }
    }
    pub fn some(v: DynValue) -> DynValue {
        DynValue::Some { value: Box::new(v) }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TickRequest {
    /// Active clock slots.
    pub active: Vec<u16>,
    /// One entry per input slot.
    pub inputs: Vec<Option<DynValue>>,
    /// One raw reading per provider, in provider order (the generated
    /// `adapter::provide`'s parameters): the input half of the adapter on
    /// the host.  A provided Source's value is made from its reading and
    /// overrides `inputs` for that slot; empty for a core without providers,
    /// or to supply every input directly.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub readings: Vec<Option<DynValue>>,
}

/// Why a reading could not become an input: the value's shape, or the
/// provider's term failing (`?` on both in generated bridges).
#[derive(Clone, Debug, PartialEq)]
pub enum ReadingError {
    Bridge(BridgeError),
    Runtime(RuntimeError),
}

impl From<BridgeError> for ReadingError {
    fn from(e: BridgeError) -> Self {
        ReadingError::Bridge(e)
    }
}

impl From<RuntimeError> for ReadingError {
    fn from(e: RuntimeError) -> Self {
        ReadingError::Runtime(e)
    }
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct RunRequest {
    pub ticks: Vec<TickRequest>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TickTrace {
    pub tick: u64,
    /// One entry per declaration in plan order; `None` when not due.
    pub values: Vec<Option<DynValue>>,
    /// One entry per output slot; `None` when the driver was not due.
    pub outputs: Vec<Option<DynValue>>,
    /// One entry per machine sink (a realised output's raw command);
    /// `None` when the driver was not due.  Absent from older traces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<Option<DynValue>>,
    /// What the generated adapter did with each command this tick, in
    /// sink order, through the recording sinks of [`mock`]; empty for a
    /// core generated without a target.  Absent from older traces.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub adapter: Vec<AdapterOp>,
}

/// One physical operation the platform adapter performed — or declined —
/// for one machine sink at one tick (docs/architecture/embedded-adapter.md).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum AdapterOp {
    /// The PWM line was set to this 8-bit duty.
    Pwm { device_id: u64, duty: u8 },
    /// The digital line was set high or low.
    Level { device_id: u64, high: bool },
    /// The command was refused by the numeric policy; the line holds.
    Refused { device_id: u64, fault: String },
    /// The driver was not due: no command, the line holds.
    Held { device_id: u64 },
}

/// Recording sinks: what a target would do, as data.  A generated host
/// applies each tick's commands to these through the same `adapter::apply`
/// the firmware uses, so `TickTrace.adapter` is the firmware's operation
/// sequence for the host's inputs.
pub mod mock {
    use bdl_runtime_adapter::{Level, PwmDuty8};

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct MockPwm {
        pub duty: Option<u8>,
    }
    impl PwmDuty8 for MockPwm {
        fn set_duty8(&mut self, duty: u8) {
            self.duty = Some(duty);
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    pub struct MockLine {
        pub high: Option<bool>,
    }
    impl Level for MockLine {
        fn set_level(&mut self, high: bool) {
            self.high = Some(high);
        }
    }
}

impl AdapterOp {
    /// The record of one applied duty command.
    pub fn duty(
        device_id: u64,
        r: Option<Result<u8, bdl_runtime_adapter::CommandFault>>,
    ) -> AdapterOp {
        match r {
            None => AdapterOp::Held { device_id },
            Some(Ok(duty)) => AdapterOp::Pwm { device_id, duty },
            Some(Err(fault)) => AdapterOp::Refused {
                device_id,
                fault: format!("{fault:?}"),
            },
        }
    }
    /// The record of one applied level command.
    pub fn level(device_id: u64, r: Option<bool>) -> AdapterOp {
        match r {
            None => AdapterOp::Held { device_id },
            Some(high) => AdapterOp::Level { device_id, high },
        }
    }
}

/// [`RuntimeError`] in serialisable form.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DynError {
    MissingInput {
        decl: u64,
    },
    DivisionByZero {
        decl: u64,
    },
    NonFinite {
        decl: u64,
        op: String,
    },
    NotEvaluated {
        decl: u64,
    },
    /// The request itself was malformed for this program.
    Bridge {
        slot: usize,
        expected: String,
    },
}

impl From<RuntimeError> for DynError {
    fn from(e: RuntimeError) -> DynError {
        match e {
            RuntimeError::MissingInput { decl } => DynError::MissingInput { decl },
            RuntimeError::DivisionByZero { decl } => DynError::DivisionByZero { decl },
            RuntimeError::NonFinite { decl, op } => DynError::NonFinite {
                decl,
                op: op.into(),
            },
            RuntimeError::NotEvaluated { decl } => DynError::NotEvaluated { decl },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceError {
    pub tick: u64,
    pub error: DynError,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct Metrics {
    /// `size_of::<State>()` of the generated core.
    pub state_bytes: usize,
    pub steps: u64,
    /// Wall time spent inside `step`, nanoseconds, all ticks.
    pub step_ns: u128,
}

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct RunTrace {
    pub ticks: Vec<TickTrace>,
    /// The first failure; the state is not advanced past it and no tick
    /// after it is recorded.
    pub error: Option<TraceError>,
    pub metrics: Metrics,
}

/// What a generated `host` binary provides.
pub trait HostProgram {
    type State;
    type Inputs;
    type Tick;
    fn init() -> Self::State;
    fn step(
        state: &mut Self::State,
        active: ActiveDomains,
        inputs: &Self::Inputs,
    ) -> Result<Self::Tick, RuntimeError>;
    fn inputs_from_dyn(slots: &[Option<DynValue>]) -> Result<Self::Inputs, BridgeError>;
    /// The provided Sources' inputs from the request's raw readings, over
    /// `inputs`; a no-op for a core without providers.
    fn inputs_from_readings(
        readings: &[Option<DynValue>],
        inputs: &mut Self::Inputs,
    ) -> Result<(), ReadingError>;
    fn values_to_dyn(tick: &Self::Tick) -> Vec<Option<DynValue>>;
    fn outputs_to_dyn(tick: &Self::Tick) -> Vec<Option<DynValue>>;
    /// The raw commands of the realised outputs; empty when none.
    fn commands_to_dyn(tick: &Self::Tick) -> Vec<Option<DynValue>>;
    /// The adapter's operations for this tick's commands over recording
    /// sinks; empty for a core generated without a target.
    fn adapter_ops(tick: &Self::Tick) -> Vec<AdapterOp>;
    fn state_bytes() -> usize;
}

pub fn active_domains(slots: &[u16]) -> ActiveDomains {
    slots
        .iter()
        .fold(ActiveDomains::none(), |a, s| a.with(ClockSlot(*s)))
}

/// Run a request to completion or first error.
pub fn run<P: HostProgram>(req: &RunRequest) -> RunTrace {
    let mut state = P::init();
    let mut trace = RunTrace {
        metrics: Metrics {
            state_bytes: P::state_bytes(),
            ..Metrics::default()
        },
        ..RunTrace::default()
    };
    for (t, tr) in req.ticks.iter().enumerate() {
        let tick = t as u64;
        let mut inputs = match P::inputs_from_dyn(&tr.inputs) {
            Ok(i) => i,
            Err(e) => {
                trace.error = Some(TraceError {
                    tick,
                    error: DynError::Bridge {
                        slot: e.slot,
                        expected: e.expected,
                    },
                });
                break;
            }
        };
        if !tr.readings.is_empty() {
            match P::inputs_from_readings(&tr.readings, &mut inputs) {
                Ok(()) => {}
                Err(ReadingError::Bridge(e)) => {
                    trace.error = Some(TraceError {
                        tick,
                        error: DynError::Bridge {
                            slot: e.slot,
                            expected: e.expected,
                        },
                    });
                    break;
                }
                Err(ReadingError::Runtime(e)) => {
                    trace.error = Some(TraceError {
                        tick,
                        error: e.into(),
                    });
                    break;
                }
            }
        }
        let active = active_domains(&tr.active);
        let started = std::time::Instant::now();
        let r = P::step(&mut state, active, &inputs);
        trace.metrics.step_ns += started.elapsed().as_nanos();
        trace.metrics.steps += 1;
        match r {
            Ok(out) => trace.ticks.push(TickTrace {
                tick,
                values: P::values_to_dyn(&out),
                outputs: P::outputs_to_dyn(&out),
                commands: P::commands_to_dyn(&out),
                adapter: P::adapter_ops(&out),
            }),
            Err(e) => {
                trace.error = Some(TraceError {
                    tick,
                    error: e.into(),
                });
                break;
            }
        }
    }
    trace
}

/// The generated `host` binary's `main`: JSON request on stdin, JSON
/// trace on stdout.  Exit code 2 on a malformed request.
pub fn main_stdio<P: HostProgram>() {
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        std::process::exit(2);
    }
    let req: RunRequest = match serde_json::from_str(&input) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("bdl host: malformed request: {e}");
            std::process::exit(2);
        }
    };
    let trace = run::<P>(&req);
    let out = serde_json::to_string(&trace).unwrap_or_else(|_| "{}".into());
    let mut stdout = std::io::stdout();
    let _ = stdout.write_all(out.as_bytes());
    let _ = stdout.write_all(b"\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-written program: one input `x` in clock 0, `y = x * 2`.
    struct Doubler;
    impl HostProgram for Doubler {
        type State = ();
        type Inputs = Option<f64>;
        type Tick = (Option<f64>, Option<f64>);
        fn init() {}
        fn step(
            _: &mut (),
            active: ActiveDomains,
            inputs: &Option<f64>,
        ) -> Result<Self::Tick, RuntimeError> {
            if !active.is_active(ClockSlot(0)) {
                return Ok((None, None));
            }
            let x = bdl_runtime_core::read_input(inputs, 0)?;
            let y = bdl_runtime_core::num::mul(x, 2.0, 1)?;
            Ok((Some(x), Some(y)))
        }
        fn inputs_from_readings(
            _: &[Option<DynValue>],
            _: &mut Option<f64>,
        ) -> Result<(), ReadingError> {
            Ok(())
        }
        fn inputs_from_dyn(slots: &[Option<DynValue>]) -> Result<Option<f64>, BridgeError> {
            Ok(match slots.first() {
                Some(Some(v)) => Some(v.quantity(0)?),
                _ => None,
            })
        }
        fn values_to_dyn(t: &Self::Tick) -> Vec<Option<DynValue>> {
            vec![
                t.0.map(|value| DynValue::Quantity { value }),
                t.1.map(|value| DynValue::Quantity { value }),
            ]
        }
        fn outputs_to_dyn(_: &Self::Tick) -> Vec<Option<DynValue>> {
            vec![]
        }
        fn commands_to_dyn(_: &Self::Tick) -> Vec<Option<DynValue>> {
            vec![]
        }
        fn adapter_ops(_: &Self::Tick) -> Vec<AdapterOp> {
            vec![]
        }
        fn state_bytes() -> usize {
            0
        }
    }

    #[test]
    fn run_records_values_then_stops_at_the_first_error() {
        let req = RunRequest {
            ticks: vec![
                TickRequest {
                    active: vec![0],
                    inputs: vec![Some(DynValue::Quantity { value: 1.5 })],
                    readings: vec![],
                },
                TickRequest {
                    active: vec![],
                    inputs: vec![None],
                    readings: vec![],
                },
                TickRequest {
                    active: vec![0],
                    inputs: vec![None],
                    readings: vec![],
                },
                TickRequest {
                    active: vec![0],
                    inputs: vec![Some(DynValue::Quantity { value: 9.0 })],
                    readings: vec![],
                },
            ],
        };
        let t = run::<Doubler>(&req);
        assert_eq!(t.ticks.len(), 2);
        assert_eq!(
            t.ticks[0].values[1],
            Some(DynValue::Quantity { value: 3.0 })
        );
        assert_eq!(t.ticks[1].values, vec![None, None]);
        assert_eq!(
            t.error,
            Some(TraceError {
                tick: 2,
                error: DynError::MissingInput { decl: 0 }
            })
        );
        assert_eq!(t.metrics.steps, 3);
        // the wire form round-trips
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<RunTrace>(&json).unwrap(), t);
        let bad = RunRequest {
            ticks: vec![TickRequest {
                active: vec![0],
                inputs: vec![Some(DynValue::Bool { value: true })],
                readings: vec![],
            }],
        };
        assert!(matches!(
            run::<Doubler>(&bad).error,
            Some(TraceError {
                error: DynError::Bridge { .. },
                ..
            })
        ));
    }

    #[test]
    fn dyn_values_have_the_reference_shape() {
        let v = DynValue::sem(3, DynValue::some(DynValue::Quantity { value: 0.5 }));
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"semantic","id":3,"repr":{"kind":"some","value":{"kind":"quantity","value":0.5}}}"#
        );
        assert_eq!(
            v.semantic(3, 0)
                .unwrap()
                .option(0)
                .unwrap()
                .unwrap()
                .quantity(0)
                .unwrap(),
            0.5
        );
        assert!(v.semantic(4, 1).is_err());
    }
}
