//! Behaviour grouping and "Package as reusable component" (FV Phase 8b;
//! brief §70–§78): groups are transparent, boundaries are projections,
//! extraction is a differential witness of Theorem R.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_model::edit::EditOp;
use bdl_model::surface::{Definition, DeviceKind};
use bdl_model::{ClockId, DeclId, Dim, OutputId, SemanticId};
use bdl_reactive::{InputTrace, Schedule, Simulation, SimulationTrace, Value};
use bdl_system::*;
use support::*;

/// A top-level design with one lamp behaviour worth packaging:
///
/// ```text
/// raw (open) ─▶ tiltValue ─▶ [ dimByTilt, brightness ] ─▶ indicator
///                                      │ drives                │ drives
///                                    light                   status
/// ```
struct Lamp {
    sys: Sys,
    tilt: SemanticId,
    level: SemanticId,
    main: ClockId,
    raw: DeclId,
    tilt_value: DeclId,
    dim: DeclId,
    brightness: DeclId,
    indicator: DeclId,
    light: OutputId,
}

fn lamp() -> Lamp {
    let mut sys = Sys::new("lamp");
    let tilt = sys.base_concept("Tilt", ANGLE);
    let level = sys.base_concept("Brightness", LEVEL);
    let main = sys.base_clock("main");
    let raw = sys.base_mapping("raw", &[], tilt);
    sys.base_clock_of(raw, main);
    let tilt_value = sys.base_mapping("tiltValue", &[], tilt);
    sys.base_formula(tilt_value, "raw");
    sys.base_clock_of(tilt_value, main);
    let dim = sys.base_mapping("dimByTilt", &[tilt], level);
    sys.base_formula(dim, "Tilt / 90 deg");
    let brightness = sys.base_mapping("brightness", &[], level);
    sys.base_formula(brightness, "dimByTilt(tiltValue)");
    sys.base_clock_of(brightness, main);
    let light = sys.base_output("light", level, main);
    sys.base_drive(brightness, light);
    sys.base_device("led", DeviceKind::PwmChannel, light);
    let indicator = sys.base_mapping("indicator", &[], level);
    sys.base_formula(indicator, "brightness");
    sys.base_clock_of(indicator, main);
    let status = sys.base_output("status", level, main);
    sys.base_drive(indicator, status);
    sys.base_device("statusLed", DeviceKind::PwmChannel, status);
    let _ = status;
    Lamp {
        sys,
        tilt,
        level,
        main,
        raw,
        tilt_value,
        dim,
        brightness,
        indicator,
        light,
    }
}

fn deg(tilt: SemanticId, x: f64) -> Value {
    Value::sem(tilt, Value::q(Dim::ANGLE, x.to_radians()))
}

fn simulate(snapshot: &SystemSnapshot, raw: DeclId, tilt: SemanticId) -> SimulationTrace {
    let a = analyze_system(snapshot);
    assert!(a.analysis.causality.valid, "{:?}", a.analysis.diagnostics);
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
    let ir = a.analysis.ir.clone();
    let mut sim = Simulation::new(
        ir.clone(),
        &a.analysis.causality,
        Schedule::always(&ir),
        inputs,
    )
    .unwrap();
    sim.run(4).unwrap().clone()
}

fn levels(trace: &SimulationTrace, decl: DeclId) -> Vec<f64> {
    trace
        .series(decl)
        .iter()
        .map(|(_, v)| v.unwrap_semantic().unwrap().as_quantity().unwrap().1)
        .collect()
}

fn ids(v: &[DeclId]) -> Vec<u64> {
    v.iter().map(|d| d.raw()).collect()
}

// §70 — every group operation is invisible to every judgment.
#[test]
fn grouping_is_semantically_transparent() {
    let mut l = lamp();
    let before = analyze_system(&l.sys.snap);
    let trace_before = simulate(&l.sys.snap, l.raw, l.tilt);
    let nano = bdl_hardware::boards::arduino_nano();
    let deploy_before = bdl_compiler::analyze_deployment(&before.flattened.snapshot, &nano);
    let revision = l.sys.snap.revision;

    let g = l.sys.create_group("Lamp", &[l.dim, l.brightness]);
    let g2 = l.sys.create_group("Sensing", &[l.tilt_value]);
    l.sys.group(GroupEditOp::AddMember {
        group: g2,
        decl: l.raw,
    });
    l.sys.group(GroupEditOp::MoveMember {
        decl: l.tilt_value,
        to: g,
    });
    l.sys.group(GroupEditOp::RenameGroup {
        id: g,
        name: "Adaptive lamp".into(),
    });
    l.sys.group(GroupEditOp::SetGroupDescription {
        id: g,
        description: "dims with tilt".into(),
    });
    let split = l
        .sys
        .group(GroupEditOp::SplitGroup {
            id: g,
            name: "Dimming".into(),
            members: vec![l.dim],
        })
        .created_group
        .unwrap();
    l.sys.group(GroupEditOp::MergeGroups {
        into: g,
        from: split,
    });
    l.sys.group(GroupEditOp::RemoveMember {
        group: g,
        decl: l.tilt_value,
    });
    let after = analyze_system(&l.sys.snap);

    // no revision, no re-derivation: the flat design is the same value
    assert_eq!(l.sys.snap.revision, revision);
    assert_eq!(
        after.flattened.snapshot.design,
        before.flattened.snapshot.design
    );
    assert_eq!(after.analysis.diagnostics, before.analysis.diagnostics);
    assert_eq!(after.analysis.dependencies, before.analysis.dependencies);
    assert_eq!(after.analysis.clocks, before.analysis.clocks);
    assert_eq!(after.analysis.outputs, before.analysis.outputs);
    assert_eq!(after.acceptance, before.acceptance);
    let trace_after = simulate(&l.sys.snap, l.raw, l.tilt);
    assert_eq!(
        levels(&trace_after, l.indicator),
        levels(&trace_before, l.indicator)
    );
    let deploy_after = bdl_compiler::analyze_deployment(&after.flattened.snapshot, &nano);
    assert_eq!(deploy_after.requirements, deploy_before.requirements);
    assert_eq!(deploy_after.status, deploy_before.status);

    // the group table is what changed, and only it
    let groups = &l.sys.system().groups;
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[&g].name, "Adaptive lamp");
    assert_eq!(ids(&groups[&g].members), ids(&[l.brightness, l.dim]));
    assert_eq!(ids(&groups[&g2].members), ids(&[l.raw]));
    assert!(after.groups.contains_key(&g));

    // ungroup: the members stay
    l.sys.group(GroupEditOp::DeleteGroup { id: g });
    assert!(l.sys.system().base.mappings.contains_key(&l.brightness));
    assert_eq!(
        analyze_system(&l.sys.snap).flattened.snapshot.design,
        before.flattened.snapshot.design
    );

    // a relationship is in at most one group
    let e = l
        .sys
        .try_group(GroupEditOp::AddMember {
            group: g2,
            decl: l.raw,
        })
        .map(|_| ());
    assert_eq!(e, Ok(()));
    let g3 = l.sys.create_group("Other", &[]);
    let e = l
        .sys
        .try_group(GroupEditOp::AddMember {
            group: g3,
            decl: l.raw,
        })
        .unwrap_err();
    assert!(matches!(e, GroupEditError::AlreadyGrouped { group, .. } if group == g2));
}

// §71 — aggregate sockets are a projection: no fan-out edge.
#[test]
fn boundary_is_a_projection_and_sockets_add_no_dependency() {
    let mut l = lamp();
    let g = l.sys.create_group("Lamp", &[l.dim, l.brightness]);
    let a = analyze_system(&l.sys.snap);
    let b = &a.groups[&g];
    assert_eq!(ids(&b.crossing_in), ids(&[l.tilt_value]));
    assert_eq!(ids(&b.crossing_out), ids(&[l.brightness]));
    assert_eq!(ids(&b.external_inputs), ids(&[l.tilt_value]));
    assert_eq!(ids(&b.external_outputs), ids(&[l.brightness]));
    assert_eq!(ids(&b.open_members), ids(&[]));
    assert_eq!(ids(&b.driven_members), ids(&[l.brightness]));
    assert_eq!(ids(&b.private_candidates), ids(&[l.dim]));
    assert_eq!(b.clocks, vec![l.main]);
    assert_eq!(b.internal_edges, vec![(l.brightness, l.dim)]);
    // Theorem H: tiltValue is on the group's input socket because
    // `brightness` reads it; `dimByTilt` still does not.
    let deps = &a.analysis.dependencies;
    assert!(deps.depends_on(l.brightness, l.tilt_value));
    assert!(!deps.depends_on(l.dim, l.tilt_value));
}

// §72 — several crossing-out members are several ports, never a tuple.
#[test]
fn several_external_outputs_are_several_provided_ports() {
    let mut l = lamp();
    let mirror = l.sys.base_mapping("mirror", &[], l.tilt);
    l.sys.base_formula(mirror, "tiltValue");
    l.sys.base_clock_of(mirror, l.main);
    let g = l
        .sys
        .create_group("Lamp", &[l.tilt_value, l.dim, l.brightness]);
    let p = preview_extraction(&l.sys.snap, g, &ExtractionChoices::default()).unwrap();
    let provided: Vec<_> = p.provided.iter().map(|x| x.name.as_str()).collect();
    assert_eq!(provided, ["tiltValue", "brightness"]);
    let required: Vec<_> = p.required.iter().map(|x| x.name.as_str()).collect();
    assert_eq!(required, ["raw"]);
    l.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g,
        choices: ExtractionChoices::default(),
    });
    let c = l.sys.system().components.values().next().unwrap();
    assert_eq!(c.interface.ports_of(PortKind::Provided).count(), 2);
    assert_eq!(c.interface.ports_of(PortKind::Required).count(), 1);
    // the base keeps two open copies, each realised by its own binding
    let base = &l.sys.system().base;
    assert!(base.mappings[&l.tilt_value].definition.is_none());
    assert!(base.mappings[&l.brightness].definition.is_none());
    assert!(!base.mappings.contains_key(&l.dim));
    assert_eq!(l.sys.system().bindings.len(), 3);
}

// §73 — an open member is a decision the designer makes, exposed by default.
#[test]
fn an_open_member_is_an_input_by_default_or_stays_open_inside() {
    let mut l = lamp();
    let g = l.sys.create_group("Sensing", &[l.raw, l.tilt_value]);
    let p = preview_extraction(&l.sys.snap, g, &ExtractionChoices::default()).unwrap();
    assert_eq!(ids(&p.boundary.open_members), ids(&[l.raw]));
    assert_eq!(p.open_members.len(), 1);
    assert!(p.open_members[0].as_input);
    assert_eq!(
        p.required.iter().map(|x| x.decl).collect::<Vec<_>>(),
        vec![l.raw]
    );
    assert!(p.warnings.is_empty());

    let keep = ExtractionChoices {
        keep_internal: vec![l.raw],
        ..Default::default()
    };
    let p2 = preview_extraction(&l.sys.snap, g, &keep).unwrap();
    assert!(p2.required.is_empty());
    assert!(p2
        .warnings
        .iter()
        .any(|w| w.code.as_str() == "extract.open_member_internal"));
    assert_eq!(
        preview_extraction(
            &l.sys.snap,
            g,
            &ExtractionChoices {
                keep_internal: vec![l.dim],
                ..Default::default()
            }
        )
        .unwrap_err(),
        ExtractError::NotAnOpenMember { decl: l.dim }
    );

    // extracted with the input exposed: the port is open, the system is
    // Open (not invalid), and the flat input is the instance's port.
    l.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g,
        choices: ExtractionChoices::default(),
    });
    let a = analyze_system(&l.sys.snap);
    assert_eq!(a.acceptance, Acceptance::Open);
    let inst = *l.sys.system().instances.keys().next().unwrap();
    let raw_port = l
        .sys
        .system()
        .component_of(inst)
        .unwrap()
        .interface
        .port_for_decl(l.raw)
        .unwrap()
        .id;
    assert_eq!(a.ports[&pr(inst, raw_port)], PortStatus::Open);
}

// §74 — a physical sink is never turned into a port.
#[test]
fn a_driven_member_keeps_its_drive_and_the_sink_stays_external() {
    let mut l = lamp();
    let g = l.sys.create_group("Lamp", &[l.dim, l.brightness]);
    let p = preview_extraction(&l.sys.snap, g, &ExtractionChoices::default()).unwrap();
    assert_eq!(p.sinks.len(), 1);
    assert_eq!(p.sinks[0].output, l.light);
    assert!(!p.sinks[0].internal);
    l.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g,
        choices: ExtractionChoices::default(),
    });
    let s = l.sys.system();
    let c = s.components.values().next().unwrap();
    assert_eq!(c.external_outputs.get(&l.light), Some(&l.light));
    assert!(c.body.mappings[&l.brightness].drives == Some(l.light));
    assert!(s.base.outputs.contains_key(&l.light));
    assert_eq!(s.base.devices.len(), 2);
    // the base copy does not drive: exactly one driver in the flat design
    assert!(s.base.mappings[&l.brightness].drives.is_none());
    let a = analyze_system(&l.sys.snap);
    assert!(
        a.analysis.outputs.conflicts.is_empty(),
        "{:?}",
        a.analysis.diagnostics
    );
    // `raw` is an ordinary open input of the design, not an unbound port
    assert_eq!(a.acceptance, Acceptance::Executable);

    // internalising moves the sink and its device into the component
    let mut l2 = lamp();
    let g2 = l2.sys.create_group("Lamp", &[l2.dim, l2.brightness]);
    l2.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g2,
        choices: ExtractionChoices {
            internalize_sinks: vec![l2.light],
            ..Default::default()
        },
    });
    let s2 = l2.sys.system();
    let c2 = s2.components.values().next().unwrap();
    assert!(c2.external_outputs.is_empty());
    assert!(c2.body.outputs.contains_key(&l2.light));
    assert_eq!(c2.body.devices.len(), 1);
    assert!(!s2.base.outputs.contains_key(&l2.light));
    assert_eq!(s2.base.devices.len(), 1);
    let f = flatten(&l2.sys.snap);
    assert_eq!(f.snapshot.design.outputs.len(), 2);
    assert_eq!(f.snapshot.design.devices.len(), 2);
}

// §75 — Theorem R witnessed: the packaged system computes the same values.
#[test]
fn extraction_is_a_differential_witness_of_theorem_r() {
    let mut l = lamp();
    let before = simulate(&l.sys.snap, l.raw, l.tilt);
    let nano = bdl_hardware::boards::arduino_nano();
    let a0 = analyze_system(&l.sys.snap);
    let d0 = bdl_compiler::analyze_deployment(&a0.flattened.snapshot, &nano);

    let g = l.sys.create_group("AdaptiveLamp", &[l.dim, l.brightness]);
    let o = l.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g,
        choices: ExtractionChoices::default(),
    });
    let inst = o.created_instance.unwrap();
    let s = l.sys.system();
    assert!(s.groups.is_empty());
    assert_eq!(s.instances[&inst].name, "adaptiveLamp");
    assert_eq!(s.instances[&inst].clock_bindings[&l.main], l.main);
    let c = &s.components[&o.created_component.unwrap()];
    assert_eq!(c.name, "AdaptiveLamp");
    assert_eq!(c.interface.clock_params, vec![l.main]);
    assert_eq!(c.shared_concepts.len(), 2);
    // the crossing-in copy is unresolved inside; the member is unchanged
    assert!(c.body.mappings[&l.tilt_value].definition.is_none());
    assert_eq!(
        c.body.mappings[&l.brightness].definition,
        Some(Definition::Formula {
            source: "dimByTilt(tiltValue)".into()
        })
    );

    // reconnection: base tiltValue → port, port → base brightness (open)
    let mut ends: Vec<(BindingEnd, BindingEnd)> = s
        .bindings
        .values()
        .map(|b| (b.source, b.destination))
        .collect();
    ends.sort();
    let req = c.interface.port_for_decl(l.tilt_value).unwrap().id;
    let prov = c.interface.port_for_decl(l.brightness).unwrap().id;
    assert_eq!(
        ends,
        vec![
            (
                BindingEnd::port(inst, prov),
                BindingEnd::Base { decl: l.brightness }
            ),
            (
                BindingEnd::Base { decl: l.tilt_value },
                BindingEnd::port(inst, req)
            ),
        ]
    );

    // the flat design: the same observables, tick for tick
    let a1 = analyze_system(&l.sys.snap);
    assert_eq!(a1.acceptance, a0.acceptance);
    let flat = &a1.flattened.snapshot.design;
    assert!(matches!(
        flat.mappings[&l.brightness].definition,
        Some(Definition::Reference { .. })
    ));
    let after = simulate(&l.sys.snap, l.raw, l.tilt);
    assert_eq!(levels(&after, l.indicator), levels(&before, l.indicator));
    assert_eq!(levels(&after, l.brightness), levels(&before, l.brightness));
    let inner = l.sys.flat_decl(inst, l.brightness);
    assert_eq!(levels(&after, inner), levels(&before, l.brightness));
    assert_eq!(
        levels(&after, l.indicator),
        vec![0.0, 30.0 / 90.0, 60.0 / 90.0, 1.0]
    );
    // deployment: the same requirements
    let d1 = bdl_compiler::analyze_deployment(&a1.flattened.snapshot, &nano);
    assert_eq!(d1.requirements.len(), d0.requirements.len());
    assert_eq!(d1.status, d0.status);
}

// §76 — a bidirectional boundary is accepted: flattened causality decides.
#[test]
fn a_bidirectional_boundary_is_not_a_cycle() {
    let mut l = lamp();
    // boosted (member) reads indicator (outside), which reads brightness
    // (member): base → group → base → group, acyclic at the declarations.
    let boosted = l.sys.base_mapping("boosted", &[], l.level);
    l.sys.base_formula(boosted, "indicator");
    l.sys.base_clock_of(boosted, l.main);
    let g = l.sys.create_group("Lamp", &[l.dim, l.brightness, boosted]);
    let a = analyze_system(&l.sys.snap);
    let b = &a.groups[&g];
    assert_eq!(ids(&b.crossing_in), ids(&[l.tilt_value, l.indicator]));
    assert_eq!(ids(&b.crossing_out), ids(&[l.brightness]));
    l.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g,
        choices: ExtractionChoices::default(),
    });
    let a = analyze_system(&l.sys.snap);
    assert!(a.analysis.causality.valid, "{:?}", a.analysis.diagnostics);
    assert_ne!(a.acceptance, Acceptance::Invalid);
    assert_eq!(l.sys.system().bindings.len(), 3);
}

// §77 — every domain a member touches becomes a parameter, `sync` included.
#[test]
fn clock_coverage_includes_domains_read_only_through_sync() {
    let mut l = lamp();
    let aux = l.sys.base_clock("aux");
    let slow = l.sys.base_mapping("slow", &[], l.level);
    l.sys.base_formula(slow, "sync(main, 0, brightness)");
    l.sys.base_clock_of(slow, aux);
    let held = l.sys.base_mapping("held", &[], l.level);
    l.sys.base_formula(held, "sync(aux, 0, slow)");
    l.sys.base_clock_of(held, l.main);
    let g = l.sys.create_group("Hold", &[held]);
    let a = analyze_system(&l.sys.snap);
    // `held` runs in main and observes aux through sync: both are clocks
    // of the group, even though Κ alone lists only main.
    assert_eq!(a.groups[&g].clocks, vec![l.main, aux]);
    let p = preview_extraction(&l.sys.snap, g, &ExtractionChoices::default()).unwrap();
    assert_eq!(p.clocks, vec![l.main, aux]);
    l.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g,
        choices: ExtractionChoices::default(),
    });
    let a = analyze_system(&l.sys.snap);
    assert!(
        !a.analysis.diagnostics.iter().any(|d| d.is_error()),
        "{:?}",
        a.analysis.diagnostics
    );
    let c = l.sys.system().components.values().next().unwrap();
    assert_eq!(c.interface.clock_params, vec![l.main, aux]);
    assert_eq!(c.body.clocks.len(), 2);
}

// §78 — fan-out, deletion and a transported binding coexist after packaging.
#[test]
fn fan_out_delete_and_transport_coexist() {
    let mut l = lamp();
    let g = l.sys.create_group("AdaptiveLamp", &[l.dim, l.brightness]);
    let o = l.sys.apply(SystemEditOp::ExtractGroupAsComponent {
        group: g,
        choices: ExtractionChoices::default(),
    });
    let inst = o.created_instance.unwrap();
    let comp = o.created_component.unwrap();
    let req = l.sys.system().components[&comp]
        .interface
        .port_for_decl(l.tilt_value)
        .unwrap()
        .id;
    let prov = l.sys.system().components[&comp]
        .interface
        .port_for_decl(l.brightness)
        .unwrap()
        .id;
    // fan-out: a second instance reads the same base relationship
    let second = l.sys.instance(comp, "second");
    l.sys.clock_arg(second, l.main, l.main);
    l.sys
        .bind(BindingEnd::Base { decl: l.tilt_value }, pr(second, req));
    let a = analyze_system(&l.sys.snap);
    assert_eq!(
        a.ports[&pr(second, req)],
        PortStatus::Bound {
            binding: *l.sys.system().bindings.keys().last().unwrap()
        }
    );
    // a second driver of `light` would be a conflict, not a priority: the
    // second instance drives the same external sink
    assert!(a.analysis.outputs.conflicts.contains_key(&l.light));

    // deleting the base copy's binding leaves it open (a legal state)
    let into_base = l
        .sys
        .system()
        .bindings
        .values()
        .find(|b| b.destination == BindingEnd::Base { decl: l.brightness })
        .unwrap()
        .id;
    l.sys
        .apply(SystemEditOp::UnbindPorts { binding: into_base });
    let a = analyze_system(&l.sys.snap);
    assert!(a.flattened.snapshot.design.mappings[&l.brightness]
        .definition
        .is_none());

    // a transported binding across a new domain coexists with the rest
    let aux = l.sys.base_clock("aux");
    let tap = l.sys.base_mapping("tap", &[], l.level);
    l.sys.base_clock_of(tap, aux);
    l.sys
        .bind_transported(pr(inst, prov), BindingEnd::Base { decl: tap }, "0");
    let a = analyze_system(&l.sys.snap);
    assert!(
        !a.composition
            .iter()
            .any(|d| d.code.as_str().starts_with("system.transport")),
        "{:?}",
        a.composition
    );
    assert!(matches!(
        a.flattened.snapshot.design.mappings[&tap].definition,
        Some(Definition::Reference {
            transport: Some(_),
            ..
        })
    ));
    // ... and a same-domain base end needs none; a mismatch is diagnosed
    let direct = l.sys.base_mapping("direct", &[], l.level);
    l.sys.base_clock_of(direct, aux);
    l.sys
        .bind(pr(inst, prov), BindingEnd::Base { decl: direct });
    let a = analyze_system(&l.sys.snap);
    assert!(a
        .composition
        .iter()
        .any(|d| d.code.as_str() == "system.binding_needs_transport"));

    // a base end must be open: a defined relationship is refused
    let e = l
        .sys
        .try_apply(SystemEditOp::BindPorts {
            source: pr(inst, prov).into(),
            destination: BindingEnd::Base { decl: l.indicator },
            transport: None,
        })
        .unwrap_err();
    assert!(matches!(e, SystemEditError::BaseNotOpen { .. }));

    // deleting a member relationship prunes it from its group
    let g2 = l.sys.create_group("Rest", &[l.indicator, tap]);
    l.sys.base(EditOp::DeleteMapping { id: tap });
    assert_eq!(
        ids(&l.sys.system().groups[&g2].members),
        ids(&[l.indicator])
    );
    // and the destructive delete removes the relationships themselves
    l.sys
        .apply(SystemEditOp::DeleteGroupWithMembers { group: g2 });
    assert!(!l.sys.system().base.mappings.contains_key(&l.indicator));
    assert!(l.sys.system().groups.is_empty());
}

// Groups persist as identity, name, description and members — nothing else.
#[test]
fn groups_persist_and_survive_reopening() {
    let mut l = lamp();
    let g = l.sys.create_group("Lamp", &[l.dim, l.brightness]);
    l.sys.group(GroupEditOp::SetGroupDescription {
        id: g,
        description: "dims with tilt".into(),
    });
    let dir = tempfile::tempdir().unwrap();
    persist::save_system_project(
        dir.path(),
        &l.sys.snap,
        &bdl_model::layout::Layout::default(),
        "test",
    )
    .unwrap();
    let json = std::fs::read_to_string(dir.path().join(persist::SYSTEM_FILE)).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let groups = &v["system"]["groups"];
    assert_eq!(groups["0"]["name"], "Lamp");
    assert_eq!(groups["0"]["description"], "dims with tilt");
    assert_eq!(
        groups["0"]["members"],
        serde_json::json!([l.dim.raw(), l.brightness.raw()])
    );
    assert!(groups["0"].get("collapsed").is_none());
    let loaded = persist::load_system_project(dir.path()).unwrap();
    assert_eq!(loaded.snapshot.system.groups, l.sys.system().groups);
    assert_eq!(loaded.snapshot.system, *l.sys.system());
}

// ---- scoped groups (ADR-0019 amendment): a component's body has groups too ----

/// A component `AdaptiveLighting` with three body relationships and one
/// instance, plus a base group, for the scope tests.
struct Lighting {
    sys: Sys,
    comp: ComponentId,
    inst: ComponentInstanceId,
    dim: DeclId,
    color: DeclId,
    warning: DeclId,
    base_decl: DeclId,
}

fn lighting() -> Lighting {
    let mut sys = Sys::new("lighting");
    let level = sys.base_concept("Brightness", LEVEL);
    let main = sys.base_clock("main");
    // Fillers so the base relationship's id is not also a body id: scope is
    // explicit, never inferred from ids, and the test must not rely on a
    // collision either way.
    for name in ["a0", "a1", "a2", "a3"] {
        sys.base_mapping(name, &[], level);
    }
    let base_decl = sys.base_mapping("ambient", &[], level);
    sys.base_clock_of(base_decl, main);
    let comp = sys.component("AdaptiveLighting");
    let l = sys.body_concept(comp, "Brightness", Some(LEVEL));
    sys.share(comp, l, level);
    let tick = sys.body_clock(comp, "tick");
    sys.clock_param(comp, tick);
    let dim = sys.body_mapping(comp, "dimBrightness", &[], l);
    sys.body_formula(comp, dim, "1");
    sys.body_clock_of(comp, dim, tick);
    let color = sys.body_mapping(comp, "colorTemp", &[], l);
    sys.body_formula(comp, color, "dimBrightness");
    sys.body_clock_of(comp, color, tick);
    let warning = sys.body_mapping(comp, "warning", &[], l);
    sys.body_formula(comp, warning, "colorTemp");
    sys.body_clock_of(comp, warning, tick);
    sys.port(comp, warning, PortKind::Provided, "warning");
    let inst = sys.instance(comp, "lighting");
    sys.clock_arg(inst, tick, main);
    Lighting {
        sys,
        comp,
        inst,
        dim,
        color,
        warning,
        base_decl,
    }
}

// §31 — a component-local group changes nothing about the component.
#[test]
fn a_component_local_group_is_adjacent_to_the_body_not_part_of_it() {
    let mut l = lighting();
    let scope = GroupScope::Component { component: l.comp };
    let before = l.sys.system().clone();
    let a0 = analyze_system(&l.sys.snap);
    let flat_ids_before = l.sys.system().flat_ids.clone();
    let revision = l.sys.snap.revision;

    let g = l.sys.create_group_in(scope, "Dimming", &[l.dim, l.color]);
    let s = l.sys.system();
    let group = &s.groups[&g];
    assert_eq!(group.scope, scope);
    assert_eq!(ids(&group.members), ids(&[l.dim, l.color]));
    // the component is the same value: body, stamps, interface
    assert_eq!(s.components[&l.comp], before.components[&l.comp]);
    assert_eq!(
        s.components[&l.comp].body_stamp,
        before.components[&l.comp].body_stamp
    );
    assert_eq!(
        s.components[&l.comp].interface_stamp,
        before.components[&l.comp].interface_stamp
    );
    assert_eq!(s.flat_ids, flat_ids_before);
    assert_eq!(l.sys.snap.revision, revision);
    let a1 = analyze_system(&l.sys.snap);
    assert_eq!(a1.flattened.snapshot.design, a0.flattened.snapshot.design);
    assert_eq!(a1.components, a0.components, "Realizes unchanged");
    assert_eq!(a1.analysis.diagnostics, a0.analysis.diagnostics);
    // the boundary is in the body's own ids, off the body's own analysis
    let b = &a1.groups[&g];
    assert_eq!(ids(&b.members), ids(&[l.dim, l.color]));
    assert_eq!(ids(&b.crossing_in), ids(&[]));
    assert_eq!(ids(&b.crossing_out), ids(&[l.color]));
    assert_eq!(ids(&b.private_candidates), ids(&[l.dim]));
    assert_eq!(b.internal_edges, vec![(l.color, l.dim)]);
    assert!(b.crossing_edges.is_empty());
    let g2 = l.sys.create_group_in(scope, "Alerts", &[l.warning]);
    let b2 = &analyze_system(&l.sys.snap).groups[&g2];
    assert_eq!(ids(&b2.crossing_in), ids(&[l.color]));
    assert_eq!(b2.crossing_edges, vec![(l.warning, l.color)]);

    // persistence: scope, name, members
    let dir = tempfile::tempdir().unwrap();
    persist::save_system_project(
        dir.path(),
        &l.sys.snap,
        &bdl_model::layout::Layout::default(),
        "test",
    )
    .unwrap();
    let loaded = persist::load_system_project(dir.path()).unwrap();
    assert_eq!(loaded.snapshot.system.groups, l.sys.system().groups);
    let json = std::fs::read_to_string(dir.path().join(persist::SYSTEM_FILE)).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["system"]["groups"][&g.raw().to_string()]["scope"],
        serde_json::json!({ "kind": "component", "component": l.comp.raw() })
    );
}

// §32 — a group never spans two designs.
#[test]
fn groups_never_cross_a_component_boundary() {
    let mut l = lighting();
    let scope = GroupScope::Component { component: l.comp };
    // a base relationship cannot join a component group, nor the reverse
    let e = l
        .sys
        .try_group(GroupEditOp::CreateGroup {
            scope,
            name: "Mixed".into(),
            description: String::new(),
            members: vec![l.base_decl],
        })
        .unwrap_err();
    assert!(matches!(e, GroupEditError::NotABaseDeclaration { .. }));
    let local = l.sys.create_group_in(scope, "Dimming", &[l.dim]);
    let base = l.sys.create_group("Ambient", &[l.base_decl]);
    let e = l
        .sys
        .try_group(GroupEditOp::MoveMember {
            decl: l.base_decl,
            to: local,
        })
        .unwrap_err();
    assert!(matches!(e, GroupEditError::NotABaseDeclaration { .. }));
    let e = l
        .sys
        .try_group(GroupEditOp::MergeGroups {
            into: base,
            from: local,
        })
        .unwrap_err();
    assert_eq!(e, GroupEditError::ScopeMismatch);
    // names are unique per design: two components may both have "Dimming"
    let other = l.sys.component("Other");
    l.sys
        .create_group_in(GroupScope::Component { component: other }, "Dimming", &[]);
    l.sys.create_group("Dimming", &[]);
    assert_eq!(l.sys.system().groups.len(), 4);
    // an unknown component is refused
    let e = l
        .sys
        .try_group(GroupEditOp::CreateGroup {
            scope: GroupScope::Component {
                component: ComponentId::from_raw(99),
            },
            name: "Ghost".into(),
            description: String::new(),
            members: vec![],
        })
        .unwrap_err();
    assert!(matches!(e, GroupEditError::UnknownComponent { .. }));
    // packaging is for the system's own design
    assert!(matches!(
        preview_extraction(&l.sys.snap, local, &ExtractionChoices::default()),
        Err(ExtractError::NotABaseGroup { .. })
    ));
}

// §37 / §38 / §39 — deletion prunes, component deletion retires, versions copy.
#[test]
fn component_groups_follow_their_component() {
    let mut l = lighting();
    let scope = GroupScope::Component { component: l.comp };
    let g = l.sys.create_group_in(scope, "Dimming", &[l.dim, l.color]);
    // deleting a body relationship prunes it; the group stays
    l.sys.body(l.comp, EditOp::DeleteMapping { id: l.dim });
    assert_eq!(ids(&l.sys.system().groups[&g].members), ids(&[l.color]));
    l.sys.body(l.comp, EditOp::DeleteMapping { id: l.color });
    assert!(l.sys.system().groups[&g].members.is_empty());
    assert!(l.sys.system().groups.contains_key(&g));
    // a version copies the groups under fresh ids, same local members
    let v2 = l
        .sys
        .apply(SystemEditOp::DuplicateComponent {
            id: l.comp,
            name: "AdaptiveLighting v2".into(),
        })
        .created_component
        .unwrap();
    let copies: Vec<&BehaviorGroup> = l
        .sys
        .system()
        .groups_in(GroupScope::Component { component: v2 })
        .collect();
    assert_eq!(copies.len(), 1);
    assert_ne!(copies[0].id, g);
    assert_eq!(copies[0].name, "Dimming");
    assert_eq!(copies[0].members, l.sys.system().groups[&g].members);
    // deleting a component retires its groups; nothing orphaned
    l.sys.apply(SystemEditOp::DeleteComponent { id: v2 });
    assert!(l
        .sys
        .system()
        .groups
        .values()
        .all(|x| x.scope != GroupScope::Component { component: v2 }));
    assert!(l.sys.system().groups.contains_key(&g));
    let _ = l.inst;
    let _ = l.warning;
}
