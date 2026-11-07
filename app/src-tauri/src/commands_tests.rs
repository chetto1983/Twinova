use super::*;
use std::sync::mpsc::{self, Receiver};
use tauri::test::{MockRuntime, mock_app};
use tauri::{App, Listener, Manager};

/// Long enough for a loaded CI runner; a passing run returns as soon as the event arrives.
const WAIT: Duration = Duration::from_secs(10);

/// The default contract plus an integer division by zero, which truST reports as a cycle error
/// (trust-runtime src/runtime/installed_cycle_contract_tests.rs, DivisionByZero).
const FAULTING: &str = "PROGRAM Main\nVAR sensor : BOOL; start : BOOL; stop : BOOL; motor : BOOL;\n\
    zero : INT; result : INT; END_VAR\nresult := 10 / zero;\nEND_PROGRAM\n";

fn app() -> App<MockRuntime> {
    let app = mock_app();
    app.manage(AppState::default());
    app
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
fn every_command_but_build_needs_a_built_program() {
    let app = app();
    assert_eq!(
        step(app.state()).err().as_deref(),
        Some("build a program first")
    );
    assert!(press(app.state(), true, false).is_err());
    assert!(set_outlet_blocked(app.state(), true).is_err());
    assert!(run(app.handle().clone(), app.state()).is_err());
}

#[test]
fn a_failed_build_keeps_no_engine() {
    let app = app();
    assert!(build(app.state(), "PROGRAM Main END_PROGRAM".into()).is_err());
    assert!(step(app.state()).is_err());
}

#[test]
fn the_commands_drive_the_engine_and_reset_rebuilds_the_last_program() {
    let app = app();
    assert_eq!(default_program(), DEFAULT_PROGRAM);
    let snapshot = build(app.state(), DEFAULT_PROGRAM.into()).unwrap();
    assert_eq!(snapshot.time_ms, 0);
    press(app.state(), true, false).unwrap();
    let snapshot = step(app.state()).unwrap();
    assert_eq!((snapshot.time_ms, snapshot.motor), (10, true));
    assert!(
        set_outlet_blocked(app.state(), true)
            .unwrap()
            .outlet_blocked
    );
    let snapshot = reset(app.state()).unwrap();
    assert_eq!(snapshot.time_ms, 0);
    assert!(!snapshot.motor && !snapshot.outlet_blocked);
}

#[test]
fn run_emits_snapshots_until_paused() {
    let app = app();
    let snapshots = events(&app, "snapshot");
    let stopped = events(&app, "stopped");
    build(app.state(), DEFAULT_PROGRAM.into()).unwrap();
    run(app.handle().clone(), app.state()).unwrap();
    run(app.handle().clone(), app.state()).expect("a second run while running does nothing");
    snapshots.recv_timeout(WAIT).expect("a snapshot");
    pause(app.state());
    stopped.recv_timeout(WAIT).expect("the loop stops");
}

#[test]
fn a_scan_error_stops_the_run_and_is_reported() {
    let app = app();
    let errors = events(&app, "engine-error");
    let stopped = events(&app, "stopped");
    build(app.state(), FAULTING.into()).unwrap();
    run(app.handle().clone(), app.state()).unwrap();
    let error = errors.recv_timeout(WAIT).expect("an engine error");
    assert!(error.to_lowercase().contains("division"), "{error}");
    stopped.recv_timeout(WAIT).expect("the loop stops");
}
