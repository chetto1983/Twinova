//! A project's line as the physics world needs it (spec 2026-11-09-component-behaviour-design.md, "Transport with
//! physics"; specs 2026-11-10-diverter-line-design.md, 2026-11-10-curves-belt-options-design.md and
//! 2026-11-11-stopper-pusher-turntable-design.md): conveyors (belts, roller conveyors, their curves and turntables'
//! lanes) with their pose, size, guards and skew, the devices mounted on them (`layout_devices.rs`), the sources and
//! sinks at their exits, the sensors on their beams. A family this build cannot simulate makes the whole line
//! unrunnable, with its name, as 4.2 requires.

use rapier3d::prelude::{Pose, Real, Rotation, Vector};

#[path = "layout_conveyor.rs"]
mod conveyor;
#[path = "layout_devices.rs"]
mod devices;
#[path = "layout_merge.rs"]
mod merge;
#[path = "layout_sensors.rs"]
mod sensors;
pub(crate) use conveyor::{conveyor, faces};
pub use devices::{PUSHER_OPENING, Pusher, Stopper, Transfer, roller_gap};
pub(crate) use devices::{add, device, fitting};
pub use merge::MERGE_REACH;
pub(crate) use sensors::scanner_fits;
pub use sensors::{Scanner, Sensor, SensorKind};
use twinova_project::Project;
use twinova_project::model::{Connection, Definition, Instance, Item, ParamValue, rgb};

/// A belt's grip against a standard carton, the figure a belt datasheet quotes (OIP `BELT_FRICTION`,
/// `src/core/types/base.ts` at `1e7488c`).
pub const BELT_GRIP: Real = 0.7;

/// How far a belt's end drums reach past its run: OIP's `BELT_HEIGHT / 2` (`src/core/types/base.ts` at `0572868`),
/// where its snap ends and its collider stop, and where Twinova's view draws them (spec 2026-11-11-snapping-design.md,
/// decision 7).
pub const BELT_FACE: Real = 0.085;

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
    /// How far a straight conveyor's deck and guards reach past each end of its run, to its end faces: `BELT_FACE` for a
    /// belt, 0 for rollers. A distance along it still counts from its run's start.
    pub face: Real,
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
    pub products: Products,
}

/// What a source makes (spec 2026-11-11-sensors-products-design.md; OIP `box-spawner/behavior.ts`).
#[derive(Debug, Clone, PartialEq)]
pub struct Products {
    /// Length along the flow, width across it, height, m.
    pub size: Vector,
    pub mass: Real,
    /// Each product's size drawn from OIP's range instead.
    pub random_size: bool,
    /// Handed out one per product, in step: product n takes colour n and code n.
    pub colors: Vec<[u8; 3]>,
    pub codes: Vec<i32>,
    /// Draw each product's colour and code at random from the lists instead of walking them.
    pub random_order: bool,
    pub seed: u32,
}

/// A source pinned before 4.4d has no product parameters: it makes M3's 0.3 m carton, at the spec's 2.7 kg.
const DEFAULT_SIZE: Real = 0.3;
const DEFAULT_MASS: Real = 2.7;
const DEFAULT_SEED: u32 = 1;

impl Products {
    /// M3's carton, unpainted and without a code.
    pub fn carton() -> Self {
        Self {
            size: Vector::splat(DEFAULT_SIZE),
            mass: DEFAULT_MASS,
            random_size: false,
            colors: Vec::new(),
            codes: Vec::new(),
            random_order: false,
            seed: DEFAULT_SEED,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sink {
    pub id: String,
    pub exit: Exit,
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
    pub scanners: Vec<Scanner>,
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

/// A scalar parameter as a number, if the definition has it.
pub(super) fn number(inst: &Instance, def: &Definition, name: &str) -> Option<f64> {
    match inst.parameter(def, name)? {
        ParamValue::Real(x) => Some(*x),
        ParamValue::Int(n) => Some(*n as f64),
        ParamValue::Bool(b) => Some(f64::from(u8::from(*b))),
        ParamValue::List(_) => None,
    }
}

pub(super) fn param(inst: &Instance, def: &Definition, name: &str) -> f64 {
    number(inst, def, name).unwrap_or(0.0)
}

/// A list parameter's items; none if the definition does not have it.
fn items<'a>(inst: &'a Instance, def: &'a Definition, name: &str) -> &'a [Item] {
    match inst.parameter(def, name) {
        Some(ParamValue::List(items)) => items,
        _ => &[],
    }
}

/// A COLOR_LIST parameter's colours; the format checked its items.
pub(super) fn colors(inst: &Instance, def: &Definition, name: &str) -> Vec<[u8; 3]> {
    (items(inst, def, name).iter())
        .filter_map(|item| match item {
            Item::Text(t) => rgb(t),
            Item::Int(_) => None,
        })
        .collect()
}

fn products(inst: &Instance, def: &Definition) -> Products {
    let real = |name, default: Real| number(inst, def, name).map_or(default, |x| x as Real);
    Products {
        size: Vector::new(
            real("length", DEFAULT_SIZE),
            real("width", DEFAULT_SIZE),
            real("height", DEFAULT_SIZE),
        ),
        mass: real("mass", DEFAULT_MASS),
        random_size: param(inst, def, "random_size") != 0.0,
        // The format checked both lists' items.
        colors: colors(inst, def, "colors"),
        codes: (items(inst, def, "codes").iter())
            .filter_map(|item| match item {
                Item::Int(n) => i32::try_from(*n).ok(),
                Item::Text(_) => None,
            })
            .collect(),
        random_order: param(inst, def, "random_order") != 0.0,
        // A DINT seed's bits, as mulberry32 takes a 32-bit state.
        seed: number(inst, def, "seed").map_or(DEFAULT_SEED, |n| n as i32 as u32),
    }
}

pub(crate) fn pose(inst: &Instance) -> Pose {
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
                    | "twinova.sensor_through_beam/1"
                    | "twinova.sensor_color/1"
                    | "twinova.sensor_laser/1"
                    | "twinova.scanner_barcode/1"
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
        let Some(mut belt) = conveyor(inst, def) else {
            continue;
        };
        belt.closed_exit &= flow_peer(project, (&inst.id, "out")).is_none();
        if belt.turntable {
            out.turntables.push(out.belts.len());
        }
        out.belts.push(belt);
    }
    merge::merges(project, &mut out)?;
    devices::devices(project, &instances, &mut out)?;
    let belt_at = |end: (&str, &str), what: &str| -> Result<usize, String> {
        let peer = flow_peer(project, end).ok_or_else(|| format!("{} is not connected", what))?;
        out.belt(peer.0)
            .ok_or_else(|| format!("{what} is connected to {}, which is not a belt", peer.0))
    };
    let mut sources = Vec::new();
    let mut sinks = Vec::new();
    let mut sensors = Vec::new();
    let mut scanners = Vec::new();
    for (inst, def) in &instances {
        match def.generator.as_str() {
            "twinova.source/1" => sources.push(Source {
                id: inst.id.clone(),
                belt: belt_at((&inst.id, "out"), &format!("{}.out", inst.id))?,
                interval_ns: param(inst, def, "interval") as i64,
                first_ns: param(inst, def, "first_arrival") as i64,
                max_count: param(inst, def, "max_count") as i64,
                products: products(inst, def),
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
            "twinova.sensor_diffuse/1"
            | "twinova.sensor_through_beam/1"
            | "twinova.sensor_color/1"
            | "twinova.sensor_laser/1" => sensors.push(sensors::sensor(project, &out, inst, def)?),
            "twinova.scanner_barcode/1" => {
                scanners.push(sensors::scanner(project, &out, inst, def)?)
            }
            _ => {}
        }
    }
    out.sources = sources;
    out.sinks = sinks;
    out.sensors = sensors;
    out.scanners = scanners;
    Ok(out)
}
