//! Device kinds → requirements.  Mechanical and declarative: the solver
//! never learns what an IMU is, only that something needs SDA and SCL on
//! the same unit.

use crate::model::{Capability, GroupId, Requirement, RequirementId, ResourceId, UnitRel};
use bdl_model::surface::{DeviceBinding, DeviceKind};
use std::collections::BTreeMap;

/// One requirement template of a device kind: capability, label, and the
/// unit-relation group it belongs to (by local group index).
struct Need {
    cap: Capability,
    label: &'static str,
    group: Option<(u16, UnitRel)>,
}

fn needs(kind: DeviceKind) -> Vec<Need> {
    use Capability::*;
    match kind {
        DeviceKind::PwmChannel => vec![Need {
            cap: Pwm,
            label: "PWM",
            group: None,
        }],
        DeviceKind::DigitalOutput => vec![Need {
            cap: DigitalOut,
            label: "digital out",
            group: None,
        }],
        DeviceKind::HBridgeChannel => vec![
            Need {
                cap: Pwm,
                label: "PWM",
                group: None,
            },
            Need {
                cap: DigitalOut,
                label: "direction",
                group: None,
            },
        ],
        DeviceKind::I2cSensor => vec![
            Need {
                cap: I2cSda,
                label: "SDA",
                group: Some((0, UnitRel::Same)),
            },
            Need {
                cap: I2cScl,
                label: "SCL",
                group: Some((0, UnitRel::Same)),
            },
        ],
        DeviceKind::QuadratureEncoder => vec![
            Need {
                cap: Interrupt,
                label: "A",
                group: None,
            },
            Need {
                cap: Interrupt,
                label: "B",
                group: None,
            },
        ],
        DeviceKind::Uart => vec![
            Need {
                cap: UartTx,
                label: "TX",
                group: Some((0, UnitRel::Same)),
            },
            Need {
                cap: UartRx,
                label: "RX",
                group: Some((0, UnitRel::Same)),
            },
        ],
    }
}

/// The requirements of one device binding.  Manual pins become `fixed`
/// resources by name; they are resolved against the target by the solver's
/// unary check.
pub fn requirements_for(device: &DeviceBinding) -> Vec<Requirement> {
    needs(device.kind)
        .into_iter()
        .enumerate()
        .map(|(i, need)| {
            let index = i as u16;
            Requirement {
                id: RequirementId {
                    device: device.id,
                    index,
                },
                capability: need.cap,
                fixed: device
                    .fixed_pins
                    .get(&index)
                    .map(|p| ResourceId::new(p.clone())),
                group: need.group.map(|(g, rel)| {
                    (
                        GroupId {
                            device: device.id,
                            index: g,
                        },
                        rel,
                    )
                }),
                label: format!("{} {}", device.name, need.label),
            }
        })
        .collect()
}

/// Requirements of every device, in device-id order.
pub fn requirements_for_all<'a>(
    devices: impl IntoIterator<Item = &'a DeviceBinding>,
) -> Vec<Requirement> {
    let ordered: BTreeMap<_, _> = devices.into_iter().map(|d| (d.id, d)).collect();
    ordered.values().flat_map(|d| requirements_for(d)).collect()
}

/// The per-index labels of a kind, for editors offering manual pins.
pub fn requirement_labels(kind: DeviceKind) -> Vec<(u16, Capability, &'static str)> {
    needs(kind)
        .into_iter()
        .enumerate()
        .map(|(i, n)| (i as u16, n.cap, n.label))
        .collect()
}
