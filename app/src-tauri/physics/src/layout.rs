//! A project's line as the physics world needs it (spec 2026-11-09-component-behaviour-design.md, "Transport with
//! physics"): belts with their pose and size, the sources and sinks at their ends, the sensors on their beams. A
//! family this build cannot simulate makes the whole line unrunnable, with its name, as 4.2 requires.

use rapier3d::prelude::{Pose, Real, Rotation, Vector};
use twinova_project::Project;
use twinova_project::model::{Connection, Definition, Instance, ParamValue};

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
    /// Whether its exit feeds nothing: products are held there rather than lost (M4.1).
    pub closed_exit: bool,
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
    /// The belt whose exit it is.
    pub belt: usize,
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
    pub sources: Vec<Source>,
    pub sinks: Vec<Sink>,
    pub sensors: Vec<Sensor>,
}

impl Layout {
    pub fn belt(&self, id: &str) -> Option<usize> {
        self.belts.iter().position(|b| b.id == id)
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
    for (inst, def) in instances
        .iter()
        .filter(|(_, d)| d.generator == "twinova.belt/1")
    {
        let exit = flow_peer(project, (&inst.id, "out"));
        out.belts.push(Belt {
            id: inst.id.clone(),
            pose: pose(inst),
            length: param(inst, def, "length") as Real,
            width: param(inst, def, "width") as Real,
            height: param(inst, def, "height") as Real,
            speed: param(inst, def, "speed") as Real,
            closed_exit: exit.is_none(),
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
            "twinova.sink/1" => sinks.push(Sink {
                id: inst.id.clone(),
                belt: belt_at((&inst.id, "in"), &format!("{}.in", inst.id))?,
            }),
            "twinova.sensor_diffuse/1" => {
                let mount = project.scene.connections.iter().find_map(|c| match c {
                    Connection::Mount {
                        host,
                        device,
                        position,
                        ..
                    } if device.0 == inst.id => Some((host.0.as_str(), position[0])),
                    _ => None,
                });
                let (host, at) = mount.ok_or_else(|| format!("{} is not mounted", inst.id))?;
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
