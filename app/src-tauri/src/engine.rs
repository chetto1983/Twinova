//! The run engine of the app: a project's line (`line.rs`) driven one controller scan of virtual time at a time,
//! paced to the wall clock in Run. The API is M1's (spec 2026-11-07-tauri-shell-design.md, "The engine"), on a
//! project since 4.3 (spec 2026-11-09-component-behaviour-design.md).

use serde::Serialize;
use trust_runtime::value::Value;
use twinova_physics::Carton;
use twinova_project::Project;

use crate::line::Line;

const NS_PER_MS: i64 = 1_000_000;
/// At most this many scans per pacing tick, so a stalled UI thread cannot freeze the loop; the lag stays visible.
pub const MAX_SCANS_PER_TICK: u64 = 100;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SinkState {
    pub id: String,
    pub delivered: u64,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TransferState {
    pub id: String,
    pub lift: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConveyorState {
    pub id: String,
    pub travel: f64,
}

/// A scene signal or a panel button with its value in the controller's image, for the I/O panel.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SignalState {
    pub name: String,
    pub address: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Snapshot {
    pub time_ms: i64,
    pub cartons: Vec<Carton>,
    pub sinks: Vec<SinkState>,
    /// Each chain transfer's lift, 0 down to 1 up, for the view to raise its chains.
    pub transfers: Vec<TransferState>,
    /// Each conveyor's surface travel, m, for the view to scroll its fabric.
    pub conveyors: Vec<ConveyorState>,
    pub signals: Vec<SignalState>,
    /// M3's jam alarm, when the program declares `jam`.
    pub jam: Option<bool>,
    /// How far virtual time trails the wall clock in Run mode, in ms. Zero when paused or stepping.
    pub lag_ms: i64,
}

/// An input from outside the line, as the engine logs it for replay.
#[derive(Debug, Clone, PartialEq)]
enum Input {
    Press(String),
    SinkBlocked(usize, bool),
}

/// A state to come back to (spec 2026-11-08-run-modes-design.md, decision 3): truST cannot save its VM's state, so a
/// snapshot is the inputs that led there, replayed on a fresh engine. The line is deterministic (`line_tests.rs`).
#[derive(Debug, Clone, PartialEq)]
pub struct Saved {
    inputs: Vec<(i64, Input)>,
    time_ns: i64,
}

pub struct Engine {
    line: Line,
    /// The scene's signals and the panel's buttons, by name and address.
    signals: Vec<(String, String)>,
    /// Every external input with its virtual time in ns.
    inputs: Vec<(i64, Input)>,
    fault: Option<String>,
    pub lag_ms: i64,
}

fn text(value: &Value) -> String {
    match value {
        Value::Bool(true) => "TRUE".into(),
        Value::Bool(false) => "FALSE".into(),
        // The image is bit-string typed: a DINT at %ID reads back as a DWORD.
        Value::DWord(bits) => (*bits as i32).to_string(),
        other => format!("{other:?}"),
    }
}

impl Engine {
    /// The engine of `project` at time zero, with the warnings Build shows.
    pub fn new(project: &Project) -> Result<(Self, Vec<String>), String> {
        let (line, warnings) = Line::build(project)?;
        let scene = &project.scene;
        let signals = scene
            .instances
            .iter()
            .flat_map(|i| {
                i.signals
                    .iter()
                    .map(move |(name, address)| (format!("{}.{name}", i.id), address.clone()))
            })
            .chain(
                scene
                    .controller
                    .panel
                    .iter()
                    .map(|b| (b.name.clone(), b.address.clone())),
            )
            .collect();
        let engine = Self {
            line,
            signals,
            inputs: Vec::new(),
            fault: None,
            lag_ms: 0,
        };
        Ok((engine, warnings))
    }

    /// A fresh engine on `project` brought to the saved state: each input at its time, then up to the saved time.
    pub fn restore(project: &Project, saved: &Saved) -> Result<Self, String> {
        let (mut engine, _) = Self::new(project)?;
        for (time_ns, input) in &saved.inputs {
            engine.advance_to(*time_ns)?;
            engine.apply(input.clone());
        }
        engine.advance_to(saved.time_ns)?;
        Ok(engine)
    }

    pub fn save(&self) -> Saved {
        Saved {
            inputs: self.inputs.clone(),
            time_ns: self.line.time_ns(),
        }
    }

    /// A panel button, held for the next scan only.
    pub fn press(&mut self, address: &str) {
        self.apply(Input::Press(address.to_string()));
    }

    pub fn set_sink_blocked(&mut self, sink: usize, blocked: bool) {
        self.apply(Input::SinkBlocked(sink, blocked));
    }

    fn apply(&mut self, input: Input) {
        match &input {
            Input::Press(address) => self.line.press(address),
            Input::SinkBlocked(sink, blocked) => self.line.set_sink_blocked(*sink, *blocked),
        }
        self.inputs.push((self.line.time_ns(), input));
    }

    pub fn time_ms(&self) -> i64 {
        self.line.time_ns() / NS_PER_MS
    }

    fn advance_to(&mut self, time_ns: i64) -> Result<(), String> {
        while self.line.time_ns() < time_ns {
            if let Err(fault) = self.line.step_ms() {
                self.fault = Some(fault.clone());
                return Err(fault);
            }
        }
        Ok(())
    }

    /// One scan of the program's period. A runtime fault is returned, and every later step returns it again.
    pub fn step(&mut self) -> Result<(), String> {
        if let Some(fault) = &self.fault {
            return Err(fault.clone());
        }
        self.advance_to(self.line.time_ns() + self.line.scan_ns())
    }

    /// Runs the scans that wall-clock time `wall_ms` at `scale` (since Run started at virtual `origin_ms`) asks for,
    /// capped at `MAX_SCANS_PER_TICK`, and records the remaining lag.
    pub fn catch_up(&mut self, origin_ms: i64, wall_ms: i64, scale: f64) -> Result<(), String> {
        let target = (origin_ms as f64 * NS_PER_MS as f64
            + wall_ms as f64 * scale * NS_PER_MS as f64) as i64;
        let scan = self.line.scan_ns();
        let mut done = 0;
        while self.line.time_ns() + scan <= target && done < MAX_SCANS_PER_TICK {
            self.step()?;
            done += 1;
        }
        self.lag_ms = (target - self.line.time_ns() - scan).max(0) / NS_PER_MS;
        Ok(())
    }

    pub fn snapshot(&self) -> Snapshot {
        let world = self.line.world();
        let controller = self.line.controller();
        Snapshot {
            time_ms: self.time_ms(),
            cartons: world.cartons(),
            sinks: (0..world.sinks())
                .map(|k| SinkState {
                    id: world.sink_id(k).to_string(),
                    delivered: world.delivered(k),
                    blocked: world.sink_blocked(k),
                })
                .collect(),
            transfers: (0..world.transfers())
                .map(|k| TransferState {
                    id: world.transfer_id(k).to_string(),
                    lift: world.transfer_lift(k),
                })
                .collect(),
            conveyors: (0..world.belts())
                .map(|k| ConveyorState {
                    id: world.belt_id(k).to_string(),
                    travel: world.belt_travel(k),
                })
                .collect(),
            signals: self
                .signals
                .iter()
                .map(|(name, address)| SignalState {
                    name: name.clone(),
                    address: address.clone(),
                    value: controller.read(address).map_or_else(|e| e, |v| text(&v)),
                })
                .collect(),
            jam: match controller.variable("jam") {
                Some(Value::Bool(b)) => Some(b),
                _ => None,
            },
            lag_ms: self.lag_ms,
        }
    }
}

#[cfg(test)]
#[path = "engine_tests.rs"]
mod tests;
