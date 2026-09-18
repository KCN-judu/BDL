//! The behaviour-system vertical slice (brief §48–§57): TiltSource +
//! 2 × AdaptiveLamp, flattened into the existing compiler, simulator,
//! deployment and backend — and compared with a hand-written flat design.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_compiler::{MappingStatus, ProjectAnalysis};
use bdl_model::edit::EditOp;
use bdl_model::surface::{Definition, DeviceKind};
use bdl_model::{DeclId, Dim};
use bdl_reactive::{InputTrace, Schedule, Simulation, SimulationTrace, Value};
use bdl_system::*;
use std::collections::BTreeMap;
use support::*;

fn deg(tilt: bdl_model::SemanticId, x: f64) -> Value {
    Value::sem(tilt, Value::q(Dim::ANGLE, x.to_radians()))
}

/// Simulate a flat snapshot for four ticks with the given tilt series fed
/// to `raw`; returns the trace and the analysis.
fn simulate(
    snapshot: &bdl_model::surface::ProjectSnapshot,
    raw: DeclId,
    tilt: bdl_model::SemanticId,
) -> (SimulationTrace, ProjectAnalysis) {
    let a = bdl_compiler::analyze(snapshot);
    assert!(a.causality.valid, "{:?}", a.diagnostics);
    let mut inputs = InputTrace::default();
    inputs.series(
        raw,
        [
            deg(tilt, 0.0),
            deg(tilt, 30.0),
            deg(tilt, 60.0),
            deg(tilt, 90.0),
        ],
    );
    let mut sim =
        Simulation::new(a.ir.clone(), &a.causality, Schedule::always(&a.ir), inputs).unwrap();
    let trace = sim.run(4).unwrap().clone();
    (trace, a)
}

fn levels(trace: &SimulationTrace, decl: DeclId) -> Vec<f64> {
    trace
        .series(decl)
        .iter()
        .map(|(_, v)| v.unwrap_semantic().unwrap().as_quantity().unwrap().1)
        .collect()
}

#[test]
fn the_slice_flattens_checks_and_simulates_with_two_equal_lamps() {
    let s = vertical_slice();
    let a = analyze_system(&s.sys.snap);
    assert!(a.composition.is_empty(), "{:?}", a.composition);
    assert!(
        a.analysis.diagnostics.iter().all(|d| !d.is_error()),
        "{:?}",
        a.analysis.diagnostics
    );
    assert!(a.analysis.causality.valid && a.analysis.clocks.valid);
    assert!(a.analysis.output_complete);
    // both required ports are realised through their bindings
    assert_eq!(
        a.ports[&pr(s.lamp_a, s.lamp_in)],
        PortStatus::Bound { binding: s.bind_a }
    );
    assert_eq!(
        a.ports[&pr(s.lamp_b, s.lamp_in)],
        PortStatus::Bound { binding: s.bind_b }
    );
    assert_eq!(a.ports[&pr(s.sensor, s.source_port)], PortStatus::Provided);
    // the flattened lamps are ClockConsistent, their bindings are references by identity
    let flat = &a.flattened.snapshot;
    let a_in = s.sys.flat_decl(s.lamp_a, s.lamp_tilt_value);
    let src = s.sys.flat_decl(s.sensor, s.source_tilt_value);
    assert_eq!(
        flat.design.mappings[&a_in].definition,
        Some(Definition::Reference {
            target: src,
            transport: None
        })
    );
    assert_eq!(
        a.analysis.mappings[&a_in].status,
        MappingStatus::ClockConsistent
    );
    assert_eq!(
        a.analysis.ir.decls[&a_in].realization,
        Some(bdl_ir::Expr::decl(src))
    );
    // the sensor reading is the system's only input; executable
    assert_eq!(a.acceptance, Acceptance::Executable, "{:?}", a.projected);
    // simulate: lampA and lampB agree tick for tick
    let raw = s.sys.flat_decl(s.sensor, s.source_raw);
    let (trace, _) = simulate(flat, raw, s.tilt);
    let ba = levels(&trace, s.sys.flat_decl(s.lamp_a, s.lamp_brightness));
    let bb = levels(&trace, s.sys.flat_decl(s.lamp_b, s.lamp_brightness));
    assert_eq!(ba, bb);
    assert_eq!(ba[0], 0.0);
    assert_eq!(ba[3], 1.0);
    assert!((ba[1] - 1.0 / 3.0).abs() < 1e-12);
    // traces can be labelled in the system's terms
    let lbl = a
        .flattened
        .origins
        .label_of_decl(s.sys.flat_decl(s.lamp_a, s.lamp_brightness), "brightness");
    assert_eq!(lbl.as_deref(), Some("lampA.brightness"));
}

#[test]
fn instances_never_alias_and_shared_identity_is_kept() {
    let s = vertical_slice();
    let sys = s.sys.system();
    // private declarations, concepts, clocks (none private here), outputs, devices differ
    for e in [
        LocalEntity::Decl(s.lamp_brightness),
        LocalEntity::Decl(s.lamp_dim),
        LocalEntity::Sem(s.lamp_brightness_concept),
        LocalEntity::Output(s.lamp_light),
    ] {
        assert_ne!(
            sys.flat_ids.get(s.lamp_a, e),
            sys.flat_ids.get(s.lamp_b, e),
            "{e:?}"
        );
    }
    let dev_a: Vec<_> = sys
        .flat_ids
        .for_instance(s.lamp_a)
        .filter(|e| matches!(e.local, LocalEntity::Device(_)))
        .map(|e| e.flat)
        .collect();
    let dev_b: Vec<_> = sys
        .flat_ids
        .for_instance(s.lamp_b)
        .filter(|e| matches!(e.local, LocalEntity::Device(_)))
        .map(|e| e.flat)
        .collect();
    assert_eq!(dev_a.len(), 1);
    assert_ne!(dev_a, dev_b);
    // the shared Tilt is the system's concept in both, and is not freshened
    assert!(sys
        .flat_ids
        .get(s.lamp_a, LocalEntity::Sem(s.lamp_tilt))
        .is_none());
    let f = flatten(&s.sys.snap);
    let a_in = s.sys.flat_decl(s.lamp_a, s.lamp_tilt_value);
    let b_in = s.sys.flat_decl(s.lamp_b, s.lamp_tilt_value);
    assert_eq!(f.snapshot.design.mappings[&a_in].signature.output, s.tilt);
    assert_eq!(f.snapshot.design.mappings[&b_in].signature.output, s.tilt);
    assert_eq!(
        f.snapshot
            .design
            .concepts
            .values()
            .filter(|c| c.name.ends_with("Brightness"))
            .count(),
        2
    );
    assert_eq!(
        f.snapshot
            .design
            .concepts
            .values()
            .filter(|c| c.name == "Tilt")
            .count(),
        1
    );
    // fresh ids never collide with the system's own
    assert!(!f
        .snapshot
        .design
        .concepts
        .values()
        .any(|c| c.id == s.tilt && c.name != "Tilt"));
    // origins recover instance and local entity
    let o = &f.origins.decls[&a_in];
    assert_eq!(
        (o.instance, o.component, o.local),
        (s.lamp_a, s.lamp, LocalEntity::Decl(s.lamp_tilt_value))
    );
    assert_eq!(f.origins.ports[&a_in], pr(s.lamp_a, s.lamp_in));
    assert_eq!(
        f.origins.decl_of_port(pr(s.sensor, s.source_port)),
        Some(s.sys.flat_decl(s.sensor, s.source_tilt_value))
    );
}

#[test]
fn renames_and_unrelated_instances_do_not_churn_identity() {
    let mut s = vertical_slice();
    let before = s.sys.system().flat_ids.clone();
    let flat_before = flatten(&s.sys.snap);
    // rename lampA, the sensor, the component, the port
    s.sys.apply(SystemEditOp::RenameInstance {
        id: s.lamp_a,
        name: "leftLamp".into(),
    });
    s.sys.apply(SystemEditOp::RenameInstance {
        id: s.sensor,
        name: "imu".into(),
    });
    s.sys.apply(SystemEditOp::RenameComponent {
        id: s.lamp,
        name: "Lamp".into(),
    });
    s.sys.apply(SystemEditOp::RenamePort {
        component: s.source,
        port: s.source_port,
        name: "angle".into(),
    });
    assert_eq!(s.sys.system().flat_ids, before, "renames touch no identity");
    let flat_after = flatten(&s.sys.snap);
    assert_eq!(flat_after.origins.decls, flat_before.origins.decls);
    assert_eq!(
        flat_after
            .snapshot
            .design
            .mappings
            .keys()
            .collect::<Vec<_>>(),
        flat_before
            .snapshot
            .design
            .mappings
            .keys()
            .collect::<Vec<_>>()
    );
    // the binding is still there and still valid (§50)
    let a = analyze_system(&s.sys.snap);
    assert!(a.composition.is_empty());
    assert_eq!(a.acceptance, Acceptance::Executable);
    assert_eq!(
        a.ports[&pr(s.lamp_a, s.lamp_in)],
        PortStatus::Bound { binding: s.bind_a }
    );
    // an unrelated instance C changes nothing for A and B (§49)
    let c = s.sys.instance(s.lamp, "lampC");
    for e in &before.entries {
        assert_eq!(
            s.sys.system().flat_ids.get(e.instance, e.local),
            Some(e.flat)
        );
    }
    assert!(s.sys.system().flat_ids.for_instance(c).count() > 0);
    let flat_c = flatten(&s.sys.snap);
    for (d, o) in &flat_before.origins.decls {
        assert_eq!(flat_c.origins.decls.get(d), Some(o));
    }
    // and the display names in the flattened design follow the renames
    let a_in = s.sys.flat_decl(s.lamp_a, s.lamp_tilt_value);
    assert_eq!(
        flat_c.snapshot.design.mappings[&a_in].name,
        "leftLamp.tiltValue"
    );
}

#[test]
fn composition_is_by_identity_never_by_name() {
    // Two components whose ports and internals share display names, and a
    // base relationship with the same name as a body relationship: every
    // binding resolves by identity; flattening never consults names.
    let mut s = vertical_slice();
    // a base mapping called `tiltValue` too
    let base_tilt_value = s
        .sys
        .base(EditOp::CreateMapping {
            name: "tiltValue".into(),
            description: String::new(),
            signature: bdl_model::surface::Signature {
                inputs: vec![],
                output: s.tilt,
            },
        })
        .inner
        .unwrap()
        .created_mapping
        .unwrap();
    s.sys.base(EditOp::SetMappingClock {
        id: base_tilt_value,
        clock: Some(s.main),
    });
    // rename the source port to collide with the lamp's required port name (allowed: different components)
    s.sys.apply(SystemEditOp::RenamePort {
        component: s.source,
        port: s.source_port,
        name: "brightness".into(),
    });
    let a = analyze_system(&s.sys.snap);
    assert!(a.composition.is_empty(), "{:?}", a.composition);
    let a_in = s.sys.flat_decl(s.lamp_a, s.lamp_tilt_value);
    let src = s.sys.flat_decl(s.sensor, s.source_tilt_value);
    assert_eq!(
        a.analysis.ir.decls[&a_in].realization,
        Some(bdl_ir::Expr::decl(src)),
        "bound to the sensor's port, not to the base `tiltValue`"
    );
    // the lamp's own formula `dimByTilt(tiltValue)` reads the lamp's port, not lampB's nor the base's
    let a_br = s.sys.flat_decl(s.lamp_a, s.lamp_brightness);
    let refs = a.analysis.ir.decls[&a_br]
        .realization
        .as_ref()
        .unwrap()
        .refs();
    assert!(refs.contains(&a_in) && !refs.contains(&base_tilt_value));
    let b_in = s.sys.flat_decl(s.lamp_b, s.lamp_tilt_value);
    assert!(!refs.contains(&b_in));
    // no flattened definition is a synthesized formula naming anything
    for m in a.flattened.snapshot.design.mappings.values() {
        if let Some(Definition::Formula { source }) = &m.definition {
            assert!(
                !a.flattened.origins.decls.contains_key(&m.id),
                "instance formulas are scoped, never plain text: {source}"
            );
        }
    }
}

#[test]
fn an_unbound_required_port_is_open_not_invalid() {
    let s = vertical_slice();
    let mut s2 = Sys {
        snap: s.sys.snap.clone(),
    };
    s2.apply(SystemEditOp::UnbindPorts { binding: s.bind_a });
    let a = analyze_system(&s2.snap);
    assert!(a.composition.is_empty(), "{:?}", a.composition);
    assert!(
        a.analysis.diagnostics.iter().all(|d| !d.is_error()),
        "{:?}",
        a.analysis.diagnostics
    );
    assert_eq!(a.ports[&pr(s.lamp_a, s.lamp_in)], PortStatus::Open);
    let a_in = s.sys.flat_decl(s.lamp_a, s.lamp_tilt_value);
    assert_eq!(a.analysis.mappings[&a_in].status, MappingStatus::Declared);
    assert_eq!(a.acceptance, Acceptance::Open);
    // exporting it makes the system executable again, with an extra input
    let export = s2
        .apply(SystemEditOp::ExportPort {
            port: pr(s.lamp_a, s.lamp_in),
            name: "leftTilt".into(),
        })
        .created_export
        .unwrap();
    let a = analyze_system(&s2.snap);
    assert_eq!(
        a.ports[&pr(s.lamp_a, s.lamp_in)],
        PortStatus::Exported { export }
    );
    assert_eq!(a.acceptance, Acceptance::Executable);
    // binding an exported port is refused; so is a second binding
    let e = s2
        .try_apply(SystemEditOp::BindPorts {
            source: pr(s.sensor, s.source_port).into(),
            destination: pr(s.lamp_a, s.lamp_in).into(),
            transport: None,
        })
        .unwrap_err();
    assert!(matches!(e, SystemEditError::PortExported { .. }));
    let e = s2
        .try_apply(SystemEditOp::BindPorts {
            source: pr(s.sensor, s.source_port).into(),
            destination: pr(s.lamp_b, s.lamp_in).into(),
            transport: None,
        })
        .unwrap_err();
    assert!(matches!(e, SystemEditError::DestinationBound { binding } if binding == s.bind_b));
}

#[test]
fn two_locally_causal_components_can_close_an_instantaneous_cycle() {
    let mut sys = Sys::new("loop");
    let level = sys.base_concept("Level", LEVEL);
    let main = sys.base_clock("main");
    // A: requires y, provides x = y + 1 ; B: requires x, provides y = x * 2
    let mk = |sys: &mut Sys, name: &str, req: &str, prov: &str, formula: &str| {
        let c = sys.component(name);
        let l = sys.body_concept(c, "Level", Some(LEVEL));
        sys.share(c, l, level);
        let tick = sys.body_clock(c, "tick");
        sys.clock_param(c, tick);
        let r = sys.body_mapping(c, req, &[], l);
        sys.body_clock_of(c, r, tick);
        let p = sys.body_mapping(c, prov, &[], l);
        sys.body_formula(c, p, formula);
        sys.body_clock_of(c, p, tick);
        let rp = sys.port(c, r, PortKind::Required, req);
        let pp = sys.port(c, p, PortKind::Provided, prov);
        (c, tick, rp, pp)
    };
    let (a, at, a_req, a_prov) = mk(&mut sys, "A", "y", "x", "y + 1");
    let (b, bt, b_req, b_prov) = mk(&mut sys, "B", "x", "y", "x * 2");
    // each alone is valid (open)
    let ia = sys.instance(a, "a");
    sys.clock_arg(ia, at, main);
    let alone = analyze_system(&sys.snap);
    assert_eq!(alone.acceptance, Acceptance::Open);
    assert!(alone.analysis.causality.valid);
    let ib = sys.instance(b, "b");
    sys.clock_arg(ib, bt, main);
    sys.bind(pr(ia, a_prov), pr(ib, b_req));
    sys.bind(pr(ib, b_prov), pr(ia, a_req));
    let an = analyze_system(&sys.snap);
    assert!(an.composition.is_empty());
    assert!(
        !an.analysis.causality.valid,
        "the existing causality analysis rejects the composed loop"
    );
    assert_eq!(an.acceptance, Acceptance::Invalid);
    let cyc: Vec<_> = an
        .projected
        .iter()
        .filter(|p| p.diagnostic.code.as_str() == "reactive.instantaneous_cycle")
        .collect();
    assert!(cyc.len() >= 2);
    let insts: std::collections::BTreeSet<_> = cyc
        .iter()
        .filter_map(|p| p.origin.as_ref().map(|o| o.instance))
        .collect();
    assert_eq!(
        insts,
        [ia, ib].into_iter().collect(),
        "projected to both instances"
    );
    assert!(
        cyc.iter().any(|p| p.port.is_some()),
        "and to the ports the bindings close"
    );
    // breaking the loop with a transported binding makes it causal again
    let bind_back = an.flattened.origins.decl_of_port(pr(ia, a_req)).unwrap();
    let _ = bind_back;
    let mut s2 = Sys {
        snap: sys.snap.clone(),
    };
    let b2 = s2
        .system()
        .bindings
        .values()
        .find(|b| b.destination == pr(ia, a_req).into())
        .unwrap()
        .id;
    s2.apply(SystemEditOp::UnbindPorts { binding: b2 });
    s2.bind_transported(pr(ib, b_prov), pr(ia, a_req), "0");
    let fixed = analyze_system(&s2.snap);
    assert!(fixed.analysis.causality.valid, "{:?}", fixed.projected);
    assert_eq!(fixed.acceptance, Acceptance::Executable);
}

#[test]
fn cross_domain_binding_needs_transport_and_keeps_strictly_before() {
    let mut s = vertical_slice();
    let slow = s.sys.base_clock("slow");
    s.sys.clock_arg(s.lamp_b, s.lamp_tick, slow);
    // direct binding across domains: not clock-consistent, said in system terms
    let a = analyze_system(&s.sys.snap);
    assert!(
        a.composition
            .iter()
            .any(|d| d.code.as_str() == "system.binding_needs_transport"
                && d.message.contains("lampB.tiltValue")),
        "{:?}",
        a.composition
    );
    assert!(!a.analysis.clocks.valid);
    assert_eq!(a.acceptance, Acceptance::Invalid);
    // transported binding: consistent, and simulation keeps the sync rule
    s.sys.apply(SystemEditOp::UnbindPorts { binding: s.bind_b });
    s.sys.bind_transported(
        pr(s.sensor, s.source_port),
        pr(s.lamp_b, s.lamp_in),
        "45 deg",
    );
    let a = analyze_system(&s.sys.snap);
    assert!(a.composition.is_empty(), "{:?}", a.composition);
    assert!(a.analysis.clocks.valid && a.analysis.causality.valid);
    assert_eq!(a.acceptance, Acceptance::Executable, "{:?}", a.projected);
    let b_in = s.sys.flat_decl(s.lamp_b, s.lamp_tilt_value);
    assert!(
        matches!(a.analysis.ir.decls[&b_in].realization, Some(bdl_ir::Expr::Sync { src, .. }) if src == s.main)
    );
    // both domains every tick: lampB sees the sensor's PREVIOUS activation
    let raw = s.sys.flat_decl(s.sensor, s.source_raw);
    let (trace, _) = simulate(&a.flattened.snapshot, raw, s.tilt);
    let ba = levels(&trace, s.sys.flat_decl(s.lamp_a, s.lamp_brightness));
    let bb = levels(&trace, s.sys.flat_decl(s.lamp_b, s.lamp_brightness));
    assert_eq!(ba[..3], [0.0, ba[1], ba[2]]);
    assert_eq!(
        bb[0], 0.5,
        "the init: 45 deg / 90 deg, before any sensor activation"
    );
    assert_eq!(
        bb[1..],
        ba[..3],
        "one activation behind, never the same tick"
    );
}

#[test]
fn two_instances_driving_one_shared_sink_hit_the_existing_single_driver_rule() {
    let mut sys = Sys::new("conflict");
    let level = sys.base_concept("Level", LEVEL);
    let main = sys.base_clock("main");
    let sink = sys.base_output("lamp", level, main);
    let c = sys.component("Driver");
    let l = sys.body_concept(c, "Level", Some(LEVEL));
    sys.share(c, l, level);
    let tick = sys.body_clock(c, "tick");
    sys.clock_param(c, tick);
    let out = sys.body_output(c, "lamp", l, tick);
    sys.apply(SystemEditOp::ExternalizeOutput {
        component: c,
        local: out,
        system: Some(sink),
    });
    let v = sys.body_mapping(c, "level", &[], l);
    sys.body_formula(c, v, "0.5");
    sys.body_clock_of(c, v, tick);
    sys.body_drive(c, v, out);
    let i1 = sys.instance(c, "first");
    sys.clock_arg(i1, tick, main);
    let one = analyze_system(&sys.snap);
    assert_eq!(
        one.acceptance,
        Acceptance::Executable,
        "{:?}",
        one.projected
    );
    let i2 = sys.instance(c, "second");
    sys.clock_arg(i2, tick, main);
    let two = analyze_system(&sys.snap);
    assert_eq!(two.acceptance, Acceptance::Invalid);
    let md: Vec<_> = two
        .projected
        .iter()
        .filter(|p| p.diagnostic.code.as_str() == "output.multiple_drivers")
        .collect();
    assert_eq!(md.len(), 2);
    let insts: std::collections::BTreeSet<_> = md
        .iter()
        .map(|p| p.origin.as_ref().unwrap().instance)
        .collect();
    assert_eq!(insts, [i1, i2].into_iter().collect());
    assert!(
        two.composition.is_empty(),
        "no system-specific arbitration rule"
    );
    // the external sink is the system's, not freshened
    assert_eq!(two.flattened.snapshot.design.outputs.len(), 1);
}

#[test]
fn private_devices_multiply_hardware_requirements() {
    let s = vertical_slice();
    let f = flatten(&s.sys.snap);
    assert_eq!(f.snapshot.design.devices.len(), 2);
    assert_eq!(f.snapshot.design.outputs.len(), 2);
    let nano = bdl_hardware::boards::arduino_nano();
    let d = bdl_compiler::analyze_deployment(&f.snapshot, &nano);
    assert_eq!(d.requirements.len(), 2);
    assert_eq!(d.status, bdl_compiler::DeploymentStatus::Feasible);
    let pins: Vec<_> = d
        .assignment
        .as_ref()
        .unwrap()
        .values()
        .map(|r| r.0.clone())
        .collect();
    assert_eq!(pins, ["D3", "D5"]);
    // and the read model names the instances' devices
    let a = bdl_compiler::analyze(&f.snapshot);
    let report = bdl_compiler::deployment_report(&f.snapshot, &a, &d, &nano);
    let names: Vec<_> = report.rows.iter().map(|r| r.device_name.clone()).collect();
    assert_eq!(names, ["lampA.led", "lampB.led"]);
    assert!(report.deployable);
}

#[test]
fn system_and_hand_written_flat_agree_on_every_observable() {
    let s = vertical_slice();
    let f = flatten(&s.sys.snap);
    let flat = hand_written_flat();
    let raw_s = s.sys.flat_decl(s.sensor, s.source_raw);
    let tilt_flat = flat
        .snap
        .design
        .concepts
        .values()
        .find(|c| c.name == "Tilt")
        .unwrap()
        .id;
    let (ts, a_s) = simulate(&f.snapshot, raw_s, s.tilt);
    let (tf, a_f) = simulate(&flat.snap, flat.raw, tilt_flat);
    // analysis: same ladder, same verdicts, same output completeness
    let statuses = |a: &ProjectAnalysis| {
        let mut v: Vec<MappingStatus> = a.mappings.values().map(|m| m.status).collect();
        v.sort();
        v
    };
    assert_eq!(statuses(&a_s), statuses(&a_f));
    assert_eq!(
        (a_s.causality.valid, a_s.clocks.valid, a_s.output_complete),
        (a_f.causality.valid, a_f.clocks.valid, a_f.output_complete)
    );
    assert!(a_s.diagnostics.is_empty() && a_f.diagnostics.is_empty());
    // traces: brightness A/B tick for tick
    let ba_s = levels(&ts, s.sys.flat_decl(s.lamp_a, s.lamp_brightness));
    let bb_s = levels(&ts, s.sys.flat_decl(s.lamp_b, s.lamp_brightness));
    assert_eq!(ba_s, levels(&tf, flat.brightness_a));
    assert_eq!(bb_s, levels(&tf, flat.brightness_b));
    // outputs: the sinks receive the same values
    let outs = |a: &ProjectAnalysis, trace: &SimulationTrace| -> Vec<Vec<f64>> {
        trace
            .ticks
            .iter()
            .map(|t| {
                bdl_output::output_values(t, &a.outputs.valid_bindings)
                    .values()
                    .map(|v| v.unwrap_semantic().unwrap().as_quantity().unwrap().1)
                    .collect()
            })
            .collect()
    };
    assert_eq!(outs(&a_s, &ts), outs(&a_f, &tf));
    // the backend: both compile, and the generated programs agree on the host
    let opts = bdl_compiler::CompileOptions {
        require_complete: true,
        ..Default::default()
    };
    let art_s = bdl_compiler::compile(&f.snapshot, &opts);
    let art_f = bdl_compiler::compile(&flat.snap, &opts);
    assert!(art_s.succeeded(), "{:?}", art_s.diagnostics);
    assert!(art_f.succeeded(), "{:?}", art_f.diagnostics);
    let es = art_s.exec_ir.as_ref().unwrap();
    let ef = art_f.exec_ir.as_ref().unwrap();
    assert_eq!(
        (
            es.cells.len(),
            es.inputs.len(),
            es.outputs.len(),
            es.clocks.len()
        ),
        (
            ef.cells.len(),
            ef.inputs.len(),
            ef.outputs.len(),
            ef.clocks.len()
        )
    );
    // the exec-IR interpreter (the in-process leg of the backend's differential tests)
    let run = |exec: &bdl_exec_ir::ExecIr, tilt| -> Vec<Vec<Option<f64>>> {
        let inputs_at = |t: u64| -> Vec<Option<Value>> {
            exec.inputs
                .iter()
                .map(|_| Some(deg(tilt, [0.0, 30.0, 60.0, 90.0][t as usize])))
                .collect()
        };
        let active: Vec<_> = exec.clocks.iter().map(|c| c.slot).collect();
        let (ticks, err) = bdl_exec_ir::interp::run(exec, 4, &|_| active.clone(), &inputs_at);
        assert!(err.is_none());
        ticks
            .iter()
            .map(|t| {
                t.outputs
                    .iter()
                    .map(|o| {
                        o.as_ref()
                            .map(|v| v.unwrap_semantic().unwrap().as_quantity().unwrap().1)
                    })
                    .collect()
            })
            .collect()
    };
    assert_eq!(run(es, s.tilt), run(ef, tilt_flat));
}

#[test]
fn packaging_a_system_makes_a_component_that_instantiates_again() {
    let s = vertical_slice();
    let mut s2 = Sys {
        snap: s.sys.snap.clone(),
    };
    // expose lampA.brightness as a provided port of the package, main as a clock parameter
    let pkg = package_system(
        &s2.snap,
        "TwinLamps",
        PackageInterface {
            ports: vec![package::PackagePort {
                decl: package::PackageDecl::Port(pr(s.lamp_a, s.lamp_out)),
                kind: PortKind::Provided,
                name: "leftBrightness".into(),
            }],
            clock_params: vec![s.main],
            shared_concepts: BTreeMap::new(),
            external_outputs: BTreeMap::new(),
        },
    )
    .unwrap();
    assert_eq!(
        pkg.body.mappings.len(),
        s2.system().base.mappings.len() + 2 + 3 * 2
    );
    // install it into a fresh outer system and instantiate it twice
    let mut outer = Sys::new("hall");
    let hall = outer.base_clock("hall");
    let c = outer
        .apply(SystemEditOp::InstallComponent {
            component: Box::new(pkg),
        })
        .created_component
        .unwrap();
    let port = *outer.system().components[&c]
        .interface
        .ports
        .keys()
        .next()
        .unwrap();
    let i1 = outer.instance(c, "west");
    outer.clock_arg(i1, s.main, hall);
    let i2 = outer.instance(c, "east");
    outer.clock_arg(i2, s.main, hall);
    let a = analyze_system(&outer.snap);
    assert!(a.composition.is_empty(), "{:?}", a.composition);
    assert_eq!(a.acceptance, Acceptance::Executable, "{:?}", a.projected);
    assert_eq!(a.ports[&pr(i1, port)], PortStatus::Provided);
    // two packages: two sensors, four lamps, four devices, all distinct
    assert_eq!(a.flattened.snapshot.design.devices.len(), 4);
    assert_eq!(
        a.flattened
            .snapshot
            .design
            .mappings
            .values()
            .filter(|m| m.name.ends_with(".brightness"))
            .count(),
        4
    );
    assert_eq!(
        a.flattened
            .snapshot
            .design
            .mappings
            .values()
            .filter(|m| m.definition.is_none())
            .count(),
        2,
        "two sensor readings"
    );
    // a package whose chosen required port is not open is refused
    let bad = package_system(
        &s2.snap,
        "Bad",
        PackageInterface {
            ports: vec![package::PackagePort {
                decl: package::PackageDecl::Port(pr(s.lamp_a, s.lamp_in)),
                kind: PortKind::Required,
                name: "tilt".into(),
            }],
            ..Default::default()
        },
    );
    assert!(matches!(bad, Err(PackageError::PortShape { .. })));
    let _ = &mut s2;
}

#[test]
fn a_flat_design_is_the_degenerate_system() {
    let flat = hand_written_flat();
    let sys = SystemSnapshot::new(BehaviorSystem::from_flat(flat.snap.design.clone()));
    assert!(sys.system.is_flat());
    let f = flatten(&sys);
    assert_eq!(f.snapshot.design, flat.snap.design);
    assert!(f.origins.decls.is_empty() && f.diagnostics.is_empty());
    let a = analyze_system(&sys);
    let mut expected = bdl_compiler::analyze(&flat.snap);
    expected.revision = sys.revision;
    assert_eq!(a.analysis, expected);
    assert_eq!(a.acceptance, Acceptance::Executable);
}

#[test]
fn system_projects_persist_the_source_and_derive_the_flat_design() {
    let s = vertical_slice();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("rover");
    persist::save_system_project(&root, &s.sys.snap, &Default::default(), "test").unwrap();
    assert!(root.join("design/system.bdl.json").is_file());
    assert!(
        !root.join("design/project.bdl.json").exists(),
        "the flat design is derived, never written"
    );
    let manifest = bdl_model::persist::read_manifest(&root).unwrap();
    // a legacy system project (manifest schema 1), as the migration reads it
    assert_eq!(
        manifest.legacy_kind(),
        Some(bdl_model::persist::ProjectKind::System)
    );
    let loaded = persist::load_system_project(&root).unwrap();
    assert_eq!(loaded.snapshot.system, s.sys.snap.system);
    assert_eq!(
        flatten(&loaded.snapshot).snapshot.design,
        flatten(&s.sys.snap).snapshot.design
    );
    // the flat loader refuses it with a clear reason; a flat project is untouched by all this
    assert!(matches!(
        bdl_model::persist::load_project(&root),
        Err(bdl_model::persist::PersistError::NotFlat { .. })
    ));
    let flat_root = dir.path().join("flat");
    let created = bdl_model::persist::init_project(&flat_root, "plain", "test").unwrap();
    assert_eq!(
        bdl_model::persist::read_manifest(&flat_root)
            .unwrap()
            .legacy_kind(),
        Some(bdl_model::persist::ProjectKind::Flat)
    );
    assert!(persist::load_system_project(&flat_root).is_err());
    assert_eq!(
        bdl_model::persist::load_project(&flat_root)
            .unwrap()
            .snapshot,
        created.snapshot
    );
    // the system file round-trips through serde exactly
    let json = serde_json::to_string(&s.sys.snap.system).unwrap();
    assert_eq!(
        serde_json::from_str::<BehaviorSystem>(&json).unwrap(),
        s.sys.snap.system
    );
}

#[test]
fn edits_invalidate_only_what_they_touch() {
    let mut s = vertical_slice();
    // a body edit of AdaptiveLamp reaches lampA and lampB, not the sensor
    let o = s.sys.body(
        s.lamp,
        EditOp::ReplaceDefinition {
            id: s.lamp_dim,
            definition: Some(Definition::Formula {
                source: "Tilt / 45 deg".into(),
            }),
        },
    );
    assert_eq!(o.instances, [s.lamp_a, s.lamp_b].into_iter().collect());
    assert!(
        o.origin_decls
            .contains(&s.sys.flat_decl(s.lamp_a, s.lamp_dim))
            && o.origin_decls
                .contains(&s.sys.flat_decl(s.lamp_b, s.lamp_dim))
    );
    assert!(!o
        .origin_decls
        .contains(&s.sys.flat_decl(s.sensor, s.source_tilt_value)));
    // a rename is a refinement with nothing invalidated
    let o = s.sys.apply(SystemEditOp::RenameInstance {
        id: s.lamp_a,
        name: "L".into(),
    });
    assert_eq!(o.kind, Some(bdl_model::edit::EditKind::Refinement));
    assert!(o.invalidates.is_empty());
    // a clock argument change is a clock invalidation of one instance
    let slow = s.sys.base_clock("slow");
    let o = s.sys.apply(SystemEditOp::SetClockArgument {
        instance: s.lamp_b,
        parameter: s.lamp_tick,
        clock: Some(slow),
    });
    assert_eq!(o.instances, [s.lamp_b].into_iter().collect());
    assert!(o
        .invalidates
        .contains(&bdl_model::edit::Invalidation::Clock));
    // deleting an instantiated component / a bound instance is refused
    assert!(matches!(
        s.sys
            .try_apply(SystemEditOp::DeleteComponent { id: s.lamp }),
        Err(SystemEditError::ComponentInUse { .. })
    ));
    assert!(matches!(
        s.sys
            .try_apply(SystemEditOp::DeleteInstance { id: s.lamp_a }),
        Err(SystemEditError::InstanceInUse { .. })
    ));
    // a body edit that would remove a port's declaration is refused
    assert!(matches!(
        s.sys.try_apply(SystemEditOp::EditComponentBody {
            component: s.lamp,
            op: EditOp::DeleteMapping {
                id: s.lamp_brightness
            }
        }),
        Err(SystemEditError::PortBacked { .. })
    ));
    // a parameter: a closed value realises it; a name does not
    let gain_c = s.sys.body_concept(s.lamp, "Gain", Some(LEVEL));
    let gain = s.sys.body_mapping(s.lamp, "gain", &[], gain_c);
    let gain_port = s.sys.port(s.lamp, gain, PortKind::Parameter, "gain");
    s.sys.apply(SystemEditOp::SetParameterArgument {
        instance: s.lamp_a,
        port: gain_port,
        value: Some(ParameterValue { source: "2".into() }),
    });
    s.sys.apply(SystemEditOp::SetParameterArgument {
        instance: s.lamp_b,
        port: gain_port,
        value: Some(ParameterValue {
            source: "tiltValue".into(),
        }),
    });
    let a = analyze_system(&s.sys.snap);
    let ga = s.sys.flat_decl(s.lamp_a, gain);
    let gb = s.sys.flat_decl(s.lamp_b, gain);
    assert_eq!(
        a.analysis.mappings[&ga].status,
        MappingStatus::ClockConsistent
    );
    assert_eq!(
        a.analysis.mappings[&gb].status,
        MappingStatus::Invalid,
        "a parameter value cannot name a relationship"
    );
    assert_eq!(a.ports[&pr(s.lamp_a, gain_port)], PortStatus::Valued);
    let _ = DeviceKind::PwmChannel;
}

/// The generated Rust programs of the system and of the hand-written flat
/// design agree on the host, tick for tick (brief §57, "where practical").
#[test]
fn generated_programs_of_system_and_flat_agree_on_the_host() {
    use bdl_runtime_host::harness::{write_crate, Cargo};
    use bdl_runtime_host::{DynValue, RunRequest, TickRequest};
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap();
    let opts = bdl_compiler::CompileOptions {
        require_complete: true,
        codegen: bdl_codegen_rust::CodegenOptions {
            runtime_core_path: root.join("runtime/bdl-runtime-core").display().to_string(),
            runtime_host_path: root.join("runtime/bdl-runtime-host").display().to_string(),
        },
        ..Default::default()
    };
    let s = vertical_slice();
    let f = flatten(&s.sys.snap);
    let flat = hand_written_flat();
    let run = |snapshot: &bdl_model::surface::ProjectSnapshot,
               name: &str,
               tilt: bdl_model::SemanticId|
     -> Vec<Vec<Option<DynValue>>> {
        let art = bdl_compiler::compile(snapshot, &opts);
        assert!(art.succeeded(), "{name}: {:?}", art.diagnostics);
        let g = art.generated.as_ref().unwrap();
        let exec = art.exec_ir.as_ref().unwrap();
        let dir = root.join("target/bdl-generated").join(name);
        write_crate(&dir, &g.files).unwrap();
        let cargo = Cargo::new(dir, root.join("target/bdl-generated/target"));
        let req = RunRequest {
            ticks: (0..4)
                .map(|t| TickRequest {
                    active: exec.clocks.iter().map(|c| c.slot.0).collect(),
                    inputs: exec
                        .inputs
                        .iter()
                        .map(|_| {
                            Some(DynValue::sem(
                                tilt.raw(),
                                DynValue::Quantity {
                                    value: [0.0f64, 30.0, 60.0, 90.0][t].to_radians(),
                                },
                            ))
                        })
                        .collect(),
                })
                .collect(),
        };
        let trace = cargo
            .run_host(&req)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(trace.error.is_none());
        // outputs carry the (intentionally different) concept ids: compare the levels
        trace
            .ticks
            .iter()
            .map(|t| {
                t.outputs
                    .iter()
                    .map(|o| match o {
                        Some(DynValue::Semantic { repr, .. }) => Some((**repr).clone()),
                        other => other.clone(),
                    })
                    .collect()
            })
            .collect()
    };
    let tilt_flat = flat
        .snap
        .design
        .concepts
        .values()
        .find(|c| c.name == "Tilt")
        .unwrap()
        .id;
    let a = run(&f.snapshot, "system_rover", s.tilt);
    let b = run(&flat.snap, "flat_rover", tilt_flat);
    assert_eq!(a, b);
    assert_eq!(
        a[3],
        vec![
            Some(DynValue::Quantity { value: 1.0 }),
            Some(DynValue::Quantity { value: 1.0 })
        ]
    );
}
