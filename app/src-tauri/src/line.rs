//! A project's line, run in lockstep on virtual time (spec 2026-11-09-component-behaviour-design.md, "Transport with
//! physics"; research 16: the fixed-slice lockstep of SIMIT's bus-synchronous mode). Every millisecond:
//! 1. the physics step (`twinova_physics::World`);
//! 2. the native models write the controller's inputs: diffuse sensors, sink counters, each conveyor's drive feedback
//!    (`drive.rs`: velocity, encoder and pulse of the millisecond just stepped), the `running` of a belt without a
//!    behaviour;
//! 3. the field cycle, which reads the controller's outputs and writes its inputs, the conveyors' surface speeds
//!    (spec 2026-11-10-curves-belt-options-design.md) and the chain transfers' lift (spec
//!    2026-11-10-diverter-line-design.md);
//! 4. the controller's scan, when its period is due.

use trust_runtime::value::Value;
use twinova_physics::{STEP_NS, World, layout};
use twinova_project::Project;

use crate::controller::{Controller, Port};
use crate::drive::Feedback;
use crate::field::Field;

/// A conveyor's signals, and whether its speed comes from its behaviour or its motor straight from the controller.
struct BeltIo {
    instance: String,
    run: Option<Port>,
    running: Option<Port>,
    behaviour: bool,
    /// The controller inputs its drive feedback is bound to, and the feedback's state.
    velocity: Option<Port>,
    encoder: Option<Port>,
    pulse: Option<Port>,
    feedback: Feedback,
}

pub struct Line {
    world: World,
    field: Option<Field>,
    controller: Controller,
    belts: Vec<BeltIo>,
    /// Per layout chain transfer, its instance in the field.
    transfers: Vec<String>,
    /// Per layout sensor, the controller input its `detected` is bound to.
    sensors: Vec<Option<Port>>,
    /// Per layout sink, the controller input its `count` is bound to.
    sinks: Vec<Option<Port>>,
    /// The controller outputs the field reads and the inputs it writes: the address as the field names it, and the
    /// controller's, parsed. Every controller address is resolved here at Build, and an input only when the program
    /// declares it: one it does not is skipped, as Build warned.
    commands: Vec<(String, Port)>,
    feedback: Vec<(String, Port)>,
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

/// Writes a bound input.
fn input(controller: &mut Controller, port: &Option<Port>, value: Value) -> Result<(), String> {
    match port {
        Some(port) => controller.write_at(port, value),
        None => Ok(()),
    }
}

/// A REAL as the image carries it at `%ID`: the double word of its bits (measured 2026-11-10).
fn real(x: f32) -> Value {
    Value::DWord(x.to_bits())
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
        let declared = |instance: &str, name: &str| {
            signal(project, instance, name).and_then(|a| controller.declared_at(&a))
        };
        let mut belts = Vec::new();
        for b in &layout.belts {
            belts.push(BeltIo {
                instance: b.id.clone(),
                run: signal(project, &b.id, "run")
                    .map(|a| Port::parse(&a))
                    .transpose()?,
                running: declared(&b.id, "running"),
                behaviour: has_behaviour(project, &b.id),
                velocity: declared(&b.id, "velocity"),
                encoder: declared(&b.id, "encoder"),
                pulse: declared(&b.id, "pulse"),
                feedback: Feedback::of(project, &b.id),
            });
        }
        // Its change-over is its block's, so a chain transfer has no native fallback (spec, "The families").
        let transfers: Vec<String> = layout.transfers.iter().map(|t| t.id.clone()).collect();
        if let Some(t) = transfers.iter().find(|t| !has_behaviour(project, t)) {
            return Err(format!("{t}: twinova.chain_transfer needs its behaviour"));
        }
        let sensors = layout
            .sensors
            .iter()
            .map(|s| declared(&s.id, "detected"))
            .collect();
        let sinks = layout
            .sinks
            .iter()
            .map(|s| declared(&s.id, "count"))
            .collect();
        let (mut commands, mut feedback) = (Vec::new(), Vec::new());
        if let Some(field) = &field {
            for address in field.commands() {
                commands.push((address.clone(), Port::parse(address)?));
            }
            for address in field.feedback() {
                if let Some(port) = controller.declared_at(address) {
                    feedback.push((address.clone(), port));
                }
            }
        }
        let next_scan_ns = controller.scan_ns();
        let line = Self {
            world: World::new(layout),
            field,
            controller,
            belts,
            transfers,
            sensors,
            sinks,
            commands,
            feedback,
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

    fn output(&self, port: &Option<Port>) -> Result<bool, String> {
        match port {
            Some(port) => self.controller.read_at(port).map(|v| is_true(&v)),
            None => Ok(false),
        }
    }

    /// One millisecond of the line, in the order of the module comment. A fault of either runtime is returned.
    pub fn step_ms(&mut self) -> Result<(), String> {
        self.world.step();
        self.t_ns += STEP_NS;
        let controller = &mut self.controller;
        for (k, address) in self.sensors.iter().enumerate() {
            input(controller, address, Value::Bool(self.world.sensor(k)))?;
        }
        for (k, address) in self.sinks.iter().enumerate() {
            // The image is bit-string typed: a DINT at %ID travels as a DWORD (measured 2026-11-08).
            let count = u32::try_from(self.world.delivered(k)).unwrap_or(u32::MAX);
            input(controller, address, Value::DWord(count))?;
        }
        for (k, belt) in self.belts.iter_mut().enumerate() {
            let speed = self.world.belt_speed(k);
            input(
                controller,
                &belt.velocity,
                real(belt.feedback.velocity(speed)),
            )?;
            input(
                controller,
                &belt.encoder,
                real(self.world.belt_travel(k) as f32),
            )?;
            if belt.pulse.is_some() {
                input(
                    controller,
                    &belt.pulse,
                    Value::Bool(belt.feedback.pulse(speed)),
                )?;
            }
        }
        if let Some(field) = self.field.as_mut() {
            for (address, port) in &self.commands {
                field.set_command(address, self.controller.read_at(port)?)?;
            }
            field.cycle()?;
            for (address, port) in &self.feedback {
                self.controller
                    .write_at(port, field.feedback_value(address)?)?;
            }
        }
        for k in 0..self.belts.len() {
            let belt = &self.belts[k];
            match (&self.field, belt.behaviour) {
                (Some(field), true) => {
                    let Some(Value::Real(speed)) = field.native(&belt.instance, "surface_speed")
                    else {
                        unreachable!("Build checked {}.surface_speed : REAL", belt.instance);
                    };
                    self.world.set_belt_speed(k, speed);
                }
                _ => {
                    let motor = self.output(&belt.run)?;
                    input(&mut self.controller, &belt.running, Value::Bool(motor))?;
                    self.world.set_belt_running(k, motor);
                }
            }
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

#[cfg(test)]
#[path = "line_drive_tests.rs"]
mod drive_tests;
