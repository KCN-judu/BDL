//! The vertical slice of the behaviour-system milestone (brief §48), built
//! through the system edit model, plus the hand-written flat equivalent.

#![allow(dead_code, clippy::unwrap_used)]

use bdl_model::edit::EditOp;
use bdl_model::surface::{Definition, Design, DeviceKind, Representation, Signature};
use bdl_model::{ClockId, DeclId, Dim, OutputId, SemanticId};
use bdl_system::*;

/// A builder over the system edit model that keeps the ids it created.
pub struct Sys {
    pub snap: SystemSnapshot,
}

impl Sys {
    pub fn new(name: &str) -> Sys {
        Sys {
            snap: SystemSnapshot::new(BehaviorSystem::empty(name)),
        }
    }
    pub fn apply(&mut self, op: SystemEditOp) -> SystemEditOutcome {
        let a = apply_system_edit(&self.snap, &op).unwrap_or_else(|e| panic!("{op:?}: {e}"));
        self.snap = a.snapshot;
        a.outcome
    }
    pub fn try_apply(&mut self, op: SystemEditOp) -> Result<SystemEditOutcome, SystemEditError> {
        let a = apply_system_edit(&self.snap, &op)?;
        self.snap = a.snapshot;
        Ok(a.outcome)
    }
    pub fn system(&self) -> &BehaviorSystem {
        &self.snap.system
    }

    // ---- base --------------------------------------------------------------

    pub fn base(&mut self, op: EditOp) -> SystemEditOutcome {
        self.apply(SystemEditOp::Base { op })
    }
    pub fn base_concept(&mut self, name: &str, rep: Representation) -> SemanticId {
        self.base(EditOp::CreateConcept {
            name: name.into(),
            description: String::new(),
            representation: Some(rep),
        })
        .inner
        .unwrap()
        .created_concept
        .unwrap()
    }
    pub fn base_clock(&mut self, name: &str) -> ClockId {
        self.base(EditOp::CreateClockDomain { name: name.into() })
            .inner
            .unwrap()
            .created_clock
            .unwrap()
    }
    pub fn base_output(&mut self, name: &str, accepts: SemanticId, clock: ClockId) -> OutputId {
        self.base(EditOp::CreateOutput {
            name: name.into(),
            description: String::new(),
            accepts,
            clock: Some(clock),
        })
        .inner
        .unwrap()
        .created_output
        .unwrap()
    }

    // ---- components ---------------------------------------------------------

    pub fn component(&mut self, name: &str) -> ComponentId {
        self.apply(SystemEditOp::CreateComponent {
            name: name.into(),
            description: String::new(),
        })
        .created_component
        .unwrap()
    }
    pub fn body(&mut self, c: ComponentId, op: EditOp) -> SystemEditOutcome {
        self.apply(SystemEditOp::EditComponentBody { component: c, op })
    }
    pub fn body_concept(
        &mut self,
        c: ComponentId,
        name: &str,
        rep: Option<Representation>,
    ) -> SemanticId {
        self.body(
            c,
            EditOp::CreateConcept {
                name: name.into(),
                description: String::new(),
                representation: rep,
            },
        )
        .inner
        .unwrap()
        .created_concept
        .unwrap()
    }
    pub fn body_clock(&mut self, c: ComponentId, name: &str) -> ClockId {
        self.body(c, EditOp::CreateClockDomain { name: name.into() })
            .inner
            .unwrap()
            .created_clock
            .unwrap()
    }
    pub fn body_mapping(
        &mut self,
        c: ComponentId,
        name: &str,
        inputs: &[SemanticId],
        output: SemanticId,
    ) -> DeclId {
        self.body(
            c,
            EditOp::CreateMapping {
                name: name.into(),
                description: String::new(),
                signature: Signature {
                    inputs: inputs.to_vec(),
                    output,
                },
            },
        )
        .inner
        .unwrap()
        .created_mapping
        .unwrap()
    }
    pub fn body_formula(&mut self, c: ComponentId, m: DeclId, source: &str) {
        self.body(
            c,
            EditOp::AttachDefinition {
                id: m,
                definition: Definition::Formula {
                    source: source.into(),
                },
            },
        );
    }
    pub fn body_clock_of(&mut self, c: ComponentId, m: DeclId, clock: ClockId) {
        self.body(
            c,
            EditOp::SetMappingClock {
                id: m,
                clock: Some(clock),
            },
        );
    }
    pub fn body_output(
        &mut self,
        c: ComponentId,
        name: &str,
        accepts: SemanticId,
        clock: ClockId,
    ) -> OutputId {
        self.body(
            c,
            EditOp::CreateOutput {
                name: name.into(),
                description: String::new(),
                accepts,
                clock: Some(clock),
            },
        )
        .inner
        .unwrap()
        .created_output
        .unwrap()
    }
    pub fn body_drive(&mut self, c: ComponentId, m: DeclId, o: OutputId) {
        self.body(
            c,
            EditOp::SetMappingDrive {
                id: m,
                output: Some(o),
            },
        );
    }
    pub fn body_device(&mut self, c: ComponentId, name: &str, kind: DeviceKind, output: OutputId) {
        self.body(
            c,
            EditOp::CreateDevice {
                name: name.into(),
                kind,
                output: Some(output),
            },
        );
    }
    pub fn share(&mut self, c: ComponentId, local: SemanticId, system: SemanticId) {
        self.apply(SystemEditOp::ShareConcept {
            component: c,
            local,
            system: Some(system),
        });
    }
    pub fn clock_param(&mut self, c: ComponentId, clock: ClockId) {
        self.apply(SystemEditOp::SetClockParameter {
            component: c,
            clock,
            parameter: true,
        });
    }
    pub fn port(&mut self, c: ComponentId, decl: DeclId, kind: PortKind, name: &str) -> PortId {
        self.apply(SystemEditOp::DeclarePort {
            component: c,
            decl,
            kind,
            name: name.into(),
            description: String::new(),
        })
        .created_port
        .unwrap()
    }

    // ---- instances and bindings --------------------------------------------

    pub fn instance(&mut self, c: ComponentId, name: &str) -> ComponentInstanceId {
        self.apply(SystemEditOp::CreateInstance {
            component: c,
            name: name.into(),
        })
        .created_instance
        .unwrap()
    }
    pub fn clock_arg(&mut self, i: ComponentInstanceId, param: ClockId, clock: ClockId) {
        self.apply(SystemEditOp::SetClockArgument {
            instance: i,
            parameter: param,
            clock: Some(clock),
        });
    }
    pub fn bind(&mut self, source: PortRef, destination: PortRef) -> BindingId {
        self.apply(SystemEditOp::BindPorts {
            source,
            destination,
            transport: None,
        })
        .created_binding
        .unwrap()
    }
    pub fn bind_transported(
        &mut self,
        source: PortRef,
        destination: PortRef,
        init: &str,
    ) -> BindingId {
        self.apply(SystemEditOp::BindPorts {
            source,
            destination,
            transport: Some(BindingTransport { init: init.into() }),
        })
        .created_binding
        .unwrap()
    }
    pub fn flat_decl(&self, i: ComponentInstanceId, local: DeclId) -> DeclId {
        DeclId::from_raw(
            self.system()
                .flat_ids
                .get(i, LocalEntity::Decl(local))
                .unwrap(),
        )
    }
    pub fn flat_sem(&self, i: ComponentInstanceId, local: SemanticId) -> SemanticId {
        SemanticId::from_raw(
            self.system()
                .flat_ids
                .get(i, LocalEntity::Sem(local))
                .unwrap(),
        )
    }
}

pub fn pr(instance: ComponentInstanceId, port: PortId) -> PortRef {
    PortRef { instance, port }
}

pub const ANGLE: Representation = Representation::Quantity { dim: Dim::ANGLE };
pub const LEVEL: Representation = Representation::Quantity { dim: Dim::ZERO };

/// Every id the vertical slice creates.
pub struct Slice {
    pub sys: Sys,
    pub tilt: SemanticId,
    pub main: ClockId,
    // TiltSource
    pub source: ComponentId,
    pub source_raw: DeclId,
    pub source_tilt_value: DeclId,
    pub source_tick: ClockId,
    pub source_port: PortId,
    // AdaptiveLamp
    pub lamp: ComponentId,
    pub lamp_tilt: SemanticId,
    pub lamp_brightness_concept: SemanticId,
    pub lamp_tick: ClockId,
    pub lamp_tilt_value: DeclId,
    pub lamp_dim: DeclId,
    pub lamp_brightness: DeclId,
    pub lamp_light: OutputId,
    pub lamp_in: PortId,
    pub lamp_out: PortId,
    // instances
    pub sensor: ComponentInstanceId,
    pub lamp_a: ComponentInstanceId,
    pub lamp_b: ComponentInstanceId,
    pub bind_a: BindingId,
    pub bind_b: BindingId,
}

/// Shared `Tilt : Angle`; `TiltSource` provides `tiltValue : () -> Tilt`
/// (from an unresolved sensor reading `raw`); `AdaptiveLamp` requires
/// `tiltValue`, owns `Brightness`, provides `brightness = dimByTilt(tiltValue)`
/// and drives a private `light` sink with a PWM device; `sensor`, `lampA`,
/// `lampB` all in the system domain `main`; `sensor.tiltValue` bound into
/// both lamps.
pub fn vertical_slice() -> Slice {
    let mut sys = Sys::new("rover");
    let tilt = sys.base_concept("Tilt", ANGLE);
    let main = sys.base_clock("main");

    let source = sys.component("TiltSource");
    let st = sys.body_concept(source, "Tilt", Some(ANGLE));
    sys.share(source, st, tilt);
    let source_tick = sys.body_clock(source, "tick");
    sys.clock_param(source, source_tick);
    let source_raw = sys.body_mapping(source, "raw", &[], st);
    sys.body_clock_of(source, source_raw, source_tick);
    let source_tilt_value = sys.body_mapping(source, "tiltValue", &[], st);
    sys.body_formula(source, source_tilt_value, "raw");
    sys.body_clock_of(source, source_tilt_value, source_tick);
    let source_port = sys.port(source, source_tilt_value, PortKind::Provided, "tiltValue");

    let lamp = sys.component("AdaptiveLamp");
    let lamp_tilt = sys.body_concept(lamp, "Tilt", Some(ANGLE));
    sys.share(lamp, lamp_tilt, tilt);
    let lamp_brightness_concept = sys.body_concept(lamp, "Brightness", Some(LEVEL));
    let lamp_tick = sys.body_clock(lamp, "tick");
    sys.clock_param(lamp, lamp_tick);
    let lamp_tilt_value = sys.body_mapping(lamp, "tiltValue", &[], lamp_tilt);
    sys.body_clock_of(lamp, lamp_tilt_value, lamp_tick);
    let lamp_dim = sys.body_mapping(lamp, "dimByTilt", &[lamp_tilt], lamp_brightness_concept);
    sys.body_formula(lamp, lamp_dim, "Tilt / 90 deg");
    let lamp_brightness = sys.body_mapping(lamp, "brightness", &[], lamp_brightness_concept);
    sys.body_formula(lamp, lamp_brightness, "dimByTilt(tiltValue)");
    sys.body_clock_of(lamp, lamp_brightness, lamp_tick);
    let lamp_light = sys.body_output(lamp, "light", lamp_brightness_concept, lamp_tick);
    sys.body_drive(lamp, lamp_brightness, lamp_light);
    sys.body_device(lamp, "led", DeviceKind::PwmChannel, lamp_light);
    let lamp_in = sys.port(lamp, lamp_tilt_value, PortKind::Required, "tiltValue");
    let lamp_out = sys.port(lamp, lamp_brightness, PortKind::Provided, "brightness");

    let sensor = sys.instance(source, "sensor");
    sys.clock_arg(sensor, source_tick, main);
    let lamp_a = sys.instance(lamp, "lampA");
    sys.clock_arg(lamp_a, lamp_tick, main);
    let lamp_b = sys.instance(lamp, "lampB");
    sys.clock_arg(lamp_b, lamp_tick, main);
    let bind_a = sys.bind(pr(sensor, source_port), pr(lamp_a, lamp_in));
    let bind_b = sys.bind(pr(sensor, source_port), pr(lamp_b, lamp_in));

    Slice {
        sys,
        tilt,
        main,
        source,
        source_raw,
        source_tilt_value,
        source_tick,
        source_port,
        lamp,
        lamp_tilt,
        lamp_brightness_concept,
        lamp_tick,
        lamp_tilt_value,
        lamp_dim,
        lamp_brightness,
        lamp_light,
        lamp_in,
        lamp_out,
        sensor,
        lamp_a,
        lamp_b,
        bind_a,
        bind_b,
    }
}

/// The same behaviour, written by hand as one flat design with the
/// ordinary edit ops: two lamps, each with its own Brightness concept and
/// light sink, reading one tilt value.
pub struct Flat {
    pub snap: bdl_model::surface::ProjectSnapshot,
    pub raw: DeclId,
    pub tilt_value: DeclId,
    pub brightness_a: DeclId,
    pub brightness_b: DeclId,
    pub light_a: OutputId,
    pub light_b: OutputId,
}

pub fn hand_written_flat() -> Flat {
    use bdl_model::edit::apply_edit;
    use bdl_model::surface::ProjectSnapshot;
    let mut s = ProjectSnapshot::new(Design::empty("rover-flat"));
    let mut ap = |s: &mut ProjectSnapshot, op: EditOp| {
        let a = apply_edit(s, &op).unwrap();
        *s = a.snapshot;
        a.outcome
    };
    let concept =
        |s: &mut ProjectSnapshot,
         ap: &mut dyn FnMut(&mut ProjectSnapshot, EditOp) -> bdl_model::edit::EditOutcome,
         name: &str,
         rep| {
            ap(
                s,
                EditOp::CreateConcept {
                    name: name.into(),
                    description: String::new(),
                    representation: Some(rep),
                },
            )
            .created_concept
            .unwrap()
        };
    let tilt = concept(&mut s, &mut ap, "Tilt", ANGLE);
    let ba = concept(&mut s, &mut ap, "BrightnessA", LEVEL);
    let bb = concept(&mut s, &mut ap, "BrightnessB", LEVEL);
    let main = ap(
        &mut s,
        EditOp::CreateClockDomain {
            name: "main".into(),
        },
    )
    .created_clock
    .unwrap();
    let mapping =
        |s: &mut ProjectSnapshot,
         ap: &mut dyn FnMut(&mut ProjectSnapshot, EditOp) -> bdl_model::edit::EditOutcome,
         name: &str,
         inputs: Vec<SemanticId>,
         output,
         formula: Option<&str>,
         clock: Option<ClockId>| {
            let id = ap(
                s,
                EditOp::CreateMapping {
                    name: name.into(),
                    description: String::new(),
                    signature: Signature { inputs, output },
                },
            )
            .created_mapping
            .unwrap();
            if let Some(f) = formula {
                ap(
                    s,
                    EditOp::AttachDefinition {
                        id,
                        definition: Definition::Formula { source: f.into() },
                    },
                );
            }
            if clock.is_some() {
                ap(s, EditOp::SetMappingClock { id, clock });
            }
            id
        };
    let raw = mapping(&mut s, &mut ap, "raw", vec![], tilt, None, Some(main));
    let tilt_value = mapping(
        &mut s,
        &mut ap,
        "tiltValue",
        vec![],
        tilt,
        Some("raw"),
        Some(main),
    );
    let tilt_a = mapping(
        &mut s,
        &mut ap,
        "tiltA",
        vec![],
        tilt,
        Some("tiltValue"),
        Some(main),
    );
    let tilt_b = mapping(
        &mut s,
        &mut ap,
        "tiltB",
        vec![],
        tilt,
        Some("tiltValue"),
        Some(main),
    );
    let _dim_a = mapping(
        &mut s,
        &mut ap,
        "dimA",
        vec![tilt],
        ba,
        Some("Tilt / 90 deg"),
        None,
    );
    let _dim_b = mapping(
        &mut s,
        &mut ap,
        "dimB",
        vec![tilt],
        bb,
        Some("Tilt / 90 deg"),
        None,
    );
    let brightness_a = mapping(
        &mut s,
        &mut ap,
        "brightnessA",
        vec![],
        ba,
        Some("dimA(tiltA)"),
        Some(main),
    );
    let brightness_b = mapping(
        &mut s,
        &mut ap,
        "brightnessB",
        vec![],
        bb,
        Some("dimB(tiltB)"),
        Some(main),
    );
    let _ = (tilt_a, tilt_b);
    let light_a = ap(
        &mut s,
        EditOp::CreateOutput {
            name: "lightA".into(),
            description: String::new(),
            accepts: ba,
            clock: Some(main),
        },
    )
    .created_output
    .unwrap();
    let light_b = ap(
        &mut s,
        EditOp::CreateOutput {
            name: "lightB".into(),
            description: String::new(),
            accepts: bb,
            clock: Some(main),
        },
    )
    .created_output
    .unwrap();
    ap(
        &mut s,
        EditOp::SetMappingDrive {
            id: brightness_a,
            output: Some(light_a),
        },
    );
    ap(
        &mut s,
        EditOp::SetMappingDrive {
            id: brightness_b,
            output: Some(light_b),
        },
    );
    ap(
        &mut s,
        EditOp::CreateDevice {
            name: "ledA".into(),
            kind: DeviceKind::PwmChannel,
            output: Some(light_a),
        },
    );
    ap(
        &mut s,
        EditOp::CreateDevice {
            name: "ledB".into(),
            kind: DeviceKind::PwmChannel,
            output: Some(light_b),
        },
    );
    Flat {
        snap: s,
        raw,
        tilt_value,
        brightness_a,
        brightness_b,
        light_a,
        light_b,
    }
}
