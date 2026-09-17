//! The textual workspace end to end: sources → system → analysis, and the
//! model written back as text with identities intact (ADR-0020).

use bdl_model::surface::{Definition, Representation};
use bdl_model::Dim;
use bdl_system::{analyze_system, Acceptance, BindingEnd, PortKind, SystemSnapshot};
use bdl_text::build::{AnchorRole, TextEntity};
use bdl_text::identity::{IdentityTable, KeyKind, SourceKey};
use bdl_text::{load_workspace, write_back, SourceFile};

const SYSTEM: &str = include_str!("../../bdl-syntax/test_data/valid/system.bdl");

fn files(pairs: &[(&str, &str)]) -> Vec<SourceFile> {
    pairs
        .iter()
        .map(|(p, t)| SourceFile {
            path: (*p).into(),
            text: (*t).into(),
        })
        .collect()
}

#[test]
fn the_system_corpus_loads_into_the_authored_model_and_analyses() {
    let fs = files(&[("src/system.bdl", SYSTEM)]);
    let b = load_workspace("lamp", &fs, &IdentityTable::default());
    let errors: Vec<_> = b.faults.iter().filter(|f| !f.is_open()).collect();
    assert!(errors.is_empty(), "{errors:#?}");
    let s = &b.system;
    assert_eq!(s.base.concepts.len(), 3);
    assert_eq!(s.base.clocks.len(), 2);
    assert_eq!(s.base.outputs.len(), 2);
    assert_eq!(s.base.devices.len(), 2);
    assert_eq!(s.base.mappings.len(), 5);
    let light = s.base.outputs.values().find(|o| o.name == "light").unwrap();
    assert!(light.required);
    let indicator = s
        .base
        .outputs
        .values()
        .find(|o| o.name == "indicator")
        .unwrap();
    assert!(!indicator.required);
    let brightness = s
        .base
        .mappings
        .values()
        .find(|m| m.name == "brightness")
        .unwrap();
    assert_eq!(brightness.drives, Some(light.id));
    let tilt_value = s
        .base
        .mappings
        .values()
        .find(|m| m.name == "tiltValue")
        .unwrap();
    assert_eq!(
        tilt_value.definition,
        Some(Definition::Formula {
            source: "tilt".into()
        })
    );
    let pwm = s
        .base
        .devices
        .values()
        .find(|d| d.name == "pwmLight")
        .unwrap();
    assert_eq!(pwm.output, Some(light.id));
    assert_eq!(pwm.fixed_pins.get(&0).map(String::as_str), Some("D3"));

    // The component: shared concepts, a timing parameter, a private clock,
    // three ports backed by body relationships, one body relationship.
    assert_eq!(s.components.len(), 1);
    let c = s.components.values().next().unwrap();
    assert_eq!(c.name, "AdaptiveLamp");
    assert_eq!(c.shared_concepts.len(), 3);
    assert_eq!(c.interface.clock_params.len(), 1);
    assert_eq!(c.body.clocks.len(), 2);
    assert_eq!(c.interface.ports.len(), 3);
    let kinds: Vec<PortKind> = c.interface.ports.values().map(|p| p.kind).collect();
    assert_eq!(
        kinds,
        vec![PortKind::Required, PortKind::Parameter, PortKind::Provided]
    );
    let dim = c
        .body
        .mappings
        .values()
        .find(|m| m.name == "dimByTilt")
        .unwrap();
    assert_eq!(dim.parameters, vec!["t".to_owned()]);
    assert_eq!(c.body.mappings.len(), 4);

    // Instances, bindings with fan-out and a transport, an export.
    assert_eq!(s.instances.len(), 2);
    let lamp_a = s.instances.values().find(|i| i.name == "lampA").unwrap();
    assert_eq!(lamp_a.clock_bindings.len(), 1);
    assert_eq!(lamp_a.parameter_bindings.len(), 1);
    assert_eq!(s.bindings.len(), 5);
    let transported: Vec<_> = s
        .bindings
        .values()
        .filter(|b| b.transport.is_some())
        .collect();
    assert_eq!(transported.len(), 1);
    assert_eq!(transported[0].transport.as_ref().unwrap().init, "0");
    assert!(matches!(
        transported[0].destination,
        BindingEnd::Base { .. }
    ));
    assert_eq!(s.exports.len(), 1);

    // The existing pipeline accepts it as it accepts a Studio-authored system.
    let analysis = analyze_system(&SystemSnapshot::new(s.clone()));
    assert_eq!(
        analysis.acceptance,
        Acceptance::Executable,
        "{:#?} / {:#?}",
        analysis.composition,
        analysis
            .analysis
            .diagnostics
            .iter()
            .map(|d| format!("{} {:?} {}", d.code.as_str(), d.entity, d.message))
            .collect::<Vec<_>>()
    );
    assert!(analysis.analysis.causality.cycles.is_empty());
    assert!(analysis.analysis.output_complete);

    // Anchors: every declaring entity has an item and a name.
    let n_items = b
        .anchors
        .iter()
        .filter(|a| a.role == AnchorRole::Item)
        .count();
    assert!(n_items >= 24, "{n_items}");
    assert!(b
        .anchors
        .iter()
        .any(|a| matches!(a.entity, TextEntity::Port(_, _)) && a.role == AnchorRole::Name));
}

#[test]
fn identities_survive_a_reload_a_rename_a_move_and_a_split() {
    let a = "concept Tilt : Angle\nconcept Brightness : Scalar\n\nclock main\n";
    let l = "mapping tilt : Tilt @main\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(t) = t / (90 deg)\nmapping brightness : Brightness @main\nbrightness() = dimByTilt(tilt)\n";
    let fs = files(&[("src/concepts.bdl", a), ("src/lighting.bdl", l)]);
    let b1 = load_workspace("lamp", &fs, &IdentityTable::default());
    assert!(!b1.has_errors(), "{:#?}", b1.faults);
    let tilt = b1
        .system
        .base
        .concepts
        .values()
        .find(|c| c.name == "Tilt")
        .unwrap()
        .id;
    let dim = b1
        .system
        .base
        .mappings
        .values()
        .find(|m| m.name == "dimByTilt")
        .unwrap()
        .id;
    // Reload against the table: everything retained, nothing allocated.
    let b2 = load_workspace("lamp", &fs, &b1.table);
    assert_eq!(b2.reconciliation.allocated, Vec::<String>::new());
    assert_eq!(b2.system, b1.system);

    // A raw rename of the concept in the editor: the unique same-file swap
    // keeps the identity; the relationship's signature follows the name.
    let a2 = a.replace("concept Tilt", "concept Angle_");
    let l2 = l.replace(": Tilt", ": Angle_");
    let fs2 = files(&[("src/concepts.bdl", &a2), ("src/lighting.bdl", &l2)]);
    let b3 = load_workspace("lamp", &fs2, &b2.table);
    assert!(!b3.has_errors(), "{:#?}", b3.faults);
    assert_eq!(
        b3.reconciliation.renamed,
        vec![("concept:Tilt".to_owned(), "concept:Angle_".to_owned())]
    );
    let renamed = b3
        .system
        .base
        .concepts
        .values()
        .find(|c| c.name == "Angle_")
        .unwrap();
    assert_eq!(renamed.id, tilt);
    assert_eq!(b3.system.base.mappings[&dim].signature.inputs, vec![tilt]);

    // Move the relationship to another file: same key, same id.
    let l3 = "mapping tilt : Angle_ @main\nmapping brightness : Brightness @main\nbrightness() = dimByTilt(tilt)\n";
    let d3 = "mapping dimByTilt : Angle_ -> Brightness\ndimByTilt(t) = t / (90 deg)\n";
    let fs3 = files(&[
        ("src/concepts.bdl", &a2),
        ("src/dimming.bdl", d3),
        ("src/lighting.bdl", l3),
    ]);
    let b4 = load_workspace("lamp", &fs3, &b3.table);
    assert!(!b4.has_errors(), "{:#?}", b4.faults);
    assert_eq!(b4.system.base.mappings[&dim].name, "dimByTilt");
    assert!(b4.reconciliation.allocated.is_empty());
    assert!(b4.reconciliation.dropped.is_empty());

    // Copy/paste and rename: a fresh id, not an alias.
    let d4 =
        format!("{d3}mapping dimByTilt2 : Angle_ -> Brightness\ndimByTilt2(t) = t / (45 deg)\n");
    let fs4 = files(&[
        ("src/concepts.bdl", &a2),
        ("src/dimming.bdl", &d4),
        ("src/lighting.bdl", l3),
    ]);
    let b5 = load_workspace("lamp", &fs4, &b4.table);
    let copy = b5
        .system
        .base
        .mappings
        .values()
        .find(|m| m.name == "dimByTilt2")
        .unwrap();
    assert_ne!(copy.id, dim);
    assert_eq!(b5.reconciliation.allocated, vec!["mapping:dimByTilt2"]);

    // Delete, "save" (the table now lacks it), recreate with the same name:
    // a new identity by default.
    let b6 = load_workspace("lamp", &fs3, &b5.table);
    assert_eq!(b6.reconciliation.dropped, vec!["mapping:dimByTilt2"]);
    let b7 = load_workspace("lamp", &fs4, &b6.table);
    let again = b7
        .system
        .base
        .mappings
        .values()
        .find(|m| m.name == "dimByTilt2")
        .unwrap();
    assert_ne!(again.id, copy.id);
}

#[test]
fn ambiguous_renames_get_fresh_ids_and_a_fault() {
    let l = "concept A : Scalar\nmapping x : A\nmapping y : A\n";
    let fs = files(&[("src/l.bdl", l)]);
    let b1 = load_workspace("p", &fs, &IdentityTable::default());
    let l2 = "concept A : Scalar\nmapping p : A\nmapping q : A\n";
    let b2 = load_workspace("p", &files(&[("src/l.bdl", l2)]), &b1.table);
    assert_eq!(b2.reconciliation.ambiguous.len(), 2);
    assert!(b2
        .faults
        .iter()
        .any(|f| f.code() == "text.ambiguous_identity"));
}

#[test]
fn duplicates_and_unknown_names_are_faults_not_panics() {
    let src = "concept A : Scalar\nconcept A : Bool\nmapping f : Nope -> A\nclock c\noutput o : A @nowhere\ndrive o = g\ninstance i : Missing\nbind i.p = f\n";
    let b = load_workspace(
        "p",
        &files(&[("src/x.bdl", src)]),
        &IdentityTable::default(),
    );
    let codes: Vec<String> = b.faults.iter().map(|f| f.code()).collect();
    assert!(
        codes.contains(&"text.duplicate_item".to_owned()),
        "{codes:?}"
    );
    assert!(codes.contains(&"text.unknown_concept".to_owned()));
    assert!(codes.contains(&"text.unknown_clock".to_owned()));
    assert!(codes.contains(&"text.unknown_relationship".to_owned()));
    assert!(codes.contains(&"text.unknown_component".to_owned()));
    assert!(codes.contains(&"text.unknown_instance".to_owned()));
    assert_eq!(b.system.base.concepts.len(), 1);
    assert_eq!(
        b.system
            .base
            .concepts
            .values()
            .next()
            .unwrap()
            .representation,
        Some(Representation::Quantity { dim: Dim::ZERO })
    );
}

#[test]
fn write_back_splices_only_what_changed_and_keeps_comments() {
    let src = "// The lamp.\n\nconcept Tilt : Angle   // how far\nconcept Brightness : Scalar\n\nclock main\n\n// inputs\nmapping tilt : Tilt @main\n\nmapping dimByTilt : Tilt -> Brightness\ndimByTilt(t) =\n  t / (90 deg)\n\nmapping brightness : Brightness @main\nbrightness() = dimByTilt(tilt)\n\noutput light : Brightness @main\ndrive light = brightness\n";
    let fs = files(&[("src/lamp.bdl", src)]);
    let b = load_workspace("lamp", &fs, &IdentityTable::default());
    assert!(!b.has_errors(), "{:#?}", b.faults);

    // Nothing changed: nothing written.
    let same = write_back(&b, &fs, &b.system);
    assert!(same.changed.is_empty());
    assert_eq!(same.files[0].text, src);

    // Studio renames a concept, changes a formula, adds a concept and a
    // relationship, deletes the output's drive and adds a device.
    let mut edited = b.system.clone();
    let tilt = edited
        .base
        .concepts
        .values_mut()
        .find(|c| c.name == "Tilt")
        .unwrap();
    tilt.name = "HeadTilt".into();
    let tilt_id = tilt.id;
    let dim = edited
        .base
        .mappings
        .values_mut()
        .find(|m| m.name == "dimByTilt")
        .unwrap();
    dim.definition = Some(Definition::Formula {
        source: "t / (45 deg)".into(),
    });
    let br = edited
        .base
        .mappings
        .values_mut()
        .find(|m| m.name == "brightness")
        .unwrap();
    br.drives = None;
    let (held, ids) = edited.base.ids.fresh_semantic();
    edited.base.ids = ids;
    edited.base.concepts.insert(
        held,
        bdl_model::surface::Concept {
            id: held,
            name: "Held".into(),
            description: String::new(),
            representation: Some(Representation::Boolean),
        },
    );
    let (dev, ids) = edited.base.ids.fresh_device();
    edited.base.ids = ids;
    let light = edited.base.outputs.values().next().unwrap().id;
    edited.base.devices.insert(
        dev,
        bdl_model::surface::DeviceBinding {
            id: dev,
            name: "pwm".into(),
            kind: bdl_model::surface::DeviceKind::PwmChannel,
            output: Some(light),
            fixed_pins: [(0u16, "D3".to_owned())].into_iter().collect(),
        },
    );

    let wb = write_back(&b, &fs, &edited);
    assert_eq!(wb.changed, vec![0]);
    let text = &wb.files[0].text;
    // Comments and untouched items are byte-identical.
    assert!(
        text.starts_with("// The lamp.\n\nconcept HeadTilt : Angle   // how far\n"),
        "{text}"
    );
    assert!(
        text.contains("// inputs\nmapping tilt : HeadTilt @main\n"),
        "{text}"
    );
    assert!(text.contains("dimByTilt(t) =\n  t / (45 deg)\n"), "{text}");
    assert!(!text.contains("drive light"), "{text}");
    assert!(
        text.ends_with(
            "concept Held : Bool\n\ndevice pwm : pwm_channel for light { pin 0 = D3 }\n"
        ),
        "{text}"
    );

    // Reloading the written text against the written table gives the
    // edited model, identities included.
    let b2 = load_workspace("lamp", &wb.files, &wb.table);
    assert!(!b2.has_errors(), "{:#?}", b2.faults);
    assert!(
        b2.reconciliation.allocated.is_empty(),
        "{:?}",
        b2.reconciliation
    );
    assert_eq!(b2.system.base.concepts[&tilt_id].name, "HeadTilt");
    assert_eq!(b2.system.base.concepts[&held].name, "Held");
    assert_eq!(b2.system.base.devices[&dev].name, "pwm");
    assert_eq!(b2.system.base, edited.base);
    assert_eq!(
        b2.table
            .id_of(&SourceKey::top(KeyKind::Concept, "HeadTilt")),
        Some(tilt_id.raw())
    );
}

#[test]
fn write_back_of_a_component_body_and_of_system_items() {
    let fs = files(&[("src/system.bdl", SYSTEM)]);
    let b = load_workspace("lamp", &fs, &IdentityTable::default());
    let mut edited = b.system.clone();
    let cid = *edited.components.keys().next().unwrap();
    // Edit the body's formula and add a body relationship.
    {
        let c = edited.components.get_mut(&cid).unwrap();
        let dim = c
            .body
            .mappings
            .values_mut()
            .find(|m| m.name == "dimByTilt")
            .unwrap();
        dim.definition = Some(Definition::Formula {
            source: "t / (60 deg)".into(),
        });
        let (id, ids) = c.body.ids.fresh_decl();
        c.body.ids = ids;
        let bright = c
            .body
            .concepts
            .values()
            .find(|x| x.name == "Brightness")
            .unwrap()
            .id;
        c.body.mappings.insert(
            id,
            bdl_model::surface::MappingBlock {
                id,
                name: "half".into(),
                description: String::new(),
                signature: bdl_model::surface::Signature {
                    inputs: vec![bright],
                    output: bright,
                },
                definition: Some(Definition::Formula {
                    source: "Brightness / 2".into(),
                }),
                clock: None,
                drives: None,
                parameters: Vec::new(),
            },
        );
    }
    // Rename an instance: the bindings that name it re-render.
    let inst = edited
        .instances
        .values_mut()
        .find(|i| i.name == "lampB")
        .unwrap();
    inst.name = "lampC".into();
    let wb = write_back(&b, &fs, &edited);
    let text = &wb.files[0].text;
    assert!(
        text.contains("  dimByTilt(t) =\n    t / (60 deg)\n"),
        "{text}"
    );
    assert!(text.contains("\n\n  mapping half : Brightness -> Brightness\n  half(Brightness) =\n    Brightness / 2\n}"), "{text}");
    assert!(
        text.contains("instance lampC : AdaptiveLamp { main = interaction, gain = 1 }"),
        "{text}"
    );
    assert!(text.contains("bind lampC.tiltValue = tiltValue"), "{text}");
    assert!(text.contains("export lampC.tiltValue as tiltIn"), "{text}");
    let b2 = load_workspace("lamp", &wb.files, &wb.table);
    assert!(!b2.has_errors(), "{:#?}", b2.faults);
    assert!(
        b2.reconciliation.allocated.is_empty(),
        "{:?}",
        b2.reconciliation
    );
    let c2 = &b2.system.components[&cid];
    assert_eq!(c2.body.mappings.len(), 5);
    assert_eq!(
        b2.system
            .instances
            .values()
            .find(|i| i.name == "lampC")
            .map(|i| i.id),
        edited
            .instances
            .values()
            .find(|i| i.name == "lampC")
            .map(|i| i.id)
    );
    let analysis = analyze_system(&SystemSnapshot::new(b2.system.clone()));
    assert_eq!(analysis.acceptance, Acceptance::Executable);
}

#[test]
fn doc_comments_are_descriptions_and_round_trip() {
    let src = "// a plain comment stays outside\n\n/// How far the head is tilted.\n/// Two lines.\nconcept Tilt : Angle\n\n/// The lamp's level.\nconcept Brightness : Scalar\n\nmapping dimByTilt : Tilt -> Brightness\n";
    let fs = files(&[("src/l.bdl", src)]);
    let b = load_workspace("lamp", &fs, &IdentityTable::default());
    assert!(!b.has_errors(), "{:#?}", b.faults);
    let tilt = b
        .system
        .base
        .concepts
        .values()
        .find(|c| c.name == "Tilt")
        .unwrap();
    assert_eq!(tilt.description, "How far the head is tilted.\nTwo lines.");
    let tilt_id = tilt.id;
    let mut edited = b.system.clone();
    edited.base.concepts.get_mut(&tilt_id).unwrap().description = "Tilt of the head.".into();
    let dim = edited.base.mappings.values_mut().next().unwrap();
    dim.description = "Upright is off.".into();
    let wb = write_back(&b, &fs, &edited);
    let text = &wb.files[0].text;
    assert!(text.starts_with("// a plain comment stays outside\n\n/// Tilt of the head.\nconcept Tilt : Angle\n\n/// The lamp's level.\nconcept Brightness : Scalar\n\n/// Upright is off.\nmapping dimByTilt : Tilt -> Brightness\n"), "{text}");
    let b2 = load_workspace("lamp", &wb.files, &wb.table);
    assert_eq!(b2.system.base, edited.base);
}

/// The text of a system and the system are one thing: rendering the
/// loaded system and loading the rendering gives the same system, the
/// same flat design and the same executable plan — ids included, since
/// the identity table carries them across.
#[test]
fn text_and_model_agree_down_to_the_executable_plan() {
    let fs = files(&[("src/system.bdl", SYSTEM)]);
    let first = load_workspace("lamp", &fs, &IdentityTable::default());
    assert!(
        first.faults.iter().all(|f| f.is_open()),
        "{:#?}",
        first.faults
    );
    let rendered = bdl_text::print::render_system(&first.system);
    let again = load_workspace(
        "lamp",
        &files(&[("src/system.bdl", &rendered)]),
        &first.table,
    );
    assert!(
        again.faults.iter().all(|f| f.is_open()),
        "{:#?}",
        again.faults
    );
    assert_eq!(again.system, first.system);
    assert_eq!(again.table, first.table);

    let flat = |s: &bdl_system::BehaviorSystem| {
        bdl_system::flatten(&SystemSnapshot::new(s.clone())).snapshot
    };
    let (a, b) = (flat(&first.system), flat(&again.system));
    assert_eq!(a.design, b.design);
    let options = bdl_compiler::CompileOptions {
        require_complete: false,
        codegen: Default::default(),
    };
    let (ca, cb) = (
        bdl_compiler::compile(&a, &options),
        bdl_compiler::compile(&b, &options),
    );
    assert!(ca.exec_ir.is_some(), "{:#?}", ca.diagnostics);
    assert_eq!(ca.exec_ir, cb.exec_ir);
    assert_eq!(ca.generated.map(|g| g.files), cb.generated.map(|g| g.files));
}

/// Loading stays proportional to the source: a project of many files
/// and thousands of items builds in well under the time a person waits
/// for an editor.
#[test]
fn a_large_project_loads_in_bounded_time() {
    let mut fs = Vec::new();
    let mut concepts = String::from("clock main\n");
    for i in 0..200 {
        concepts.push_str(&format!("concept C{i} : Scalar\n"));
    }
    fs.push(SourceFile {
        path: "src/concepts.bdl".into(),
        text: concepts,
    });
    for f in 0..40 {
        let mut text = String::new();
        for i in 0..50 {
            let c = (f * 50 + i) % 200;
            text.push_str(&format!(
                "mapping m{f}_{i} : C{c} -> C{c}\nm{f}_{i}(x) = x * 2 + 1\n\n"
            ));
        }
        fs.push(SourceFile {
            path: format!("src/f{f:02}.bdl"),
            text,
        });
    }
    let started = std::time::Instant::now();
    let b = load_workspace("big", &fs, &IdentityTable::default());
    let loaded = started.elapsed();
    assert!(
        b.faults.iter().all(|f| f.is_open()),
        "{:#?}",
        &b.faults[..b.faults.len().min(3)]
    );
    assert_eq!(b.system.base.mappings.len(), 2000);
    let again = load_workspace("big", &fs, &b.table);
    let reloaded = started.elapsed() - loaded;
    assert_eq!(again.table, b.table);
    eprintln!("load {loaded:?}, reload {reloaded:?}");
    assert!(
        loaded < std::time::Duration::from_secs(5),
        "load took {loaded:?}"
    );
    assert!(
        reloaded < std::time::Duration::from_secs(5),
        "reload took {reloaded:?}"
    );
}
