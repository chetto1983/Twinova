use super::*;
use std::time::{Duration, Instant};
use tauri::Manager;
use tauri::test::MockRuntime;

use crate::commands::{HOLD, pause, reset, run, scan_timing, step};
use crate::monitor::{watch_range, watched};
use crate::test_fixtures::opened;

const MAIN: &str = "plc/Main.st";

fn held(app: &tauri::App<MockRuntime>) -> Hold {
    let started = Instant::now();
    loop {
        if let Some(hold) = hold(app.state()).unwrap() {
            return hold;
        }
        assert!(started.elapsed() < Duration::from_secs(10), "no stop");
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn the_app_answers_while_the_line_is_held_and_reset_lets_it_go() {
    let app = opened("two_belts_diverter", true);
    assert_eq!(
        hold(app.state()).unwrap(),
        None,
        "no debugger before the first breakpoint"
    );
    assert_eq!(
        resume(app.state(), Resume::Continue).unwrap_err(),
        "no breakpoint is set"
    );
    assert_eq!(
        set_breakpoints(app.state(), MAIN.into(), vec![34, 37]).unwrap(),
        [37]
    );
    watch_range(app.state(), MAIN.into(), 40, 40, None, vec![]).unwrap();
    run(app.handle().clone(), app.state(), None).unwrap();
    assert_eq!(held(&app).line, 37);

    let started = Instant::now();
    assert_eq!(scan_timing(app.state()).unwrap_err(), HOLD);
    assert!(
        started.elapsed() < Duration::from_millis(100),
        "no wait on the held engine"
    );
    let values = watched(app.state()).unwrap();
    assert_eq!(
        values[0],
        Some("0".into()),
        "line 40's count, from the stop"
    );
    let rows = watch_range(app.state(), MAIN.into(), 40, 40, None, vec![])
        .unwrap()
        .rows;
    assert_eq!(rows.len(), 20, "while held, the table keeps its rows");

    resume(app.state(), Resume::Over).unwrap();
    assert_eq!(held(&app).line, 38);
    reset(app.state()).unwrap();
    assert_eq!(
        hold(app.state()).unwrap(),
        None,
        "Reset let the line go and dropped the debugger"
    );
    assert!(scan_timing(app.state()).is_ok());
    run(app.handle().clone(), app.state(), None).unwrap();
    assert_eq!(
        held(&app).line,
        37,
        "the session's breakpoint is armed again at Run"
    );
    pause(app.state());
    assert_eq!(
        set_breakpoints(app.state(), MAIN.into(), vec![]).unwrap(),
        Vec::<u32>::new()
    );
    tauri::async_runtime::block_on(step(app.state())).unwrap();
    assert_eq!(
        hold(app.state()).unwrap(),
        None,
        "no breakpoint left to arm"
    );
}

#[test]
fn a_step_stopped_by_a_breakpoint_returns_once_resumed() {
    let app = opened("two_belts_diverter", true);
    set_breakpoints(app.state(), MAIN.into(), vec![38]).unwrap();
    std::thread::scope(|scope| {
        let handle = app.handle().clone();
        let stepping = scope.spawn(move || tauri::async_runtime::block_on(step(handle.state())));
        assert_eq!(held(&app).line, 38);
        assert!(!stepping.is_finished(), "the step waits in its scan");
        run_to_cursor(app.state(), MAIN.into(), 39).unwrap();
        assert_eq!(held(&app).line, 39, "Run to cursor, in the same scan");
        resume(app.state(), Resume::Continue).unwrap();
        assert!(stepping.join().unwrap().is_ok());
    });
}
