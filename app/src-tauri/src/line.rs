//! A project's line, run in lockstep on virtual time (spec 2026-11-09-component-behaviour-design.md, "Transport with
//! physics"; research 16: the fixed-slice lockstep of SIMIT's bus-synchronous mode). Every millisecond:
//! 1. the physics step (`twinova_physics::World`);
//! 2. the native models write the controller's inputs: diffuse sensors, sink counters, the feedback of a belt
//!    without a behaviour;
//! 3. the field cycle, which reads the controller's outputs and writes its inputs, the belts' motors and the chain
//!    transfers' lift (spec 2026-11-10-diverter-line-design.md);
//! 4. the controller's scan, when its period is due.

use trust_runtime::value::Value;
use twinova_physics::{STEP_NS, World, layout};
use twinova_project::Project;

use crate::controller::Controller;
use crate::field::Field;

/// A belt's signals, and whether its motor comes from its behaviour or straight from the controller.
struct BeltIo {
    instance: String,
    run: Option<String>,
    running: Option<String>,
    behaviour: bool,
}

pub struct Line {
    world: World,
    field: Option<Field>,
    controller: Controller,
    belts: Vec<BeltIo>,
    /// Per layout chain transfer, its instance in the field.
    transfers: Vec<String>,
    /// Per layout sensor, the controller input its `detected` is bound to.
    sensors: Vec<Option<String>>,
    /// Per layout sink, the controller input its `count` is bound to.
    sinks: Vec<Option<String>>,
    t_ns: i64,
    next_scan_ns: i64,
}

fn signal(project: &Project, instance: &str, name: &str) -> Option<String> {
    project
        .scene
        .instances
        .iter()
        .find(|i| i.id == instance)
        .and_then(|i| i.signals.get(name).cloned())
}

fn is_true(value: &Value) -> bool {
    matches!(value, Value::Bool(true))
}

fn has_behaviour(project: &Project, instance: &str) -> bool {
    project
        .scene
        .instances
        .iter()
        .find(|i| i.id == instance)
        .is_some_and(|i| project.definitions[&i.definition].behaviour.is_some())
}

impl Line {
    /// Builds the world, the field and the controller of `project`, with the warnings Build shows.
    pub fn build(project: &Project) -> Result<(Self, Vec<String>), String> {
        let layout = layout(project)?;
        let field = Field::build(project)?;
        let (controller, warnings) = Controller::build(project)?;
        let belts = layout
            .belts
            .iter()
            .map(|b| BeltIo {
                instance: b.id.clone(),
                run: signal(project, &b.id, "run"),
                running: signal(project, &b.id, "running"),
                behaviour: has_behaviour(project, &b.id),
            })
            .collect();
        // Its change-over is its block's, so a chain transfer has no native fallback (spec, "The families").
        let transfers: Vec<String> = layout.transfers.iter().map(|t| t.id.clone()).collect();
        if let Some(t) = transfers.iter().find(|t| !has_behaviour(project, t)) {
            return Err(format!("{t}: twinova.chain_transfer needs its behaviour"));
        }
        let sensors = layout
            .sensors
            .iter()
            .map(|s| signal(project, &s.id, "detected"))
            .collect();
        let sinks = layout
            .sinks
            .iter()
            .map(|s| signal(project, &s.id, "count"))
            .collect();
        let next_scan_ns = controller.scan_ns();
        let line = Self {
            world: World::new(layout),
            field,
            controller,
            belts,
            transfers,
            sensors,
            sinks,
            t_ns: 0,
            next_scan_ns,
        };
        Ok((line, warnings))
    }

    pub fn time_ns(&self) -> i64 {
        self.t_ns
    }

    pub fn scan_ns(&self) -> i64 {
        self.controller.scan_ns()
    }

    pub fn controller(&self) -> &Controller {
        &self.controller
    }

    pub fn world(&self) -> &World {
        &self.world
    }

    /// A panel button: its input bit is true for the next scan.
    pub fn press(&mut self, address: &str) {
        self.controller.press(address);
    }

    pub fn set_sink_blocked(&mut self, sink: usize, blocked: bool) {
        self.world.set_sink_blocked(sink, blocked);
    }

    /// Writes an input the program declares; one it does not is skipped, as Build warned.
    fn input(&mut self, address: &Option<String>, value: Value) -> Result<(), String> {
        match address {
            Some(a) if self.controller.declares(a) => self.controller.write(a, value),
            _ => Ok(()),
        }
    }

    fn output(&self, address: &Option<String>) -> Result<bool, String> {
        match address {
            Some(a) => self.controller.read(a).map(|v| is_true(&v)),
            None => Ok(false),
        }
    }

    /// One millisecond of the line, in the order of the module comment. A fault of either runtime is returned.
    pub fn step_ms(&mut self) -> Result<(), String> {
        self.world.step();
        self.t_ns += STEP_NS;
        for (k, address) in self.sensors.clone().iter().enumerate() {
            let seen = self.world.sensor(k);
            self.input(address, Value::Bool(seen))?;
        }
        for (k, address) in self.sinks.clone().iter().enumerate() {
            // The image is bit-string typed: a DINT at %ID travels as a DWORD (measured 2026-11-08).
            let count = u32::try_from(self.world.delivered(k)).unwrap_or(u32::MAX);
            self.input(address, Value::DWord(count))?;
        }
        if let Some(field) = self.field.as_mut() {
            for address in field.commands().to_vec() {
                field.set_command(&address, self.controller.read(&address)?)?;
            }
            field.cycle()?;
            for address in field.feedback().to_vec() {
                let value = field.feedback_value(&address)?;
                if self.controller.declares(&address) {
                    self.controller.write(&address, value)?;
                }
            }
        }
        for k in 0..self.belts.len() {
            let belt = &self.belts[k];
            let motor = match (&self.field, belt.behaviour) {
                (Some(field), true) => field
                    .native(&belt.instance, "motor")
                    .is_some_and(|v| is_true(&v)),
                _ => self.output(&belt.run)?,
            };
            if !belt.behaviour {
                let running = belt.running.clone();
                self.input(&running, Value::Bool(motor))?;
            }
            self.world.set_belt_running(k, motor);
        }
        if let Some(field) = &self.field {
            for (k, instance) in self.transfers.iter().enumerate() {
                let Some(Value::Real(lift)) = field.native(instance, "lift") else {
                    unreachable!("Build checked {instance}.lift : REAL");
                };
                let raised = field
                    .native(instance, "raised")
                    .is_some_and(|v| is_true(&v));
                self.world.set_transfer(k, lift, raised);
            }
        }
        if self.t_ns >= self.next_scan_ns {
            self.controller.scan()?;
            self.next_scan_ns += self.controller.scan_ns();
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "line_tests.rs"]
mod tests;
