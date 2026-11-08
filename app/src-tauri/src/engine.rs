//! The run engine of the app: the nexosim bench (`bench.rs`) driven one scan of virtual time at a time, paced to the
//! wall clock in Run. The public API is M1's (spec 2026-11-07-tauri-shell-design.md, "The engine"); the lockstep it
//! used to compute by hand is now a same-instant message chain in the bench.

use serde::Serialize;

pub use crate::bench::SCAN_MS;
use crate::bench::{BeltState, Bench, ControllerState};
use crate::plc::Plc;
use crate::scene::Part;

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
    bench: Bench,
    controller: ControllerState,
    belt: BeltState,
    pub lag_ms: i64,
}

impl Engine {
    pub fn new(plc: Plc) -> Result<Self, String> {
        let (bench, controller, belt) = Bench::new(plc)?;
        Ok(Self {
            bench,
            controller,
            belt,
            lag_ms: 0,
        })
    }

    /// Queues a start or stop button press; it is held for the next scan only.
    pub fn press(&mut self, start: bool, stop: bool) -> Result<(), String> {
        self.bench.press(start, stop)
    }

    pub fn set_outlet_blocked(&mut self, blocked: bool) -> Result<(), String> {
        self.bench.set_outlet_blocked(blocked)?;
        self.refresh();
        Ok(())
    }

    pub fn time_ms(&self) -> i64 {
        self.bench.time_ms()
    }

    /// One scan of `SCAN_MS`. A runtime fault of the program is returned, and every later step returns it again.
    pub fn step(&mut self) -> Result<(), String> {
        self.fault()?;
        self.bench.advance_to(self.time_ms() + SCAN_MS)?;
        self.refresh();
        self.fault()
    }

    /// Runs the scans that wall-clock time `wall_ms` (since Run started at virtual `origin_ms`) asks for, capped at
    /// `MAX_SCANS_PER_TICK`, and records the remaining lag.
    pub fn catch_up(&mut self, origin_ms: i64, wall_ms: i64) -> Result<(), String> {
        let target = origin_ms + wall_ms;
        let mut done = 0;
        while self.time_ms() + SCAN_MS <= target && done < MAX_SCANS_PER_TICK {
            self.step()?;
            done += 1;
        }
        self.lag_ms = (target - self.time_ms() - SCAN_MS).max(0);
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            time_ms: self.time_ms(),
            parts: self.belt.parts.clone(),
            delivered: self.belt.delivered,
            outlet_blocked: self.belt.outlet_blocked,
            sensor: self.belt.sensor,
            motor: self.controller.motor,
            jam: self.controller.jam,
            lag_ms: self.lag_ms,
        }
    }

    fn refresh(&mut self) {
        let (controller, belt) = self.bench.latest();
        self.controller = controller.unwrap_or_else(|| self.controller.clone());
        self.belt = belt.unwrap_or_else(|| self.belt.clone());
    }

    fn fault(&self) -> Result<(), String> {
        self.controller.fault.clone().map_or(Ok(()), Err)
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
