//! The sensors mounted on a conveyor (specs 2026-11-10-diverter-line-design.md and 2026-11-11-sensors-products-design.md):
//! the diffuse, through-beam, colour and laser sensors on a beam across it, at a position along it (an angle into a
//! curve), and the barcode scanner over a straight conveyor.

use rapier3d::prelude::Real;
use twinova_project::Project;
use twinova_project::model::{Definition, Instance};

use super::devices::mount;
use super::{Layout, colors, number, param};

#[derive(Debug, Clone, PartialEq)]
pub struct Sensor {
    pub id: String,
    pub belt: usize,
    /// Along the belt, from its entry; on a curve, the angle into its arc, in radians.
    pub at: Real,
    pub kind: SensorKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SensorKind {
    /// A diffuse photo eye: its beam from the bed edge, as far as `range`.
    Diffuse { range: Real },
    /// An emitter and a receiver facing each other across the conveyor.
    ThroughBeam,
    /// It classifies the colour of what it sees within `range` (OIP's colour sensor).
    Color {
        range: Real,
        colors: Vec<[u8; 3]>,
        tolerance: u8,
    },
    /// It measures the distance to what it sees, `max_range` when clear.
    Laser { max_range: Real },
}

/// A barcode scanner's arch over a straight conveyor, at a position along it.
#[derive(Debug, Clone, PartialEq)]
pub struct Scanner {
    pub id: String,
    pub belt: usize,
    pub at: Real,
    /// How far along the conveyor its read window reaches, centred on the scan line (OIP `readDepth`).
    pub depth: Real,
}

/// The conveyor `inst` is mounted on, and where along it: an angle in radians into a curve.
fn mounted(project: &Project, out: &Layout, inst: &Instance) -> Result<(usize, Real), String> {
    let (host, at) = mount(project, &inst.id)?;
    let belt = (out.belt(host))
        .ok_or_else(|| format!("{} is mounted on {host}, which is not a belt", inst.id))?;
    let at = at as Real;
    let at = if out.belts[belt].arc.is_some() {
        at.to_radians()
    } else {
        at
    };
    Ok((belt, at))
}

pub(super) fn sensor(
    project: &Project,
    out: &Layout,
    inst: &Instance,
    def: &Definition,
) -> Result<Sensor, String> {
    let (belt, at) = mounted(project, out, inst)?;
    let real = |name| param(inst, def, name) as Real;
    let kind = match def.generator.as_str() {
        "twinova.sensor_diffuse/1" => SensorKind::Diffuse {
            range: real("range"),
        },
        "twinova.sensor_through_beam/1" => SensorKind::ThroughBeam,
        "twinova.sensor_color/1" => SensorKind::Color {
            range: real("range"),
            colors: colors(inst, def, "colors"),
            // The format keeps it within 0 to 255 by the definition's bounds.
            tolerance: number(inst, def, "tolerance").map_or(0, |t| t.clamp(0.0, 255.0) as u8),
        },
        _ => SensorKind::Laser {
            max_range: real("max_range"),
        },
    };
    Ok(Sensor {
        id: inst.id.clone(),
        belt,
        at,
        kind,
    })
}

pub(super) fn scanner(
    project: &Project,
    out: &Layout,
    inst: &Instance,
    def: &Definition,
) -> Result<Scanner, String> {
    let (belt, at) = mounted(project, out, inst)?;
    if out.belts[belt].arc.is_some() {
        return Err(format!(
            "{} is mounted on {}, a curve: a barcode scanner stands over a straight conveyor",
            inst.id, out.belts[belt].id
        ));
    }
    Ok(Scanner {
        id: inst.id.clone(),
        belt,
        at,
        depth: param(inst, def, "read_depth") as Real,
    })
}
