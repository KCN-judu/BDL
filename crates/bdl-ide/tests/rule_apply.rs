//! A rule nothing applies (`reactive.rule_unapplied`) and the action that
//! puts it to work: `Add a value that applies <rule>`.  Ready when every
//! concept the rule reads has exactly one value producing it; a choice —
//! one option per combination — when a concept has several; blocked, with
//! the reason, when one has none or the arguments live in two domains.
//! The design stays legal throughout: the note is never an error.

mod support;

use bdl_ide::*;
use bdl_model::edit::{apply_edit, EditOp};
use bdl_model::surface::{Definition, Design, ProjectSnapshot, Representation, Signature};
use bdl_model::{ClockId, DeclId, Dim, SemanticId};
use support::*;

/// `TempSensor : () -> RoomTemp`, `ButtonInput : () -> ButtonHeld`,
/// `AirConditionerCtrl : RoomTemp -> ButtonHeld -> SwitchState` defined
/// and applied by nothing.
struct Ac {
    snapshot: ProjectSnapshot,
    room_temp: SemanticId,
    button_held: SemanticId,
    switch_state: SemanticId,
    temp_sensor: DeclId,
    button_input: DeclId,
    ctrl: DeclId,
}

fn created_mapping(s: &ProjectSnapshot, op: EditOp) -> (ProjectSnapshot, DeclId) {
    let a = apply_edit(s, &op).expect("fixture edit applies");
    (a.snapshot, a.outcome.created_mapping.expect("a mapping"))
}

fn created_concept(s: &ProjectSnapshot, op: EditOp) -> (ProjectSnapshot, SemanticId) {
    let a = apply_edit(s, &op).expect("fixture edit applies");
    (a.snapshot, a.outcome.created_concept.expect("a concept"))
}

fn air_conditioner() -> Ac {
    let s = ProjectSnapshot::new(Design::empty("ac"));
    let (s, room_temp) = created_concept(
        &s,
        concept(
            "RoomTemp",
            Some(Representation::Quantity {
                dim: Dim::TEMPERATURE,
            }),
        ),
    );
    let (s, button_held) =
        created_concept(&s, concept("ButtonHeld", Some(Representation::Boolean)));
    let (s, switch_state) =
        created_concept(&s, concept("SwitchState", Some(Representation::Boolean)));
    let (s, temp_sensor) = created_mapping(&s, mapping("TempSensor", vec![], room_temp));
    let (s, button_input) = created_mapping(&s, mapping("ButtonInput", vec![], button_held));
    let (s, ctrl) = created_mapping(
        &s,
        mapping(
            "AirConditionerCtrl",
            vec![room_temp, button_held],
            switch_state,
        ),
    );
    let s = edit(&s, formula(ctrl, "RoomTemp > 299.15 K && ButtonHeld"));
    Ac {
        snapshot: s,
        room_temp,
        button_held,
        switch_state,
        temp_sensor,
        button_input,
        ctrl,
    }
}

fn unapplied_note(snap: &AnalysisSnapshot, rule: DeclId) -> Option<SemanticDiagnostic> {
    diagnostics(snap, DiagnosticScope::Entity(EntityRef::Mapping(rule)))
        .items
        .into_iter()
        .find(|d| d.code == "reactive.rule_unapplied")
}

fn the_action(snap: &AnalysisSnapshot, rule: DeclId) -> SemanticAction {
    let d = unapplied_note(snap, rule).expect("the note");
    let acts = actions_for(snap, &d);
    assert_eq!(acts.len(), 1, "one fix: {:?}", acts);
    let a = acts.into_iter().next().expect("one");
    assert_eq!(a.id.0, format!("rule.apply:{rule}"));
    assert_eq!(a.kind, ActionKind::QuickFix);
    assert_eq!(a.addresses, vec!["reactive.rule_unapplied".to_string()]);
    assert_eq!(a.title, "Add a value that applies `AirConditionerCtrl`");
    a
}

#[test]
fn the_note_is_open_and_names_the_call_when_every_read_has_one_value() {
    let ac = air_conditioner();
    let mut host = IdeHost::new(ac.snapshot.clone());
    let snap = host.snapshot();
    let d = unapplied_note(&snap, ac.ctrl).expect("the note");
    assert_eq!(d.severity, SemanticSeverity::Open);
    assert_eq!(d.primary.entity, EntityRef::Mapping(ac.ctrl));
    assert_eq!(
        d.message,
        "AirConditionerCtrl is a rule nothing applies yet."
    );
    assert!(
        d.explanation
            .contains("`AirConditionerCtrl(TempSensor, ButtonInput)`"),
        "{}",
        d.explanation
    );
    for word in ["declRef", "DeclId", "β", "realization"] {
        assert!(!d.message.contains(word) && !d.explanation.contains(word));
    }
    assert!(d
        .actions
        .contains(&SemanticActionId(format!("rule.apply:{}", ac.ctrl))));
    // the whole set has no error: the design is legal
    assert!(diagnostics(&snap, DiagnosticScope::Project)
        .errors()
        .next()
        .is_none());
    // the values are never noted
    assert!(unapplied_note(&snap, ac.temp_sensor).is_none());
}

#[test]
fn ready_creates_the_defined_value_with_a_fresh_lower_camel_name() {
    let ac = air_conditioner();
    let mut host = IdeHost::new(ac.snapshot.clone());
    let snap = host.snapshot();
    let a = the_action(&snap, ac.ctrl);
    assert!(a.is_ready(), "{:?}", a.applicability);
    let plan = a.plan.as_ref().expect("plan");
    let edits: Vec<&EditOp> = plan.model_edits().collect();
    assert_eq!(edits.len(), 1);
    let EditOp::CreateMapping {
        name,
        signature,
        definition,
        clock,
        ..
    } = edits[0]
    else {
        panic!("{:?}", edits[0])
    };
    assert_eq!(name, "airConditionerCtrl");
    assert_eq!(
        *signature,
        Signature {
            inputs: vec![],
            output: ac.switch_state
        }
    );
    assert_eq!(
        *definition,
        Some(Definition::Formula {
            source: "AirConditionerCtrl(TempSensor, ButtonInput)".into()
        })
    );
    assert_eq!(*clock, None, "a pure rule over agnostic values: no domain");
    assert!(a.explanation.contains("has no value of its own"));

    // Applying it: the note is gone, the value checks and has a value per
    // tick (a unit-domain, clock-consistent declaration the simulator shows).
    let applied = apply_edit(&ac.snapshot, edits[0]).expect("applies");
    let value = applied.outcome.created_mapping.expect("created");
    let mut host = IdeHost::new(applied.snapshot);
    let snap = host.snapshot();
    assert!(unapplied_note(&snap, ac.ctrl).is_none());
    let status = snap.analysis().mappings[&value].status;
    assert_eq!(status, bdl_compiler::MappingStatus::ClockConsistent);
}

#[test]
fn a_taken_name_gets_a_counter_and_a_lower_camel_rule_gets_value() {
    let ac = air_conditioner();
    // the name the fix would pick is taken by a concept
    let (s, _) = created_concept(&ac.snapshot, concept("airConditionerCtrl", None));
    let mut host = IdeHost::new(s);
    let snap = host.snapshot();
    let a = the_action(&snap, ac.ctrl);
    let EditOp::CreateMapping { name, .. } = a
        .plan
        .as_ref()
        .expect("plan")
        .model_edits()
        .next()
        .expect("edit")
    else {
        panic!()
    };
    assert_eq!(name, "airConditionerCtrl2");

    // a rule already spelled lowerCamel
    let lamp = lamp();
    let (s, _) = created_mapping(&lamp.snapshot, mapping("tilt", vec![], lamp.tilt));
    let s = edit(&s, formula(lamp.dim_by_tilt, "Tilt / 90 deg"));
    let mut host = IdeHost::new(s);
    let snap = host.snapshot();
    let d = unapplied_note(&snap, lamp.dim_by_tilt).expect("note");
    let a = actions_for(&snap, &d).into_iter().next().expect("fix");
    let EditOp::CreateMapping {
        name, definition, ..
    } = a
        .plan
        .as_ref()
        .expect("plan")
        .model_edits()
        .next()
        .expect("edit")
    else {
        panic!()
    };
    assert_eq!(name, "dimByTiltValue");
    assert_eq!(
        *definition,
        Some(Definition::Formula {
            source: "dimByTilt(tilt)".into()
        })
    );
}

#[test]
fn two_values_for_one_read_concept_is_a_choice_never_a_guess() {
    let ac = air_conditioner();
    // a second value producing RoomTemp
    let (s, _) = created_mapping(&ac.snapshot, mapping("TempSetpoint", vec![], ac.room_temp));
    let mut host = IdeHost::new(s);
    let snap = host.snapshot();
    let a = the_action(&snap, ac.ctrl);
    assert!(a.plan.is_none());
    let Applicability::NeedsChoice { options } = &a.applicability else {
        panic!("{:?}", a.applicability)
    };
    let labels: Vec<&str> = options.iter().map(|o| o.label.as_str()).collect();
    assert_eq!(
        labels,
        vec![
            "AirConditionerCtrl(TempSensor, ButtonInput)",
            "AirConditionerCtrl(TempSetpoint, ButtonInput)",
        ],
        "one option per combination, in declaration order"
    );
    for o in options {
        let EditOp::CreateMapping {
            name, definition, ..
        } = &o.edit
        else {
            panic!()
        };
        assert_eq!(name, "airConditionerCtrl");
        assert_eq!(
            *definition,
            Some(Definition::Formula {
                source: o.label.clone()
            })
        );
    }
    assert!(
        a.explanation.contains("choose which ones"),
        "{}",
        a.explanation
    );
    // the note itself no longer spells a call: the choice is the designer's
    let d = unapplied_note(&snap, ac.ctrl).expect("note");
    assert!(
        d.explanation.contains("(RoomTemp, ButtonHeld)"),
        "{}",
        d.explanation
    );
}

#[test]
fn no_value_for_a_read_concept_blocks_with_the_concept_named() {
    let lamp = lamp();
    let s = edit(&lamp.snapshot, formula(lamp.dim_by_tilt, "Tilt / 90 deg"));
    let mut host = IdeHost::new(s);
    let snap = host.snapshot();
    let d = unapplied_note(&snap, lamp.dim_by_tilt).expect("note");
    let a = actions_for(&snap, &d).into_iter().next().expect("fix");
    assert_eq!(a.title, "Add a value that applies `dimByTilt`");
    let Applicability::Blocked { reason } = &a.applicability else {
        panic!("{:?}", a.applicability)
    };
    assert_eq!(
        reason,
        "no value produces `Tilt` yet; add a Source or a computed value that produces it first"
    );
    assert!(a.plan.is_none());
}

#[test]
fn arguments_in_two_domains_block_and_a_shared_domain_is_the_values() {
    let ac = air_conditioner();
    let s = ac.snapshot;
    let a = apply_edit(
        &s,
        &EditOp::CreateClockDomain {
            name: "sensing".into(),
        },
    )
    .expect("clock");
    let sensing: ClockId = a.outcome.created_clock.expect("clock");
    let s = edit(
        &a.snapshot,
        EditOp::SetMappingClock {
            id: ac.temp_sensor,
            clock: Some(sensing),
        },
    );
    // one argument in `sensing`, the other agnostic: the value updates in
    // `sensing`
    {
        let mut host = IdeHost::new(s.clone());
        let snap = host.snapshot();
        let act = the_action(&snap, ac.ctrl);
        assert!(act.is_ready());
        let EditOp::CreateMapping { clock, .. } = act
            .plan
            .as_ref()
            .expect("plan")
            .model_edits()
            .next()
            .expect("edit")
        else {
            panic!()
        };
        assert_eq!(*clock, Some(sensing));
    }
    // the other argument in a second domain: blocked, both domains named
    let a = apply_edit(&s, &EditOp::CreateClockDomain { name: "ui".into() }).expect("clock");
    let ui = a.outcome.created_clock.expect("clock");
    let s = edit(
        &a.snapshot,
        EditOp::SetMappingClock {
            id: ac.button_input,
            clock: Some(ui),
        },
    );
    let mut host = IdeHost::new(s);
    let snap = host.snapshot();
    let act = the_action(&snap, ac.ctrl);
    let Applicability::Blocked { reason } = &act.applicability else {
        panic!("{:?}", act.applicability)
    };
    assert!(reason.contains("`ButtonInput` updates in `ui`"), "{reason}");
    assert!(reason.contains("`sensing`"), "{reason}");
    assert!(reason.contains("transport"), "{reason}");
    let _ = (ac.button_held, ac.switch_state);
}
