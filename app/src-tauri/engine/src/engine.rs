//! The run engine of the app: a project's line (`line.rs`) driven one controller scan of virtual time at a time,
//! paced to the wall clock in Run. The API is M1's (spec 2026-11-07-tauri-shell-design.md, "The engine"), on a
//! project since 4.3 (spec 2026-11-09-component-behaviour-design.md).

use serde::Serialize;
use trust_runtime::value::Value;
use twinova_physics::{Carton, World};
use twinova_project::Project;
use twinova_project::model::SignalType;

use crate::controller::Target;
use crate::line::Line;
use crate::watch::{self, Compiled, Scope, Watch};

const NS_PER_MS: i64 = 1_000_000;
/// At most this many scans per pacing tick, so a stalled UI thread cannot freeze the loop; the lag stays visible.
pub const MAX_SCANS_PER_TICK: u64 = 100;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SinkState {
    pub id: String,
    pub delivered: u64,
    pub blocked: bool,
}

/// Where a moving device stands: a chain transfer's or a stopper's lift and a pusher's reach, 0 to 1, or a
/// turntable's deck angle in degrees, counter-clockwise seen from above.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ActuatorState {
    pub id: String,
    pub value: f32,
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
    /// Each moving device's position, for the view to pose it.
    pub actuators: Vec<ActuatorState>,
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
    /// An online change of the line program, its files as (path, text).
    Program(Vec<(String, String)>),
    /// The user's write, force and release (spec 2026-11-13-online-monitoring-design.md, decision 3).
    Write(Target, Value),
    Force(Target, Value),
    Release(Target),
}

/// A state to come back to (spec 2026-11-08-run-modes-design.md, decision 3): truST cannot save its VM's state, so a
/// snapshot is the inputs that led there, replayed on a fresh engine. The line is deterministic (`line_tests.rs`). It
/// starts from the program the engine was built with, whatever online changes came after (spec
/// 2026-11-12-st-vscode-design.md, decision 9).
#[derive(Debug, Clone, PartialEq)]
pub struct Saved {
    program: Vec<(String, String)>,
    inputs: Vec<(i64, Input)>,
    time_ns: i64,
}

pub struct Engine {
    line: Line,
    /// The project the engine runs, its program the current one: the scene an online change is checked against.
    project: Project,
    /// The program the engine was built with, where a snapshot's replay starts.
    built_program: Vec<(String, String)>,
    /// The scene's signals and the panel's buttons, by name, address and type.
    signals: Vec<(String, String, SignalType)>,
    /// Every external input with its virtual time in ns.
    inputs: Vec<(i64, Input)>,
    fault: Option<String>,
    pub lag_ms: i64,
    /// What the editor monitors, ready to read.
    watches: Vec<Compiled>,
}

fn text(value: &Value, ty: SignalType) -> String {
    match (value, ty) {
        (Value::Bool(true), _) => "TRUE".into(),
        (Value::Bool(false), _) => "FALSE".into(),
        // The image is bit-string typed: a DINT or a REAL at %ID reads back as a DWORD of its bits.
        (Value::DWord(bits), SignalType::REAL) => f32::from_bits(*bits).to_string(),
        (Value::DWord(bits), _) => (*bits as i32).to_string(),
        (other, _) => format!("{other:?}"),
    }
}

/// Every moving device's position, by kind: chain transfers, stoppers, pushers, turntables.
fn actuators(world: &World) -> Vec<ActuatorState> {
    let at = |id: &str, value: f32| ActuatorState {
        id: id.to_string(),
        value,
    };
    let transfers =
        (0..world.transfers()).map(|k| at(world.transfer_id(k), world.transfer_lift(k)));
    let stoppers = (0..world.stoppers()).map(|k| at(world.stopper_id(k), world.stopper_lift(k)));
    let pushers = (0..world.pushers()).map(|k| at(world.pusher_id(k), world.pusher_reach(k)));
    let turntables = (0..world.turntables())
        .map(|k| at(world.turntable_id(k), world.turntable_angle(k).to_degrees()));
    transfers
        .chain(stoppers)
        .chain(pushers)
        .chain(turntables)
        .collect()
}

impl Engine {
    pub fn scan_timing(&self) -> crate::timing::ScanTiming {
        self.line.controller().timing()
    }

    /// The engine of `project` at time zero, with the warnings Build shows.
    pub fn new(project: &Project) -> Result<(Self, Vec<String>), String> {
        let (line, warnings) = Line::build(project)?;
        let scene = &project.scene;
        let signals = scene
            .instances
            .iter()
            .flat_map(|i| {
                let def = &project.definitions[&i.definition];
                i.signals.iter().map(move |(name, address)| {
                    let ty = def.signals.iter().find(|s| &s.name == name);
                    let ty = ty
                        .expect("the format checks a bound signal is the definition's")
                        .ty;
                    (format!("{}.{name}", i.id), address.clone(), ty)
                })
            })
            .chain(
                scene
                    .controller
                    .panel
                    .iter()
                    .map(|b| (b.name.clone(), b.address.clone(), SignalType::BOOL)),
            )
            .collect();
        let engine = Self {
            line,
            project: project.clone(),
            built_program: project.program.clone(),
            signals,
            inputs: Vec::new(),
            fault: None,
            lag_ms: 0,
            watches: Vec::new(),
        };
        Ok((engine, warnings))
    }

    /// A fresh engine on `project` brought to the saved state: each input at its time, then up to the saved time.
    pub fn restore(project: &Project, saved: &Saved) -> Result<Self, String> {
        let built = Project {
            program: saved.program.clone(),
            ..project.clone()
        };
        let (mut engine, _) = Self::new(&built)?;
        for (time_ns, input) in &saved.inputs {
            engine.advance_to(*time_ns)?;
            engine.apply(input.clone())?;
        }
        engine.advance_to(saved.time_ns)?;
        Ok(engine)
    }

    pub fn save(&self) -> Saved {
        Saved {
            program: self.built_program.clone(),
            inputs: self.inputs.clone(),
            time_ns: self.line.time_ns(),
        }
    }

    /// A panel button, held for the next scan only.
    pub fn press(&mut self, address: &str) {
        let applied = self.apply(Input::Press(address.to_string()));
        applied.expect("a press always applies");
    }

    pub fn set_sink_blocked(&mut self, sink: usize, blocked: bool) {
        let applied = self.apply(Input::SinkBlocked(sink, blocked));
        applied.expect("a sink's block always applies");
    }

    /// An online change of the line program (spec 2026-11-12-st-vscode-design.md, decision 1): the PLC takes `program`
    /// now, between two milliseconds, while the plant goes on. A fault stays until Reset or Build. On an error nothing
    /// changes. Returns Build's warnings for the new program.
    pub fn change_program(
        &mut self,
        program: Vec<(String, String)>,
    ) -> Result<Vec<String>, String> {
        self.apply(Input::Program(program))
    }

    /// Writes `text` into a program variable once, at the next scan.
    pub fn write(&mut self, target: Target, text: &str) -> Result<(), String> {
        let value = self.line.controller_mut().value_for(&target, text)?;
        self.apply(Input::Write(target, value)).map(drop)
    }

    /// Holds a variable or an address at `text`'s value until released.
    pub fn force(&mut self, target: Target, text: &str) -> Result<(), String> {
        let value = self.line.controller_mut().value_for(&target, text)?;
        self.apply(Input::Force(target, value)).map(drop)
    }

    pub fn release(&mut self, target: Target) -> Result<(), String> {
        self.apply(Input::Release(target)).map(drop)
    }

    /// The forces in effect, with their values as the editor shows them.
    pub fn forces(&self) -> Vec<(Target, String)> {
        self.line.controller().forces()
    }

    /// Whether the PLC's debug control is attached (spec 2026-11-13-online-monitoring-design.md, decision 2).
    pub fn debugged(&self) -> bool {
        self.line.controller().debugged()
    }

    /// What the editor monitors from now on: each path kept, each condition parsed once. A watch on the field of a
    /// project without blocks reads nothing.
    pub fn watch(&mut self, watches: &[Watch]) {
        let line = &mut self.line;
        self.watches = (watches.iter())
            .map(|w| match w {
                Watch::Path(Scope::Plc, _) | Watch::Condition(Scope::Plc, _) => {
                    watch::compile(line.controller_mut().harness_mut(), w)
                }
                Watch::Path(Scope::Field, _) | Watch::Condition(Scope::Field, _) => (line
                    .field_mut())
                .map_or(Compiled::Nothing, |f| watch::compile(f.harness_mut(), w)),
            })
            .collect();
    }

    /// Whether the editor monitors anything.
    pub fn watching(&self) -> bool {
        !self.watches.is_empty()
    }

    /// The monitored values after the last scan, in the order they were watched.
    pub fn watched(&mut self) -> Vec<Option<String>> {
        let line = &mut self.line;
        (self.watches.iter())
            .map(|c| match c {
                Compiled::Path(Scope::Plc, _) | Compiled::Expr(Scope::Plc, _) => {
                    watch::read(line.controller_mut().harness_mut(), c)
                }
                Compiled::Path(Scope::Field, _) | Compiled::Expr(Scope::Field, _) => line
                    .field_mut()
                    .and_then(|f| watch::read(f.harness_mut(), c)),
                Compiled::Nothing => None,
            })
            .collect()
    }

    /// The program the engine runs now, with its online changes.
    pub fn program(&self) -> &[(String, String)] {
        &self.project.program
    }

    /// Applies an input and logs it for replay; one that fails is not logged.
    fn apply(&mut self, input: Input) -> Result<Vec<String>, String> {
        let warnings = match &input {
            Input::Press(address) => {
                self.line.press(address);
                Vec::new()
            }
            Input::SinkBlocked(sink, blocked) => {
                self.line.set_sink_blocked(*sink, *blocked);
                Vec::new()
            }
            Input::Program(program) => {
                let changed = Project {
                    program: program.clone(),
                    ..self.project.clone()
                };
                let warnings = self.line.change_program(&changed)?;
                self.project = changed;
                warnings
            }
            Input::Write(target, value) => {
                (self.line.controller_mut()).write_once(target, value.clone())?;
                Vec::new()
            }
            Input::Force(target, value) => {
                (self.line.controller_mut()).force(target, value.clone())?;
                Vec::new()
            }
            Input::Release(target) => {
                self.line.controller_mut().release(target)?;
                Vec::new()
            }
        };
        self.inputs.push((self.line.time_ns(), input));
        Ok(warnings)
    }

    /// The physics world's fingerprint: equal for equal runs, on every platform.
    pub fn fingerprint(&self) -> u64 {
        self.line.world().fingerprint()
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
            actuators: actuators(world),
            conveyors: (0..world.belts())
                .map(|k| ConveyorState {
                    id: world.belt_id(k).to_string(),
                    travel: world.belt_travel(k),
                })
                .collect(),
            signals: self
                .signals
                .iter()
                .map(|(name, address, ty)| SignalState {
                    name: name.clone(),
                    address: address.clone(),
                    value: controller
                        .read(address)
                        .map_or_else(|e| e, |v| text(&v, *ty)),
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

#[cfg(test)]
#[path = "engine_change_tests.rs"]
mod change_tests;

#[cfg(test)]
#[path = "engine_monitor_tests.rs"]
mod monitor_tests;
