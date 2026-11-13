//! The rest of ROADMAP 4.9 (spec 2026-11-13-breakpoints-complete-design.md): conditions, hit counts, logpoints,
//! breakpoints in blocks, and writing while held, on the 4.4a line.
use super::*;
use crate::debugger_kinds::{Piece, message, pieces};
use crate::engine::Engine;
use std::path::Path;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use trust_runtime::debug::HitCondition;
use twinova_project::storage::open_folder;

const MAIN: &str = "plc/Main.st";
const TRANSFER: &str = "components/twinova.chain_transfer@1.st";

fn project() -> twinova_project::model::Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/two_belts_diverter");
    open_folder(&dir).unwrap().opened.project
}

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

fn at(line: u32) -> Breakpoint {
    Breakpoint {
        line,
        condition: None,
        hit: None,
        log: None,
    }
}

#[test]
fn a_condition_stops_only_when_true_with_names_of_the_stopped_frame() {
    let mut engine = Engine::new(&project()).unwrap().0;
    let debugger = engine.debugger();
    let when = Breakpoint {
        condition: Some("count = 2".into()),
        ..at(41)
    };
    assert_eq!(
        debugger.set(MAIN, std::slice::from_ref(&when)).unwrap(),
        [when]
    );
    let (_engine, time, thread) = running(engine, 9_000);
    assert_eq!(held(&debugger).line, 41);
    assert!(
        time.load(Ordering::SeqCst) > 7_000,
        "the second carton, not the first"
    );
    assert_eq!(debugger.read(&["Main.count".into()]), [Some("2".into())]);
    debugger.let_go();
    thread.join().unwrap();
}

#[test]
fn a_hit_count_stops_at_its_hit() {
    let mut engine = Engine::new(&project()).unwrap().0;
    let debugger = engine.debugger();
    let third = Breakpoint {
        hit: Some(Hit {
            op: HitOp::Equal,
            count: 3,
        }),
        ..at(40)
    };
    debugger.set(MAIN, &[third]).unwrap();
    let (_engine, time, thread) = running(engine, 12_000);
    assert_eq!(held(&debugger).line, 40);
    assert!(time.load(Ordering::SeqCst) > 9_000, "the third carton");
    debugger.let_go();
    thread.join().unwrap();
}

#[test]
fn each_hit_operator_is_trust_s() {
    let hit = |op, count| Hit { op, count }.condition();
    assert_eq!(hit(HitOp::Equal, 3), HitCondition::Equal(3));
    assert_eq!(hit(HitOp::AtLeast, 2), HitCondition::AtLeast(2));
    assert_eq!(hit(HitOp::GreaterThan, 1), HitCondition::GreaterThan(1));
}

#[test]
fn a_logpoint_logs_without_stopping_a_number_as_text_and_a_time_as_trust_shows_it() {
    let mut engine = Engine::new(&project()).unwrap().0;
    let debugger = engine.debugger();
    let log = Breakpoint {
        log: Some("count={count}, PT={carry.PT}".into()),
        ..at(41)
    };
    debugger.set(MAIN, &[log]).unwrap();
    while engine.time_ms() < 8_000 {
        engine.step().unwrap();
    }
    assert_eq!(debugger.held(), None, "a logpoint never stops");
    let logs = debugger.logs();
    let messages: Vec<&str> = logs.iter().map(|l| l.message.as_str()).collect();
    let pt = "Time(Duration { nanos: 1500000000 })";
    assert_eq!(
        messages,
        [format!("count=1, PT={pt}"), format!("count=2, PT={pt}")],
        "two cartons by 8 s"
    );
    assert_eq!((logs[0].file.as_str(), logs[0].line), (MAIN, 41));
    assert!(debugger.logs().is_empty(), "drained");
}

#[test]
fn a_logged_line_keeps_each_expression_s_text_or_its_bare_value() {
    assert_eq!(message("a=\u{2}1\u{3}DInt(1)\u{4}!"), "a=1!");
    assert_eq!(
        message("t=\u{2}<error: type mismatch>\u{3}Time(x)\u{4}"),
        "t=Time(x)"
    );
    assert_eq!(message("plain"), "plain");
    assert_eq!(message("\u{2}x\u{3}y\u{4}\u{2}z\u{3}w\u{4}"), "xz");
}

#[test]
fn a_message_splits_into_text_and_expressions() {
    assert_eq!(
        pieces("a={x}, b={ y + 1 }!").unwrap(),
        [
            Piece::Text("a="),
            Piece::Expr("x"),
            Piece::Text(", b="),
            Piece::Expr("y + 1"),
            Piece::Text("!")
        ]
    );
    assert_eq!(pieces("plain").unwrap(), [Piece::Text("plain")]);
    assert_eq!(pieces("a{x").unwrap_err(), "'{' without '}' in the message");
    assert_eq!(pieces("a}x").unwrap_err(), "'}' without '{' in the message");
    assert_eq!(pieces("a{}").unwrap_err(), "an empty {} in the message");
}

#[test]
fn a_condition_or_a_message_that_does_not_parse_is_refused() {
    let mut engine = Engine::new(&project()).unwrap().0;
    let debugger = engine.debugger();
    let bad = Breakpoint {
        condition: Some("count = = 2".into()),
        ..at(41)
    };
    assert!(
        debugger
            .set(MAIN, &[bad])
            .unwrap_err()
            .starts_with("line 41: the condition 'count = = 2' ")
    );
    let bad = Breakpoint {
        log: Some("x={count +}".into()),
        ..at(41)
    };
    assert!(
        debugger
            .set(MAIN, &[bad])
            .unwrap_err()
            .starts_with("line 41: the expression 'count +' ")
    );
}

#[test]
fn a_block_s_breakpoint_holds_the_line_and_names_its_device() {
    let mut engine = Engine::new(&project()).unwrap().0;
    let field = engine.field_debugger().expect("the 4.4a line has blocks");
    assert_eq!(field.set(TRANSFER, &[at(20)]).unwrap(), [at(20)]);
    assert_eq!(
        field.set(MAIN, &[at(37)]).unwrap_err(),
        "plc/Main.st is not a file of the field"
    );
    let (shared, _time, thread) = running(engine, 100);
    let hold = held(&field);
    assert_eq!(
        (hold.file.as_str(), hold.line, hold.device.as_deref()),
        (TRANSFER, 20, Some("chain_transfer"))
    );
    assert_eq!(hold.frames.first(), Some(&(TRANSFER.to_string(), 20)));
    assert!(
        shared.try_lock().is_err(),
        "HOLD: the engine is in its step"
    );
    assert_eq!(
        field
            .write("Field.chain_transfer.raised", "TRUE")
            .unwrap_err(),
        "a block is read-only"
    );
    field.let_go();
    thread.join().unwrap();
}

#[test]
fn a_write_while_held_lands_at_the_next_cycle_and_replays_to_the_same_trajectory() {
    let project = project();
    let mut engine = Engine::new(&project).unwrap().0;
    let debugger = engine.debugger();
    assert_eq!(
        debugger.write("Main.count", "100").unwrap_err(),
        "the PLC is not held"
    );
    debugger.set(MAIN, &[at(40)]).unwrap();
    let (shared, _time, thread) = running(engine, 9_000);
    assert_eq!(held(&debugger).line, 40);
    assert_eq!(
        debugger.write("Main.carry", "1").unwrap_err(),
        "Main.carry has no single value to write"
    );
    assert_eq!(
        debugger.write("Main.nothing", "1").unwrap_err(),
        "Main.nothing is not a variable of the PLC"
    );
    assert_eq!(
        debugger.write("Main.count", "TRUE").unwrap_err(),
        "'TRUE' is not DINT"
    );
    debugger.write("Main.count", "100").unwrap();
    assert_eq!(
        debugger.read(&["Main.count".into()]),
        [Some("100".into())],
        "shown at once"
    );
    debugger.resume(Resume::Into);
    assert_eq!(held(&debugger).line, 41);
    assert_eq!(
        debugger.read(&["Main.count".into()]),
        [Some("1".into())],
        "the held scan does not see it"
    );
    debugger.let_go();
    thread.join().unwrap();
    let mut engine = shared.lock().unwrap();
    engine.watch(&[crate::watch::Watch::Path(
        crate::watch::Scope::Plc,
        "Main.count".into(),
    )]);
    assert_eq!(engine.watched(), [Some("101".into())], "100, then a carton");
    let replayed = Engine::restore(&project, &engine.save()).unwrap();
    assert_eq!(
        replayed.fingerprint(),
        engine.fingerprint(),
        "the write replays one step after the held one"
    );
}
