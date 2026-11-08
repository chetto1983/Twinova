use super::*;
use crate::scene::{BELT_SPEED_M_S, PART_LENGTH_M};

const DEFAULT_PROGRAM: &str = include_str!("default_program.st");

fn started(outlet_blocked: bool) -> Engine {
    let mut engine = Engine::new(Plc::build(DEFAULT_PROGRAM).expect("default program builds"))
        .expect("bench builds");
    engine.set_outlet_blocked(outlet_blocked).unwrap();
    engine.press(true, false).unwrap();
    engine
}

fn run_for(engine: &mut Engine, seconds: i64) -> Vec<Snapshot> {
    (0..seconds * 1000 / SCAN_MS)
        .map(|_| {
            engine.step().expect("scan");
            engine.snapshot()
        })
        .collect()
}

/// FNV-1a over an explicit byte layout of every scan, so the value is stable across Rust versions (unlike
/// `DefaultHasher`).
fn fingerprint(history: &[Snapshot]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |bytes: &[u8]| {
        for byte in bytes {
            hash = (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for s in history {
        feed(&s.time_ms.to_le_bytes());
        feed(&[
            u8::from(s.motor),
            u8::from(s.sensor),
            s.jam.map_or(2, u8::from),
        ]);
        feed(&s.delivered.to_le_bytes());
        for part in &s.parts {
            feed(&part.id.to_le_bytes());
            feed(&part.front_m.to_bits().to_le_bytes());
        }
    }
    hash
}

/// Every scan of both 60 s scenarios, pinned on the closed-form belt (spec 2026-11-08-analytic-belt-design.md). They
/// were first pinned on M1's stepped belt; the analytic belt moved M1's edges 10 ms earlier, which M1's floating-point
/// sum had delayed, and changed the last bits of the positions. Any engine change must reproduce these exactly.
#[test]
fn every_scan_of_the_reference_scenarios_is_reproduced() {
    assert_eq!(
        fingerprint(&run_for(&mut started(false), 60)),
        12_522_855_918_933_968_443,
        "outlet free"
    );
    assert_eq!(
        fingerprint(&run_for(&mut started(true), 60)),
        2_752_821_455_799_209_310,
        "outlet blocked"
    );
}

/// The events at their closed-form times, an independent check of the pinned scans: the first part arrives at 10 ms
/// with its front at 0.30 m, so at 0.5 m/s it reaches the photocell (2.70 m) at 4.81 s and leaves (3.00 m) at
/// 5.41 s; the jam trips 2 s after the photocell is covered.
#[test]
fn the_events_fall_at_their_closed_form_times() {
    let first =
        |h: &[Snapshot], f: &dyn Fn(&Snapshot) -> bool| h.iter().find(|s| f(s)).map(|s| s.time_ms);
    let free = run_for(&mut started(false), 60);
    assert_eq!(first(&free, &|s| s.sensor), Some(4_810));
    assert_eq!(first(&free, &|s| s.delivered == 1), Some(5_410));
    assert_eq!(free.last().unwrap().delivered, 28);
    let blocked = run_for(&mut started(true), 60);
    assert_eq!(first(&blocked, &|s| s.sensor), Some(4_810));
    assert_eq!(first(&blocked, &|s| s.jam == Some(true)), Some(6_810));
    assert_eq!(first(&blocked, &|s| !s.motor), Some(6_810));
}

/// The same inputs at the same scans give the same states: Start, a blocked outlet, Stop, a freed outlet, Start.
fn scripted_run() -> u64 {
    fingerprint(&script(
        &mut Engine::new(Plc::build(DEFAULT_PROGRAM).unwrap()).unwrap(),
        0,
        3_000,
    ))
}

#[test]
fn a_replay_of_the_same_inputs_reaches_the_same_states() {
    assert_eq!(scripted_run(), scripted_run());
}

#[test]
fn with_the_outlet_free_the_line_runs_and_delivers() {
    let mut engine = started(false);
    let history = run_for(&mut engine, 60);
    let last = history.last().unwrap();
    assert!(last.motor, "motor still on");
    assert_eq!(last.jam, Some(false));
    assert!(last.delivered >= 25, "delivered {}", last.delivered);
}

#[test]
fn with_the_outlet_blocked_a_covered_photocell_trips_the_jam_within_two_seconds() {
    let mut engine = started(true);
    let history = run_for(&mut engine, 60);
    let covered_from = history
        .windows(2)
        .rposition(|w| !w[0].sensor && w[1].sensor)
        .map_or(history[0].time_ms, |i| history[i + 1].time_ms);
    let jam_at = history
        .iter()
        .find(|s| s.jam == Some(true))
        .expect("a jam")
        .time_ms;
    let delay = jam_at - covered_from;
    assert!(
        (2000..=2000 + 2 * SCAN_MS).contains(&delay),
        "jam {delay} ms after the photocell was covered"
    );
    let last = history.last().unwrap();
    assert!(!last.motor && last.sensor && last.delivered == 0);
}

#[test]
fn each_scan_moves_the_belt_by_one_scan_of_virtual_time() {
    let mut engine = started(false);
    run_for(&mut engine, 1);
    // The first scan latches the motor and the part enters after it moves: 99 scans of movement.
    let travelled = BELT_SPEED_M_S * 99.0 * SCAN_MS as f64 / 1000.0;
    let front = engine.snapshot().parts[0].front_m;
    assert!(
        (front - (PART_LENGTH_M + travelled)).abs() < 1e-9,
        "front at {front} m"
    );
}

#[test]
fn a_program_with_a_periodic_task_is_scanned_at_its_interval() {
    let source = "PROGRAM Main\nVAR sensor : BOOL; start : BOOL; stop : BOOL; motor : BOOL; END_VAR\n\
        motor := NOT motor;\nEND_PROGRAM\nCONFIGURATION Cell\nRESOURCE CPU ON PLC\n\
        TASK Fast(INTERVAL := T#5ms, PRIORITY := 1);\nPROGRAM P WITH Fast : Main;\nEND_RESOURCE\nEND_CONFIGURATION\n";
    let mut engine = Engine::new(Plc::build(source).unwrap()).unwrap();
    let scans: Vec<(i64, bool)> = (0..4)
        .map(|_| {
            engine.step().unwrap();
            let s = engine.snapshot();
            (s.time_ms, s.motor)
        })
        .collect();
    assert_eq!(scans, [(5, true), (10, false), (15, true), (20, false)]);
}

/// The scripted inputs of `scripted_run`, from scan `from` to scan `to`, on `engine`; the snapshots after each scan.
fn script(engine: &mut Engine, from: usize, to: usize) -> Vec<Snapshot> {
    (from..to)
        .map(|scan| {
            match scan {
                0 | 2_000 => engine.press(true, false).unwrap(),
                700 => engine.set_outlet_blocked(true).unwrap(),
                1_500 => engine.press(false, true).unwrap(),
                1_900 => engine.set_outlet_blocked(false).unwrap(),
                _ => {}
            }
            engine.step().unwrap();
            engine.snapshot()
        })
        .collect()
}

#[test]
fn a_restored_state_continues_exactly_like_the_uninterrupted_run() {
    let program = || Plc::build(DEFAULT_PROGRAM).unwrap();
    let mut straight = Engine::new(program()).unwrap();
    let uninterrupted = script(&mut straight, 0, 3_000);
    // Saved after inputs at scans 0 and 700, before the ones at 1500, 1900 and 2000.
    let mut first = Engine::new(program()).unwrap();
    script(&mut first, 0, 1_200);
    let mut restored = Engine::restore(program(), &first.save()).unwrap();
    assert_eq!(
        fingerprint(&[restored.snapshot()]),
        fingerprint(&uninterrupted[1_199..1_200])
    );
    let resumed = script(&mut restored, 1_200, 3_000);
    assert_eq!(fingerprint(&resumed), fingerprint(&uninterrupted[1_200..]));
}

#[test]
fn catch_up_scales_the_wall_clock() {
    let mut fast = started(false);
    fast.catch_up(0, 500, 2.0).unwrap();
    assert_eq!((fast.time_ms(), fast.lag_ms), (1_000, 0));
    let mut slow = started(false);
    slow.catch_up(0, 400, 0.5).unwrap();
    assert_eq!((slow.time_ms(), slow.lag_ms), (200, 0));
}

#[test]
fn the_motor_waits_for_start() {
    let mut engine = Engine::new(Plc::build(DEFAULT_PROGRAM).unwrap()).unwrap();
    let history = run_for(&mut engine, 5);
    assert!(history.iter().all(|s| !s.motor));
}

#[test]
fn stop_stops_the_line_on_the_next_scan() {
    let mut engine = started(false);
    run_for(&mut engine, 3);
    engine.press(false, true).unwrap();
    engine.step().unwrap();
    assert!(!engine.snapshot().motor);
}

#[test]
fn catch_up_follows_the_wall_clock_and_reports_what_it_could_not_do() {
    let mut engine = started(false);
    engine.catch_up(0, 250, 1.0).unwrap();
    assert_eq!(engine.time_ms(), 250);
    assert_eq!(engine.lag_ms, 0);
    engine.catch_up(0, 250 + 5000, 1.0).unwrap();
    assert_eq!(engine.time_ms(), 250 + MAX_SCANS_PER_TICK as i64 * SCAN_MS);
    assert_eq!(engine.lag_ms, 5250 - engine.time_ms() - SCAN_MS);
}
