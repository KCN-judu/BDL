#![allow(clippy::unwrap_used)]
//! Behaviour systems over the real `bdld`: a system project is created,
//! composed through `ApplySystemEdit`, read back as designer-level views,
//! analysed as a system, and — because its flat design is derived — served
//! to every existing request (analysis, simulation, deployment) unchanged.
//! Reopening yields the same system; flat edits on a system project are
//! refused; a flat project is untouched.

use bdl_protocol::framing;
use bdl_protocol::pb::{self, client_message::Payload as Req, response::Payload as Resp};
use bytes::BytesMut;
use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

struct Client {
    child: Child,
    buf: BytesMut,
    next_id: u64,
    last_revision: u64,
    events: Vec<pb::Event>,
}

impl Client {
    fn spawn() -> Client {
        let child = Command::new(env!("CARGO_BIN_EXE_bdld"))
            .arg("serve")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("spawn bdld");
        Client {
            child,
            buf: BytesMut::new(),
            next_id: 1,
            last_revision: 0,
            events: Vec::new(),
        }
    }

    fn call(&mut self, payload: Req) -> Resp {
        let id = self.next_id;
        self.next_id += 1;
        let mut out = BytesMut::new();
        framing::encode(
            &pb::ClientMessage {
                request_id: id,
                payload: Some(payload),
            },
            &mut out,
        )
        .unwrap();
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(&out).unwrap();
        stdin.flush().unwrap();
        loop {
            let msg = loop {
                if let Some(m) = framing::decode::<pb::ServerMessage>(&mut self.buf).unwrap() {
                    break m;
                }
                let mut chunk = [0u8; 4096];
                let n = self
                    .child
                    .stdout
                    .as_mut()
                    .unwrap()
                    .read(&mut chunk)
                    .unwrap();
                assert!(n > 0, "daemon closed stdout");
                self.buf.extend_from_slice(&chunk[..n]);
            };
            match msg.payload.unwrap() {
                pb::server_message::Payload::Response(r) => {
                    assert_eq!(r.request_id, id);
                    let payload = r.payload.unwrap();
                    match &payload {
                        Resp::Project(p) => {
                            self.last_revision = p.project.as_ref().unwrap().revision
                        }
                        Resp::EditApplied(e) => {
                            self.last_revision = e.project.as_ref().unwrap().revision
                        }
                        Resp::SystemEditApplied(e) => {
                            self.last_revision = e.project.as_ref().unwrap().revision
                        }
                        _ => {}
                    }
                    return payload;
                }
                pb::server_message::Payload::Event(e) => self.events.push(e),
            }
        }
    }

    fn sys(&mut self, op: pb::system_edit_op::Op) -> pb::SystemEditApplied {
        let base = self.last_revision;
        match self.call(Req::ApplySystemEdit(pb::ApplySystemEditRequest {
            base_revision: base,
            op: Some(pb::SystemEditOp { op: Some(op) }),
        })) {
            Resp::SystemEditApplied(e) => e,
            other => panic!("system edit failed: {other:?}"),
        }
    }

    fn base(&mut self, op: pb::edit_op::Op) -> pb::EditOutcome {
        self.sys(pb::system_edit_op::Op::Base(pb::EditOp { op: Some(op) }))
            .outcome
            .unwrap()
            .inner
            .unwrap()
    }

    fn body(&mut self, component: u64, op: pb::edit_op::Op) -> pb::EditOutcome {
        self.sys(pb::system_edit_op::Op::EditComponentBody(
            pb::EditComponentBody {
                component,
                op: Some(pb::EditOp { op: Some(op) }),
            },
        ))
        .outcome
        .unwrap()
        .inner
        .unwrap()
    }

    fn system(&mut self) -> pb::SystemView {
        match self.call(Req::GetSystem(pb::GetSystemRequest {})) {
            Resp::System(s) => s.system.unwrap(),
            other => panic!("{other:?}"),
        }
    }

    fn system_analysis(&mut self) -> pb::SystemAnalysisView {
        match self.call(Req::RunSystemAnalysis(pb::RunSystemAnalysisRequest {})) {
            Resp::SystemAnalysis(a) => a.analysis.unwrap(),
            other => panic!("{other:?}"),
        }
    }
}

fn quantity(dim: pb::Dim) -> Option<pb::Representation> {
    Some(pb::Representation {
        kind: Some(pb::representation::Kind::Quantity(dim)),
    })
}

fn concept(name: &str, dim: pb::Dim) -> pb::edit_op::Op {
    pb::edit_op::Op::CreateConcept(pb::CreateConcept {
        name: name.into(),
        description: String::new(),
        representation: quantity(dim),
    })
}

fn mapping(name: &str, inputs: Vec<u64>, output: u64) -> pb::edit_op::Op {
    pb::edit_op::Op::CreateMapping(pb::CreateMapping {
        name: name.into(),
        description: String::new(),
        signature: Some(pb::Signature { inputs, output }),
        ..Default::default()
    })
}

fn formula(id: u64, source: &str) -> pb::edit_op::Op {
    pb::edit_op::Op::AttachDefinition(pb::AttachDefinition {
        id,
        definition: Some(pb::Definition {
            kind: Some(pb::definition::Kind::Formula(source.into())),
        }),
    })
}

fn port_ref(instance: u64, port: u64) -> Option<pb::PortRefView> {
    Some(pb::PortRefView {
        instance,
        port,
        base_decl: None,
    })
}

#[test]
fn a_system_project_composes_analyses_simulates_deploys_and_reopens() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("rover");
    let mut c = Client::spawn();
    c.call(Req::Handshake(pb::HandshakeRequest {
        client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
        client_name: "system-e2e".into(),
        client_version: "0".into(),
    }));
    let Resp::Project(p) = c.call(Req::InitSystemProject(pb::InitSystemProjectRequest {
        root_path: root.to_string_lossy().into(),
        name: "rover".into(),
    })) else {
        panic!()
    };
    let p = p.project.unwrap();
    // one kind of project (ADR-0023): the field always reports it
    assert_eq!(p.kind(), pb::ProjectKind::Text);
    assert_eq!(p.revision, 0);

    // a flat edit is the system edit `Base { op }`: it lands, as one
    // revision, and is undone together with every other edit
    let Resp::EditApplied(applied) = c.call(Req::ApplyEdit(pb::ApplyEditRequest {
        base_revision: 0,
        op: Some(pb::EditOp {
            op: Some(concept(
                "Probe",
                pb::Dim {
                    angle: 1,
                    ..Default::default()
                },
            )),
        }),
    })) else {
        panic!()
    };
    assert_eq!(applied.project.unwrap().revision, 1);
    let Resp::SystemEditApplied(undone) = c.call(Req::Undo(pb::UndoRequest {})) else {
        panic!()
    };
    assert!(undone.project.unwrap().concepts.is_empty());

    // the system: shared Tilt, domain main
    let angle = pb::Dim {
        angle: 1,
        ..Default::default()
    };
    let tilt = c.base(concept("Tilt", angle)).created_concept.unwrap();
    let main = c
        .base(pb::edit_op::Op::CreateClockDomain(pb::CreateClockDomain {
            name: "main".into(),
        }))
        .created_clock
        .unwrap();

    // AdaptiveLamp: shared Tilt, private Brightness, clock parameter tick,
    // required tiltValue, dimByTilt, provided brightness
    let lamp = c
        .sys(pb::system_edit_op::Op::CreateComponent(
            pb::CreateComponent {
                name: "AdaptiveLamp".into(),
                description: String::new(),
            },
        ))
        .outcome
        .unwrap()
        .created_component
        .unwrap();
    let l_tilt = c
        .body(lamp, concept("Tilt", angle))
        .created_concept
        .unwrap();
    c.sys(pb::system_edit_op::Op::ShareConcept(pb::ShareConcept {
        component: lamp,
        local: l_tilt,
        system: Some(tilt),
    }));
    let l_bright = c
        .body(lamp, concept("Brightness", pb::Dim::default()))
        .created_concept
        .unwrap();
    let l_tick = c
        .body(
            lamp,
            pb::edit_op::Op::CreateClockDomain(pb::CreateClockDomain {
                name: "tick".into(),
            }),
        )
        .created_clock
        .unwrap();
    c.sys(pb::system_edit_op::Op::SetClockParameter(
        pb::SetClockParameter {
            component: lamp,
            clock: l_tick,
            parameter: true,
        },
    ));
    let l_in = c
        .body(lamp, mapping("tiltValue", vec![], l_tilt))
        .created_mapping
        .unwrap();
    c.body(
        lamp,
        pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
            id: l_in,
            clock_id: Some(l_tick),
        }),
    );
    let l_dim = c
        .body(lamp, mapping("dimByTilt", vec![l_tilt], l_bright))
        .created_mapping
        .unwrap();
    c.body(lamp, formula(l_dim, "Tilt / 90 deg"));
    let l_out = c
        .body(lamp, mapping("brightness", vec![], l_bright))
        .created_mapping
        .unwrap();
    c.body(lamp, formula(l_out, "dimByTilt(tiltValue)"));
    c.body(
        lamp,
        pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
            id: l_out,
            clock_id: Some(l_tick),
        }),
    );
    let mut dp = pb::DeclarePort {
        component: lamp,
        decl: l_in,
        name: "tiltValue".into(),
        description: String::new(),
        ..Default::default()
    };
    dp.set_kind(pb::PortKind::Required);
    let port_in = c
        .sys(pb::system_edit_op::Op::DeclarePort(dp))
        .outcome
        .unwrap()
        .created_port
        .unwrap();
    let mut dp = pb::DeclarePort {
        component: lamp,
        decl: l_out,
        name: "brightness".into(),
        description: String::new(),
        ..Default::default()
    };
    dp.set_kind(pb::PortKind::Provided);
    let port_out = c
        .sys(pb::system_edit_op::Op::DeclarePort(dp))
        .outcome
        .unwrap()
        .created_port
        .unwrap();

    // TiltSource: provides tiltValue from an unresolved reading
    let source = c
        .sys(pb::system_edit_op::Op::CreateComponent(
            pb::CreateComponent {
                name: "TiltSource".into(),
                description: String::new(),
            },
        ))
        .outcome
        .unwrap()
        .created_component
        .unwrap();
    let s_tilt = c
        .body(source, concept("Tilt", angle))
        .created_concept
        .unwrap();
    c.sys(pb::system_edit_op::Op::ShareConcept(pb::ShareConcept {
        component: source,
        local: s_tilt,
        system: Some(tilt),
    }));
    let s_tick = c
        .body(
            source,
            pb::edit_op::Op::CreateClockDomain(pb::CreateClockDomain {
                name: "tick".into(),
            }),
        )
        .created_clock
        .unwrap();
    c.sys(pb::system_edit_op::Op::SetClockParameter(
        pb::SetClockParameter {
            component: source,
            clock: s_tick,
            parameter: true,
        },
    ));
    let s_raw = c
        .body(source, mapping("raw", vec![], s_tilt))
        .created_mapping
        .unwrap();
    c.body(
        source,
        pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
            id: s_raw,
            clock_id: Some(s_tick),
        }),
    );
    let s_val = c
        .body(source, mapping("tiltValue", vec![], s_tilt))
        .created_mapping
        .unwrap();
    c.body(source, formula(s_val, "raw"));
    c.body(
        source,
        pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
            id: s_val,
            clock_id: Some(s_tick),
        }),
    );
    let mut dp = pb::DeclarePort {
        component: source,
        decl: s_val,
        name: "tiltValue".into(),
        description: String::new(),
        ..Default::default()
    };
    dp.set_kind(pb::PortKind::Provided);
    let s_port = c
        .sys(pb::system_edit_op::Op::DeclarePort(dp))
        .outcome
        .unwrap()
        .created_port
        .unwrap();

    // instances and bindings
    let inst = |c: &mut Client, comp: u64, name: &str, param: u64| -> u64 {
        let id = c
            .sys(pb::system_edit_op::Op::CreateInstance(pb::CreateInstance {
                component: comp,
                name: name.into(),
            }))
            .outcome
            .unwrap()
            .created_instance
            .unwrap();
        c.sys(pb::system_edit_op::Op::SetClockArgument(
            pb::SetClockArgument {
                instance: id,
                parameter: param,
                clock: Some(main),
            },
        ));
        id
    };
    let sensor = inst(&mut c, source, "sensor", s_tick);
    let lamp_a = inst(&mut c, lamp, "lampA", l_tick);
    let lamp_b = inst(&mut c, lamp, "lampB", l_tick);
    let applied = c.sys(pb::system_edit_op::Op::BindPorts(pb::BindPorts {
        source: port_ref(sensor, s_port),
        destination: port_ref(lamp_a, port_in),
        transport_init: None,
    }));
    let outcome = applied.outcome.unwrap();
    assert_eq!(outcome.kind(), pb::EditKind::Refinement);
    assert_eq!(outcome.instances, vec![sensor, lamp_a]);
    assert_eq!(
        outcome.origin_decls.len(),
        1,
        "the destination's flat declaration"
    );
    // the derived flat design came back with the edit
    let flat = applied.project.unwrap();
    assert_eq!(flat.kind(), pb::ProjectKind::Text);
    assert!(flat.mappings.iter().any(|m| m.name == "lampA.tiltValue"));
    c.sys(pb::system_edit_op::Op::BindPorts(pb::BindPorts {
        source: port_ref(sensor, s_port),
        destination: port_ref(lamp_b, port_in),
        transport_init: None,
    }));
    // a second binding into the same port is refused with a stable code
    let base = c.last_revision;
    let Resp::Error(e) = c.call(Req::ApplySystemEdit(pb::ApplySystemEditRequest {
        base_revision: base,
        op: Some(pb::SystemEditOp {
            op: Some(pb::system_edit_op::Op::BindPorts(pb::BindPorts {
                source: port_ref(sensor, s_port),
                destination: port_ref(lamp_b, port_in),
                transport_init: None,
            })),
        }),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "system_edit.destination_bound");

    // the designer-level view
    let v = c.system();
    assert_eq!(v.revision, c.last_revision);
    assert!(!v.is_flat);
    assert_eq!(v.components.len(), 2);
    assert_eq!(
        v.instances
            .iter()
            .map(|i| i.name.as_str())
            .collect::<Vec<_>>(),
        ["sensor", "lampA", "lampB"],
        "in id order"
    );
    assert_eq!(v.bindings.len(), 2);
    let lamp_view = v.components.iter().find(|x| x.id == lamp).unwrap();
    assert_eq!(lamp_view.ports.len(), 2);
    // provenance: the flat brightness of lampA points back at the instance and its port
    let o = v
        .origins
        .iter()
        .find(|o| o.instance == lamp_a && o.local == l_out && o.sort() == pb::LocalSort::Decl)
        .unwrap();
    assert_eq!(o.port, Some(port_out));
    let flat_brightness_a = o.flat;
    let flat_raw = v
        .origins
        .iter()
        .find(|o| o.instance == sensor && o.local == s_raw && o.sort() == pb::LocalSort::Decl)
        .unwrap()
        .flat;

    // system analysis: executable, every port accounted for
    let a = c.system_analysis();
    assert_eq!(
        a.acceptance(),
        pb::SystemAcceptance::Executable,
        "{:?}",
        a.projected
    );
    assert!(a.composition.is_empty());
    let st = |a: &pb::SystemAnalysisView, i, p| {
        a.ports
            .iter()
            .find(|x| x.port == port_ref(i, p))
            .unwrap()
            .status()
    };
    assert_eq!(st(&a, lamp_a, port_in), pb::PortStatusKind::Bound);
    assert_eq!(st(&a, lamp_b, port_in), pb::PortStatusKind::Bound);
    assert_eq!(st(&a, sensor, s_port), pb::PortStatusKind::Provided);
    assert!(
        a.analysis.as_ref().unwrap().output_complete
            || a.analysis.as_ref().unwrap().outputs.is_empty()
    );

    // the existing flat requests work on the derived design unchanged
    let Resp::Analysis(fa) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {})) else {
        panic!()
    };
    let fa = fa.analysis.unwrap();
    assert!(fa.causal && fa.clock_consistent);
    assert_eq!(fa.revision, c.last_revision);
    let Resp::Simulation(sim) = c.call(Req::StartSimulation(pb::StartSimulationRequest {
        inputs: (0..4)
            .map(|t| pb::SimulationInput {
                mapping_id: flat_raw,
                tick: t,
                value: Some(pb::Value {
                    kind: Some(pb::value::Kind::Semantic(Box::new(pb::SemanticValue {
                        concept_id: tilt,
                        repr: Some(Box::new(pb::Value {
                            kind: Some(pb::value::Kind::Quantity(pb::Quantity {
                                dim: Some(angle),
                                value: (t as f64 * 30.0f64).to_radians(),
                            })),
                        })),
                    }))),
                }),
            })
            .collect(),
        schedule: vec![pb::SchedulePeriod {
            clock_id: main,
            period: 1,
        }],
    })) else {
        panic!()
    };
    assert!(sim.error.is_none(), "{:?}", sim.error);
    let Resp::Simulation(sim) = c.call(Req::StepSimulation(pb::StepSimulationRequest { ticks: 4 }))
    else {
        panic!()
    };
    assert!(sim.error.is_none(), "{:?}", sim.error);
    assert_eq!(sim.samples.len(), 4);
    let last = &sim.samples[3];
    let ba = last
        .values
        .iter()
        .find(|v| v.mapping_id == flat_brightness_a)
        .unwrap();
    assert!(
        ba.rendered.contains("1"),
        "lampA.brightness at 90°: {}",
        ba.rendered
    );

    // deployment sees no devices yet: incomplete, not infeasible
    let Resp::Deployment(d) = c.call(Req::AnalyzeDeployment(pb::AnalyzeDeploymentRequest {
        target_id: "arduino_nano".into(),
        revision: Some(c.last_revision),
    })) else {
        panic!()
    };
    let d = d.deployment.unwrap();
    assert_eq!(d.status(), pb::DeploymentStatus::Feasible);
    assert!(d.design_ready);

    // undo the last binding: lampB's port is open again; redo restores it
    // (a system project answers with the system alongside)
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {})) else {
        panic!()
    };
    assert_eq!(u.project.as_ref().unwrap().revision, c.last_revision);
    assert!(u.system.is_some());
    let a = c.system_analysis();
    assert_eq!(st(&a, lamp_b, port_in), pb::PortStatusKind::Open);
    assert_eq!(a.acceptance(), pb::SystemAcceptance::Open);
    c.call(Req::Redo(pb::RedoRequest {}));
    let a = c.system_analysis();
    assert_eq!(st(&a, lamp_b, port_in), pb::PortStatusKind::Bound);

    // the public contract is on the wire, renderable without the body:
    // "Requires Tilt" / "Provides Brightness" / "Updates in tick"
    let v = c.system();
    let lamp_view = v.components.iter().find(|x| x.id == lamp).unwrap();
    let req = lamp_view.ports.iter().find(|p| p.id == port_in).unwrap();
    let k = req.contract.as_ref().unwrap();
    assert_eq!(req.kind(), pb::PortKind::Required);
    assert_eq!((k.input_names.len(), k.output_name.as_str()), (0, "Tilt"));
    assert_eq!(k.clock_kind(), pb::ClockContractKind::Parameter);
    assert_eq!((k.clock_id, k.clock_name.as_str()), (Some(l_tick), "tick"));
    assert_eq!(
        k.shared,
        vec![pb::IdPair {
            local: l_tilt,
            system: tilt
        }],
        "over the shared system concept"
    );
    let prov = lamp_view.ports.iter().find(|p| p.id == port_out).unwrap();
    assert_eq!(prov.contract.as_ref().unwrap().output_name, "Brightness");
    assert!(
        prov.contract.as_ref().unwrap().shared.is_empty(),
        "a private concept"
    );
    assert!(lamp_view.interface_stamp > 0 && lamp_view.stamp > 0);
    // a private body edit moves the implementation stamp only; the contract is unchanged
    let before = req.contract.clone();
    let applied = c.body(
        lamp,
        pb::edit_op::Op::ReplaceDefinition(pb::ReplaceDefinition {
            id: l_dim,
            definition: Some(pb::Definition {
                kind: Some(pb::definition::Kind::Formula("Tilt / 45 deg".into())),
            }),
        }),
    );
    assert_eq!(applied.kind(), pb::EditKind::Edit);
    let v_after = c.system();
    let lamp_after = v_after.components.iter().find(|x| x.id == lamp).unwrap();
    assert_eq!(lamp_after.interface_stamp, lamp_view.interface_stamp);
    assert_eq!(lamp_after.stamp, lamp_view.stamp + 1);
    assert_eq!(
        lamp_after
            .ports
            .iter()
            .find(|p| p.id == port_in)
            .unwrap()
            .contract,
        before
    );
    c.body(
        lamp,
        pb::edit_op::Op::ReplaceDefinition(pb::ReplaceDefinition {
            id: l_dim,
            definition: Some(pb::Definition {
                kind: Some(pb::definition::Kind::Formula("Tilt / 90 deg".into())),
            }),
        }),
    );
    // an explicit contract change is refused with a stable code when malformed, classified when applied
    let base = c.last_revision;
    let Resp::Error(e) = c.call(Req::ApplySystemEdit(pb::ApplySystemEditRequest {
        base_revision: base,
        op: Some(pb::SystemEditOp {
            op: Some(pb::system_edit_op::Op::ChangePortContract(
                pb::ChangePortContract {
                    component: lamp,
                    port: port_in,
                    contract: None,
                },
            )),
        }),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "protocol.invalid_edit");
    let mut changed = before.clone().unwrap();
    changed.signature = Some(pb::Signature {
        inputs: vec![],
        output: l_bright,
    });
    let applied = c.sys(pb::system_edit_op::Op::ChangePortContract(
        pb::ChangePortContract {
            component: lamp,
            port: port_in,
            contract: Some(changed),
        },
    ));
    let o = applied.outcome.unwrap();
    assert_eq!(o.kind(), pb::EditKind::Edit);
    assert_eq!(
        o.bindings.len(),
        2,
        "both bindings on the port are reopened"
    );
    let a = c.system_analysis();
    assert_eq!(a.acceptance(), pb::SystemAcceptance::Invalid);
    assert!(!a.components.iter().find(|x| x.id == lamp).unwrap().realizes);
    assert!(a
        .composition
        .iter()
        .any(|d| d.code == "component.port_signature_mismatch"));
    c.call(Req::Undo(pb::UndoRequest {}));
    let a = c.system_analysis();
    assert!(a.components.iter().all(|x| x.realizes));
    assert_eq!(a.acceptance(), pb::SystemAcceptance::Executable);
    assert_eq!(lamp_view.clock_params, vec![l_tick]);
    assert_eq!(lamp_view.shared_concepts[0].system, tilt);
    assert_eq!(lamp_view.body.as_ref().unwrap().mappings.len(), 3);
    let v = c.system();
    // save, reopen: the sources and the identity sidecar are the truth
    // (ADR-0023); no JSON design file is written
    let Resp::Project(saved) = c.call(Req::SaveProject(pb::SaveProjectRequest { force: false }))
    else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);
    assert!(root.join("src/main.bdl").is_file());
    assert!(root.join(".bdl/identities.json").is_file());
    assert!(!root.join("design").exists());
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let Resp::Project(re) = c.call(Req::OpenProject(pb::OpenProjectRequest {
        root_path: root.to_string_lossy().into(),
    })) else {
        panic!()
    };
    let re = re.project.unwrap();
    assert_eq!(re.kind(), pb::ProjectKind::Text);
    assert_eq!(re.revision, 0);
    let v2 = c.system();
    // stamps are session edit counters, never persisted semantics
    let unstamped = |cs: &[pb::ComponentView]| -> Vec<pb::ComponentView> {
        cs.iter()
            .cloned()
            .map(|mut c| {
                c.stamp = 0;
                c.interface_stamp = 0;
                c
            })
            .collect()
    };
    assert_eq!(unstamped(&v2.components), unstamped(&v.components));
    assert_eq!(v2.instances, v.instances);
    assert_eq!(v2.bindings, v.bindings);
    assert_eq!(
        v2.origins, v.origins,
        "flat ids are stable across save and reopen"
    );

    // a flat project is untouched by all this: GetSystem is not for it
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let flat_root = dir.path().join("plain");
    let Resp::Project(fp) = c.call(Req::InitProject(pb::InitProjectRequest {
        root_path: flat_root.to_string_lossy().into(),
        name: "plain".into(),
    })) else {
        panic!()
    };
    // a new project is the same one kind (ADR-0023): a behaviour system
    // with sources, whose system view is served and whose flat edits land
    assert_eq!(fp.project.unwrap().kind(), pb::ProjectKind::Text);
    let Resp::System(sv) = c.call(Req::GetSystem(pb::GetSystemRequest {})) else {
        panic!()
    };
    assert!(sv.system.unwrap().is_flat);
    let Resp::EditApplied(_) = c.call(Req::ApplyEdit(pb::ApplyEditRequest {
        base_revision: 0,
        op: Some(pb::EditOp {
            op: Some(concept("Tilt", angle)),
        }),
    })) else {
        panic!()
    };

    let Resp::Ack(_) = c.call(Req::Shutdown(pb::ShutdownRequest {})) else {
        panic!()
    };
    assert!(c.child.wait().unwrap().success());
}

/// Phase 8b over the wire: group edits move the authoring generation and
/// nothing else; the boundary is served with the analysis; a preview
/// changes nothing; extraction packages the group, reconnects it through
/// base-end bindings, and the component's body is editable in its own
/// scope (component-scoped drafts).
#[test]
fn grouping_and_extraction_over_the_wire() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("lamp");
    let mut c = Client::spawn();
    c.call(Req::Handshake(pb::HandshakeRequest {
        client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
        client_name: "system-e2e".into(),
        client_version: "0".into(),
    }));
    c.call(Req::InitSystemProject(pb::InitSystemProjectRequest {
        root_path: root.to_string_lossy().into(),
        name: "lamp".into(),
    }));
    c.call(Req::SubscribeProject(pb::SubscribeProjectRequest {}));
    let angle = pb::Dim {
        angle: 1,
        ..Default::default()
    };
    let tilt = c.base(concept("Tilt", angle)).created_concept.unwrap();
    let level = c
        .base(concept("Brightness", pb::Dim::default()))
        .created_concept
        .unwrap();
    let main = c
        .base(pb::edit_op::Op::CreateClockDomain(pb::CreateClockDomain {
            name: "main".into(),
        }))
        .created_clock
        .unwrap();
    let clock_of = |c: &mut Client, id: u64| {
        c.base(pb::edit_op::Op::SetMappingClock(pb::SetMappingClock {
            id,
            clock_id: Some(main),
        }));
    };
    let raw = c
        .base(mapping("raw", vec![], tilt))
        .created_mapping
        .unwrap();
    clock_of(&mut c, raw);
    let tilt_value = c
        .base(mapping("tiltValue", vec![], tilt))
        .created_mapping
        .unwrap();
    c.base(formula(tilt_value, "raw"));
    clock_of(&mut c, tilt_value);
    let dim = c
        .base(mapping("dimByTilt", vec![tilt], level))
        .created_mapping
        .unwrap();
    c.base(formula(dim, "Tilt / 90 deg"));
    let brightness = c
        .base(mapping("brightness", vec![], level))
        .created_mapping
        .unwrap();
    c.base(formula(brightness, "dimByTilt(tiltValue)"));
    clock_of(&mut c, brightness);
    let indicator = c
        .base(mapping("indicator", vec![], level))
        .created_mapping
        .unwrap();
    c.base(formula(indicator, "brightness"));
    clock_of(&mut c, indicator);
    let revision = c.last_revision;
    c.events.clear();

    // --- group edits: no revision, no events, a new authoring generation ---
    let group_edit = |c: &mut Client, op: pb::group_edit_op::Op| -> pb::SystemView {
        match c.call(Req::ApplyGroupEdit(pb::ApplyGroupEditRequest {
            op: Some(pb::GroupEditOp { op: Some(op) }),
            base_generation: None,
        })) {
            Resp::System(s) => s.system.unwrap(),
            other => panic!("{other:?}"),
        }
    };
    let v = group_edit(
        &mut c,
        pb::group_edit_op::Op::CreateGroup(pb::CreateGroup {
            name: "Lamp".into(),
            description: "dims with tilt".into(),
            members: vec![dim],
            component: None,
        }),
    );
    assert_eq!(v.authoring_generation, 1);
    assert_eq!(v.revision, revision);
    let group = v.groups[0].id;
    let v = group_edit(
        &mut c,
        pb::group_edit_op::Op::AddMember(pb::AddGroupMember {
            group,
            decl: brightness,
        }),
    );
    assert_eq!(v.authoring_generation, 2);
    assert_eq!(v.groups[0].members, vec![dim, brightness]);
    assert_eq!(v.groups[0].description, "dims with tilt");
    // group edits push nothing: every event seen is about the last commit
    for e in &c.events {
        if let Some(pb::event::Payload::ProjectChanged(pc)) = &e.payload {
            assert_eq!(pc.project.as_ref().unwrap().revision, revision);
        }
        if let Some(pb::event::Payload::AnalysisReady(ar)) = &e.payload {
            assert_eq!(ar.analysis.as_ref().unwrap().revision, revision);
        }
    }
    let Resp::Project(p) = c.call(Req::GetProject(pb::GetProjectRequest {})) else {
        panic!()
    };
    assert_eq!(p.project.unwrap().revision, revision);
    // a refusal is a group_edit.* code
    let Resp::Error(e) = c.call(Req::ApplyGroupEdit(pb::ApplyGroupEditRequest {
        op: Some(pb::GroupEditOp {
            op: Some(pb::group_edit_op::Op::AddMember(pb::AddGroupMember {
                group: 99,
                decl: dim,
            })),
        }),
        base_generation: None,
    })) else {
        panic!()
    };
    assert_eq!(e.code, "group_edit.unknown_group");

    // --- authoring history: undo/redo of group edits move no revision ---
    let saved_before = c.call(Req::SaveProject(pb::SaveProjectRequest { force: false }));
    let Resp::Project(sp) = saved_before else {
        panic!()
    };
    assert!(!sp.project.unwrap().dirty);
    assert!(!c.system().dirty);
    group_edit(
        &mut c,
        pb::group_edit_op::Op::RenameGroup(pb::RenameGroup {
            id: group,
            name: "Adaptive lamp".into(),
        }),
    );
    // a group edit dirties the project although the revision is unchanged
    let v = c.system();
    assert!(v.dirty, "a group edit needs saving");
    assert_eq!(v.revision, revision);
    let Resp::Project(p) = c.call(Req::GetProject(pb::GetProjectRequest {})) else {
        panic!()
    };
    assert!(p.project.unwrap().dirty);
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {})) else {
        panic!()
    };
    let v = u.system.unwrap();
    assert_eq!(v.groups[0].name, "Lamp");
    assert_eq!(v.revision, revision, "an authoring undo is not a revision");
    assert_eq!(u.project.as_ref().unwrap().revision, revision);
    assert!(v.authoring_generation > 3);
    let Resp::SystemEditApplied(r) = c.call(Req::Redo(pb::RedoRequest {})) else {
        panic!()
    };
    assert_eq!(r.system.unwrap().groups[0].name, "Adaptive lamp");
    assert_eq!(c.last_revision, revision);
    // undo twice more: the member add, then the group's creation
    c.call(Req::Undo(pb::UndoRequest {}));
    c.call(Req::Undo(pb::UndoRequest {}));
    let Resp::SystemEditApplied(u) = c.call(Req::Undo(pb::UndoRequest {})) else {
        panic!()
    };
    assert!(u.system.unwrap().groups.is_empty());
    assert_eq!(c.last_revision, revision);
    c.call(Req::Redo(pb::RedoRequest {}));
    c.call(Req::Redo(pb::RedoRequest {}));
    c.call(Req::Redo(pb::RedoRequest {}));
    let v = c.system();
    assert_eq!(v.groups[0].name, "Adaptive lamp");
    assert_eq!(v.groups[0].members, vec![dim, brightness]);
    group_edit(
        &mut c,
        pb::group_edit_op::Op::RenameGroup(pb::RenameGroup {
            id: group,
            name: "Lamp".into(),
        }),
    );
    let generation = c.system().authoring_generation;

    // a stale authoring generation is refused; the current one is accepted
    let Resp::Error(e) = c.call(Req::ApplyGroupEdit(pb::ApplyGroupEditRequest {
        op: Some(pb::GroupEditOp {
            op: Some(pb::group_edit_op::Op::SetGroupDescription(
                pb::SetGroupDescription {
                    id: group,
                    description: "stale".into(),
                },
            )),
        }),
        base_generation: Some(generation - 1),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "group_edit.stale_generation");
    assert_eq!(c.system().groups[0].description, "dims with tilt");
    let Resp::System(_) = c.call(Req::ApplyGroupEdit(pb::ApplyGroupEditRequest {
        op: Some(pb::GroupEditOp {
            op: Some(pb::group_edit_op::Op::SetGroupDescription(
                pb::SetGroupDescription {
                    id: group,
                    description: "dims with tilt".into(),
                },
            )),
        }),
        base_generation: Some(generation),
    })) else {
        panic!()
    };

    // --- the boundary comes with the analysis ---
    let a = c.system_analysis();
    let b = a.groups.iter().find(|g| g.id == group).unwrap();
    assert_eq!(b.crossing_in, vec![tilt_value]);
    assert_eq!(b.crossing_out, vec![brightness]);
    assert_eq!(b.external_inputs, vec![tilt_value]);
    assert_eq!(b.private_candidates, vec![dim]);
    assert_eq!(b.clocks, vec![main]);

    // --- preview: nothing changes ---
    let Resp::ExtractionPreview(p) = c.call(Req::PreviewComponentExtraction(
        pb::PreviewComponentExtractionRequest {
            group,
            choices: Some(pb::ExtractionChoices {
                name: "AdaptiveLamp".into(),
                ..Default::default()
            }),
        },
    )) else {
        panic!()
    };
    let p = p.preview.unwrap();
    assert_eq!(p.name, "AdaptiveLamp");
    assert_eq!(p.instance_name, "adaptiveLamp");
    assert_eq!(p.required.len(), 1);
    assert_eq!(p.required[0].name, "tiltValue");
    assert_eq!(p.provided[0].name, "brightness");
    assert_eq!(c.last_revision, revision);
    assert_eq!(c.system().groups.len(), 1);

    // --- extraction: one system edit, one revision ---
    let applied = c.sys(pb::system_edit_op::Op::ExtractGroupAsComponent(
        pb::ExtractGroupAsComponent {
            group,
            choices: Some(pb::ExtractionChoices {
                name: "AdaptiveLamp".into(),
                ..Default::default()
            }),
        },
    ));
    let o = applied.outcome.unwrap();
    let comp = o.created_component.unwrap();
    let inst = o.created_instance.unwrap();
    let v = applied.system.unwrap();
    assert!(v.groups.is_empty());
    assert_eq!(v.components.len(), 1);
    assert_eq!(v.instances[0].name, "adaptiveLamp");
    assert_eq!(v.bindings.len(), 2);
    // base ends on the wire
    let into_port = v
        .bindings
        .iter()
        .find(|b| b.destination.as_ref().unwrap().base_decl.is_none())
        .unwrap();
    assert_eq!(
        into_port.source.as_ref().unwrap().base_decl,
        Some(tilt_value)
    );
    assert_eq!(into_port.destination.as_ref().unwrap().instance, inst);
    let into_base = v
        .bindings
        .iter()
        .find(|b| b.destination.as_ref().unwrap().base_decl.is_some())
        .unwrap();
    assert_eq!(
        into_base.destination.as_ref().unwrap().base_decl,
        Some(brightness)
    );
    // the flat design: the base copy is a reference; still causal
    let a = c.system_analysis();
    assert_eq!(a.acceptance(), pb::SystemAcceptance::Executable);
    let flat = applied.project.unwrap();
    let copy = flat.mappings.iter().find(|m| m.id == brightness).unwrap();
    assert!(copy
        .definition
        .as_ref()
        .unwrap()
        .kind
        .as_ref()
        .is_some_and(|k| matches!(k, pb::definition::Kind::Reference(_))));
    // the component analysis of the body comes with the system analysis
    let ca = a
        .component_analyses
        .iter()
        .find(|x| x.id == comp)
        .unwrap()
        .analysis
        .as_ref()
        .unwrap();
    assert_eq!(ca.mappings.len(), 3);

    // --- a component-scoped draft is judged in the body's own scope ---
    let rev = c.last_revision;
    let Resp::DefinitionDraft(d) = c.call(Req::AnalyzeDefinitionDraft(
        pb::AnalyzeDefinitionDraftRequest {
            revision: rev,
            mapping_id: brightness,
            generation: 1,
            source: "dimByTilt(tiltValue) * 2".into(),
            component: Some(comp),
        },
    )) else {
        panic!()
    };
    assert!(d.parse_ok);
    assert_eq!(
        d.analysis.as_ref().unwrap().status(),
        pb::MappingStatus::ClockConsistent,
        "{:?}",
        d.analysis
    );
    let Resp::DraftCompletion(cmp) = c.call(Req::CompleteDefinitionDraft(
        pb::CompleteDefinitionDraftRequest {
            revision: rev,
            mapping_id: brightness,
            source: "dim".into(),
            offset: 3,
            component: Some(comp),
        },
    )) else {
        panic!()
    };
    assert!(
        cmp.items.iter().any(|i| i.label == "dimByTilt(Tilt)"),
        "{:?}",
        cmp.items
    );
    assert!(
        !cmp.items.iter().any(|i| i.label.contains('.')),
        "body scope, not flat names"
    );
    let Resp::Ack(_) = c.call(Req::DiscardDefinitionDraft(
        pb::DiscardDefinitionDraftRequest {
            mapping_id: brightness,
            component: Some(comp),
        },
    )) else {
        panic!()
    };
    // the same mapping id in the flat scope is the base copy (unrelated)
    let Resp::Error(e) = c.call(Req::AnalyzeDefinitionDraft(
        pb::AnalyzeDefinitionDraftRequest {
            revision: rev,
            mapping_id: brightness,
            generation: 2,
            source: "1".into(),
            component: Some(99),
        },
    )) else {
        panic!()
    };
    assert_eq!(e.code, "system.unknown_component");

    // --- layout carries instances, groups and component canvases ---
    c.call(Req::SetLayout(pb::SetLayoutRequest {
        layout: Some(pb::Layout {
            instances: vec![pb::NodePosition {
                id: inst,
                x: 10.0,
                y: 20.0,
            }],
            groups: vec![pb::GroupBox {
                id: 7,
                x: 1.0,
                y: 2.0,
                width: 300.0,
                height: 200.0,
                collapsed: true,
            }],
            components: vec![pb::ComponentLayout {
                id: comp,
                layout: Some(pb::Layout {
                    mappings: vec![pb::NodePosition {
                        id: dim,
                        x: 5.0,
                        y: 6.0,
                    }],
                    ..Default::default()
                }),
            }],
            ..Default::default()
        }),
    }));
    c.call(Req::SaveProject(pb::SaveProjectRequest { force: false }));
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let Resp::Project(re) = c.call(Req::OpenProject(pb::OpenProjectRequest {
        root_path: root.to_string_lossy().into(),
    })) else {
        panic!()
    };
    let layout = re.project.unwrap().layout.unwrap();
    // what was placed by hand is where it was left; the rest was placed
    // by the layout service on open (ADR-0023 §7)
    assert!(layout.instances.iter().any(|p| p.id == inst));
    assert!(layout.groups[0].collapsed);
    let body = layout.components[0].layout.as_ref().unwrap();
    assert_eq!(body.mappings.iter().find(|p| p.id == dim).unwrap().x, 5.0);
    assert!(
        !body.concepts.is_empty(),
        "body concepts were placed on open"
    );
    let v2 = c.system();
    assert_eq!(v2.bindings.len(), 2);
    assert_eq!(v2.authoring_generation, 0);

    let Resp::Ack(_) = c.call(Req::Shutdown(pb::ShutdownRequest {})) else {
        panic!()
    };
    assert!(c.child.wait().unwrap().success());
}

/// The role across the component boundary (docs/spec/... relationship
/// roles; ADR-0032): one answer per design, stated by the daemon.  Inside
/// the body a declaration that backs a required or parameter port is a
/// Source of the body (provided through the port); the flattening makes an
/// instance's bound copy a Value and leaves an unbound required port a
/// Source of the system (FV Theorem H); a base relationship a binding
/// realises is a Value at the top level; a rule is a Rule everywhere; a
/// flattened rule inside an instance offers no apply action here.
#[test]
fn the_role_is_one_answer_across_the_component_boundary() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("probe");
    let mut c = Client::spawn();
    c.call(Req::Handshake(pb::HandshakeRequest {
        client_protocol_version: Some(bdl_protocol::PROTOCOL_VERSION),
        client_name: "e2e".into(),
        client_version: "0".into(),
    }));
    let Resp::Project(p) = c.call(Req::InitProject(pb::InitProjectRequest {
        root_path: root.to_string_lossy().into(),
        name: "probe".into(),
    })) else {
        panic!()
    };
    c.last_revision = p.project.unwrap().revision;
    let angle = pb::Dim {
        angle: 1,
        ..Default::default()
    };
    let tilt = c.base(concept("Tilt", angle)).created_concept.unwrap();
    let bright = c
        .base(concept("Brightness", pb::Dim::default()))
        .created_concept
        .unwrap();
    // base: a Source and an open value a provided port will realise
    let tilt_sensor = c
        .base(mapping("tiltSensor", vec![], tilt))
        .created_mapping
        .unwrap();
    let level_out = c
        .base(mapping("level", vec![], bright))
        .created_mapping
        .unwrap();
    let role_of =
        |p: &pb::ProjectProjection, id: u64| p.mappings.iter().find(|m| m.id == id).unwrap().role();

    // Probe: tiltValue (required), level (provided, defined), gain
    // (parameter), an internal Source, a rule nothing applies
    let probe = c
        .sys(pb::system_edit_op::Op::CreateComponent(
            pb::CreateComponent {
                name: "Probe".into(),
                description: String::new(),
            },
        ))
        .outcome
        .unwrap()
        .created_component
        .unwrap();
    let p_tilt = c
        .body(probe, concept("Tilt", angle))
        .created_concept
        .unwrap();
    c.sys(pb::system_edit_op::Op::ShareConcept(pb::ShareConcept {
        component: probe,
        local: p_tilt,
        system: Some(tilt),
    }));
    let p_bright = c
        .body(probe, concept("Brightness", pb::Dim::default()))
        .created_concept
        .unwrap();
    let p_in = c
        .body(probe, mapping("tiltValue", vec![], p_tilt))
        .created_mapping
        .unwrap();
    let p_level = c
        .body(probe, mapping("level", vec![], p_bright))
        .created_mapping
        .unwrap();
    c.body(probe, formula(p_level, "1"));
    let p_gain = c
        .body(probe, mapping("gain", vec![], p_bright))
        .created_mapping
        .unwrap();
    let p_internal = c
        .body(probe, mapping("internalReading", vec![], p_tilt))
        .created_mapping
        .unwrap();
    let p_rule = c
        .body(probe, mapping("dim", vec![p_tilt], p_bright))
        .created_mapping
        .unwrap();
    c.body(probe, formula(p_rule, "Tilt / 90 deg"));
    let declare = |c: &mut Client, decl: u64, name: &str, kind: pb::PortKind| -> u64 {
        let mut dp = pb::DeclarePort {
            component: probe,
            decl,
            name: name.into(),
            description: String::new(),
            ..Default::default()
        };
        dp.set_kind(kind);
        c.sys(pb::system_edit_op::Op::DeclarePort(dp))
            .outcome
            .unwrap()
            .created_port
            .unwrap()
    };
    let port_in = declare(&mut c, p_in, "tiltValue", pb::PortKind::Required);
    let port_level = declare(&mut c, p_level, "level", pb::PortKind::Provided);
    let port_gain = declare(&mut c, p_gain, "gain", pb::PortKind::Parameter);

    // inside the body: every role from the body's own state
    let sv = c.system();
    let body = sv.components[0].body.as_ref().unwrap();
    assert_eq!(
        role_of(body, p_in),
        pb::RelationshipRole::Source,
        "required port: a Source of the body"
    );
    assert_eq!(
        role_of(body, p_gain),
        pb::RelationshipRole::Source,
        "parameter: a Source of the body"
    );
    assert_eq!(role_of(body, p_internal), pb::RelationshipRole::Source);
    assert_eq!(
        role_of(body, p_level),
        pb::RelationshipRole::Value,
        "provided, realised inside"
    );
    assert_eq!(role_of(body, p_rule), pb::RelationshipRole::Rule);
    let base = sv.base.as_ref().unwrap();
    assert_eq!(role_of(base, tilt_sensor), pb::RelationshipRole::Source);
    assert_eq!(
        role_of(base, level_out),
        pb::RelationshipRole::Source,
        "open, nothing binds it yet"
    );

    // two instances: one fully bound, one with its required port unbound
    let inst = |c: &mut Client, name: &str| -> u64 {
        let id = c
            .sys(pb::system_edit_op::Op::CreateInstance(pb::CreateInstance {
                component: probe,
                name: name.into(),
            }))
            .outcome
            .unwrap()
            .created_instance
            .unwrap();
        c.sys(pb::system_edit_op::Op::SetParameterArgument(
            pb::SetParameterArgument {
                instance: id,
                port: port_gain,
                value: Some("2".into()),
            },
        ));
        id
    };
    let bound = inst(&mut c, "bound");
    let open = inst(&mut c, "open");
    c.sys(pb::system_edit_op::Op::BindPorts(pb::BindPorts {
        source: Some(pb::PortRefView {
            instance: 0,
            port: 0,
            base_decl: Some(tilt_sensor),
        }),
        destination: port_ref(bound, port_in),
        transport_init: None,
    }));
    let applied = c.sys(pb::system_edit_op::Op::BindPorts(pb::BindPorts {
        source: port_ref(bound, port_level),
        destination: Some(pb::PortRefView {
            instance: 0,
            port: 0,
            base_decl: Some(level_out),
        }),
        transport_init: None,
    }));

    // the top level: the bound base relationship is a Value now
    let sv = applied.system.unwrap();
    let base = sv.base.as_ref().unwrap();
    assert_eq!(
        role_of(base, level_out),
        pb::RelationshipRole::Value,
        "a binding realises it"
    );
    assert_eq!(role_of(base, tilt_sensor), pb::RelationshipRole::Source);
    // the flat design: the instance's bound copy a Value, the unbound one
    // a Source of the system, the internal Source a Source, the rule a Rule
    let flat = applied.project.unwrap();
    let by_name = |n: &str| flat.mappings.iter().find(|m| m.name == n).unwrap().clone();
    assert_eq!(
        by_name("bound.tiltValue").role(),
        pb::RelationshipRole::Value
    );
    assert!(matches!(
        by_name("bound.tiltValue").definition.as_ref().unwrap().kind,
        Some(pb::definition::Kind::Reference(_))
    ));
    assert_eq!(
        by_name("open.tiltValue").role(),
        pb::RelationshipRole::Source
    );
    assert_eq!(
        by_name("bound.gain").role(),
        pb::RelationshipRole::Value,
        "a parameter argument realises it"
    );
    assert_eq!(
        by_name("bound.internalReading").role(),
        pb::RelationshipRole::Source
    );
    assert_eq!(by_name("bound.level").role(), pb::RelationshipRole::Value);
    assert_eq!(by_name("bound.dim").role(), pb::RelationshipRole::Rule);
    assert_eq!(by_name("level").role(), pb::RelationshipRole::Value);
    assert_eq!(by_name("tiltSensor").role(), pb::RelationshipRole::Source);
    // the analysis states the same roles, and who applies what
    let Resp::Analysis(a) = c.call(Req::RunAnalysis(pb::RunAnalysisRequest {})) else {
        panic!()
    };
    let a = a.analysis.unwrap();
    for m in &flat.mappings {
        let ma = a.mappings.iter().find(|x| x.id == m.id).unwrap();
        assert_eq!(ma.role(), m.role(), "{}", m.name);
    }
    let rule = by_name("bound.dim");
    let ma = a.mappings.iter().find(|x| x.id == rule.id).unwrap();
    assert!(ma.applied_by.is_empty());
    assert!(ma
        .diagnostics
        .iter()
        .any(|d| d.code == "reactive.rule_unapplied"));
    // the simulation's inputs are exactly the flat Sources
    let sources: Vec<&str> = flat
        .mappings
        .iter()
        .filter(|m| m.role() == pb::RelationshipRole::Source)
        .map(|m| m.name.as_str())
        .collect();
    assert_eq!(
        sources,
        vec![
            "tiltSensor",
            "bound.internalReading",
            "open.tiltValue",
            "open.internalReading"
        ]
    );
    // hover on the flattened bound copy: a Value, bound, backing the port
    let Resp::DraftHover(h) = c.call(Req::HoverEntity(pb::HoverEntityRequest {
        revision: c.last_revision,
        entity: Some(pb::EntityRef {
            kind: Some(pb::entity_ref::Kind::MappingId(
                by_name("bound.tiltValue").id,
            )),
        }),
    })) else {
        panic!()
    };
    let detail = |k: &str| {
        h.details
            .iter()
            .find(|d| d.label == k)
            .map(|d| d.value.clone())
    };
    assert_eq!(detail("role").as_deref(), Some("Value"));
    assert!(detail("definition").unwrap().starts_with("bound to"));
    // the apply action for a rule inside an instance is blocked here: the
    // value belongs in the component's source (DI-40)
    let Resp::SemanticActions(acts) =
        c.call(Req::ListSemanticActions(pb::ListSemanticActionsRequest {
            revision: c.last_revision,
            entity: Some(pb::EntityRef {
                kind: Some(pb::entity_ref::Kind::MappingId(rule.id)),
            }),
        }))
    else {
        panic!()
    };
    let apply = acts
        .actions
        .iter()
        .find(|x| x.id.starts_with("rule.apply:"))
        .expect("the apply action is listed");
    assert_eq!(apply.applicability(), pb::ActionApplicability::Blocked);
    assert!(
        apply.reason.contains("inside the instance `bound`"),
        "{}",
        apply.reason
    );
    assert!(apply.edits.is_empty());
    let _ = (open, port_level);
    c.call(Req::Shutdown(pb::ShutdownRequest {}));
}
