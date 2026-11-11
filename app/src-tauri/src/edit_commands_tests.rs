//! The editor's commands (spec 2026-11-11-editor-basics-design.md, acceptance 2 and 3): each edit through the command,
//! one undo step each, a refused edit changing nothing; opening another project forgets the history, Build keeps it.

use super::*;
use crate::commands::{AppState, build};
use crate::project_commands::{FlowView, MountView, open_project, save_project};
use serde_json::{from_value, json};
use std::path::Path;
use tauri::Manager;
use tauri::test::{MockRuntime, mock_app};

fn app() -> tauri::App<MockRuntime> {
    let app = mock_app();
    app.manage(AppState::default());
    app.manage(ProjectState::default());
    app
}

fn op(value: serde_json::Value) -> Edit {
    from_value(value).expect("an edit")
}

fn send(app: &tauri::App<MockRuntime>, value: serde_json::Value) -> Result<ProjectView, String> {
    edit(app.state(), op(value))
}

const NO_TURN: [f64; 4] = [0.0, 0.0, 0.0, 1.0];

/// From a new project: a source, a belt, a sink and a sensor placed, connected, mounted and bound.
fn small_line(app: &tauri::App<MockRuntime>) -> ProjectView {
    let empty = new_project(app.state()).unwrap();
    assert!(empty.instances.is_empty() && !empty.can_undo && empty.folder.is_none());
    for (family, x) in [
        ("twinova.source@1", -0.5),
        ("twinova.belt@1", 0.0),
        ("twinova.sink@1", 3.2),
    ] {
        send(
            app,
            json!({ "op": "place", "family": family, "position": [x, 0, 0], "rotation": NO_TURN }),
        )
        .unwrap();
    }
    send(app, json!({ "op": "place", "family": "twinova.sensor_diffuse@1", "position": [2, 0, 0], "rotation": NO_TURN }))
        .unwrap();
    send(
        app,
        json!({ "op": "set_flow", "id": "belt_1", "port": "in", "peer": ["source_1", "out"] }),
    )
    .unwrap();
    send(
        app,
        json!({ "op": "set_flow", "id": "belt_1", "port": "out", "peer": ["sink_1", "in"] }),
    )
    .unwrap();
    send(
        app,
        json!({ "op": "set_mount", "device": "sensor_diffuse_1", "host": "belt_1", "at": 2.5 }),
    )
    .unwrap();
    send(
        app,
        json!({ "op": "bind_signal", "id": "belt_1", "signal": "run", "address": "%QX0.0" }),
    )
    .unwrap();
    send(
        app,
        json!({ "op": "bind_signal", "id": "belt_1", "signal": "running", "address": "%IX1.0" }),
    )
    .unwrap();
    send(app, json!({ "op": "bind_signal", "id": "sensor_diffuse_1", "signal": "detected", "address": " %IX0.0 " }))
        .unwrap();
    send(
        app,
        json!({ "op": "bind_signal", "id": "sink_1", "signal": "count", "address": "%ID4" }),
    )
    .unwrap();
    send(
        app,
        json!({ "op": "set_parameter", "id": "belt_1", "name": "length", "value": 3.0 }),
    )
    .unwrap()
}

#[test]
fn a_line_is_built_by_hand_from_a_new_project_and_builds() {
    let app = app();
    let view = small_line(&app);
    assert_eq!(
        view.flows,
        [
            FlowView {
                from: ("source_1".into(), "out".into()),
                to: ("belt_1".into(), "in".into())
            },
            FlowView {
                from: ("belt_1".into(), "out".into()),
                to: ("sink_1".into(), "in".into())
            },
        ]
    );
    assert_eq!(
        view.mounts,
        [MountView {
            device: "sensor_diffuse_1".into(),
            host: "belt_1".into(),
            at: 2.5
        }]
    );
    let sensor = view
        .instances
        .iter()
        .find(|i| i.id == "sensor_diffuse_1")
        .unwrap();
    assert_eq!(
        sensor.signals["detected"], "%IX0.0",
        "an address is trimmed"
    );
    assert!(view.can_undo && !view.can_redo);
    let program = vec![(
        "plc/Main.st".to_string(),
        "PROGRAM Main\nVAR\n    run AT %QX0.0 : BOOL;\n    running AT %IX1.0 : BOOL;\n    seen AT %IX0.0 : BOOL;\n    count AT %ID4 : DINT;\nEND_VAR\nrun := TRUE;\nEND_PROGRAM\n"
            .to_string(),
    )];
    let built = build(app.state(), app.state(), program, view.behaviours.clone()).unwrap();
    assert_eq!(built.warnings, Vec::<String>::new());
    let after = crate::project_commands::current_project(app.state()).unwrap();
    assert!(after.can_undo, "Build keeps the history");
}

#[test]
fn undo_walks_back_to_the_empty_scene_and_redo_forward_to_the_line() {
    let app = app();
    let line = small_line(&app);
    let mut steps = 0;
    let mut view = line.clone();
    while view.can_undo {
        view = undo(app.state()).unwrap();
        steps += 1;
    }
    assert_eq!(steps, 12, "one step per edit");
    assert!(view.instances.is_empty() && view.definitions.is_empty() && view.can_redo);
    while view.can_redo {
        view = redo(app.state()).unwrap();
    }
    assert_eq!(view, line);
    assert_eq!(undo(app.state()).unwrap().instances.len(), 4);
}

#[test]
fn a_refused_edit_changes_nothing_and_says_why() {
    let app = app();
    let line = small_line(&app);
    for (value, error) in [
        (
            json!({ "op": "rename", "id": "belt_1", "name": "" }),
            "a name cannot be empty",
        ),
        (
            json!({ "op": "remove", "id": "ghost" }),
            "no instance ghost",
        ),
        (
            json!({ "op": "set_parameter", "id": "belt_1", "name": "length", "value": "long" }),
            "belt_1.length is a REAL: \"long\" is not one",
        ),
        (
            json!({ "op": "place", "family": "twinova.robot@1", "position": [0, 0, 0], "rotation": NO_TURN }),
            "no family twinova.robot@1 in the library",
        ),
    ] {
        assert_eq!(send(&app, value), Err(error.into()));
    }
    assert_eq!(
        crate::project_commands::current_project(app.state()).unwrap(),
        line
    );
}

#[test]
fn moving_renaming_unmounting_and_removing_through_the_command() {
    let app = app();
    small_line(&app);
    let quarter = [
        0.0,
        std::f64::consts::FRAC_1_SQRT_2,
        0.0,
        std::f64::consts::FRAC_1_SQRT_2,
    ];
    send(
        &app,
        json!({ "op": "move", "id": "belt_1", "position": [1, 0, 2], "rotation": quarter }),
    )
    .unwrap();
    send(
        &app,
        json!({ "op": "rename", "id": "belt_1", "name": "Infeed" }),
    )
    .unwrap();
    send(
        &app,
        json!({ "op": "bind_signal", "id": "belt_1", "signal": "running", "address": null }),
    )
    .unwrap();
    send(
        &app,
        json!({ "op": "set_parameter", "id": "belt_1", "name": "length", "value": null }),
    )
    .unwrap();
    send(
        &app,
        json!({ "op": "set_flow", "id": "sink_1", "port": "in", "peer": null }),
    )
    .unwrap();
    let view = send(
        &app,
        json!({ "op": "set_mount", "device": "sensor_diffuse_1", "host": null, "at": 0 }),
    )
    .unwrap();
    let belt = view.instances.iter().find(|i| i.id == "belt_1").unwrap();
    assert_eq!(
        (belt.position, belt.rotation, belt.name.as_str()),
        ([1.0, 0.0, 2.0], quarter, "Infeed")
    );
    assert!(!belt.signals.contains_key("running") && !belt.parameters.contains_key("length"));
    assert_eq!(view.flows.len(), 1);
    assert!(view.mounts.is_empty());
    let view = send(&app, json!({ "op": "remove", "id": "belt_1" })).unwrap();
    assert_eq!(view.instances.len(), 3);
    assert!(view.flows.is_empty(), "its flows went with it");
}

#[test]
fn opening_another_project_forgets_the_history_and_a_new_project_saves() {
    let app = app();
    small_line(&app);
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("project/tests/fixtures/valid/demo_m3");
    let view = open_project(app.state(), fixture.display().to_string()).unwrap();
    assert!(!view.can_undo && !view.can_redo);
    assert!(!undo(app.state()).unwrap().can_redo, "nothing to undo");
    small_line(&app);
    let dir = tempfile::tempdir().unwrap();
    let saved = save_project(app.state(), dir.path().display().to_string()).unwrap();
    assert!(saved.can_undo, "saving keeps the history");
    let reopened = open_project(app.state(), dir.path().display().to_string()).unwrap();
    assert_eq!(reopened.instances, saved.instances);
    assert_eq!(reopened.name, "Untitled");
}
