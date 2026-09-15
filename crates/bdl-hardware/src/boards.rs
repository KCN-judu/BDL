//! Concrete targets as data.  These constructors are the source of truth
//! for the checked-in `hardware/boards/*.toml` (a test keeps them equal);
//! a board package later loads the TOML instead.

use crate::model::{Capability, Hardware, Resource, ResourceId, UnitId};
use std::collections::{BTreeMap, BTreeSet};

fn res(id: &str, caps: &[Capability], units: &[(Capability, u32)]) -> Resource {
    Resource {
        id: ResourceId::new(id),
        capabilities: caps.iter().copied().collect(),
        units: units.iter().map(|(c, u)| (*c, UnitId(*u))).collect(),
    }
}

/// The Arduino Nano (ATmega328P) as the paper's Phase-7 table: PWM on D3,
/// D5, D6, D9, D10, D11 backed by timers 2, 0, 0, 1, 1, 2; external
/// interrupts on D2 and D3; I2C on A4/A5 (unit 0); SPI unit 0 on D10–D13;
/// UART unit 0 on D0/D1.  Buses are shareable, everything else exclusive.
pub fn arduino_nano() -> Hardware {
    use Capability::*;
    let dio = [DigitalIn, DigitalOut];
    Hardware {
        name: "arduino_nano".into(),
        resources: vec![
            res("D0", &[DigitalIn, DigitalOut, UartRx], &[(UartRx, 0)]),
            res("D1", &[DigitalIn, DigitalOut, UartTx], &[(UartTx, 0)]),
            res("D2", &[DigitalIn, DigitalOut, Interrupt], &[]),
            res("D3", &[DigitalIn, DigitalOut, Pwm, Interrupt], &[(Pwm, 2)]),
            res("D4", &dio, &[]),
            res("D5", &[DigitalIn, DigitalOut, Pwm], &[(Pwm, 0)]),
            res("D6", &[DigitalIn, DigitalOut, Pwm], &[(Pwm, 0)]),
            res("D7", &dio, &[]),
            res("D8", &dio, &[]),
            res("D9", &[DigitalIn, DigitalOut, Pwm], &[(Pwm, 1)]),
            res(
                "D10",
                &[DigitalIn, DigitalOut, Pwm, SpiSs],
                &[(Pwm, 1), (SpiSs, 0)],
            ),
            res(
                "D11",
                &[DigitalIn, DigitalOut, Pwm, SpiMosi],
                &[(Pwm, 2), (SpiMosi, 0)],
            ),
            res("D12", &[DigitalIn, DigitalOut, SpiMiso], &[(SpiMiso, 0)]),
            res("D13", &[DigitalIn, DigitalOut, SpiSck], &[(SpiSck, 0)]),
            res("A0", &[AnalogIn, DigitalIn, DigitalOut], &[]),
            res("A1", &[AnalogIn, DigitalIn, DigitalOut], &[]),
            res("A2", &[AnalogIn, DigitalIn, DigitalOut], &[]),
            res("A3", &[AnalogIn, DigitalIn, DigitalOut], &[]),
            res(
                "A4",
                &[AnalogIn, DigitalIn, DigitalOut, I2cSda],
                &[(I2cSda, 0)],
            ),
            res(
                "A5",
                &[AnalogIn, DigitalIn, DigitalOut, I2cScl],
                &[(I2cScl, 0)],
            ),
            res("A6", &[AnalogIn], &[]),
            res("A7", &[AnalogIn], &[]),
        ],
        shareable: [I2cSda, I2cScl].into_iter().collect(),
    }
}

/// The paper's mock larger board: the Nano plus six more PWM pins on three
/// more timers.
pub fn big_board() -> Hardware {
    use Capability::*;
    let mut hw = arduino_nano();
    hw.name = "big_board".into();
    for (pin, timer) in [
        ("D40", 3),
        ("D41", 3),
        ("D42", 4),
        ("D43", 4),
        ("D44", 5),
        ("D45", 5),
    ] {
        hw.resources
            .push(res(pin, &[DigitalIn, DigitalOut, Pwm], &[(Pwm, timer)]));
    }
    hw
}

/// Every built-in target, by id.
pub fn registry() -> BTreeMap<String, Hardware> {
    [arduino_nano(), big_board()]
        .into_iter()
        .map(|h| (h.name.clone(), h))
        .collect()
}

pub fn by_name(name: &str) -> Option<Hardware> {
    registry().remove(name)
}

/// Sanity: every unit-bearing capability is one the resource has.
pub fn well_formed(hw: &Hardware) -> bool {
    let ids: BTreeSet<&ResourceId> = hw.resource_ids().collect();
    ids.len() == hw.resources.len()
        && hw
            .resources
            .iter()
            .all(|r| r.units.keys().all(|c| r.capabilities.contains(c)))
}
