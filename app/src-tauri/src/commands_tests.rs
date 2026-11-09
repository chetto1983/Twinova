use super::*;
use std::sync::mpsc::{self, Receiver};
use tauri::test::{MockRuntime, mock_app};
use tauri::{App, Listener, Manager};

/// Long enough for a loaded CI runner; a passing run returns as soon as the event arrives.
const WAIT: Duration = Duration::from_secs(10);

fn app() -> App<MockRuntime> {
    let app = mock_app();
    app.manage(AppState::default());
    app.manage(ProjectState::default());
    app
}

/// The demo's own sources, as the editor would send them back.
fn sources(app: &App<MockRuntime>) -> (Vec<(String, String)>, BTreeMap<String, String>) {
    let project = app.state::<ProjectState>().project().unwrap();
    (project.program, project.behaviours)
}

fn build_demo(app: &App<MockRuntime>) -> Built {
    let (program, behaviours) = sources(app);
    build(app.state(), app.state(), program, behaviours).unwrap()
}

fn value<'a>(s: &'a Snapshot, name: &str) -> &'a str {
    &s.signals.iter().find(|x| x.name == name).unwrap().value
}

/// Collects the payloads of one event name.
fn events(app: &App<MockRuntime>, name: &'static str) -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    let tx = Mutex::new(tx);
    app.listen_any(name, move |e| {
        drop(tx.lock().unwrap().send(e.payload().to_string()))
    });
    rx
}

#[test]
fn every_command_but_build_needs_a_built_project() {
    let app = app();
    assert_eq!(
        step(app.state()).err().as_deref(),
        Some("build the project first")
    );
    assert!(press(app.state(), "%IX1.0".into()).is_err());
    assert!(set_sink_blocked(app.state(), 0, true).is_err());
    assert!(reset(app.state()).is_err());
    assert!(run(app.handle().clone(), app.state(), Some(1.0)).is_err());
}

#[test]
fn a_failed_build_keeps_the_running_line_and_the_project_sources() {
    let app = app();
    let built = build_demo(&app);
    assert_eq!(
        built.warnings,
        ["sink.count at %ID4: the program does not declare it"]
    );
    step(app.state()).unwrap();
    let (program, mut behaviours) = sources(&app);
    let belt = behaviours.get_mut("twinova.belt@1").unwrap();
    *belt = belt.replace("motor := run;", "motor := ;");
    let error = build(app.state(), app.state(), program.clone(), behaviours).unwrap_err();
    assert!(error.contains("components/twinova.belt@1.st:15"), "{error}");
    assert_eq!(
        step(app.state()).unwrap().time_ms,
        20,
        "the line keeps running"
    );
    let kept = app.state::<ProjectState>().project().unwrap();
    assert!(kept.behaviours["twinova.belt@1"].contains("motor := run;"));
    let error = build(app.state(), app.state(), program, BTreeMap::new()).unwrap_err();
    assert!(
        error.contains("the behaviour of twinova.belt@1, is not listed"),
        "{error}"
    );
}

#[test]
fn an_accepted_build_becomes_the_project_sources() {
    let app = app();
    let (program, mut behaviours) = sources(&app);
    let belt = behaviours.get_mut("twinova.belt@1").unwrap();
    *belt = belt.replace("motor := run;", "motor := run; (* edited *)");
    build(app.state(), app.state(), program, behaviours).unwrap();
    let kept = app.state::<ProjectState>().project().unwrap();
    assert!(kept.behaviours["twinova.belt@1"].contains("(* edited *)"));
}

#[test]
fn the_commands_drive_the_engine_and_reset_rebuilds_the_last_project() {
    let app = app();
    assert_eq!(build_demo(&app).snapshot.time_ms, 0);
    press(app.state(), "%IX1.0".into()).unwrap();
    let snapshot = step(app.state()).unwrap();
    assert_eq!(
        (snapshot.time_ms, value(&snapshot, "belt_a.run")),
        (10, "TRUE")
    );
    assert!(set_sink_blocked(app.state(), 0, true).unwrap().sinks[0].blocked);
    let snapshot = reset(app.state()).unwrap();
    assert_eq!(snapshot.time_ms, 0);
    assert_eq!(value(&snapshot, "belt_a.run"), "FALSE");
    assert!(!snapshot.sinks[0].blocked);
}

#[test]
fn run_emits_snapshots_until_paused() {
    let app = app();
    let snapshots = events(&app, "snapshot");
    let stopped = events(&app, "stopped");
    build_demo(&app);
    run(app.handle().clone(), app.state(), Some(1.0)).unwrap();
    run(app.handle().clone(), app.state(), Some(1.0))
        .expect("a second run while running does nothing");
    snapshots.recv_timeout(WAIT).expect("a snapshot");
    pause(app.state());
    stopped.recv_timeout(WAIT).expect("the loop stops");
}

#[test]
fn a_scan_error_stops_the_run_and_is_reported() {
    let app = app();
    let errors = events(&app, "engine-error");
    let stopped = events(&app, "stopped");
    let (mut program, behaviours) = sources(&app);
    program[0].1 = program[0]
        .1
        .replace(
            "IF start THEN",
            "zero := 0;\nresult := 10 / zero;\nIF start THEN",
        )
        .replace(
            "jam : BOOL;",
            "jam : BOOL;\n    zero : INT;\n    result : INT;",
        );
    build(app.state(), app.state(), program, behaviours).unwrap();
    run(app.handle().clone(), app.state(), Some(1.0)).unwrap();
    let error = errors.recv_timeout(WAIT).expect("an engine error");
    assert!(error.to_lowercase().contains("division"), "{error}");
    stopped.recv_timeout(WAIT).expect("the loop stops");
}

#[test]
fn a_saved_state_comes_back_on_restore() {
    let app = app();
    assert!(
        save_state(app.state()).is_err(),
        "nothing to save before a build"
    );
    assert!(restore_state(app.state()).is_err(), "nothing saved yet");
    build_demo(&app);
    press(app.state(), "%IX1.0".into()).unwrap();
    for _ in 0..50 {
        step(app.state()).unwrap();
    }
    save_state(app.state()).unwrap();
    let saved = step(app.state()).unwrap();
    for _ in 0..100 {
        step(app.state()).unwrap();
    }
    let restored = restore_state(app.state()).unwrap();
    assert_eq!(restored.time_ms, saved.time_ms - 10);
    assert_eq!(value(&restored, "belt_a.run"), "TRUE");
    assert_eq!(step(app.state()).unwrap(), saved);
}

#[test]
fn run_as_fast_as_possible_outruns_the_wall_clock() {
    let app = app();
    let snapshots = events(&app, "snapshot");
    let stopped = events(&app, "stopped");
    build_demo(&app);
    run(app.handle().clone(), app.state(), None).unwrap();
    let first = snapshots.recv_timeout(WAIT).expect("a snapshot");
    pause(app.state());
    stopped.recv_timeout(WAIT).expect("the loop stops");
    let time: serde_json::Value = serde_json::from_str(&first).unwrap();
    // One tick scans for 25 ms of wall time; the demo does more than 25 ms of virtual time in that.
    assert!(time["time_ms"].as_i64().unwrap() > 25, "{first}");
}
