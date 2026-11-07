//! Lockstep of scene and program with a fixed scan of virtual time (spec 2026-11-07-tauri-shell-design.md,
//! "The engine"): inputs, PLC time, one cycle, motor, scene.

use serde::Serialize;

use crate::plc::{Inputs, Plc};
use crate::scene::{Part, Scene};

pub const SCAN_MS: i64 = 10;
/// At most this many scans per pacing tick, so a stalled UI thread cannot freeze the loop; the lag stays visible.
pub const MAX_SCANS_PER_TICK: u64 = 100;

#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    pub time_ms: i64,
    pub parts: Vec<Part>,
    pub delivered: u64,
    pub outlet_blocked: bool,
    pub sensor: bool,
    pub motor: bool,
    pub jam: Option<bool>,
    /// How far virtual time trails the wall clock in Run mode, in ms. Zero when paused or stepping.
    pub lag_ms: i64,
}

pub struct Engine {
    scene: Scene,
    plc: Plc,
    time_ms: i64,
    motor: bool,
    pulse: Inputs,
    pub lag_ms: i64,
}

impl Engine {
    pub fn new(plc: Plc) -> Self {
        Self {
            scene: Scene::default(),
            plc,
            time_ms: 0,
            motor: false,
            pulse: Inputs::default(),
            lag_ms: 0,
        }
    }

    /// Queues a start or stop button press; it is held for the next scan only.
    pub fn press(&mut self, start: bool, stop: bool) {
        self.pulse.start |= start;
        self.pulse.stop |= stop;
    }

    pub fn set_outlet_blocked(&mut self, blocked: bool) {
        self.scene.outlet_blocked = blocked;
    }

    pub fn time_ms(&self) -> i64 {
        self.time_ms
    }

    /// One scan of `SCAN_MS`.
    pub fn step(&mut self) -> Result<(), String> {
        let inputs = Inputs {
            sensor: self.scene.photocell(),
            ..self.pulse
        };
        self.motor = self.plc.scan(inputs, SCAN_MS)?;
        self.pulse = Inputs::default();
        self.scene.step(SCAN_MS as f64 / 1000.0, self.motor);
        self.time_ms += SCAN_MS;
        Ok(())
    }

    /// Runs the scans that wall-clock time `wall_ms` (since Run started at virtual `origin_ms`) asks for, capped at
    /// `MAX_SCANS_PER_TICK`, and records the remaining lag.
    pub fn catch_up(&mut self, origin_ms: i64, wall_ms: i64) -> Result<(), String> {
        let target = origin_ms + wall_ms;
        let mut done = 0;
        while self.time_ms + SCAN_MS <= target && done < MAX_SCANS_PER_TICK {
            self.step()?;
            done += 1;
        }
        self.lag_ms = (target - self.time_ms - SCAN_MS).max(0);
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            time_ms: self.time_ms,
            parts: self.scene.parts.clone(),
            delivered: self.scene.delivered,
            outlet_blocked: self.scene.outlet_blocked,
            sensor: self.scene.photocell(),
            motor: self.motor,
            jam: self.plc.bool_var("jam"),
            lag_ms: self.lag_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_PROGRAM: &str = include_str!("default_program.st");

    fn started(outlet_blocked: bool) -> Engine {
        let mut engine = Engine::new(Plc::build(DEFAULT_PROGRAM).expect("default program builds"));
        engine.set_outlet_blocked(outlet_blocked);
        engine.press(true, false);
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
    fn the_motor_waits_for_start() {
        let mut engine = Engine::new(Plc::build(DEFAULT_PROGRAM).unwrap());
        let history = run_for(&mut engine, 5);
        assert!(history.iter().all(|s| !s.motor));
    }

    #[test]
    fn stop_stops_the_line_on_the_next_scan() {
        let mut engine = started(false);
        run_for(&mut engine, 3);
        engine.press(false, true);
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
}
