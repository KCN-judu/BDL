#![allow(clippy::unwrap_used)]

use bdl_hardware::boards::{arduino_nano, big_board};
use bdl_hardware::devices::requirements_for_all;
use bdl_hardware::solve::{brute_force_satisfiable, candidates};
use bdl_hardware::*;
use bdl_model::surface::{DeviceBinding, DeviceKind};
use bdl_model::DeviceId;
use std::collections::{BTreeMap, BTreeSet};

fn dev(n: u64, name: &str, kind: DeviceKind) -> DeviceBinding {
    DeviceBinding {
        id: DeviceId::from_raw(n),
        name: name.into(),
        kind,
        output: None,
        realization: None,
        fixed_pins: BTreeMap::new(),
    }
}

fn pinned(mut d: DeviceBinding, index: u16, pin: &str) -> DeviceBinding {
    d.fixed_pins.insert(index, pin.into());
    d
}

fn rid(s: &str) -> ResourceId {
    ResourceId::new(s)
}

fn assigned(a: &Assignment, device: u64, index: u16) -> &str {
    &a[&RequirementId {
        device: DeviceId::from_raw(device),
        index,
    }]
        .0
}

#[test]
fn nano_four_motors_and_an_imu_are_satisfiable() {
    let devices: Vec<DeviceBinding> = (1..=4)
        .map(|n| dev(n, &format!("M{n}"), DeviceKind::HBridgeChannel))
        .chain([dev(5, "IMU", DeviceKind::I2cSensor)])
        .collect();
    let reqs = requirements_for_all(&devices);
    assert_eq!(reqs.len(), 10);
    let hw = arduino_nano();
    let a = solve(&hw, &reqs).expect("SAT");
    assert!(validate(&hw, &reqs, &a).is_empty());
    // the IMU's two lines land on the I2C unit, each motor has a distinct
    // PWM pin and a direction pin; the exact placement is the documented
    // deterministic one
    assert_eq!(assigned(&a, 5, 0), "A4");
    assert_eq!(assigned(&a, 5, 1), "A5");
    let pwms: BTreeSet<&str> = (1..=4).map(|n| assigned(&a, n, 0)).collect();
    assert_eq!(pwms.len(), 4);
    for p in &pwms {
        assert!(hw.supports(&rid(p), Capability::Pwm));
    }
    assert_eq!(pwms, ["D3", "D5", "D6", "D9"].into_iter().collect());
    let dirs: BTreeSet<&str> = (1..=4).map(|n| assigned(&a, n, 1)).collect();
    assert_eq!(dirs, ["D0", "D1", "D2", "D4"].into_iter().collect());
    assert_eq!(solve(&hw, &reqs), Some(a), "deterministic");
}

#[test]
fn seven_pwm_is_unsat_on_the_nano_and_sat_on_the_big_board() {
    let devices: Vec<DeviceBinding> = (1..=7)
        .map(|n| dev(n, &format!("L{n}"), DeviceKind::PwmChannel))
        .collect();
    let reqs = requirements_for_all(&devices);
    let nano = arduino_nano();
    assert_eq!(solve(&nano, &reqs), None);
    let dead = diagnose(&nano, &reqs).unwrap();
    // the seventh PWM finds every PWM pin taken; the diagnosis names who holds each
    assert_eq!(
        dead.requirement,
        RequirementId {
            device: DeviceId::from_raw(7),
            index: 0
        }
    );
    let DeadEndReason::Blocked { candidates } = dead.reason else {
        panic!("{:?}", dead.reason)
    };
    assert_eq!(candidates.len(), 6);
    assert_eq!(dead.placed.len(), 6);
    // same design, larger target: feasible — feasibility is target-relative
    let big = big_board();
    let a = solve(&big, &reqs).expect("SAT on the bigger board");
    assert!(validate(&big, &reqs, &a).is_empty());
}

#[test]
fn counting_capabilities_is_not_feasibility() {
    // 2 interrupts + 6 PWM: the Nano has exactly 2 interrupt pins and 6 PWM
    // pins, but D3 is both.
    let devices: Vec<DeviceBinding> = [dev(0, "enc", DeviceKind::QuadratureEncoder)]
        .into_iter()
        .chain((1..=6).map(|n| dev(n, &format!("L{n}"), DeviceKind::PwmChannel)))
        .collect();
    let reqs = requirements_for_all(&devices);
    let nano = arduino_nano();
    assert!(solve(&nano, &reqs).is_none());
    // every capability is individually present; the shared pin is the point
    assert_eq!(candidates(&nano, &reqs[0]).len(), 2);
    assert!(reqs[2..].iter().all(|r| candidates(&nano, r).len() == 6));
}

#[test]
fn fixed_pins_are_honoured_or_reported() {
    let hw = arduino_nano();
    let ok = requirements_for_all(&[pinned(dev(0, "L", DeviceKind::PwmChannel), 0, "D3")]);
    assert_eq!(assigned(&solve(&hw, &ok).unwrap(), 0, 0), "D3");
    // pinned to a non-PWM pin
    let bad = requirements_for_all(&[pinned(dev(0, "L", DeviceKind::PwmChannel), 0, "D4")]);
    assert!(solve(&hw, &bad).is_none());
    let d = diagnose(&hw, &bad).unwrap();
    assert_eq!(
        d.reason,
        DeadEndReason::FixedUnavailable { fixed: rid("D4") }
    );
    // two requirements pinned to one exclusive pin
    let clash = requirements_for_all(&[
        pinned(dev(0, "L1", DeviceKind::PwmChannel), 0, "D3"),
        pinned(dev(1, "L2", DeviceKind::PwmChannel), 0, "D3"),
    ]);
    assert!(solve(&hw, &clash).is_none());
    // a manual pin can make an otherwise feasible design infeasible:
    // an encoder needs D2 and D3; pin a PWM to D3
    let squeezed = requirements_for_all(&[
        pinned(dev(0, "L", DeviceKind::PwmChannel), 0, "D3"),
        dev(1, "enc", DeviceKind::QuadratureEncoder),
    ]);
    assert!(solve(&hw, &squeezed).is_none());
    let d = diagnose(&hw, &squeezed).unwrap();
    assert!(matches!(d.reason, DeadEndReason::Blocked { .. }));
}

#[test]
fn bus_lines_are_shareable_exclusive_pins_are_not() {
    let hw = arduino_nano();
    let two_sensors = requirements_for_all(&[
        dev(0, "imu", DeviceKind::I2cSensor),
        dev(1, "baro", DeviceKind::I2cSensor),
    ]);
    let a = solve(&hw, &two_sensors).unwrap();
    assert_eq!(assigned(&a, 0, 0), "A4");
    assert_eq!(assigned(&a, 1, 0), "A4");
    assert!(validate(&hw, &two_sensors, &a).is_empty());
    // sharing is capability-specific: A4 as a digital output is exclusive
    let mixed = requirements_for_all(&[
        pinned(dev(0, "imu", DeviceKind::I2cSensor), 0, "A4"),
        pinned(dev(1, "led", DeviceKind::DigitalOutput), 0, "A4"),
    ]);
    assert!(solve(&hw, &mixed).is_none());
}

#[test]
fn unit_relations_same_and_distinct() {
    use Capability::*;
    let hw = arduino_nano();
    // UART TX/RX on the same unit: D1/D0 (unit 0)
    let uart = requirements_for_all(&[dev(0, "link", DeviceKind::Uart)]);
    let a = solve(&hw, &uart).unwrap();
    assert_eq!(
        hw.unit_of(&rid(assigned(&a, 0, 0)), UartTx),
        hw.unit_of(&rid(assigned(&a, 0, 1)), UartRx)
    );
    // Same on a board where the only TX and RX are on different units: UNSAT
    let mut split = hw.clone();
    for r in &mut split.resources {
        if r.id.0 == "D0" {
            r.units.insert(UartRx, UnitId(9));
        }
    }
    assert!(solve(&split, &uart).is_none());
    assert!(!brute_force_satisfiable(&split, &uart));
    // Distinct: four PWM lines that must be on independent timers — the
    // Nano has three timers, so UNSAT although six PWM pins exist
    let dev0 = DeviceId::from_raw(0);
    let group = GroupId {
        device: dev0,
        index: 0,
    };
    let four: Vec<Requirement> = (0..4)
        .map(|i| Requirement {
            id: RequirementId {
                device: dev0,
                index: i,
            },
            capability: Pwm,
            fixed: None,
            group: Some((group, UnitRel::Distinct)),
            label: format!("ch{i}"),
        })
        .collect();
    assert!(solve(&hw, &four).is_none());
    let three = &four[..3];
    let a = solve(&hw, three).unwrap();
    let units: BTreeSet<_> = three.iter().map(|r| hw.unit_of(&a[&r.id], Pwm)).collect();
    assert_eq!(units.len(), 3);
}

#[test]
fn unsupported_capability_is_a_clear_dead_end() {
    let hw = Hardware {
        name: "gpio_only".into(),
        display_name: String::new(),
        description: String::new(),
        family: String::new(),
        resources: vec![Resource {
            id: rid("P0"),
            capabilities: [Capability::DigitalOut].into_iter().collect(),
            units: BTreeMap::new(),
        }],
        shareable: BTreeSet::new(),
    };
    let reqs = requirements_for_all(&[dev(0, "L", DeviceKind::PwmChannel)]);
    assert!(solve(&hw, &reqs).is_none());
    assert_eq!(
        diagnose(&hw, &reqs).unwrap().reason,
        DeadEndReason::NoCapableResource
    );
}

#[test]
fn validate_reports_every_violation_kind() {
    let hw = arduino_nano();
    let reqs = requirements_for_all(&[
        pinned(dev(0, "L", DeviceKind::PwmChannel), 0, "D3"),
        dev(1, "M", DeviceKind::PwmChannel),
    ]);
    let mut a = Assignment::new();
    a.insert(reqs[0].id, rid("D5")); // ignores fixed
    a.insert(reqs[1].id, rid("D5")); // collides
    a.insert(
        RequirementId {
            device: DeviceId::from_raw(7),
            index: 0,
        },
        rid("D6"),
    ); // unknown
    let v = validate(&hw, &reqs, &a);
    assert!(v
        .iter()
        .any(|x| matches!(x, Violation::FixedIgnored { .. })));
    assert!(v
        .iter()
        .any(|x| matches!(x, Violation::Incompatible { .. })));
    assert!(v
        .iter()
        .any(|x| matches!(x, Violation::UnknownRequirement { .. })));
    let v = validate(&hw, &reqs, &Assignment::new());
    assert_eq!(v.len(), 2);
    assert!(v.iter().all(|x| matches!(x, Violation::Unassigned { .. })));
}

#[test]
fn boards_round_trip_through_toml_and_match_the_checked_in_files() {
    for hw in [arduino_nano(), big_board()] {
        assert!(bdl_hardware::boards::well_formed(&hw));
        let text = toml::to_string_pretty(&hw).unwrap();
        let back: Hardware = toml::from_str(&text).unwrap();
        assert_eq!(back, hw);
        let path = format!(
            "{}/../../hardware/boards/{}.toml",
            env!("CARGO_MANIFEST_DIR"),
            hw.name
        );
        let write = std::env::var("BDL_WRITE_BOARDS").is_ok();
        match std::fs::read_to_string(&path) {
            Ok(on_disk) if on_disk == text => {}
            Ok(_) | Err(_) if write => std::fs::write(&path, &text).unwrap(),
            Ok(_) => panic!("{path} is stale: run the test with BDL_WRITE_BOARDS=1"),
            Err(e) => panic!("{path}: {e} (set BDL_WRITE_BOARDS=1 to generate)"),
        }
    }
}

// ---- properties -----------------------------------------------------------

mod props {
    use super::*;
    use proptest::prelude::*;

    fn arb_cap() -> impl Strategy<Value = Capability> {
        prop_oneof![
            Just(Capability::DigitalOut),
            Just(Capability::Pwm),
            Just(Capability::I2cSda),
            Just(Capability::UartTx),
        ]
    }

    fn arb_hardware() -> impl Strategy<Value = Hardware> {
        (1usize..=4, any::<bool>()).prop_flat_map(|(n, share_i2c)| {
            proptest::collection::vec(
                (proptest::collection::btree_set(arb_cap(), 1..=3), 0u32..3),
                n,
            )
            .prop_map(move |rs| Hardware {
                name: "gen".into(),
                display_name: String::new(),
                description: String::new(),
                family: String::new(),
                resources: rs
                    .into_iter()
                    .enumerate()
                    .map(|(i, (caps, unit))| Resource {
                        id: rid(&format!("P{i}")),
                        units: caps.iter().map(|c| (*c, UnitId(unit))).collect(),
                        capabilities: caps,
                    })
                    .collect(),
                shareable: if share_i2c {
                    [Capability::I2cSda].into_iter().collect()
                } else {
                    BTreeSet::new()
                },
            })
        })
    }

    fn arb_requirements() -> impl Strategy<Value = Vec<Requirement>> {
        proptest::collection::vec(
            (
                arb_cap(),
                proptest::option::of(0usize..4),
                proptest::option::of((0u16..2, any::<bool>())),
            ),
            0..=4,
        )
        .prop_map(|rs| {
            rs.into_iter()
                .enumerate()
                .map(|(i, (cap, fixed, group))| Requirement {
                    id: RequirementId {
                        device: DeviceId::from_raw(0),
                        index: i as u16,
                    },
                    capability: cap,
                    fixed: fixed.map(|f| rid(&format!("P{f}"))),
                    group: group.map(|(g, same)| {
                        (
                            GroupId {
                                device: DeviceId::from_raw(0),
                                index: g,
                            },
                            if same {
                                UnitRel::Same
                            } else {
                                UnitRel::Distinct
                            },
                        )
                    }),
                    label: String::new(),
                })
                .collect()
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(300))]

        #[test]
        fn solver_is_sound(hw in arb_hardware(), reqs in arb_requirements()) {
            if let Some(a) = solve(&hw, &reqs) {
                prop_assert!(validate(&hw, &reqs, &a).is_empty(), "{a:?}");
            }
        }

        #[test]
        fn solver_agrees_with_brute_force(hw in arb_hardware(), reqs in arb_requirements()) {
            prop_assert_eq!(solve(&hw, &reqs).is_some(), brute_force_satisfiable(&hw, &reqs));
        }

        #[test]
        fn solver_is_deterministic_and_diagnosis_only_when_unsat(hw in arb_hardware(), reqs in arb_requirements()) {
            let a = solve(&hw, &reqs);
            prop_assert_eq!(&a, &solve(&hw, &reqs));
            if a.is_none() {
                prop_assert!(diagnose(&hw, &reqs).is_some());
            }
        }
    }
}
