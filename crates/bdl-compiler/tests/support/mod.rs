//! Shared fixtures for the backend tests: a corpus of small Design IRs
//! with schedules and input traces, the reference run, the generated run,
//! and the comparison policy (docs/architecture/codegen-rust.md §Differential testing).

#![allow(dead_code, clippy::unwrap_used)]

use bdl_compiler::{compile_design_ir, CompileArtifact, CompileOptions};
use bdl_ir::{
    ConceptBinding, Declaration, DesignIr, Expr, Interface, OutputSpec, Prim, Scalar, Ty,
};
use bdl_model::{ClockId, DeclId, Dim, OutputId, SemanticId};
use bdl_reactive::eval::{self, RuntimeError, State, TickInput};
use bdl_reactive::{InputTrace, Schedule, Value};
use bdl_runtime_host::harness::Cargo;
use bdl_runtime_host::{DynError, DynValue, RunRequest, RunTrace, TickRequest};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

pub fn d(n: u64) -> DeclId {
    DeclId::from_raw(n)
}
pub fn c(n: u64) -> ClockId {
    ClockId::from_raw(n)
}
pub fn s(n: u64) -> SemanticId {
    SemanticId::from_raw(n)
}
pub fn o(n: u64) -> OutputId {
    OutputId::from_raw(n)
}
pub fn lit(v: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim: Dim::ZERO,
        value: Scalar(v),
    })
}
pub fn lit_dim(dim: Dim, v: f64) -> Expr {
    Expr::prim(Prim::Lit {
        dim,
        value: Scalar(v),
    })
}
pub fn add(a: Expr, b: Expr) -> Expr {
    Expr::apps(Expr::prim(Prim::Add { dim: Dim::ZERO }), [a, b])
}
pub fn sub(a: Expr, b: Expr) -> Expr {
    Expr::apps(Expr::prim(Prim::Sub { dim: Dim::ZERO }), [a, b])
}
pub fn mul(a: Expr, b: Expr) -> Expr {
    Expr::apps(
        Expr::prim(Prim::Mul {
            d1: Dim::ZERO,
            d2: Dim::ZERO,
        }),
        [a, b],
    )
}
pub fn div(a: Expr, b: Expr) -> Expr {
    Expr::apps(
        Expr::prim(Prim::Div {
            d1: Dim::ZERO,
            d2: Dim::ZERO,
        }),
        [a, b],
    )
}

/// A declaration with an explicit type, realization and domain.
pub fn decl(ir: &mut DesignIr, n: u64, name: &str, ty: Ty, real: Option<Expr>, clock: Option<u64>) {
    ir.decls.insert(
        d(n),
        Declaration {
            id: d(n),
            name: name.into(),
            interface: Interface {
                expected_type: ty,
                commitments: vec![],
            },
            realization: real,
        },
    );
    if let Some(cl) = clock {
        ir.clocks.insert(d(n), c(cl));
    }
}
pub fn qdecl(ir: &mut DesignIr, n: u64, name: &str, real: Option<Expr>, clock: Option<u64>) {
    decl(ir, n, name, Ty::q(Dim::ZERO), real, clock)
}
pub fn concept(ir: &mut DesignIr, n: u64, name: &str, rep: Ty) {
    ir.concepts.insert(
        s(n),
        ConceptBinding {
            id: s(n),
            name: name.into(),
            representation: Some(rep),
            ordered: false,
        },
    );
}

/// One corpus entry: a design, how long to run it, which domains tick
/// when, and what the inputs are.
pub struct Case {
    pub name: &'static str,
    pub ir: DesignIr,
    pub ticks: u64,
    pub schedule: Schedule,
    pub inputs: InputTrace,
}

impl Case {
    fn new(name: &'static str, ir: DesignIr, ticks: u64) -> Case {
        let schedule = Schedule::always(&ir);
        Case {
            name,
            ir,
            ticks,
            schedule,
            inputs: InputTrace::default(),
        }
    }
}

pub fn deg(x: f64) -> Value {
    Value::sem(s(0), Value::q(Dim::ANGLE, x.to_radians()))
}

/// tilt : Tilt (input, c0) · dimByTilt : Tilt → Brightness (λ) ·
/// brightness := dimByTilt tilt (c0); with `output` also light : Brightness
/// in c0 driven by brightness.
pub fn lamp(with_output: bool) -> DesignIr {
    let mut ir = DesignIr::default();
    concept(&mut ir, 0, "Tilt", Ty::q(Dim::ANGLE));
    concept(&mut ir, 1, "Brightness", Ty::q(Dim::ZERO));
    let f = Expr::Lam {
        dom: Ty::sem(s(0)),
        body: Box::new(Expr::mk(
            s(1),
            Expr::apps(
                Expr::prim(Prim::Div {
                    d1: Dim::ANGLE,
                    d2: Dim::ANGLE,
                }),
                [
                    Expr::rep(Expr::Var { index: 0 }),
                    lit_dim(Dim::ANGLE, std::f64::consts::FRAC_PI_2),
                ],
            ),
        )),
    };
    decl(&mut ir, 0, "tilt", Ty::sem(s(0)), None, Some(0));
    decl(
        &mut ir,
        1,
        "dimByTilt",
        Ty::arr(Ty::sem(s(0)), Ty::sem(s(1))),
        Some(f),
        None,
    );
    decl(
        &mut ir,
        2,
        "brightness",
        Ty::sem(s(1)),
        Some(Expr::app(Expr::decl(d(1)), Expr::decl(d(0)))),
        Some(0),
    );
    ir.clock_names.insert(c(0), "interaction".into());
    if with_output {
        ir.outputs.insert(
            o(4),
            OutputSpec {
                id: o(4),
                name: "light".into(),
                accepts: Ty::sem(s(1)),
                clock: c(0),
            },
        );
        ir.drives.insert(d(2), o(4));
    }
    ir
}

pub fn corpus() -> Vec<Case> {
    let mut cases = Vec::new();

    let mut case = Case::new("lamp", lamp(false), 4);
    case.inputs
        .series(d(0), [deg(0.0), deg(30.0), deg(60.0), deg(90.0)]);
    cases.push(case);

    let mut case = Case::new("lamp_output", lamp(true), 4);
    case.inputs
        .series(d(0), [deg(0.0), deg(30.0), deg(60.0), deg(90.0)]);
    cases.push(case);

    // Pure arithmetic with dimensions and a Count.
    let mut ir = DesignIr::default();
    decl(
        &mut ir,
        0,
        "area",
        Ty::q(Dim::LENGTH + Dim::LENGTH),
        Some(Expr::apps(
            Expr::prim(Prim::Mul {
                d1: Dim::LENGTH,
                d2: Dim::LENGTH,
            }),
            [lit_dim(Dim::LENGTH, 2.5), lit_dim(Dim::LENGTH, 4.0)],
        )),
        Some(0),
    );
    decl(
        &mut ir,
        1,
        "speed",
        Ty::q(Dim::LENGTH - Dim::TIME),
        Some(Expr::apps(
            Expr::prim(Prim::Div {
                d1: Dim::LENGTH,
                d2: Dim::TIME,
            }),
            [lit_dim(Dim::LENGTH, 100.0), lit_dim(Dim::TIME, 9.58)],
        )),
        Some(0),
    );
    qdecl(
        &mut ir,
        2,
        "poly",
        Some(sub(
            add(mul(lit(2.0), lit(3.0)), lit(0.1)),
            div(lit(1.0), lit(3.0)),
        )),
        Some(0),
    );
    decl(
        &mut ir,
        3,
        "count",
        Ty::Nat,
        Some(Expr::NatLit { value: 7 }),
        Some(0),
    );
    ir.clock_names.insert(c(0), "main".into());
    cases.push(Case::new("arith", ir, 2));

    // Semantic rep/mk with a nullary mapping and a Boolean concept.
    let mut ir = DesignIr::default();
    concept(&mut ir, 0, "Tilt", Ty::q(Dim::ANGLE));
    concept(&mut ir, 1, "Held", Ty::Bool);
    decl(&mut ir, 0, "tilt", Ty::sem(s(0)), None, Some(0));
    decl(
        &mut ir,
        1,
        "steep",
        Ty::sem(s(1)),
        Some(Expr::mk(
            s(1),
            Expr::apps(
                Expr::prim(Prim::Lt { dim: Dim::ANGLE }),
                [lit_dim(Dim::ANGLE, 0.5), Expr::rep(Expr::decl(d(0)))],
            ),
        )),
        Some(0),
    );
    decl(&mut ir, 2, "held", Ty::sem(s(1)), None, Some(0));
    decl(
        &mut ir,
        3,
        "both",
        Ty::Bool,
        Some(Expr::apps(
            Expr::prim(Prim::And),
            [Expr::rep(Expr::decl(d(1))), Expr::rep(Expr::decl(d(2)))],
        )),
        Some(0),
    );
    let mut case = Case::new("semantic", ir, 3);
    case.inputs.series(d(0), [deg(10.0), deg(45.0), deg(80.0)]);
    case.inputs.series(
        d(2),
        [
            Value::sem(s(1), Value::boolean(true)),
            Value::sem(s(1), Value::boolean(true)),
            Value::sem(s(1), Value::boolean(false)),
        ],
    );
    cases.push(case);

    // Booleans, comparisons, strict if, options.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "x", None, Some(0));
    let q = Ty::q(Dim::ZERO);
    decl(
        &mut ir,
        1,
        "big",
        Ty::Bool,
        Some(Expr::apps(
            Expr::prim(Prim::Lt { dim: Dim::ZERO }),
            [lit(10.0), Expr::decl(d(0))],
        )),
        Some(0),
    );
    qdecl(
        &mut ir,
        2,
        "clamped",
        Some(Expr::apps(
            Expr::prim(Prim::Ite { ty: q.clone() }),
            [Expr::decl(d(1)), lit(10.0), Expr::decl(d(0))],
        )),
        Some(0),
    );
    decl(
        &mut ir,
        3,
        "maybe",
        Ty::opt(q.clone()),
        Some(Expr::apps(
            Expr::prim(Prim::Ite {
                ty: Ty::opt(q.clone()),
            }),
            [
                Expr::apps(
                    Expr::prim(Prim::Eq {
                        ty: Ty::q(Dim::ZERO),
                    }),
                    [Expr::decl(d(0)), lit(0.0)],
                ),
                Expr::prim(Prim::None { ty: q.clone() }),
                Expr::app(Expr::prim(Prim::Some { ty: q.clone() }), Expr::decl(d(2))),
            ],
        )),
        Some(0),
    );
    qdecl(
        &mut ir,
        4,
        "got",
        Some(Expr::apps(
            Expr::prim(Prim::GetD { ty: q.clone() }),
            [Expr::decl(d(3)), lit(-1.0)],
        )),
        Some(0),
    );
    decl(
        &mut ir,
        5,
        "flag",
        Ty::Bool,
        Some(Expr::apps(
            Expr::prim(Prim::Or),
            [
                Expr::app(
                    Expr::prim(Prim::Not),
                    Expr::app(Expr::prim(Prim::IsSome { ty: q.clone() }), Expr::decl(d(3))),
                ),
                Expr::decl(d(1)),
            ],
        )),
        Some(0),
    );
    let mut case = Case::new("bool_opt", ir, 4);
    case.inputs.series(
        d(0),
        [
            Value::scalar(0.0),
            Value::scalar(5.0),
            Value::scalar(50.0),
            Value::scalar(-3.0),
        ],
    );
    cases.push(case);

    // delay: an accumulator reading its own previous value.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "x", None, Some(0));
    qdecl(
        &mut ir,
        1,
        "acc",
        Some(Expr::delay(
            lit(0.0),
            add(Expr::decl(d(1)), Expr::decl(d(0))),
        )),
        Some(0),
    );
    qdecl(
        &mut ir,
        2,
        "twice",
        Some(mul(Expr::decl(d(1)), lit(2.0))),
        Some(0),
    );
    let mut case = Case::new("delay", ir, 5);
    case.inputs
        .series(d(0), (1..=5).map(|n| Value::scalar(n as f64)));
    cases.push(case);

    // A cycle broken by delay: a := delay 0 b ; b := a + 1.
    let mut ir = DesignIr::default();
    qdecl(
        &mut ir,
        0,
        "a",
        Some(Expr::delay(lit(0.0), Expr::decl(d(1)))),
        Some(0),
    );
    qdecl(
        &mut ir,
        1,
        "b",
        Some(add(Expr::decl(d(0)), lit(1.0))),
        Some(0),
    );
    cases.push(Case::new("delayed_cycle", ir, 4));

    // sync across two domains, both directions, with the slow domain on a
    // period of 2 so source and destination sometimes share a tick.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "x", None, Some(0));
    qdecl(
        &mut ir,
        1,
        "y",
        Some(Expr::sync(c(0), lit(-1.0), Expr::decl(d(0)))),
        Some(1),
    );
    qdecl(
        &mut ir,
        2,
        "z",
        Some(Expr::sync(c(1), lit(0.0), Expr::decl(d(1)))),
        Some(0),
    );
    ir.clock_names.insert(c(0), "fast".into());
    ir.clock_names.insert(c(1), "slow".into());
    let mut case = Case::new("sync", ir, 7);
    case.schedule.periods.insert(c(1), 2);
    case.inputs
        .series(d(0), (0..7).map(|n| Value::scalar(n as f64 * 10.0)));
    cases.push(case);

    // A domain-agnostic pure declaration used from two domains.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "k", Some(mul(lit(2.0), lit(3.0))), None);
    qdecl(
        &mut ir,
        1,
        "u",
        Some(add(Expr::decl(d(0)), lit(1.0))),
        Some(0),
    );
    qdecl(
        &mut ir,
        2,
        "v",
        Some(sub(Expr::decl(d(0)), lit(1.0))),
        Some(1),
    );
    let mut case = Case::new("agnostic", ir, 6);
    case.schedule.periods.insert(c(0), 2);
    case.schedule.periods.insert(c(1), 3);
    cases.push(case);

    // Runtime errors: division by zero at tick 2, from an input.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "x", None, Some(0));
    qdecl(
        &mut ir,
        1,
        "inv",
        Some(div(lit(1.0), Expr::decl(d(0)))),
        Some(0),
    );
    let mut case = Case::new("div_by_zero", ir, 4);
    case.inputs.series(
        d(0),
        [
            Value::scalar(2.0),
            Value::scalar(4.0),
            Value::scalar(0.0),
            Value::scalar(8.0),
        ],
    );
    cases.push(case);

    // Non-finite result at tick 1.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "x", None, Some(0));
    qdecl(
        &mut ir,
        1,
        "huge",
        Some(mul(Expr::decl(d(0)), lit(1e308))),
        Some(0),
    );
    let mut case = Case::new("non_finite", ir, 3);
    case.inputs.series(
        d(0),
        [Value::scalar(1.0), Value::scalar(10.0), Value::scalar(1.0)],
    );
    cases.push(case);

    // Missing input at tick 1.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "x", None, Some(0));
    qdecl(
        &mut ir,
        1,
        "y",
        Some(add(Expr::decl(d(0)), lit(1.0))),
        Some(0),
    );
    let mut case = Case::new("missing_input", ir, 3);
    case.inputs.set(d(0), 0, Value::scalar(1.0));
    case.inputs.set(d(0), 2, Value::scalar(3.0));
    cases.push(case);

    // Collections and grouped values (Phase 9a/9b): the library's
    // combinators at closed instances, the recursor with a primitive step,
    // structural equality, a list in a state cell.
    let mut ir = DesignIr::default();
    let q = Ty::q(Dim::ZERO);
    let lq = Ty::list(q.clone());
    let mut inst = bdl_equations::Instance::default();
    inst.subst.tys.insert(0, q.clone());
    inst.subst.tys.insert(1, q.clone());
    inst.subst.dims.insert(0, Dim::ZERO);
    inst.ordered.insert(0, bdl_equations::Ordered::Q(Dim::ZERO));
    let lib = |name: &str| (bdl_equations::lookup(name).unwrap().build)(&inst);
    decl(&mut ir, 0, "xs", lq.clone(), None, Some(0));
    // above := any(xs, x => 2 < x)
    decl(
        &mut ir,
        1,
        "above",
        Ty::Bool,
        Some(Expr::apps(
            lib("any"),
            [
                Expr::decl(d(0)),
                Expr::lam(
                    q.clone(),
                    Expr::apps(
                        Expr::prim(Prim::Lt { dim: Dim::ZERO }),
                        [lit(2.0), Expr::var(0)],
                    ),
                ),
            ],
        )),
        Some(0),
    );
    // total := sum(xs)
    qdecl(
        &mut ir,
        2,
        "total",
        Some(Expr::app(lib("sum"), Expr::decl(d(0)))),
        Some(0),
    );
    // shifted := map(xs, x => x + 5)
    decl(
        &mut ir,
        3,
        "shifted",
        lq.clone(),
        Some(Expr::apps(
            lib("map"),
            [
                Expr::decl(d(0)),
                Expr::lam(q.clone(), add(Expr::var(0), lit(5.0))),
            ],
        )),
        Some(0),
    );
    // stats := (getOrElse(head(xs), 0), length(xs))
    decl(
        &mut ir,
        4,
        "stats",
        Ty::prod(q.clone(), q.clone()),
        Some(Expr::apps(
            Expr::prim(Prim::Pair {
                fst: q.clone(),
                snd: q.clone(),
            }),
            [
                Expr::apps(
                    lib("getOrElse"),
                    [Expr::app(lib("head"), Expr::decl(d(0))), lit(0.0)],
                ),
                Expr::app(lib("length"), Expr::decl(d(0))),
            ],
        )),
        Some(0),
    );
    // first := first(stats)
    qdecl(
        &mut ir,
        5,
        "first",
        Some(Expr::app(lib("first"), Expr::decl(d(4)))),
        Some(0),
    );
    // same := xs == reverse(reverse(xs))
    decl(
        &mut ir,
        6,
        "same",
        Ty::Bool,
        Some(Expr::apps(
            Expr::prim(Prim::Eq { ty: lq.clone() }),
            [
                Expr::decl(d(0)),
                Expr::app(lib("reverse"), Expr::app(lib("reverse"), Expr::decl(d(0)))),
            ],
        )),
        Some(0),
    );
    // remembered := delay [] xs
    decl(
        &mut ir,
        7,
        "remembered",
        lq.clone(),
        Some(Expr::delay(
            Expr::prim(Prim::Nil { ty: q.clone() }),
            Expr::decl(d(0)),
        )),
        Some(0),
    );
    // pairs := zip(xs, shifted)
    decl(
        &mut ir,
        8,
        "pairs",
        Ty::list(Ty::prod(q.clone(), q.clone())),
        Some(Expr::apps(lib("zip"), [Expr::decl(d(0)), Expr::decl(d(3))])),
        Some(0),
    );
    // has2 := contains(2, xs)
    decl(
        &mut ir,
        9,
        "has2",
        Ty::Bool,
        Some(Expr::apps(lib("contains"), [lit(2.0), Expr::decl(d(0))])),
        Some(0),
    );
    // clipped := clamp(total, 1, 4); kept := filter(xs, x => x < 2)
    qdecl(
        &mut ir,
        10,
        "clipped",
        Some(Expr::apps(
            lib("clamp"),
            [Expr::decl(d(2)), lit(1.0), lit(4.0)],
        )),
        Some(0),
    );
    decl(
        &mut ir,
        11,
        "kept",
        lq.clone(),
        Some(Expr::apps(
            lib("filter"),
            [
                Expr::decl(d(0)),
                Expr::lam(
                    q.clone(),
                    Expr::apps(
                        Expr::prim(Prim::Lt { dim: Dim::ZERO }),
                        [Expr::var(0), lit(2.0)],
                    ),
                ),
            ],
        )),
        Some(0),
    );
    ir.clock_names.insert(c(0), "main".into());
    let mut case = Case::new("collections", ir, 4);
    case.inputs.series(
        d(0),
        (0..4).map(|t| Value::list((0..t).map(|i| Value::scalar(i as f64)))),
    );
    cases.push(case);

    // The Phase-9a buffer: a lossless cross-domain window as five
    // declarations over `delay`/`sync` and list data — no buffer primitive.
    //   log   @fast := cons x (delay [] log)
    //   logD  @slow := sync fast [] log
    //   seen  @slow := length logD
    //   cursor@slow := delay 0 seen
    //   window@slow := reverse (take (seen − cursor) logD)
    let mut ir = DesignIr::default();
    let nil = Expr::prim(Prim::Nil { ty: q.clone() });
    qdecl(&mut ir, 0, "x", None, Some(0));
    decl(
        &mut ir,
        1,
        "log",
        lq.clone(),
        Some(Expr::apps(
            Expr::prim(Prim::Cons { ty: q.clone() }),
            [Expr::decl(d(0)), Expr::delay(nil.clone(), Expr::decl(d(1)))],
        )),
        Some(0),
    );
    decl(
        &mut ir,
        2,
        "logD",
        lq.clone(),
        Some(Expr::sync(c(0), nil.clone(), Expr::decl(d(1)))),
        Some(1),
    );
    qdecl(
        &mut ir,
        3,
        "seen",
        Some(Expr::app(
            Expr::prim(Prim::Length { ty: q.clone() }),
            Expr::decl(d(2)),
        )),
        Some(1),
    );
    qdecl(
        &mut ir,
        4,
        "cursor",
        Some(Expr::delay(lit(0.0), Expr::decl(d(3)))),
        Some(1),
    );
    decl(
        &mut ir,
        5,
        "window",
        lq.clone(),
        Some(Expr::app(
            Expr::prim(Prim::Reverse { ty: q.clone() }),
            Expr::apps(
                Expr::prim(Prim::Take { ty: q.clone() }),
                [sub(Expr::decl(d(3)), Expr::decl(d(4))), Expr::decl(d(2))],
            ),
        )),
        Some(1),
    );
    ir.clock_names.insert(c(0), "fast".into());
    ir.clock_names.insert(c(1), "slow".into());
    let mut case = Case::new("buffer", ir, 7);
    case.schedule.periods.insert(c(1), 3);
    case.inputs
        .series(d(0), (0..7).map(|n| Value::scalar(n as f64 + 1.0)));
    cases.push(case);

    // A design with no clock domain at all.
    let mut ir = DesignIr::default();
    qdecl(&mut ir, 0, "one", Some(lit(1.0)), None);
    qdecl(
        &mut ir,
        1,
        "two",
        Some(add(Expr::decl(d(0)), Expr::decl(d(0)))),
        None,
    );
    cases.push(Case::new("no_domains", ir, 2));

    cases
}

// ---- running ---------------------------------------------------------------

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

pub fn options() -> CompileOptions {
    let root = repo_root();
    CompileOptions {
        require_complete: false,
        codegen: bdl_codegen_rust::CodegenOptions {
            runtime_core_path: root.join("runtime/bdl-runtime-core").display().to_string(),
            runtime_host_path: root.join("runtime/bdl-runtime-host").display().to_string(),
        },
    }
}

pub fn compile_case(case: &Case) -> CompileArtifact {
    compile_design_ir(case.ir.clone(), case.name, &options())
}

/// Where generated crates are written and built during tests.
pub fn generated_dir(name: &str) -> PathBuf {
    repo_root().join("target/bdl-generated").join(name)
}

pub fn cargo_for(name: &str) -> Cargo {
    Cargo::new(
        generated_dir(name),
        repo_root().join("target/bdl-generated/target"),
    )
}

/// The reference run: per tick the sample and, for inputs, what was fed.
pub struct ReferenceRun {
    pub ticks: Vec<eval::TickOutcome>,
    pub error: Option<(u64, RuntimeError)>,
    /// Every runtime error the reference *could* report at the failing
    /// tick, one per starting declaration (DI-25).
    pub error_alternatives: BTreeSet<ErrorKind>,
}

pub fn reference_run(case: &Case) -> ReferenceRun {
    let all_active = |t: u64| case.schedule.active_at(t);
    let input_at = |t: u64| TickInput {
        values: case
            .inputs
            .samples
            .iter()
            .filter_map(|(d, s)| s.get(&t).map(|v| (*d, v.clone())))
            .collect(),
    };
    let mut state = State::default();
    let mut ticks = Vec::new();
    for t in 0..case.ticks {
        let active = all_active(t);
        let input = input_at(t);
        match eval::step(&case.ir, t, &active, &state, &input) {
            Ok(o) => {
                state = o.next.clone();
                ticks.push(o);
            }
            Err(e) => {
                let due: Vec<DeclId> = eval::evaluated_this_tick(&case.ir, &active)
                    .into_iter()
                    .collect();
                let mut alts = BTreeSet::new();
                for i in 0..due.len() {
                    let mut order = due[i..].to_vec();
                    order.extend_from_slice(&due[..i]);
                    if let Err(e) =
                        eval::step_in_order(&case.ir, t, &active, &state, &input, &order)
                    {
                        alts.insert(ErrorKind::of_reference(&e));
                    }
                }
                return ReferenceRun {
                    ticks,
                    error: Some((t, e)),
                    error_alternatives: alts,
                };
            }
        }
    }
    ReferenceRun {
        ticks,
        error: None,
        error_alternatives: BTreeSet::new(),
    }
}

/// The comparison key for runtime errors: category and declaration.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorKind {
    MissingInput(u64),
    DivisionByZero(u64),
    NonFinite(u64),
    Other(String),
}

impl ErrorKind {
    pub fn of_reference(e: &RuntimeError) -> ErrorKind {
        match e {
            RuntimeError::MissingInput { decl, .. } => ErrorKind::MissingInput(decl.raw()),
            RuntimeError::DivisionByZero { decl, .. } => ErrorKind::DivisionByZero(decl.raw()),
            RuntimeError::NonFinite { decl, .. } => ErrorKind::NonFinite(decl.raw()),
            other => ErrorKind::Other(other.to_string()),
        }
    }
    pub fn of_dyn(e: &DynError) -> ErrorKind {
        match e {
            DynError::MissingInput { decl } => ErrorKind::MissingInput(*decl),
            DynError::DivisionByZero { decl } => ErrorKind::DivisionByZero(*decl),
            DynError::NonFinite { decl, .. } => ErrorKind::NonFinite(*decl),
            other => ErrorKind::Other(format!("{other:?}")),
        }
    }
}

/// Reference `Value` → `DynValue`: dimensions are static and dropped.
pub fn dyn_of(v: &Value) -> DynValue {
    match v {
        Value::Bool { value } => DynValue::Bool { value: *value },
        Value::Nat { value } => DynValue::Nat { value: *value },
        Value::Quantity { value, .. } => DynValue::Quantity { value: *value },
        Value::Semantic { id, repr } => DynValue::sem(id.raw(), dyn_of(repr)),
        Value::None => DynValue::None,
        Value::Some { value } => DynValue::some(dyn_of(value)),
        Value::List { items } => DynValue::List {
            items: items.iter().map(dyn_of).collect(),
        },
        Value::Pair { fst, snd } => DynValue::pair_of(dyn_of(fst), dyn_of(snd)),
        Value::Closure { .. } | Value::Prim { .. } => panic!("function values are never compared"),
    }
}

/// The run request for the generated program: clock slots and input
/// slots from the artefact's plan.
pub fn run_request(case: &Case, art: &CompileArtifact) -> RunRequest {
    let exec = art.exec_ir.as_ref().unwrap();
    let ticks = (0..case.ticks)
        .map(|t| {
            let active: Vec<u16> = case
                .schedule
                .active_at(t)
                .iter()
                .filter_map(|c| exec.clock_slot(*c))
                .map(|s| s.0)
                .collect();
            let inputs = exec
                .inputs
                .iter()
                .map(|i| {
                    let id = exec.decl(i.decl).unwrap().id;
                    case.inputs
                        .samples
                        .get(&id)
                        .and_then(|s| s.get(&t))
                        .map(dyn_of)
                })
                .collect();
            TickRequest { active, inputs }
        })
        .collect();
    RunRequest { ticks }
}

/// The comparison: every value of every tick, every output, and the
/// failure (tick exactly; kind within the reference's alternatives).
pub fn assert_agree(
    case: &Case,
    art: &CompileArtifact,
    reference: &ReferenceRun,
    generated: &RunTrace,
) {
    let exec = art.exec_ir.as_ref().unwrap();
    let name = case.name;
    assert_eq!(
        reference.ticks.len(),
        generated.ticks.len(),
        "{name}: tick count (generated error: {:?})",
        generated.error
    );
    for (t, (r, g)) in reference.ticks.iter().zip(&generated.ticks).enumerate() {
        let tick = t as u64;
        assert_eq!(g.tick, tick);
        let active = case.schedule.active_at(tick);
        let due = eval::evaluated_this_tick(&case.ir, &active);
        for dp in &exec.decls {
            let expected = match dp.kind {
                bdl_exec_ir::DeclKind::Input { .. } => {
                    if due.contains(&dp.id) {
                        case.inputs
                            .samples
                            .get(&dp.id)
                            .and_then(|s| s.get(&tick))
                            .map(dyn_of)
                    } else {
                        None
                    }
                }
                bdl_exec_ir::DeclKind::Computed { .. } => r.values.get(&dp.id).map(dyn_of),
            };
            assert_eq!(
                expected, g.values[dp.index.0 as usize],
                "{name} tick {t}: {} ({})",
                dp.name, dp.id
            );
        }
        let sample = bdl_reactive::TickSample {
            tick,
            active: active.clone(),
            values: r.values.clone(),
        };
        let outs = bdl_output::output_values(&sample, &art.analysis.outputs.valid_bindings);
        for op in &exec.outputs {
            assert_eq!(
                outs.get(&op.id).map(dyn_of),
                g.outputs[op.slot.0 as usize],
                "{name} tick {t}: output {}",
                op.name
            );
        }
    }
    match (&reference.error, &generated.error) {
        (None, None) => {}
        (Some((t, e)), Some(g)) => {
            assert_eq!(*t, g.tick, "{name}: failing tick");
            let gk = ErrorKind::of_dyn(&g.error);
            let rk = ErrorKind::of_reference(e);
            assert!(
                gk == rk || reference.error_alternatives.contains(&gk),
                "{name}: reference {rk:?} (alternatives {:?}) vs generated {gk:?}",
                reference.error_alternatives
            );
        }
        (r, g) => panic!("{name}: reference error {r:?} vs generated error {g:?}"),
    }
}

/// Compile, write, build and run one case against the reference.
pub fn differential(case: &Case) -> (CompileArtifact, RunTrace) {
    let art = compile_case(case);
    assert!(art.succeeded(), "{}: {:?}", case.name, art.diagnostics);
    let g = art.generated.as_ref().unwrap();
    let dir = generated_dir(case.name);
    bdl_runtime_host::harness::write_crate(&dir, &g.files).unwrap();
    let cargo = cargo_for(case.name);
    let req = run_request(case, &art);
    let trace = cargo
        .run_host(&req)
        .unwrap_or_else(|e| panic!("{}: {e}", case.name));
    let reference = reference_run(case);
    assert_agree(case, &art, &reference, &trace);
    (art, trace)
}

pub fn value_of(trace: &RunTrace, art: &CompileArtifact, decl: DeclId) -> Vec<Option<DynValue>> {
    let idx = art.exec_ir.as_ref().unwrap().decl_index(decl).unwrap().0 as usize;
    trace.ticks.iter().map(|t| t.values[idx].clone()).collect()
}

pub fn quantities(vs: &[Option<DynValue>]) -> Vec<Option<f64>> {
    vs.iter()
        .map(|v| match v {
            Some(DynValue::Quantity { value }) => Some(*value),
            Some(DynValue::Semantic { repr, .. }) => match **repr {
                DynValue::Quantity { value } => Some(value),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

pub fn all_ticks(map: BTreeMap<u64, Value>) -> BTreeMap<u64, Value> {
    map
}
