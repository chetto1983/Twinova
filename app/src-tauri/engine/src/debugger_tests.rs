//! Breakpoints that hold the 4.4a line (spec 2026-11-13-breakpoints-hold-design.md). The tests use top-level lines:
//! at the pin, a breakpoint in an IF's body stops at the IF (plan, Task 6).
use super::*;
use crate::engine::Engine;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use trust_runtime::harness::TestHarness;
use twinova_project::storage::open_folder;

const MAIN: &str = "plc/Main.st";

fn diverter() -> Engine {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/two_belts_diverter");
    Engine::new(&open_folder(&dir).unwrap().opened.project)
        .unwrap()
        .0
}

/// The engine stepping to `end_ms` on a thread that holds it, as the app's Run loop does, and its time so far.
fn running(engine: Engine, end_ms: i64) -> (Arc<Mutex<Engine>>, Arc<AtomicI64>, JoinHandle<()>) {
    let shared = Arc::new(Mutex::new(engine));
    let time = Arc::new(AtomicI64::new(0));
    let (engine, now) = (shared.clone(), time.clone());
    let thread = std::thread::spawn(move || {
        loop {
            let mut e = engine.lock().unwrap();
            if e.time_ms() >= end_ms {
                return;
            }
            e.step().unwrap();
            now.store(e.time_ms(), Ordering::SeqCst);
        }
    });
    (shared, time, thread)
}

fn held(debugger: &Debugger) -> Hold {
    let started = Instant::now();
    loop {
        if let Some(hold) = debugger.held() {
            return hold;
        }
        assert!(started.elapsed() < Duration::from_secs(10), "no stop");
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn at(line: u32) -> Hold {
    Hold {
        file: MAIN.into(),
        line,
        frames: vec![(MAIN.into(), line)],
        device: None,
    }
}

/// Plain breakpoints at `lines`, and the lines they landed on.
fn set(debugger: &Debugger, path: &str, lines: &[u32]) -> Result<Vec<u32>, String> {
    let plain = |line| Breakpoint {
        line,
        condition: None,
        hit: None,
        log: None,
    };
    let set = debugger.set(path, &lines.iter().copied().map(plain).collect::<Vec<_>>())?;
    Ok(set.iter().map(|b| b.line).collect())
}

#[test]
fn a_breakpoint_holds_the_line_and_steps_continue_and_let_go_leave_its_trajectory() {
    const END: i64 = 2_000;
    let mut reference = diverter();
    while reference.time_ms() < END {
        reference.step().unwrap();
    }
    let mut engine = diverter();
    let debugger = engine.debugger();
    assert_eq!(debugger.held(), None);
    assert_eq!(
        debugger.read(&["Main.count".into()]),
        [None],
        "nothing is read while it runs"
    );
    assert_eq!(set(&debugger, MAIN, &[37]).unwrap(), [37]);
    let (shared, time, thread) = running(engine, END);
    assert_eq!(held(&debugger), at(37), "the first scan stops");
    let stopped_at = time.load(Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(
        time.load(Ordering::SeqCst),
        stopped_at,
        "HOLD: virtual time stands"
    );
    assert!(shared.try_lock().is_err(), "the engine is in its scan");
    assert_eq!(
        debugger.read(&[
            "Main.count".into(),
            "Main.count + 2".into(),
            "Main.nothing".into(),
            "Main.carry".into()
        ]),
        [Some("0".into()), Some("2".into()), None, None],
        "a value, an expression; no name and no single value read nothing"
    );
    debugger.resume(Resume::Over);
    assert_eq!(held(&debugger), at(38));
    debugger.resume(Resume::Into);
    assert_eq!(held(&debugger), at(39));
    debugger.resume(Resume::Continue);
    assert_eq!(held(&debugger), at(37), "the next scan stops again");
    assert!(time.load(Ordering::SeqCst) > stopped_at);
    debugger.let_go();
    thread.join().unwrap();
    assert_eq!(debugger.held(), None);
    let engine = shared.lock().unwrap();
    assert_eq!(
        engine.fingerprint(),
        reference.fingerprint(),
        "stops change nothing"
    );
}

#[test]
fn run_to_cursor_stops_once_and_leaves_the_breakpoints_as_they_were() {
    let mut engine = diverter();
    let debugger = engine.debugger();
    set(&debugger, MAIN, &[37]).unwrap();
    let (_shared, _time, thread) = running(engine, 100);
    assert_eq!(held(&debugger), at(37));
    debugger.run_to(MAIN, 38).unwrap();
    assert_eq!(held(&debugger), at(38));
    debugger.resume(Resume::Continue);
    assert_eq!(held(&debugger), at(37), "line 38 was a one-shot; 37 stays");
    debugger.resume(Resume::Out);
    assert!(
        held(&debugger).line >= 37,
        "step out at the program's level steps on"
    );
    debugger.let_go();
    thread.join().unwrap();
}

#[test]
fn a_line_takes_its_nearest_statement_and_a_line_past_the_end_none() {
    let mut engine = diverter();
    let debugger = engine.debugger();
    assert_eq!(
        set(&debugger, MAIN, &[34, 37, 38, 37]).unwrap(),
        [37, 38],
        "the comment line 34 takes 37"
    );
    assert_eq!(set(&debugger, MAIN, &[0, 999]).unwrap(), Vec::<u32>::new());
    assert_eq!(set(&debugger, MAIN, &[]).unwrap(), Vec::<u32>::new());
    assert_eq!(
        set(&debugger, "plc/Other.st", &[1]).unwrap_err(),
        "plc/Other.st is not a file of the program"
    );
    assert_eq!(
        debugger.run_to("plc/Other.st", 1).unwrap_err(),
        "plc/Other.st is not a file of the program"
    );
    assert_eq!(
        debugger.run_to(MAIN, 999).unwrap_err(),
        "line 999 of plc/Main.st has no statement"
    );
}

#[test]
fn letting_go_forgets_the_breakpoints() {
    let mut engine = diverter();
    let debugger = engine.debugger();
    set(&debugger, MAIN, &[37]).unwrap();
    debugger.let_go();
    debugger.run_to(MAIN, 38).unwrap();
    let (_shared, _time, thread) = running(engine, 100);
    assert_eq!(held(&debugger), at(38), "37 is gone");
    debugger.resume(Resume::Continue);
    let started = Instant::now();
    while !thread.is_finished() {
        assert!(
            debugger.held().is_none(),
            "no breakpoint is left to stop at"
        );
        assert!(started.elapsed() < Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(1));
    }
}

const BLOCK: &str = "FUNCTION_BLOCK Adder
VAR_INPUT a : INT; END_VAR
VAR_OUTPUT q : INT; END_VAR
q := a + 1;
END_FUNCTION_BLOCK
PROGRAM Main
VAR add : Adder; r : INT; END_VAR
add(a := 2);
r := add.q;
END_PROGRAM
";

/// A program calling a user's block, scanned on a thread until `debugger` lets it go.
fn calling() -> (Debugger, JoinHandle<()>) {
    let mut harness = TestHarness::from_source(BLOCK).unwrap();
    let control = harness.runtime_mut().enable_debug();
    let files = vec![(MAIN.to_string(), BLOCK.to_string())];
    let places = Debugger::whole_files(&files);
    let debugger = Debugger::new(
        control,
        &harness,
        vec![BLOCK.into()],
        places,
        Vec::new(),
        None,
    );
    let thread = std::thread::spawn(move || {
        for _ in 0..20 {
            harness.cycle();
        }
    });
    (debugger, thread)
}

#[test]
fn into_enters_a_block_out_leaves_it_and_the_stack_lists_the_innermost_first() {
    let (debugger, thread) = calling();
    set(&debugger, MAIN, &[8]).unwrap();
    assert_eq!(held(&debugger).line, 8);
    debugger.resume(Resume::Into);
    let inside = held(&debugger);
    assert_eq!(
        (inside.line, inside.frames),
        (4, vec![(MAIN.into(), 4), (MAIN.into(), 8)])
    );
    assert_eq!(
        debugger.read(&["a".into()]),
        [Some("2".into())],
        "the block's own frame"
    );
    debugger.resume(Resume::Out);
    assert_eq!(held(&debugger).line, 9, "out of the block, after its call");
    set(&debugger, MAIN, &[]).unwrap();
    debugger.run_to(MAIN, 8).unwrap();
    assert_eq!(held(&debugger).line, 8);
    debugger.resume(Resume::Over);
    assert_eq!(held(&debugger).line, 9, "over the call");
    debugger.let_go();
    thread.join().unwrap();
}

/// truST c9635b1c, ported to the pin through the fork (spec decision 1): a statement matches a breakpoint when it starts
/// inside it, so a breakpoint in an IF's body stops in the body, when the body runs.
#[test]
fn a_breakpoint_in_an_if_s_body_stops_only_when_the_body_runs() {
    let mut engine = diverter();
    let debugger = engine.debugger();
    set(&debugger, MAIN, &[40]).unwrap();
    let (_shared, time, thread) = running(engine, 8_000);
    assert_eq!(held(&debugger), at(40));
    let first = time.load(Ordering::SeqCst);
    assert!(
        first > 5_000,
        "the first carton enters after 5 s, not at the first scan ({first} ms)"
    );
    assert_eq!(debugger.read(&["Main.count".into()]), [Some("0".into())]);
    debugger.resume(Resume::Into);
    assert_eq!(held(&debugger), at(41));
    assert_eq!(debugger.read(&["Main.count".into()]), [Some("1".into())]);
    debugger.resume(Resume::Continue);
    assert_eq!(held(&debugger), at(40));
    assert!(
        time.load(Ordering::SeqCst) >= first + 1_900,
        "the next carton, 2 s later"
    );
    debugger.let_go();
    thread.join().unwrap();
}

#[test]
fn a_breakpoint_on_an_if_s_line_stops_at_the_if_once_a_scan() {
    let mut engine = diverter();
    let debugger = engine.debugger();
    set(&debugger, MAIN, &[39]).unwrap();
    let (_shared, time, thread) = running(engine, 100);
    assert_eq!(held(&debugger), at(39));
    let first = time.load(Ordering::SeqCst);
    debugger.resume(Resume::Continue);
    assert_eq!(held(&debugger), at(39), "the next scan");
    assert!(time.load(Ordering::SeqCst) > first);
    debugger.let_go();
    thread.join().unwrap();
}
