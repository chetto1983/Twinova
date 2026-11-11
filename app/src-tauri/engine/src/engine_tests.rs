use super::*;
use std::path::Path;
use twinova_project::storage::open_folder;

fn demo() -> Project {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/demo_m3");
    open_folder(&dir).unwrap().opened.project
}

const START: &str = "%IX1.0";
const STOP: &str = "%IX1.1";

fn engine() -> Engine {
    Engine::new(&demo()).unwrap().0
}

fn started(sink_blocked: bool) -> Engine {
    let mut engine = engine();
    engine.set_sink_blocked(0, sink_blocked);
    engine.press(START);
    engine
}

fn run_for(engine: &mut Engine, seconds: i64) -> Vec<Snapshot> {
    (0..seconds * 100)
        .map(|_| {
            engine.step().expect("scan");
            engine.snapshot()
        })
        .collect()
}

fn signal<'a>(s: &'a Snapshot, name: &str) -> &'a str {
    &s.signals.iter().find(|x| x.name == name).unwrap().value
}

#[test]
fn the_snapshot_lists_the_signals_the_sinks_and_the_jam() {
    let (engine, warnings) = Engine::new(&demo()).unwrap();
    assert_eq!(warnings.len(), 1, "{warnings:?}");
    let s = engine.snapshot();
    let names: Vec<(&str, &str)> = s
        .signals
        .iter()
        .map(|x| (x.name.as_str(), x.address.as_str()))
        .collect();
    assert_eq!(
        names,
        [
            ("belt_a.run", "%QX0.0"),
            ("belt_a.running", "%IX0.1"),
            ("sensor_a.detected", "%IX0.0"),
            ("sink.count", "%ID4"),
            ("Start", "%IX1.0"),
            ("Stop", "%IX1.1"),
        ]
    );
    assert_eq!(signal(&s, "belt_a.run"), "FALSE");
    assert_eq!(signal(&s, "sink.count"), "0");
    assert_eq!(
        s.sinks,
        [SinkState {
            id: "sink".into(),
            delivered: 0,
            blocked: false
        }]
    );
    assert_eq!((s.time_ms, s.jam, s.cartons.len()), (0, Some(false), 0));
}

#[test]
fn each_step_is_one_scan_and_the_motor_waits_for_start() {
    let mut engine = engine();
    let history = run_for(&mut engine, 3);
    assert_eq!(history[0].time_ms, 10);
    assert!(history.iter().all(|s| signal(s, "belt_a.run") == "FALSE"));
    assert_eq!(
        history.last().unwrap().cartons.len(),
        1,
        "the first carton arrives on a stopped line and holds the entry"
    );
}

#[test]
fn start_runs_the_line_and_stop_stops_it_on_the_next_scan() {
    let mut engine = started(false);
    let history = run_for(&mut engine, 20);
    assert_eq!(signal(&history[0], "belt_a.run"), "TRUE");
    let delivered = history.last().unwrap().sinks[0].delivered;
    assert!((6..=9).contains(&delivered), "{delivered}");
    engine.press(STOP);
    engine.step().unwrap();
    assert_eq!(signal(&engine.snapshot(), "belt_a.run"), "FALSE");
}

#[test]
fn a_blocked_sink_trips_the_jam() {
    let mut engine = started(true);
    let history = run_for(&mut engine, 10);
    let jam = history.iter().position(|s| s.jam == Some(true)).unwrap();
    assert!((670..700).contains(&jam), "jam at scan {jam}");
    assert!(history.last().unwrap().sinks[0].blocked);
}

/// The scripted inputs from scan `from` to scan `to`; the snapshots after each scan.
fn script(engine: &mut Engine, from: usize, to: usize) -> Vec<Snapshot> {
    (from..to)
        .map(|scan| {
            match scan {
                0 | 900 => engine.press(START),
                300 => engine.set_sink_blocked(0, true),
                600 => engine.press(STOP),
                800 => engine.set_sink_blocked(0, false),
                _ => {}
            }
            engine.step().unwrap();
            engine.snapshot()
        })
        .collect()
}

#[test]
fn a_restored_state_continues_exactly_like_the_uninterrupted_run() {
    let mut straight = engine();
    let uninterrupted = script(&mut straight, 0, 1_200);
    let mut first = engine();
    script(&mut first, 0, 500);
    let mut restored = Engine::restore(&demo(), &first.save()).unwrap();
    assert_eq!(restored.snapshot(), uninterrupted[499]);
    let resumed = script(&mut restored, 500, 1_200);
    assert_eq!(resumed, uninterrupted[500..]);
}

#[test]
fn catch_up_scales_and_counts_from_where_run_started() {
    let mut fast = started(false);
    fast.catch_up(0, 500, 2.0).unwrap();
    assert_eq!((fast.time_ms(), fast.lag_ms), (1_000, 0));
    let mut slow = started(false);
    slow.catch_up(0, 400, 0.5).unwrap();
    assert_eq!((slow.time_ms(), slow.lag_ms), (200, 0));
    slow.catch_up(200, 500, 2.0).unwrap();
    assert_eq!(slow.time_ms(), 1_200);
}

#[test]
fn catch_up_reports_what_it_could_not_do() {
    let mut engine = started(false);
    engine.catch_up(0, 250 + 5000, 1.0).unwrap();
    assert_eq!(engine.time_ms(), MAX_SCANS_PER_TICK as i64 * 10);
    assert_eq!(engine.lag_ms, 5250 - engine.time_ms() - 10);
}

#[test]
fn a_runtime_fault_stops_every_later_step() {
    let mut project = demo();
    project.program[0].1 = project.program[0]
        .1
        .replace("IF start THEN", "jamTimer.PT := T#1S / 0;\nIF start THEN");
    let (mut engine, _) = Engine::new(&project).unwrap();
    let first = engine.step().unwrap_err();
    assert_eq!(engine.step().unwrap_err(), first);
}

#[test]
fn the_snapshot_carries_each_chain_transfer_s_lift() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/two_belts_diverter");
    let project = open_folder(&dir).unwrap().opened.project;
    let (mut engine, _) = Engine::new(&project).unwrap();
    let lift = |e: &Engine| {
        let s = e.snapshot();
        assert_eq!(s.actuators.len(), 1);
        assert_eq!(s.actuators[0].id, "chain_transfer");
        s.actuators[0].value
    };
    assert_eq!(lift(&engine), 0.0);
    // The third carton reaches the transfer at about 11.4 s; the chains top out 150 ms later.
    let topped = (0..1_300).any(|_| {
        engine.step().unwrap();
        lift(&engine) == 1.0
    });
    assert!(
        topped,
        "the lift reached 1 while the third carton was diverted"
    );
}

/// The 4.4a line (spec 2026-11-10-diverter-line-design.md, acceptance 4): Step, Run at ×1 against the wall clock and
/// as fast as possible reach the same states.
#[test]
fn the_line_with_its_chain_transfer_runs_alike_under_step_x1_and_as_fast_as_possible() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../project/tests/fixtures/valid/two_belts_diverter");
    let project = open_folder(&dir).unwrap().opened.project;
    let fresh = || Engine::new(&project).unwrap().0;
    let mut stepped = fresh();
    for _ in 0..1_500 {
        stepped.step().unwrap();
    }
    let mut paced = fresh();
    let mut wall = 0;
    while paced.time_ms() < 15_000 {
        wall += 500;
        paced.catch_up(0, wall, 1.0).unwrap();
    }
    let mut fast = fresh();
    while fast.time_ms() < 15_000 {
        fast.catch_up(0, 1_000_000, 1.0).unwrap();
    }
    let end = stepped.snapshot();
    assert_eq!(end.time_ms, 15_000);
    assert_eq!(end.sinks[1].delivered, 1, "the third carton went to reject");
    assert_eq!(paced.snapshot(), end);
    assert_eq!(fast.time_ms(), 15_000);
    assert_eq!(
        Snapshot {
            lag_ms: 0,
            ..fast.snapshot()
        },
        end
    );
}

/// A REAL signal reads as its number, not as the bits of the double word it travels in (4.4b's drive feedback).
#[test]
fn a_real_signal_reads_as_a_number() {
    let dir =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../project/tests/fixtures/valid/curve_line");
    let mut engine = Engine::new(&open_folder(&dir).unwrap().opened.project)
        .unwrap()
        .0;
    let s = run_for(&mut engine, 1).pop().unwrap();
    assert_eq!(signal(&s, "curve_a.velocity"), "30");
    let travel: f32 = signal(&s, "curve_a.encoder").parse().unwrap();
    assert!((travel - 0.5).abs() < 0.011, "{travel}");
    assert_eq!(
        signal(&s, "sink.count"),
        "0",
        "a DINT still reads as an integer"
    );
}

/// The engine's fingerprint is its world's: equal for equal runs, different once the cartons have moved.
#[test]
fn the_fingerprint_follows_the_line() {
    let run = |seconds| {
        let mut engine = started(false);
        run_for(&mut engine, seconds);
        engine.fingerprint()
    };
    assert_eq!(run(3), run(3));
    assert_ne!(run(3), run(4));
    assert_ne!(run(3), engine().fingerprint());
}
