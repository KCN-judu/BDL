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
    Some(pb::PortRefView { instance, port })
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
    assert_eq!(p.kind(), pb::ProjectKind::System);
    assert_eq!(p.revision, 0);

    // flat edits are refused on a system project: the flat design is derived
    let Resp::Error(e) = c.call(Req::ApplyEdit(pb::ApplyEditRequest {
        base_revision: 0,
        op: Some(pb::EditOp {
            op: Some(concept(
                "Tilt",
                pb::Dim {
                    angle: 1,
                    ..Default::default()
                },
            )),
        }),
    })) else {
        panic!()
    };
    assert_eq!(e.code, "edit.derived_design");

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
    // the derived flat design came back with the edit, and it is marked derived
    let flat = applied.project.unwrap();
    assert_eq!(flat.kind(), pb::ProjectKind::System);
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
    let Resp::EditApplied(u) = c.call(Req::Undo(pb::UndoRequest {})) else {
        panic!()
    };
    assert_eq!(u.project.as_ref().unwrap().revision, c.last_revision);
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
    // save, reopen: the system is the truth; the flat file is not written
    let Resp::Project(saved) = c.call(Req::SaveProject(pb::SaveProjectRequest {})) else {
        panic!()
    };
    assert!(!saved.project.unwrap().dirty);
    assert!(root.join("design/system.bdl.json").is_file());
    assert!(!root.join("design/project.bdl.json").exists());
    c.call(Req::CloseProject(pb::CloseProjectRequest {}));
    let Resp::Project(re) = c.call(Req::OpenProject(pb::OpenProjectRequest {
        root_path: root.to_string_lossy().into(),
    })) else {
        panic!()
    };
    let re = re.project.unwrap();
    assert_eq!(re.kind(), pb::ProjectKind::System);
    assert_eq!(re.revision, 0);
    let v2 = c.system();
    assert_eq!(v2.components, v.components);
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
    assert_eq!(fp.project.unwrap().kind(), pb::ProjectKind::Flat);
    let Resp::Error(e) = c.call(Req::GetSystem(pb::GetSystemRequest {})) else {
        panic!()
    };
    assert_eq!(e.code, "system.not_a_system");
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
