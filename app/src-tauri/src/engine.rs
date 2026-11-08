//! The run engine of the app: the nexosim bench (`bench.rs`) driven one scan of virtual time at a time, paced to the
//! wall clock in Run. The public API is M1's (spec 2026-11-07-tauri-shell-design.md, "The engine"); the lockstep it
//! used to compute by hand is now a same-instant message chain in the bench.

use serde::Serialize;

use crate::bench::{BeltState, Bench, ControllerState};
use crate::plc::{DEFAULT_SCAN_NS, Plc};
use crate::scene::Part;

const NS_PER_MS: i64 = 1_000_000;
/// The scan period of a program without a periodic `TASK`, in ms.
pub const SCAN_MS: i64 = DEFAULT_SCAN_NS / NS_PER_MS;
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

/// An input from outside the bench, as the engine logs it for replay.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Input {
    Press { start: bool, stop: bool },
    OutletBlocked(bool),
}

/// A state to come back to (spec 2026-11-08-run-modes-design.md, decision 3): truST cannot save its VM's state, so a
/// snapshot is the inputs that led there, replayed on a fresh engine. The engine is deterministic (replay test).
#[derive(Debug, Clone, PartialEq)]
pub struct Saved {
    inputs: Vec<(i64, Input)>,
    time_ns: i64,
}

pub struct Engine {
    bench: Bench,
    controller: ControllerState,
    belt: BeltState,
    /// Every external input with its virtual time in ns.
    inputs: Vec<(i64, Input)>,
    pub lag_ms: i64,
}

impl Engine {
    pub fn new(plc: Plc) -> Result<Self, String> {
        let (bench, controller, belt) = Bench::new(plc)?;
        Ok(Self {
            bench,
            controller,
            belt,
            inputs: Vec::new(),
            lag_ms: 0,
        })
    }

    /// A fresh engine on `plc` brought to the saved state: each input at its time, then up to the saved time.
    pub fn restore(plc: Plc, saved: &Saved) -> Result<Self, String> {
        let mut engine = Self::new(plc)?;
        for &(time_ns, input) in &saved.inputs {
            engine.bench.advance_to(time_ns)?;
            engine.apply(input)?;
        }
        engine.bench.advance_to(saved.time_ns)?;
        engine.refresh();
        Ok(engine)
    }

    pub fn save(&self) -> Saved {
        Saved {
            inputs: self.inputs.clone(),
            time_ns: self.bench.time_ns(),
        }
    }

    /// Queues a start or stop button press; it is held for the next scan only.
    pub fn press(&mut self, start: bool, stop: bool) -> Result<(), String> {
        self.apply(Input::Press { start, stop })
    }

    pub fn set_outlet_blocked(&mut self, blocked: bool) -> Result<(), String> {
        self.apply(Input::OutletBlocked(blocked))
    }

    fn apply(&mut self, input: Input) -> Result<(), String> {
        match input {
            Input::Press { start, stop } => self.bench.press(start, stop)?,
            Input::OutletBlocked(blocked) => self.bench.set_outlet_blocked(blocked)?,
        }
        self.inputs.push((self.bench.time_ns(), input));
        self.refresh();
        Ok(())
    }

    pub fn time_ms(&self) -> i64 {
        self.bench.time_ns() / NS_PER_MS
    }

    /// One scan of the program's period. A runtime fault of the program is returned, and every later step returns it
    /// again.
    pub fn step(&mut self) -> Result<(), String> {
        self.fault()?;
        self.bench
            .advance_to(self.bench.time_ns() + self.bench.scan_ns())?;
        self.refresh();
        self.fault()
    }

    /// Runs the scans that wall-clock time `wall_ms` at `scale` (since Run started at virtual `origin_ms`) asks for,
    /// capped at `MAX_SCANS_PER_TICK`, and records the remaining lag.
    pub fn catch_up(&mut self, origin_ms: i64, wall_ms: i64, scale: f64) -> Result<(), String> {
        let target = (origin_ms as f64 * NS_PER_MS as f64
            + wall_ms as f64 * scale * NS_PER_MS as f64) as i64;
        let scan = self.bench.scan_ns();
        let mut done = 0;
        while self.bench.time_ns() + scan <= target && done < MAX_SCANS_PER_TICK {
            self.step()?;
            done += 1;
        }
        self.lag_ms = (target - self.bench.time_ns() - scan).max(0) / NS_PER_MS;
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
