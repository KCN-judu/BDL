//! The designs a new project can start from: one canonical hardware demo
//! in two variants, and the checked-in examples that run on an Arduino
//! Nano, all written in textual BDL and loaded through the same path as
//! any hand-written project (`bdl_text::init_project_with_source`).
//!
//! `button-lamp` is the Button → rule → Lamp product of
//! `docs/user-guide/getting-started/pico-demo.md`: a Source the
//! environment provides (`pressed`), a value that follows it (`lit`), a
//! physical output (`lamp`).  The *guided* variant ships the design alone
//! — the deployment (which device provides the button, which realises the
//! lamp, on which pads) is the designer's task; the *configured* variant
//! ships it complete for the Raspberry Pi Pico: the button on GP2 against
//! ground, the lamp on the on-board LED (GP25).

/// The Nano examples are the checked-in projects themselves
/// (`examples/`, kept faithful by `crates/bdl-compiler/tests/example_*.rs`),
/// never a copy: their devices and pins are part of the source.
const SENSOR_STATION: &str = include_str!("../../../examples/sensor_station/src/main.bdl");
const BREATHING_BEACON: &str = include_str!("../../../examples/breathing_beacon/src/main.bdl");

/// One template as `ListTemplates` reports it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Template {
    pub id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    /// The board the demo is written for.
    pub target_id: &'static str,
    /// Whether the deployment comes with the design.
    pub configured: bool,
    /// The whole `src/main.bdl`.
    pub source: String,
}

const DESIGN: &str = "\
/// Whether the button is held down.
concept Pressed : Bool

/// Whether the lamp is lit.
concept Lit : Bool

clock main

/// The push button: the environment provides it, read once per activation.
mapping pressed : Pressed @main

/// The lamp is lit exactly while the button is pressed.
mapping lit : Lit @main
lit() =
  pressed

/// The lamp itself.
output lamp : Lit @main

drive lamp by lit
";

const DEPLOYMENT: &str = "
// The Raspberry Pi Pico: the button between GP2 and ground (the line is
// pulled up, pressed reads low), the lamp on the on-board LED.
device button : digital_input for pressed { provider gpio_level_in_low, pin 0 = GP2 }

device led : digital_output for lamp { realization gpio_level, pin 0 = GP25 }
";

/// Every template, in the order a chooser lists them.
pub fn templates() -> Vec<Template> {
    vec![
        Template {
            id: "button-lamp",
            display_name: "Button → Lamp",
            description: "For the Raspberry Pi Pico: a push button and a lamp that follows it — one Source, one relationship, one physical output. The deployment (which device provides the button, which realises the lamp, on which pads) is yours to make on the Deploy page.",
            target_id: "rp2040_pico",
            configured: false,
            source: DESIGN.to_owned(),
        },
        Template {
            id: "button-lamp-configured",
            display_name: "Button → Lamp, wired",
            description: "For the Raspberry Pi Pico: the same design with its deployment made — the button between GP2 and ground, the lamp on the on-board LED. Open the Deploy page, Build, Flash.",
            target_id: "rp2040_pico",
            configured: true,
            source: format!("{DESIGN}{DEPLOYMENT}"),
        },
        Template {
            id: "sensor-station",
            display_name: "Sensor Station, wired",
            description: "For the Arduino Nano: a thermistor, a photoresistor and a trim pot read through their curves; a night light with a gamma curve, a fan with inertia and an overheat lamp with hysteresis. Wired on A0–A2, D9, D10, D13 — open the Deploy page, Build.",
            target_id: "arduino_nano",
            configured: true,
            source: SENSOR_STATION.to_owned(),
        },
        Template {
            id: "breathing-beacon",
            display_name: "Breathing Beacon, wired",
            description: "For the Arduino Nano: no sensor, only dynamics — a triangle-wave breath, a gamma curve, a first-order lag and a Schmitt trigger driving two PWM pins and the on-board LED (D9, D10, D13).",
            target_id: "arduino_nano",
            configured: true,
            source: BREATHING_BEACON.to_owned(),
        },
    ]
}

pub fn template(id: &str) -> Option<Template> {
    templates().into_iter().find(|t| t.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_configured_variant_is_the_guided_design_plus_its_deployment() {
        let guided = template("button-lamp").unwrap();
        let configured = template("button-lamp-configured").unwrap();
        assert_eq!(guided.source, DESIGN);
        assert_eq!(configured.source, format!("{DESIGN}{DEPLOYMENT}"));
        assert!(!guided.configured && configured.configured);
    }

    #[test]
    fn the_nano_demos_are_the_checked_in_examples_and_arrive_wired() {
        for (id, example) in [
            ("sensor-station", "sensor_station"),
            ("breathing-beacon", "breathing_beacon"),
        ] {
            let t = template(id).unwrap();
            let path = format!(
                "{}/../../examples/{example}/src/main.bdl",
                env!("CARGO_MANIFEST_DIR")
            );
            assert_eq!(t.source, std::fs::read_to_string(path).unwrap());
            assert!(t.configured && t.target_id == "arduino_nano");
        }
    }

    #[test]
    fn ids_are_unique_and_name_a_known_board() {
        let all = templates();
        let mut ids: Vec<_> = all.iter().map(|t| t.id).collect();
        ids.dedup();
        assert_eq!(ids.len(), all.len());
        for t in &all {
            assert!(
                bdl_hardware::boards::by_name(t.target_id).is_some(),
                "{}: unknown board {}",
                t.id,
                t.target_id
            );
        }
    }
}
