//! A project's line as the physics world needs it (spec 2026-11-09-component-behaviour-design.md, "Transport with
//! physics"; specs 2026-11-10-diverter-line-design.md, 2026-11-10-curves-belt-options-design.md and
//! 2026-11-11-stopper-pusher-turntable-design.md): conveyors (belts, roller conveyors, their curves and turntables'
//! lanes) with their pose, size, guards and skew, the devices mounted on them (`layout_devices.rs`), the sources and
//! sinks at their exits, the sensors on their beams. A family this build cannot simulate makes the whole line
//! unrunnable, with its name, as 4.2 requires.

use rapier3d::prelude::{Pose, Real, Rotation, Vector};

use crate::path::arc_length;

#[path = "layout_devices.rs"]
mod devices;
pub use devices::{PUSHER_OPENING, Pusher, Stopper, Transfer, roller_gap};
use twinova_project::Project;
use twinova_project::model::{Connection, Definition, Instance, ParamValue};

/// A belt's grip against a standard carton, the figure a belt datasheet quotes (OIP `BELT_FRICTION`,
/// `src/core/types/base.ts` at `1e7488c`).
pub const BELT_GRIP: Real = 0.7;

/// A conveyor, belt or rollers, straight or curved: a deck with a surface velocity. OIP draws a roller conveyor's
/// rollers only; its physics is one deck (`examples/parts/roller-conveyor/behavior.ts` at `1e7488c`).
#[derive(Debug, Clone, PartialEq)]
pub struct Belt {
    pub id: String,
    /// The belt's origin: its entry end, on the floor, x along the flow and y up. An inclined conveyor's pose is tilted
    /// about the centre of its surface (OIP `tiltDeg`), so its surface and its velocity follow the slope.
    pub pose: Pose,
    /// Along the centreline: a curve's stubs and arc.
    pub length: Real,
    pub width: Real,
    /// Height of the running surface above the floor.
    pub height: Real,
    pub speed: Real,
    /// Grip against a standard carton: `BELT_GRIP`, or a roller conveyor's `friction`.
    pub grip: Real,
    /// Whether its exit feeds nothing: products are held there rather than lost (M4.1).
    pub closed_exit: bool,
    /// A curve's arc; none for a straight conveyor.
    pub arc: Option<Arc>,
    /// Its left (−z) and right guards.
    pub guards: [bool; 2],
    /// A roller conveyor's skew, in radians: positive drifts the load to its left (OIP `skew`).
    pub skew: Real,
    /// Where its guards open for the pushers beside it.
    pub openings: Vec<Opening>,
    /// It is a turntable's lane: OIP's static cross for a deck, and a flow that turns with the deck
    /// (`World::set_turntable`).
    pub turntable: bool,
}

/// A turntable's lane along its start heading, in widths: OIP's deck is 1.5 widths across (`turntable-conveyor/part.js`
/// `PLATE`), and its lane box runs from rim to rim.
pub const TURNTABLE_LANE: Real = 1.5;

/// A curve's arc (OIP's curved conveyors).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arc {
    pub inner_radius: Real,
    /// In radians.
    pub sweep: Real,
    /// It turns to its right (+z) instead of its left.
    pub right: bool,
}

/// Where a product leaves the line's surfaces: a conveyor's end, a chain transfer's side, or a pusher's.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Exit {
    End(usize),
    Side(usize),
    Push(usize),
}

/// A gap in a straight conveyor's guard, where a pusher stands (OIP `guardOpenings.ts`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opening {
    /// In the right guard (+z) instead of the left.
    pub right: bool,
    /// Its centre along the conveyor, from its entry.
    pub centre: Real,
    pub length: Real,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Source {
    pub id: String,
    pub belt: usize,
    pub interval_ns: i64,
    pub first_ns: i64,
    /// 0 is unlimited.
    pub max_count: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sink {
    pub id: String,
    pub exit: Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sensor {
    pub id: String,
    pub belt: usize,
    /// Along the belt, from its entry; on a curve, the angle into its arc, in radians.
    pub at: Real,
    pub range: Real,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub belts: Vec<Belt>,
    pub transfers: Vec<Transfer>,
    pub stoppers: Vec<Stopper>,
    pub pushers: Vec<Pusher>,
    /// The conveyors that are turntables' lanes.
    pub turntables: Vec<usize>,
    pub sources: Vec<Source>,
    pub sinks: Vec<Sink>,
    pub sensors: Vec<Sensor>,
}

impl Layout {
    pub fn belt(&self, id: &str) -> Option<usize> {
        self.belts.iter().position(|b| b.id == id)
    }

    pub fn transfer(&self, id: &str) -> Option<usize> {
        self.transfers.iter().position(|t| t.id == id)
    }

    pub fn pusher(&self, id: &str) -> Option<usize> {
        self.pushers.iter().position(|p| p.id == id)
    }
}

/// A scalar parameter as a number; a list, which no scalar reads, as none.
fn param(inst: &Instance, def: &Definition, name: &str) -> f64 {
    match inst.parameter(def, name) {
        Some(ParamValue::Real(x)) => *x,
        Some(ParamValue::Int(n)) => *n as f64,
        Some(ParamValue::Bool(b)) => f64::from(u8::from(*b)),
        Some(ParamValue::List(_)) | None => 0.0,
    }
}

fn pose(inst: &Instance) -> Pose {
    let [x, y, z] = inst.position.map(|v| v as Real);
    let [qx, qy, qz, qw] = inst.rotation.map(|v| v as Real);
    Pose::from_parts(Vector::new(x, y, z), Rotation::from_xyzw(qx, qy, qz, qw))
}

fn degrees(inst: &Instance, def: &Definition, name: &str) -> Real {
    (param(inst, def, name) as Real).to_radians()
}

/// A straight conveyor's pose, tilted by its incline about the centre of its surface, head up for a positive one.
pub(crate) fn inclined(pose: Pose, length: Real, height: Real, incline: Real) -> Pose {
    if incline == 0.0 {
        return pose;
    }
    let centre = Vector::new(length / 2.0, height, 0.0);
    pose * Pose::from_translation(centre)
        * Pose::from_rotation(Rotation::from_rotation_z(incline))
        * Pose::from_translation(-centre)
}

fn definition_of<'a>(project: &'a Project, instance: &str) -> &'a Definition {
    let inst = project.scene.instances.iter().find(|i| i.id == instance);
    &project.definitions[&inst.expect("an instance of the layout").definition]
}

/// The peer of a flow port: `(instance, port)` at the other end of the flow connection that has `end`.
fn flow_peer<'a>(project: &'a Project, end: (&str, &str)) -> Option<(&'a str, &'a str)> {
    project.scene.connections.iter().find_map(|c| match c {
        Connection::Flow { from, to } if (from.0.as_str(), from.1.as_str()) == end => {
            Some((to.0.as_str(), to.1.as_str()))
        }
        Connection::Flow { from, to } if (to.0.as_str(), to.1.as_str()) == end => {
            Some((from.0.as_str(), from.1.as_str()))
        }
        _ => None,
    })
}

/// The line of `project`, in instance-id order (M4.1: construction order follows ids, not file order).
pub fn layout(project: &Project) -> Result<Layout, String> {
    let mut instances: Vec<(&Instance, &Definition)> = project
        .scene
        .instances
        .iter()
        .map(|i| (i, &project.definitions[&i.definition]))
        .collect();
    instances.sort_by(|a, b| a.0.id.cmp(&b.0.id));
    let unsupported: Vec<String> = instances
        .iter()
        .filter(|(_, d)| {
            !matches!(
                d.generator.as_str(),
                "twinova.belt/1"
                    | "twinova.roller/1"
                    | "twinova.belt_curve/1"
                    | "twinova.roller_curve/1"
                    | "twinova.turntable/1"
                    | "twinova.chain_transfer/1"
                    | "twinova.stopper/1"
                    | "twinova.pusher/1"
                    | "twinova.source/1"
                    | "twinova.sink/1"
                    | "twinova.sensor_diffuse/1"
            )
        })
        .map(|(i, d)| format!("{} ({})", i.id, d.id))
        .collect();
    if !unsupported.is_empty() {
        return Err(format!(
            "this build cannot run {}: their families have no model yet (ROADMAP 4.4)",
            unsupported.join(", ")
        ));
    }
    let mut out = Layout::default();
    for (inst, def) in &instances {
        let real = |name| param(inst, def, name) as Real;
        let (width, height) = (real("width"), real("height"));
        let (grip, arc) = match def.generator.as_str() {
            "twinova.belt/1" => (BELT_GRIP, None),
            "twinova.roller/1" | "twinova.turntable/1" => (real("friction"), None),
            "twinova.belt_curve/1" | "twinova.roller_curve/1" => {
                let arc = Arc {
                    inner_radius: real("inner_radius"),
                    sweep: degrees(inst, def, "sweep"),
                    right: real("turn_right") != 0.0,
                };
                (real("friction"), Some(arc))
            }
            _ => continue,
        };
        let turntable = def.generator == "twinova.turntable/1";
        let (pose, length) = if turntable {
            let half = TURNTABLE_LANE * width / 2.0;
            (
                pose(inst) * Pose::from_translation(Vector::new(-half, 0.0, 0.0)),
                2.0 * half,
            )
        } else {
            let length = arc.map_or_else(|| real("length"), |a| arc_length(&a, width));
            (
                inclined(pose(inst), length, height, degrees(inst, def, "incline")),
                length,
            )
        };
        if turntable {
            out.turntables.push(out.belts.len());
        }
        out.belts.push(Belt {
            id: inst.id.clone(),
            pose,
            length,
            width,
            height,
            speed: real("speed"),
            grip,
            // A turntable's ends have no wall: its deck turns them (OIP has none).
            closed_exit: !turntable && flow_peer(project, (&inst.id, "out")).is_none(),
            arc,
            guards: [real("left_guard") != 0.0, real("right_guard") != 0.0],
            openings: Vec::new(),
            skew: degrees(inst, def, "skew"),
            turntable,
        });
    }
    devices::devices(project, &instances, &mut out)?;
    let belt_at = |end: (&str, &str), what: &str| -> Result<usize, String> {
        let peer = flow_peer(project, end).ok_or_else(|| format!("{} is not connected", what))?;
        out.belt(peer.0)
            .ok_or_else(|| format!("{what} is connected to {}, which is not a belt", peer.0))
    };
    let mut sources = Vec::new();
    let mut sinks = Vec::new();
    let mut sensors = Vec::new();
    for (inst, def) in &instances {
        match def.generator.as_str() {
            "twinova.source/1" => sources.push(Source {
                id: inst.id.clone(),
                belt: belt_at((&inst.id, "out"), &format!("{}.out", inst.id))?,
                interval_ns: param(inst, def, "interval") as i64,
                first_ns: param(inst, def, "first_arrival") as i64,
                max_count: param(inst, def, "max_count") as i64,
            }),
            "twinova.sink/1" => {
                let what = format!("{}.in", inst.id);
                let peer = flow_peer(project, (&inst.id, "in"))
                    .ok_or_else(|| format!("{what} is not connected"))?;
                let exit = match (out.belt(peer.0), out.transfer(peer.0), out.pusher(peer.0)) {
                    (Some(belt), ..) => Exit::End(belt),
                    (_, Some(transfer), _) => Exit::Side(transfer),
                    (.., Some(pusher)) => Exit::Push(pusher),
                    _ => {
                        return Err(format!(
                            "{what} is connected to {}, which is not a conveyor, a chain transfer or a pusher",
                            peer.0
                        ));
                    }
                };
                sinks.push(Sink {
                    id: inst.id.clone(),
                    exit,
                });
            }
            "twinova.sensor_diffuse/1" => {
                let (host, at) = devices::mount(project, &inst.id)?;
                let belt = out.belt(host).ok_or_else(|| {
                    format!("{} is mounted on {host}, which is not a belt", inst.id)
                })?;
                let at = at as Real;
                sensors.push(Sensor {
                    id: inst.id.clone(),
                    belt,
                    at: if out.belts[belt].arc.is_some() {
                        at.to_radians()
                    } else {
                        at
                    },
                    range: param(inst, def, "range") as Real,
                });
            }
            _ => {}
        }
    }
    out.sources = sources;
    out.sinks = sinks;
    out.sensors = sensors;
    Ok(out)
}
