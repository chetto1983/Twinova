//! Where each instance meets the others (spec 2026-11-11-snapping-design.md, "Interfaces"): its ends and, on a
//! conveyor, the rail its devices mount on. They are built by the line's own layout code, so the editor snaps to the
//! geometry the physics runs, with each instance at the origin: its frames are its own, small and exact in f32, and the
//! editor places them in the world in f64 (`world`). An incomplete line still has them: an instance only lacks what its
//! own description cannot give, such as the side exit of a device not yet mounted.

use std::collections::BTreeMap;

use glamx::{DPose3, DQuat, DVec3};
use rapier3d::prelude::{Pose, Real, Rotation, Vector};
use twinova_project::Project;
use twinova_project::model::Instance;

use crate::layout::{
    Belt, Exit, Layout, add, conveyor, device, faces, fitting, scanner_fits, takes_spurs,
};
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
    /// In its own instance's frame, or in `on`'s.
    pub frame: DPose3,
    /// The host whose frame a mounted device's side exit is in: the device's own pose is not where it stands.
    pub on: Option<String>,
}

/// Where a device mounts along a conveyor, on its running surface, in the conveyor's frame.
#[derive(Debug, Clone, PartialEq)]
pub enum Rail {
    /// A straight run: from its start, along `start`'s x for `length`. A mount's distance counts from `start`.
    Line { start: DPose3, length: f64 },
    /// A curve's arc: about `centre`, from the centreline point `start`, `sweep` radians to the left or the right. A
    /// mount's distance is the angle into it, in degrees.
    Arc {
        centre: DVec3,
        start: DVec3,
        sweep: f64,
        right: bool,
    },
}

/// A conveyor's rail, its width, the generators of the devices it fits by its shape (Build also checks that a guard
/// does not close a device's side exit), and whether a spur may merge onto its flanks, `width / 2` either side of a
/// straight rail.
#[derive(Debug, Clone, PartialEq)]
pub struct Host {
    pub rail: Rail,
    pub width: f64,
    pub accepts: Vec<&'static str>,
    pub spurs: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Interfaces {
    pub ends: Vec<End>,
    pub host: Option<Host>,
}

/// An instance's pose, in f64.
pub fn place(inst: &Instance) -> DPose3 {
    let [x, y, z, w] = inst.rotation;
    DPose3::from_parts(
        DVec3::from(inst.position),
        DQuat::from_xyzw(x, y, z, w).normalize(),
    )
}

/// Where instance `id`'s `end` is in the world.
pub fn world(project: &Project, id: &str, end: &End) -> DPose3 {
    let base = end.on.as_deref().unwrap_or(id);
    let inst = project.scene.instances.iter().find(|i| i.id == base);
    place(inst.expect("an instance")) * end.frame
}

/// `inst` at the origin, unturned: its frames come out in its own.
fn at_origin(inst: &Instance) -> Instance {
    Instance {
        position: [0.0; 3],
        rotation: [0.0, 0.0, 0.0, 1.0],
        ..inst.clone()
    }
}

/// Every instance's interfaces, by id.
pub fn interfaces(project: &Project) -> BTreeMap<String, Interfaces> {
    let mut line = Layout::default();
    for inst in &project.scene.instances {
        let def = &project.definitions[&inst.definition];
        line.belts.extend(conveyor(&at_origin(inst), def));
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
            "twinova.source/1" => i.ends.push(end("out", Direction::Out, Pose::IDENTITY)),
            "twinova.sink/1" => i.ends.push(end("in", Direction::In, turned(1.0))),
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
        let mut exit = end("out", Direction::Out, frame);
        exit.on = Some(b.id.clone());
        out.get_mut(&id).expect("an instance").ends.push(exit);
    }
    out
}

fn end(name: &'static str, direction: Direction, frame: Pose) -> End {
    End {
        name,
        direction,
        frame: DPose3::from(frame),
        on: None,
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
    let [entry, exit] = faces(b);
    let mut ends = vec![
        end("in", Direction::In, entry),
        end("out", Direction::Out, exit),
    ];
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
            centre: surface.transform_point(arc.centre(b.width)).as_dvec3(),
            start: (surface * arc.frame(b.width, 0.0)).translation.as_dvec3(),
            sweep: f64::from(arc.sweep),
            right: arc.right,
        },
        None => Rail::Line {
            start: DPose3::from(surface),
            length: f64::from(b.length),
        },
    };
    let mut accepts = SENSORS.to_vec();
    if scanner_fits(b) {
        accepts.push(SCANNER);
    }
    accepts.extend(fitting(project, b));
    Host {
        rail,
        width: f64::from(b.width),
        accepts,
        spurs: takes_spurs(project, b),
    }
}

#[cfg(test)]
#[path = "interfaces_tests.rs"]
mod tests;
