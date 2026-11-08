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

/// M1's results over 60 s, pinned before the engine moves onto nexosim (plan 2026-11-08-des-core-integration,
/// task 2): any engine must reproduce every scan of both scenarios exactly.
#[test]
fn every_scan_of_the_m1_scenarios_is_reproduced() {
    assert_eq!(
        fingerprint(&run_for(&mut started(false), 60)),
        17_403_443_809_136_332_006,
        "outlet free"
    );
    assert_eq!(
        fingerprint(&run_for(&mut started(true), 60)),
        9_118_810_849_520_154_159,
        "outlet blocked"
    );
}

/// The same inputs at the same scans give the same states: Start, a blocked outlet, Stop, a freed outlet, Start.
fn scripted_run() -> u64 {
    let mut engine = Engine::new(Plc::build(DEFAULT_PROGRAM).unwrap()).unwrap();
    let mut history = Vec::new();
    for scan in 0..3_000 {
        match scan {
            0 | 2_000 => engine.press(true, false).unwrap(),
            700 => engine.set_outlet_blocked(true).unwrap(),
            1_500 => engine.press(false, true).unwrap(),
            1_900 => engine.set_outlet_blocked(false).unwrap(),
            _ => {}
        }
        engine.step().unwrap();
        history.push(engine.snapshot());
    }
    fingerprint(&history)
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
    engine.catch_up(0, 250).unwrap();
    assert_eq!(engine.time_ms(), 250);
    assert_eq!(engine.lag_ms, 0);
    engine.catch_up(0, 250 + 5000).unwrap();
    assert_eq!(engine.time_ms(), 250 + MAX_SCANS_PER_TICK as i64 * SCAN_MS);
    assert_eq!(engine.lag_ms, 5250 - engine.time_ms() - SCAN_MS);
}
