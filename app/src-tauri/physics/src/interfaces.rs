//! Where each instance meets the others (spec 2026-11-11-snapping-design.md, "Interfaces"): its ends and, on a
//! conveyor, the rail its devices mount on. Everything is in the world, built by the line's own layout code, so the
//! editor snaps to the geometry the physics runs. An incomplete line still has them: an instance only lacks what its
//! own description cannot give, such as the side exit of a device not yet mounted.

use std::collections::BTreeMap;

use rapier3d::prelude::{Pose, Real, Rotation, Vector};
use twinova_project::Project;

use crate::layout::{Belt, Exit, Layout, add, conveyor, device, fitting, pose, scanner_fits};
use crate::world::side_exit;

/// The sensors, which any conveyor carries.
const SENSORS: [&str; 4] = [
    "twinova.sensor_diffuse/1",
    "twinova.sensor_through_beam/1",
    "twinova.sensor_color/1",
    "twinova.sensor_laser/1",
];
const SCANNER: &str = "twinova.scanner_barcode/1";

/// Which way products cross an end.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    /// A turntable's rim, which feeds or takes whatever its deck faces.
    Either,
}

/// An end: its frame's origin is on its face at the running surface (on the floor for a source or a sink), and its x
/// points out of the instance.
#[derive(Debug, Clone, PartialEq)]
pub struct End {
    /// The flow port it is, or a turntable's `left` or `right` rim, which has none.
    pub name: &'static str,
    pub direction: Direction,
    pub frame: Pose,
}

/// Where a device mounts along a conveyor, on its running surface.
#[derive(Debug, Clone, PartialEq)]
pub enum Rail {
    /// A straight run: from its start, along `start`'s x for `length`. A mount's distance counts from `start`.
    Line { start: Pose, length: Real },
    /// A curve's arc: about `centre`, from the centreline point `start`, `sweep` radians to the left or the right. A
    /// mount's distance is the angle into it, in degrees.
    Arc {
        centre: Vector,
        start: Vector,
        sweep: Real,
        right: bool,
    },
}

/// A conveyor's rail and the generators of the devices it fits by its shape. Build also checks that a guard does
/// not close a device's side exit.
#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    pub rail: Rail,
    pub accepts: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Interfaces {
    pub ends: Vec<End>,
    pub host: Option<Host>,
}

/// Every instance's interfaces, by id.
pub fn interfaces(project: &Project) -> BTreeMap<String, Interfaces> {
    let mut line = Layout::default();
    for inst in &project.scene.instances {
        line.belts
            .extend(conveyor(inst, &project.definitions[&inst.definition]));
    }
    for inst in &project.scene.instances {
        let def = &project.definitions[&inst.definition];
        if let Ok(Some(d)) = device(project, inst, def, &line) {
            add(&mut line, d);
        }
    }
    let mut out: BTreeMap<String, Interfaces> = BTreeMap::new();
    for inst in &project.scene.instances {
        let mut i = Interfaces::default();
        if let Some(k) = line.belt(&inst.id) {
            let b = &line.belts[k];
            i.ends = conveyor_ends(b);
            i.host = Some(host(project, b));
        }
        match project.definitions[&inst.definition].generator.as_str() {
            "twinova.source/1" => i.ends.push(end("out", Direction::Out, pose(inst))),
            "twinova.sink/1" => i
                .ends
                .push(end("in", Direction::In, pose(inst) * turned(1.0))),
            _ => {}
        }
        out.insert(inst.id.clone(), i);
    }
    let exits = (0..line.transfers.len())
        .map(|k| (line.transfers[k].id.clone(), Exit::Side(k)))
        .chain((0..line.pushers.len()).map(|k| (line.pushers[k].id.clone(), Exit::Push(k))));
    for (id, exit) in exits.collect::<Vec<_>>() {
        let s = side_exit(&line, exit).expect("a side exit");
        let b = &line.belts[s.host];
        let side = if s.right { 1.0 } else { -1.0 };
        let frame =
            b.pose * at(s.at, b.height, side * s.edge) * Pose::from_rotation(across(s.right));
        let ends = &mut out.get_mut(&id).expect("an instance").ends;
        ends.push(end("out", Direction::Out, frame));
    }
    out
}

fn end(name: &'static str, direction: Direction, frame: Pose) -> End {
    End {
        name,
        direction,
        frame,
    }
}

fn at(x: Real, y: Real, z: Real) -> Pose {
    Pose::from_translation(Vector::new(x, y, z))
}

/// Half turns about +Y.
fn turned(halves: Real) -> Pose {
    Pose::from_rotation(Rotation::from_rotation_y(halves * std::f32::consts::PI))
}

/// The rotation that turns x across a conveyor, to its right (+z) or its left.
fn across(right: bool) -> Rotation {
    let quarter = std::f32::consts::FRAC_PI_2;
    Rotation::from_rotation_y(if right { -quarter } else { quarter })
}

/// A conveyor's ends: in and out on its faces, and a turntable's rims across its lane.
fn conveyor_ends(b: &Belt) -> Vec<End> {
    let surface = b.pose * at(0.0, b.height, 0.0);
    let mut ends = match b.arc {
        Some(arc) => vec![
            end("in", Direction::In, surface * turned(1.0)),
            end("out", Direction::Out, surface * arc.end(b.width)),
        ],
        None => vec![
            end(
                "in",
                Direction::In,
                surface * at(-b.face, 0.0, 0.0) * turned(1.0),
            ),
            end(
                "out",
                Direction::Out,
                surface * at(b.length + b.face, 0.0, 0.0),
            ),
        ],
    };
    if b.turntable {
        let half = b.length / 2.0;
        for (name, right) in [("left", false), ("right", true)] {
            let side = if right { half } else { -half };
            let frame = surface * at(half, 0.0, side) * Pose::from_rotation(across(right));
            ends.push(end(name, Direction::Either, frame));
        }
    }
    ends
}

/// A conveyor's rail and the devices it fits.
fn host(project: &Project, b: &Belt) -> Host {
    let surface = b.pose * at(0.0, b.height, 0.0);
    let rail = match b.arc {
        Some(arc) => Rail::Arc {
            centre: surface.transform_point(arc.centre(b.width)),
            start: (surface * arc.frame(b.width, 0.0)).translation,
            sweep: arc.sweep,
            right: arc.right,
        },
        None => Rail::Line {
            start: surface,
            length: b.length,
        },
    };
    let mut accepts = SENSORS.to_vec();
    if scanner_fits(b) {
        accepts.push(SCANNER);
    }
    accepts.extend(fitting(project, b));
    Host { rail, accepts }
}

#[cfg(test)]
#[path = "interfaces_tests.rs"]
mod tests;
