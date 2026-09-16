//! Contract hardening (brief §20–§24): a component's public interface is a
//! promise stored on its own, realized — or not — by the body; bindings
//! and substitution are decided on promises alone; nominal identity holds;
//! contracts persist and migrate.

#![allow(clippy::unwrap_used)]

mod support;

use bdl_model::edit::{EditKind, EditOp, Invalidation};
use bdl_model::surface::{Definition, Signature};
use bdl_system::*;
use support::*;

fn contract_of(sys: &Sys, c: ComponentId, p: PortId) -> PortContract {
    sys.system().components[&c].interface.ports[&p]
        .contract
        .clone()
}

fn codes(a: &SystemAnalysis) -> Vec<&str> {
    a.composition.iter().map(|d| d.code.as_str()).collect()
}

#[test]
fn a_private_body_edit_leaves_the_contract_untouched() {
    let mut s = vertical_slice();
    let before = contract_of(&s.sys, s.lamp, s.lamp_out);
    let stamps = {
        let c = &s.sys.system().components[&s.lamp];
        (c.body_stamp, c.interface_stamp)
    };
    // 1. a private formula edit
    let o = s.sys.body(
        s.lamp,
        EditOp::ReplaceDefinition {
            id: s.lamp_dim,
            definition: Some(Definition::Formula {
                source: "Tilt / 45 deg".into(),
            }),
        },
    );
    assert_eq!(
        contract_of(&s.sys, s.lamp, s.lamp_out),
        before,
        "byte-for-byte the same promise"
    );
    let c = &s.sys.system().components[&s.lamp];
    assert_eq!(
        (c.body_stamp, c.interface_stamp),
        (stamps.0 + 1, stamps.1),
        "an implementation change, not an interface change"
    );
    assert!(o.bindings.is_empty());
    assert_eq!(o.instances, [s.lamp_a, s.lamp_b].into_iter().collect());
    let a = analyze_system(&s.sys.snap);
    assert!(a.components[&s.lamp] && a.composition.is_empty());
    assert_eq!(a.acceptance, Acceptance::Executable);
    // a private rename: nothing semantic
    let o = s.sys.body(
        s.lamp,
        EditOp::RenameMapping {
            id: s.lamp_dim,
            name: "dim".into(),
        },
    );
    assert_eq!(o.kind, Some(EditKind::Refinement));
    assert_eq!(contract_of(&s.sys, s.lamp, s.lamp_out), before);
}

#[test]
fn changing_the_backing_declaration_shape_breaks_realizes_not_the_contract() {
    let mut s = vertical_slice();
    let before = contract_of(&s.sys, s.lamp, s.lamp_out);
    // 2. the provided port's declaration now produces Tilt instead of Brightness
    s.sys.body(
        s.lamp,
        EditOp::SetMappingSignature {
            id: s.lamp_brightness,
            signature: Signature {
                inputs: vec![],
                output: s.lamp_tilt,
            },
        },
    );
    assert_eq!(
        contract_of(&s.sys, s.lamp, s.lamp_out),
        before,
        "the promise did not follow the body"
    );
    let a = analyze_system(&s.sys.snap);
    assert!(!a.components[&s.lamp]);
    assert!(
        codes(&a).contains(&"component.port_signature_mismatch"),
        "{:?}",
        codes(&a)
    );
    let d = a
        .composition
        .iter()
        .find(|d| d.code.as_str() == "component.port_signature_mismatch")
        .unwrap();
    assert_eq!(
        d.message,
        "AdaptiveLamp's port brightness promises Brightness, but brightness is Tilt."
    );
    assert!(
        d.technical.contains(&s.lamp.to_string())
            && d.technical.contains(&s.lamp_out.to_string())
            && d.technical.contains(&s.lamp_brightness.to_string())
    );
    assert_eq!(a.acceptance, Acceptance::Invalid);
    // the clock, too: give the provided port no domain
    let mut s2 = vertical_slice();
    s2.sys.body(
        s2.lamp,
        EditOp::SetMappingClock {
            id: s2.lamp_brightness,
            clock: None,
        },
    );
    let a = analyze_system(&s2.sys.snap);
    assert!(
        codes(&a).contains(&"component.port_clock_mismatch"),
        "{:?}",
        codes(&a)
    );
    // a required port that gains a definition inside the body
    let mut s3 = vertical_slice();
    s3.sys.body(
        s3.lamp,
        EditOp::AttachDefinition {
            id: s3.lamp_tilt_value,
            definition: Definition::Formula {
                source: "30 deg".into(),
            },
        },
    );
    let a = analyze_system(&s3.sys.snap);
    assert!(
        codes(&a).contains(&"component.required_port_realized"),
        "{:?}",
        codes(&a)
    );
    // a port whose declaration is retargeted to a wrong one
    let mut s4 = vertical_slice();
    let e = s4
        .sys
        .try_apply(SystemEditOp::RebindPortDeclaration {
            component: s4.lamp,
            port: s4.lamp_out,
            decl: s4.lamp_tilt_value,
        })
        .unwrap_err();
    assert!(matches!(e, SystemEditError::DeclAlreadyExposed { .. }));
    s4.sys.apply(SystemEditOp::RebindPortDeclaration {
        component: s4.lamp,
        port: s4.lamp_out,
        decl: s4.lamp_dim,
    });
    let a = analyze_system(&s4.sys.snap);
    assert!(codes(&a).contains(&"component.port_signature_mismatch"));
    assert_eq!(
        contract_of(&s4.sys, s4.lamp, s4.lamp_out),
        before,
        "rebinding the declaration keeps the promise"
    );
}

#[test]
fn a_contract_change_is_explicit_and_classified() {
    let mut s = vertical_slice();
    let before = contract_of(&s.sys, s.lamp, s.lamp_in);
    let stamp = s.sys.system().components[&s.lamp].interface_stamp;
    let source_stamp = s.sys.system().components[&s.source].interface_stamp;
    // 3. the same contract again: a refinement, no stamp, no binding reopened
    let o = s.sys.apply(SystemEditOp::ChangePortContract {
        component: s.lamp,
        port: s.lamp_in,
        contract: before.clone(),
    });
    assert_eq!(o.kind, Some(EditKind::Refinement));
    assert!(o.bindings.is_empty());
    assert_eq!(s.sys.system().components[&s.lamp].interface_stamp, stamp);
    // a real change: an edit that names the bindings on the port and the instances
    let changed = PortContract {
        signature: Signature {
            inputs: vec![],
            output: s.lamp_brightness_concept,
        },
        ..before.clone()
    };
    let o = s.sys.apply(SystemEditOp::ChangePortContract {
        component: s.lamp,
        port: s.lamp_in,
        contract: changed.clone(),
    });
    assert_eq!(o.kind, Some(EditKind::Edit));
    assert!(o.invalidates.contains(&Invalidation::Interface));
    assert_eq!(o.bindings, [s.bind_a, s.bind_b].into_iter().collect());
    assert_eq!(
        o.instances,
        [s.sensor, s.lamp_a, s.lamp_b].into_iter().collect()
    );
    assert_eq!(
        s.sys.system().components[&s.lamp].interface_stamp,
        stamp + 1
    );
    assert_eq!(contract_of(&s.sys, s.lamp, s.lamp_in), changed);
    // now the body does not realize the promise, and the bindings are incompatible on the promise
    let a = analyze_system(&s.sys.snap);
    assert!(codes(&a).contains(&"component.port_signature_mismatch"));
    assert!(codes(&a).contains(&"system.binding_type_mismatch"));
    assert_eq!(a.acceptance, Acceptance::Invalid);
    // the source component's interface is untouched by any of this
    assert_eq!(
        s.sys.system().components[&s.source].interface_stamp,
        source_stamp
    );
}

#[test]
fn existing_bindings_survive_private_edits_and_stay_put_when_the_body_stops_realizing() {
    let mut s = vertical_slice();
    let a0 = analyze_system(&s.sys.snap);
    assert_eq!(
        a0.ports[&pr(s.lamp_a, s.lamp_in)],
        PortStatus::Bound { binding: s.bind_a }
    );
    // edit the consumer's internal realization only
    s.sys.body(
        s.lamp,
        EditOp::ReplaceDefinition {
            id: s.lamp_brightness,
            definition: Some(Definition::Formula {
                source: "dimByTilt(tiltValue)".into(),
            }),
        },
    );
    let a1 = analyze_system(&s.sys.snap);
    assert!(a1.composition.is_empty() && a1.acceptance == Acceptance::Executable);
    assert_eq!(
        a1.ports[&pr(s.lamp_a, s.lamp_in)],
        PortStatus::Bound { binding: s.bind_a }
    );
    // now make the consumer's required declaration no longer match its contract
    s.sys.body(
        s.lamp,
        EditOp::SetMappingSignature {
            id: s.lamp_tilt_value,
            signature: Signature {
                inputs: vec![],
                output: s.lamp_brightness_concept,
            },
        },
    );
    let a2 = analyze_system(&s.sys.snap);
    // the binding is still there, by identity, untouched
    assert_eq!(
        s.sys.system().bindings[&s.bind_a].source,
        pr(s.sensor, s.source_port).into()
    );
    assert_eq!(
        a2.ports[&pr(s.lamp_a, s.lamp_in)],
        PortStatus::Bound { binding: s.bind_a }
    );
    let d = s
        .sys
        .system()
        .flat_ids
        .get(s.lamp_a, LocalEntity::Decl(s.lamp_tilt_value))
        .map(bdl_model::DeclId::from_raw)
        .unwrap();
    assert!(
        matches!(
            a2.flattened.snapshot.design.mappings[&d].definition,
            Some(Definition::Reference { .. })
        ),
        "not reinterpreted, still the same reference"
    );
    // the component is invalid; the binding itself is still compatible on the promises
    assert!(!a2.components[&s.lamp]);
    assert!(codes(&a2).contains(&"component.port_signature_mismatch"));
    assert!(
        !codes(&a2).contains(&"system.binding_type_mismatch"),
        "the promise did not change, the body broke it: {:?}",
        codes(&a2)
    );
    assert_eq!(a2.acceptance, Acceptance::Invalid);
    // and the flat compiler, the final authority, rejects the reference against the actual body
    assert!(a2.analysis.mappings[&d]
        .diagnostics
        .iter()
        .any(|x| x.code.as_str() == "realization.type_mismatch"));
}

#[test]
fn equal_representations_are_not_equal_concepts() {
    // Two components: Source provides `level : () -> A`, Sink requires
    // `level : () -> B`, A and B both dimensionless quantities.
    let mut sys = Sys::new("nominal");
    let a = sys.base_concept("A", LEVEL);
    let b = sys.base_concept("B", LEVEL);
    let main = sys.base_clock("main");
    let src = sys.component("Source");
    let sa = sys.body_concept(src, "A", Some(LEVEL));
    sys.share(src, sa, a);
    let st = sys.body_clock(src, "tick");
    sys.clock_param(src, st);
    let sv = sys.body_mapping(src, "level", &[], sa);
    sys.body_formula(src, sv, "0.5");
    sys.body_clock_of(src, sv, st);
    let sp = sys.port(src, sv, PortKind::Provided, "level");
    let snk = sys.component("Sink");
    let sb = sys.body_concept(snk, "B", Some(LEVEL));
    sys.share(snk, sb, b);
    let kt = sys.body_clock(snk, "tick");
    sys.clock_param(snk, kt);
    let kv = sys.body_mapping(snk, "level", &[], sb);
    sys.body_clock_of(snk, kv, kt);
    let kp = sys.port(snk, kv, PortKind::Required, "level");
    let i1 = sys.instance(src, "s");
    sys.clock_arg(i1, st, main);
    let i2 = sys.instance(snk, "k");
    sys.clock_arg(i2, kt, main);
    let bnd = sys.bind(pr(i1, sp), pr(i2, kp));
    let inc = binding_compatibility(sys.system(), &sys.system().bindings[&bnd]);
    assert_eq!(inc, vec![Incompatibility::Signature]);
    let an = analyze_system(&sys.snap);
    let d = an
        .composition
        .iter()
        .find(|d| d.code.as_str() == "system.binding_type_mismatch")
        .unwrap();
    assert_eq!(d.message, "k.level expects B, but s.level provides A.");
    assert_eq!(an.acceptance, Acceptance::Invalid);
    // two private concepts with equal representations are different too, even in two instances of one component
    let mut s = vertical_slice();
    s.sys.apply(SystemEditOp::UnbindPorts { binding: s.bind_b });
    // Brightness (private, lampA) into lampB's port over shared Tilt: accepted as an edit, rejected on the promises
    let b = s
        .sys
        .bind(pr(s.lamp_a, s.lamp_out), pr(s.lamp_b, s.lamp_in));
    assert_eq!(
        binding_compatibility(s.sys.system(), &s.sys.system().bindings[&b]),
        vec![Incompatibility::Signature]
    );
    // and a private concept of lampA is not lampB's either: two instances' Brightness differ
    let ra = bdl_system::contract::resolve_concept(
        &s.sys.system().components[&s.lamp],
        s.lamp_a,
        s.lamp_brightness_concept,
    );
    let rb = bdl_system::contract::resolve_concept(
        &s.sys.system().components[&s.lamp],
        s.lamp_b,
        s.lamp_brightness_concept,
    );
    assert_ne!(ra, rb);
    assert_eq!(
        bdl_system::contract::resolve_concept(
            &s.sys.system().components[&s.lamp],
            s.lamp_a,
            s.lamp_tilt
        ),
        ResolvedConcept::Shared(s.tilt)
    );
}

#[test]
fn a_refining_version_substitutes_an_incompatible_one_is_refused() {
    let mut s = vertical_slice();
    // C2: a copy of AdaptiveLamp with a different body (a different curve)
    let c2 = s
        .sys
        .apply(SystemEditOp::DuplicateComponent {
            id: s.lamp,
            name: "AdaptiveLampV2".into(),
        })
        .created_component
        .unwrap();
    s.sys.body(
        c2,
        EditOp::ReplaceDefinition {
            id: s.lamp_dim,
            definition: Some(Definition::Formula {
                source: "Tilt / 180 deg".into(),
            }),
        },
    );
    assert_eq!(
        s.sys.system().components[&c2].interface.ports,
        s.sys.system().components[&s.lamp].interface.ports,
        "same ports, same ids, same promises"
    );
    assert!(component_substitutable(
        &s.sys.system().components[&s.lamp],
        &s.sys.system().components[&c2],
        &[s.lamp_in, s.lamp_out],
        &[s.lamp_tick]
    )
    .is_ok());
    let flat_before: Vec<_> = s
        .sys
        .system()
        .flat_ids
        .for_instance(s.lamp_a)
        .cloned()
        .collect();
    let o = s.sys.apply(SystemEditOp::ReplaceInstanceComponent {
        instance: s.lamp_a,
        component: c2,
    });
    assert_eq!(o.bindings, [s.bind_a].into_iter().collect());
    assert_eq!(s.sys.system().instances[&s.lamp_a].component, c2);
    // bindings untouched and still valid; the instance keeps its flat identities (a version keeps its local ids)
    assert_eq!(
        s.sys.system().bindings[&s.bind_a].destination,
        pr(s.lamp_a, s.lamp_in).into()
    );
    assert_eq!(
        s.sys
            .system()
            .flat_ids
            .for_instance(s.lamp_a)
            .cloned()
            .collect::<Vec<_>>(),
        flat_before
    );
    let a = analyze_system(&s.sys.snap);
    assert!(a.composition.is_empty(), "{:?}", codes(&a));
    assert_eq!(a.acceptance, Acceptance::Executable);
    // lampA now runs the V2 curve, lampB the original
    let raw = s.sys.flat_decl(s.sensor, s.source_raw);
    let ir = &a.analysis.ir;
    let brightness_a = s.sys.flat_decl(s.lamp_a, s.lamp_brightness);
    let brightness_b = s.sys.flat_decl(s.lamp_b, s.lamp_brightness);
    let _ = (raw, ir, brightness_a, brightness_b);
    // C3: an incompatible interface — the required port now promises Brightness
    let c3 = s
        .sys
        .apply(SystemEditOp::DuplicateComponent {
            id: s.lamp,
            name: "AdaptiveLampV3".into(),
        })
        .created_component
        .unwrap();
    s.sys.apply(SystemEditOp::ChangePortContract {
        component: c3,
        port: s.lamp_in,
        contract: PortContract {
            signature: Signature {
                inputs: vec![],
                output: s.lamp_brightness_concept,
            },
            commitments: vec![],
            clock: ClockContract::Parameter { clock: s.lamp_tick },
        },
    });
    let problems = component_substitutable(
        &s.sys.system().components[&s.lamp],
        &s.sys.system().components[&c3],
        &[s.lamp_in, s.lamp_out],
        &[s.lamp_tick],
    )
    .unwrap_err();
    assert_eq!(
        problems,
        vec![SubstitutionProblem {
            port: s.lamp_in,
            reason: SubstitutionReason::Sharing
        }]
    );
    let e = s
        .sys
        .try_apply(SystemEditOp::ReplaceInstanceComponent {
            instance: s.lamp_b,
            component: c3,
        })
        .unwrap_err();
    assert!(matches!(e, SystemEditError::NotSubstitutable { .. }));
    assert_eq!(
        s.sys.system().instances[&s.lamp_b].component,
        s.lamp,
        "refused: nothing moved"
    );
    // an uninstantiated component with a broken promise is still reported, once, on its own
    let a = analyze_system(&s.sys.snap);
    assert_eq!(codes(&a), vec!["component.port_signature_mismatch"]);
    assert!(!a.components[&c3] && a.components[&c2] && a.components[&s.lamp]);
    s.sys.apply(SystemEditOp::DeleteComponent { id: c3 });
    // a version that drops a port the system uses is refused too; one that only adds an open required port is fine
    let c4 = s
        .sys
        .apply(SystemEditOp::DuplicateComponent {
            id: s.lamp,
            name: "AdaptiveLampV4".into(),
        })
        .created_component
        .unwrap();
    let extra_c = s.sys.body_concept(c4, "Gain", Some(LEVEL));
    let extra = s.sys.body_mapping(c4, "gain", &[], extra_c);
    s.sys.port(c4, extra, PortKind::Required, "gain");
    s.sys.apply(SystemEditOp::ReplaceInstanceComponent {
        instance: s.lamp_b,
        component: c4,
    });
    let a = analyze_system(&s.sys.snap);
    assert!(a.composition.is_empty(), "{:?}", a.composition);
    assert_eq!(
        a.acceptance,
        Acceptance::Open,
        "the new required port is open, the old bindings hold"
    );
    let c5 = s
        .sys
        .apply(SystemEditOp::DuplicateComponent {
            id: s.lamp,
            name: "AdaptiveLampV5".into(),
        })
        .created_component
        .unwrap();
    // retire the provided port in V5 — nothing is bound to lampB's brightness, so this still substitutes;
    // retire the required one — refused, lampB.tiltValue is bound
    s.sys.apply(SystemEditOp::RetirePort {
        component: c5,
        port: s.lamp_out,
    });
    assert!(s
        .sys
        .try_apply(SystemEditOp::ReplaceInstanceComponent {
            instance: s.lamp_b,
            component: c5
        })
        .is_ok());
    let c6 = s
        .sys
        .apply(SystemEditOp::DuplicateComponent {
            id: s.lamp,
            name: "AdaptiveLampV6".into(),
        })
        .created_component
        .unwrap();
    s.sys.apply(SystemEditOp::RetirePort {
        component: c6,
        port: s.lamp_in,
    });
    let e = s
        .sys
        .try_apply(SystemEditOp::ReplaceInstanceComponent {
            instance: s.lamp_b,
            component: c6,
        })
        .unwrap_err();
    assert!(
        matches!(e, SystemEditError::NotSubstitutable { ref problems, .. } if problems[0].reason == SubstitutionReason::PortMissing)
    );
}

#[test]
fn clock_contracts_are_explicit_and_local() {
    let mut s = vertical_slice();
    let c = contract_of(&s.sys, s.lamp, s.lamp_in);
    assert_eq!(c.clock, ClockContract::Parameter { clock: s.lamp_tick });
    // a private clock inside the component and a port over it
    let private = s.sys.body_clock(s.lamp, "blink");
    let bl_c = s.sys.body_concept(s.lamp, "Blink", Some(LEVEL));
    let bl = s.sys.body_mapping(s.lamp, "blink", &[], bl_c);
    s.sys.body_formula(s.lamp, bl, "1");
    s.sys.body_clock_of(s.lamp, bl, private);
    let bp = s.sys.port(s.lamp, bl, PortKind::Provided, "blink");
    assert_eq!(
        contract_of(&s.sys, s.lamp, bp).clock,
        ClockContract::Private { clock: private }
    );
    // the pure relationship is agnostic
    let dp = s.sys.port(s.lamp, s.lamp_dim, PortKind::Provided, "curve");
    assert_eq!(
        contract_of(&s.sys, s.lamp, dp).clock,
        ClockContract::Agnostic
    );
    // making the private clock a parameter rewrites the promise's role explicitly
    let o = s.sys.apply(SystemEditOp::SetClockParameter {
        component: s.lamp,
        clock: private,
        parameter: true,
    });
    assert!(o.invalidates.contains(&Invalidation::Interface));
    assert_eq!(
        contract_of(&s.sys, s.lamp, bp).clock,
        ClockContract::Parameter { clock: private }
    );
    // no system ClockId ever appears in a contract
    for p in s.sys.system().components[&s.lamp].interface.ports.values() {
        if let Some(k) = p.contract.clock.local() {
            assert!(s.sys.system().components[&s.lamp]
                .body
                .clocks
                .contains_key(&k));
        }
    }
    let a = analyze_system(&s.sys.snap);
    assert!(a.components[&s.lamp], "{:?}", a.composition);
}

#[test]
fn contracts_persist_exactly_and_schema_1_migrates_once() {
    let s = vertical_slice();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("rover");
    persist::save_system_project(&root, &s.sys.snap, &Default::default(), "test").unwrap();
    let text = std::fs::read_to_string(root.join("design/system.bdl.json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["schema_version"], 2);
    let loaded = persist::load_system_project(&root).unwrap();
    assert_eq!(loaded.snapshot.system, s.sys.snap.system);
    for (cid, c) in &s.sys.snap.system.components {
        for (pid, p) in &c.interface.ports {
            assert_eq!(
                loaded.snapshot.system.components[cid].interface.ports[pid].contract,
                p.contract
            );
        }
    }
    // a schema-1 file: no contracts, one stamp
    let mut v1 = v.clone();
    v1["schema_version"] = serde_json::json!(1);
    for comp in v1["system"]["components"]
        .as_object_mut()
        .unwrap()
        .values_mut()
    {
        let stamp = comp["interface_stamp"].clone();
        comp.as_object_mut().unwrap().remove("body_stamp");
        comp.as_object_mut().unwrap().remove("interface_stamp");
        comp["stamp"] = stamp;
        for port in comp["interface"]["ports"]
            .as_object_mut()
            .unwrap()
            .values_mut()
        {
            port.as_object_mut().unwrap().remove("contract");
        }
    }
    let old_root = dir.path().join("old");
    std::fs::create_dir_all(old_root.join("design")).unwrap();
    std::fs::write(
        old_root.join("design/system.bdl.json"),
        serde_json::to_string_pretty(&v1).unwrap(),
    )
    .unwrap();
    std::fs::copy(root.join("bdl.toml"), old_root.join("bdl.toml")).unwrap();
    let migrated = persist::load_system_project(&old_root).unwrap();
    for (cid, c) in &s.sys.snap.system.components {
        for (pid, p) in &c.interface.ports {
            assert_eq!(
                migrated.snapshot.system.components[cid].interface.ports[pid].contract, p.contract,
                "derived once from the body"
            );
        }
    }
    assert_eq!(
        analyze_system(&migrated.snapshot).acceptance,
        Acceptance::Executable
    );
    // saved again, it is schema 2 with explicit contracts; the body no longer decides
    persist::save_system_project(&old_root, &migrated.snapshot, &Default::default(), "test")
        .unwrap();
    let again: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(old_root.join("design/system.bdl.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(again["schema_version"], 2);
    assert!(again["system"]["components"]
        .as_object()
        .unwrap()
        .values()
        .all(|c| c["interface"]["ports"]
            .as_object()
            .unwrap()
            .values()
            .all(|p| p.get("contract").is_some())));
    // a newer schema is refused
    let mut v3 = v.clone();
    v3["schema_version"] = serde_json::json!(3);
    std::fs::write(
        old_root.join("design/system.bdl.json"),
        serde_json::to_string(&v3).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        persist::load_system_project(&old_root),
        Err(bdl_model::persist::PersistError::UnsupportedSchema { .. })
    ));
}

#[test]
fn flattening_is_unchanged_for_a_realizing_component() {
    // The same design as before this milestone: contracts change nothing
    // about what a realizing component flattens to.
    let s = vertical_slice();
    let f = flatten(&s.sys.snap);
    let a = bdl_compiler::analyze(&f.snapshot);
    assert!(a.diagnostics.is_empty() && a.output_complete);
    assert_eq!(f.snapshot.design.mappings.len(), 2 + 3 * 2);
    let flat_hand = hand_written_flat();
    let ah = bdl_compiler::analyze(&flat_hand.snap);
    let count = |x: &bdl_compiler::ProjectAnalysis| {
        x.mappings
            .values()
            .filter(|m| m.status == bdl_compiler::MappingStatus::ClockConsistent)
            .count()
    };
    assert_eq!(count(&a), count(&ah));
}
