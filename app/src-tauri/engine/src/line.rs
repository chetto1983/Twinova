//! A project's line, run in lockstep on virtual time (spec 2026-11-09-component-behaviour-design.md, "Transport with
//! physics"; research 16: the fixed-slice lockstep of SIMIT's bus-synchronous mode). Every millisecond:
//! 1. the physics step (`twinova_physics::World`);
//! 2. the native models write the controller's inputs: the sensors and barcode scanners (spec
//!    2026-11-11-sensors-products-design.md), sink counters, each conveyor's drive feedback
//!    (`drive.rs`: velocity, encoder and pulse of the millisecond just stepped), the `running` of a belt without a
//!    behaviour;
//! 3. the field cycle, which reads the controller's outputs and writes its inputs, the conveyors' surface speeds
//!    (spec 2026-11-10-curves-belt-options-design.md), the chain transfers' lift (spec
//!    2026-11-10-diverter-line-design.md), and the stoppers', pushers' and turntables' positions (spec
//!    2026-11-11-stopper-pusher-turntable-design.md);
//! 4. the controller's scan, when its period is due.
//!
//! An online change (spec 2026-11-12-st-vscode-design.md) replaces the controller's program between two
//! milliseconds and binds the line to it again (`line_bind.rs`); the world and the field go on.

use trust_runtime::value::Value;
use twinova_physics::{Reading, STEP_NS, World, layout};
use twinova_project::Project;

use crate::controller::{Controller, Port};
use crate::drive::Feedback;
use crate::field::Field;
use crate::line_bind::{Ports, Wiring};

/// A conveyor: whether its speed comes from its behaviour or its motor straight from the controller, and its drive
/// feedback's state.
struct BeltIo {
    instance: String,
    behaviour: bool,
    feedback: Feedback,
}

/// A family whose block poses it in the world.
#[derive(Debug, Clone, Copy)]
enum Kind {
    Transfer,
    Stopper,
    Pusher,
    Turntable,
}

/// A device the field moves: its family, its index among the layout's devices of that family, its instance.
struct Device {
    kind: Kind,
    index: usize,
    instance: String,
}

pub struct Line {
    world: World,
    field: Option<Field>,
    controller: Controller,
    belts: Vec<BeltIo>,
    /// The devices the field moves.
    devices: Vec<Device>,
    wiring: Wiring,
    /// The controller addresses the line writes and reads, resolved for the current program, every one parsed once.
    ports: Ports,
    /// The controller outputs the field reads: the address as the field names it, and the controller's.
    commands: Vec<(String, Port)>,
    t_ns: i64,
    next_scan_ns: i64,
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

fn instance_of<'a>(project: &'a Project, instance: &str) -> &'a twinova_project::model::Instance {
    let found = project.scene.instances.iter().find(|i| i.id == instance);
    found.expect("an instance of the layout")
}

fn has_behaviour(project: &Project, instance: &str) -> bool {
    project.definitions[&instance_of(project, instance).definition]
        .behaviour
        .is_some()
}

impl Line {
    /// Builds the world, the field and the controller of `project`, with the warnings Build shows.
    pub fn build(project: &Project) -> Result<(Self, Vec<String>), String> {
        let layout = layout(project)?;
        let field = Field::build(project)?;
        let (controller, warnings) = Controller::build(project)?;
        let belts = (layout.belts.iter())
            .map(|b| BeltIo {
                instance: b.id.clone(),
                behaviour: has_behaviour(project, &b.id),
                feedback: Feedback::of(project, &b.id),
            })
            .collect();
        // A device's motion is its block's, so it has no native fallback (specs 2026-11-10-diverter-line-design.md and
        // 2026-11-11-stopper-pusher-turntable-design.md, "The families").
        let by_kind = [
            (
                Kind::Transfer,
                layout
                    .transfers
                    .iter()
                    .map(|t| t.id.clone())
                    .collect::<Vec<_>>(),
            ),
            (
                Kind::Stopper,
                layout.stoppers.iter().map(|s| s.id.clone()).collect(),
            ),
            (
                Kind::Pusher,
                layout.pushers.iter().map(|p| p.id.clone()).collect(),
            ),
            (
                Kind::Turntable,
                layout
                    .turntables
                    .iter()
                    .map(|&k| layout.belts[k].id.clone())
                    .collect(),
            ),
        ];
        let devices: Vec<Device> = by_kind
            .into_iter()
            .flat_map(|(kind, ids)| {
                ids.into_iter()
                    .enumerate()
                    .map(move |(index, instance)| Device {
                        kind,
                        index,
                        instance,
                    })
            })
            .collect();
        if let Some(d) = devices
            .iter()
            .find(|d| !has_behaviour(project, &d.instance))
        {
            let family = &project.definitions[&instance_of(project, &d.instance).definition].id;
            return Err(format!("{}: {family} needs its behaviour", d.instance));
        }
        let wiring = Wiring::of(project, &layout, field.as_ref())?;
        let commands = (field.iter().flat_map(|f| f.commands()))
            .map(|address| Ok((address.clone(), Port::parse(address)?)))
            .collect::<Result<_, String>>()?;
        let next_scan_ns = controller.scan_ns();
        let line = Self {
            world: World::new(layout),
            field,
            ports: wiring.bind(&controller),
            wiring,
            controller,
            belts,
            devices,
            commands,
            t_ns: 0,
            next_scan_ns,
        };
        Ok((line, warnings))
    }

    /// Replaces the program with `project`'s, checked as Build checks it, and binds the line to it again; the next scan
    /// stays where it was due, then follows the new program's period. An input the line no longer writes reads zero,
    /// as it would after Build. On an error nothing changes. Returns Build's warnings for the new program.
    pub fn change_program(&mut self, project: &Project) -> Result<Vec<String>, String> {
        let written: Vec<Port> = self.ports.inputs().into_iter().cloned().collect();
        let warnings = self.controller.reload(project)?;
        self.ports = self.wiring.bind(&self.controller);
        let still = self.ports.inputs();
        for port in written.iter().filter(|p| !still.contains(p)) {
            self.controller.clear_input(port)?;
        }
        Ok(warnings)
    }

    pub fn time_ns(&self) -> i64 {
        self.t_ns
    }

    pub fn scan_ns(&self) -> i64 {
        self.controller.scan_ns()
    }

    pub(crate) fn controller_mut(&mut self) -> &mut Controller {
        &mut self.controller
    }

    pub(crate) fn field_mut(&mut self) -> Option<&mut Field> {
        self.field.as_mut()
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
        let ports = &self.ports;
        for (k, address) in ports.sensors.iter().enumerate() {
            let value = match self.world.reading(k) {
                Reading::Detected(seen) => Value::Bool(seen),
                // The image is bit-string typed: a DINT at %ID travels as a DWORD (measured 2026-11-08).
                Reading::Color(index) => Value::DWord(index as u32),
                Reading::Distance(metres) => real(metres),
            };
            input(controller, address, value)?;
        }
        for (k, [read, count, code]) in ports.scanners.iter().enumerate() {
            let scan = self.world.scanned(k);
            input(controller, read, Value::Bool(scan.read))?;
            input(controller, count, Value::DWord(scan.count))?;
            input(controller, code, Value::DWord(scan.code as u32))?;
        }
        for (k, address) in ports.sinks.iter().enumerate() {
            // The image is bit-string typed: a DINT at %ID travels as a DWORD (measured 2026-11-08).
            let count = u32::try_from(self.world.delivered(k)).unwrap_or(u32::MAX);
            input(controller, address, Value::DWord(count))?;
        }
        for (k, (belt, at)) in self.belts.iter_mut().zip(&ports.belts).enumerate() {
            let speed = self.world.belt_speed(k);
            input(
                controller,
                &at.velocity,
                real(belt.feedback.velocity(speed)),
            )?;
            input(
                controller,
                &at.encoder,
                real(self.world.belt_travel(k) as f32),
            )?;
            if at.pulse.is_some() {
                input(
                    controller,
                    &at.pulse,
                    Value::Bool(belt.feedback.pulse(speed)),
                )?;
            }
        }
        if let Some(field) = self.field.as_mut() {
            for (address, port) in &self.commands {
                field.set_command(address, self.controller.read_at(port)?)?;
            }
            field.cycle()?;
            for (address, port) in &self.ports.feedback {
                self.controller
                    .write_at(port, field.feedback_value(address)?)?;
            }
        }
        for k in 0..self.belts.len() {
            let (belt, at) = (&self.belts[k], &self.ports.belts[k]);
            match (&self.field, belt.behaviour) {
                (Some(field), true) => {
                    let Some(Value::Real(speed)) = field.native(&belt.instance, "surface_speed")
                    else {
                        unreachable!("Build checked {}.surface_speed : REAL", belt.instance);
                    };
                    self.world.set_belt_speed(k, speed);
                }
                _ => {
                    let motor = self.output(&at.run)?;
                    input(&mut self.controller, &at.running, Value::Bool(motor))?;
                    self.world.set_belt_running(k, motor);
                }
            }
        }
        if let Some(field) = &self.field {
            for d in &self.devices {
                let real = |name: &str| match field.native(&d.instance, name) {
                    Some(Value::Real(x)) => x,
                    _ => unreachable!("Build checked {}.{name} : REAL", d.instance),
                };
                match d.kind {
                    Kind::Transfer => {
                        let raised = field
                            .native(&d.instance, "raised")
                            .is_some_and(|v| is_true(&v));
                        self.world.set_transfer(d.index, real("lift"), raised);
                    }
                    Kind::Stopper => self.world.set_stopper(d.index, real("lift")),
                    Kind::Pusher => self.world.set_pusher(d.index, real("reach")),
                    Kind::Turntable => self.world.set_turntable(
                        d.index,
                        real("deck_angle").to_radians(),
                        real("deck_rate").to_radians(),
                    ),
                }
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

#[cfg(test)]
#[path = "line_devices_tests.rs"]
mod devices_tests;

#[cfg(test)]
#[path = "line_sensors_tests.rs"]
mod sensors_tests;
