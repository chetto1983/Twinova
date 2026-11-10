//! A project's line as the physics world needs it (spec 2026-11-09-component-behaviour-design.md, "Transport with
//! physics"; spec 2026-11-10-diverter-line-design.md): straight conveyors (belts and roller conveyors) with their pose
//! and size, the chain transfers fitted in roller conveyors, the sources and sinks at their exits, the sensors on
//! their beams. A family this build cannot simulate makes the whole line unrunnable, with its name, as 4.2 requires.

use rapier3d::prelude::{Pose, Real, Rotation, Vector};
use twinova_project::Project;
use twinova_project::model::{Connection, Definition, Instance, ParamValue};

/// A belt's grip against a standard carton, the figure a belt datasheet quotes (OIP `BELT_FRICTION`,
/// `src/core/types/base.ts` at `1e7488c`).
pub const BELT_GRIP: Real = 0.7;

/// A straight conveyor, belt or rollers: one deck with a surface velocity. OIP draws a roller conveyor's rollers only;
/// its physics is one deck (`examples/parts/roller-conveyor/behavior.ts` at `1e7488c`).
#[derive(Debug, Clone, PartialEq)]
pub struct Belt {
    pub id: String,
    /// The belt's origin: its entry end, on the floor, x along the flow and y up.
    pub pose: Pose,
    pub length: Real,
    pub width: Real,
    /// Height of the running surface above the floor.
    pub height: Real,
    pub speed: Real,
    /// Grip against a standard carton: `BELT_GRIP`, or a roller conveyor's `friction`.
    pub grip: Real,
    /// Whether its exit feeds nothing: products are held there rather than lost (M4.1).
    pub closed_exit: bool,
}

/// OIP's chain transfer (`examples/parts/chain-transfer/` at `1e7488c`): a deck of chains fitted in a roller conveyor,
/// lifted to carry sideways out of its side exit.
#[derive(Debug, Clone, PartialEq)]
pub struct Transfer {
    pub id: String,
    /// The roller conveyor it is fitted in.
    pub host: usize,
    /// The deck's centre along the host, from its entry.
    pub at: Real,
    /// The deck along the host's flow (`chains × distance`) and across it (`lane_length`).
    pub along: Real,
    pub across: Real,
    pub speed: Real,
    /// It carries to the host's right (+z) instead of its left (−z).
    pub out_right: bool,
    /// Whether its side exit feeds nothing: a wall holds what it carries there.
    pub closed_exit: bool,
}

/// Where a product leaves the line's surfaces: a conveyor's end, or a chain transfer's side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Exit {
    End(usize),
    Side(usize),
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
    /// Along the belt, from its entry.
    pub at: Real,
    pub range: Real,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Layout {
    pub belts: Vec<Belt>,
    pub transfers: Vec<Transfer>,
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
}

fn param(inst: &Instance, def: &Definition, name: &str) -> f64 {
    let value = inst.parameters.get(name).copied().or_else(|| {
        def.parameters
            .iter()
            .find(|p| p.name == name)
            .map(|p| p.default)
    });
    match value {
        Some(ParamValue::Real(x)) => x,
        Some(ParamValue::Int(n)) => n as f64,
        Some(ParamValue::Bool(b)) => f64::from(u8::from(b)),
        None => 0.0,
    }
}

fn pose(inst: &Instance) -> Pose {
    let [x, y, z] = inst.position.map(|v| v as Real);
    let [qx, qy, qz, qw] = inst.rotation.map(|v| v as Real);
    Pose::from_parts(Vector::new(x, y, z), Rotation::from_xyzw(qx, qy, qz, qw))
}

fn definition_of<'a>(project: &'a Project, instance: &str) -> &'a Definition {
    let inst = project.scene.instances.iter().find(|i| i.id == instance);
    &project.definitions[&inst.expect("an instance of the layout").definition]
}

/// The host and the position along it of a mounted device.
fn mount<'a>(project: &'a Project, device: &str) -> Result<(&'a str, f64), String> {
    project
        .scene
        .connections
        .iter()
        .find_map(|c| match c {
            Connection::Mount {
                host,
                device: d,
                position,
                ..
            } if d.0 == device => Some((host.0.as_str(), position[0])),
            _ => None,
        })
        .ok_or_else(|| format!("{device} is not mounted"))
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
                    | "twinova.chain_transfer/1"
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
        let grip = match def.generator.as_str() {
            "twinova.belt/1" => BELT_GRIP,
            "twinova.roller/1" => param(inst, def, "friction") as Real,
            _ => continue,
        };
        out.belts.push(Belt {
            id: inst.id.clone(),
            pose: pose(inst),
            length: param(inst, def, "length") as Real,
            width: param(inst, def, "width") as Real,
            height: param(inst, def, "height") as Real,
            speed: param(inst, def, "speed") as Real,
            grip,
            closed_exit: flow_peer(project, (&inst.id, "out")).is_none(),
        });
    }
    for (inst, def) in &instances {
        if def.generator != "twinova.chain_transfer/1" {
            continue;
        }
        let (host, at) = mount(project, &inst.id)?;
        let host = out
            .belt(host)
            .filter(|&k| definition_of(project, &out.belts[k].id).generator == "twinova.roller/1")
            .ok_or_else(|| {
                format!(
                    "{} is mounted on {host}, which is not a roller conveyor",
                    inst.id
                )
            })?;
        out.transfers.push(Transfer {
            id: inst.id.clone(),
            host,
            at: at as Real,
            along: (param(inst, def, "chains") * param(inst, def, "distance")) as Real,
            across: param(inst, def, "lane_length") as Real,
            speed: param(inst, def, "speed") as Real,
            out_right: param(inst, def, "out_right") != 0.0,
            closed_exit: flow_peer(project, (&inst.id, "out")).is_none(),
        });
    }
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
                let exit = match (out.belt(peer.0), out.transfer(peer.0)) {
                    (Some(belt), _) => Exit::End(belt),
                    (_, Some(transfer)) => Exit::Side(transfer),
                    _ => {
                        return Err(format!(
                            "{what} is connected to {}, which is not a conveyor or a chain transfer",
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
                let (host, at) = mount(project, &inst.id)?;
                sensors.push(Sensor {
                    id: inst.id.clone(),
                    belt: out.belt(host).ok_or_else(|| {
                        format!("{} is mounted on {host}, which is not a belt", inst.id)
                    })?,
                    at: at as Real,
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
