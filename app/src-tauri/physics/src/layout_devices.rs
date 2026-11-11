//! The devices mounted on a conveyor: OIP's chain transfer (spec 2026-11-10-diverter-line-design.md), blade stop and
//! pusher (spec 2026-11-11-stopper-pusher-turntable-design.md), each at a position along its host, which must be a
//! conveyor they fit.

use rapier3d::prelude::Real;

use super::{Belt, Layout, definition_of, flow_peer, param};
use twinova_project::Project;
use twinova_project::model::{Connection, Definition, Instance};

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

/// OIP's blade stop (`examples/parts/blade-stop/` at `1e7488c`): a blade raised between two rollers of its host.
#[derive(Debug, Clone, PartialEq)]
pub struct Stopper {
    pub id: String,
    /// The roller conveyor it is fitted in.
    pub host: usize,
    /// The blade's centreline along the host, from its entry: in the gap between the drawn rollers nearest its mount.
    pub at: Real,
    /// Across the host: its width less OIP's 3 cm.
    pub width: Real,
    /// How far the raised blade stands above the surface (OIP `raiseHeight`).
    pub raise: Real,
}

/// OIP's diverter, a pusher (`examples/parts/diverter/` at `1e7488c`): a paddle beside its host that sweeps a product
/// across it, out of its other side.
#[derive(Debug, Clone, PartialEq)]
pub struct Pusher {
    pub id: String,
    pub host: usize,
    /// The paddle's centre along the host, from its entry.
    pub at: Real,
    /// OIP `divertDistance`.
    pub stroke: Real,
    /// It pushes to the host's right (+z): it stands on the left.
    pub out_right: bool,
}

/// The guard opening OIP cuts for a pusher: its footprint along the rail and a centimetre each side
/// (`src/core/snap/guardOpenings.ts`).
pub const PUSHER_OPENING: Real = 0.75;
/// The roller view's pitch and radius (OIP `roller-conveyor/part.js` `PITCH`, `R`; `app/src/lib/oip/parts/roller.ts`).
const ROLLER_PITCH: Real = 0.075;
const ROLLER_R: Real = 0.024;
/// OIP fits a blade to its bed less a rail's clearance on each side and a centimetre (`snapTrack.ts`).
const BLADE_CLEARANCE: Real = 0.03;

/// The family of a mounted device, for the host checks and their messages.
#[derive(Clone, Copy)]
enum Kind {
    Transfer,
    Stopper,
    Pusher,
}

/// The conveyors a host may be for what it carries: a roller conveyor, or any straight one.
#[derive(Clone, Copy)]
pub(super) enum Fit {
    Rollers,
    Straight,
}

impl Kind {
    fn name(self) -> &'static str {
        match self {
            Kind::Transfer => "chain transfer",
            Kind::Stopper => "stopper",
            Kind::Pusher => "pusher",
        }
    }

    fn fit(self) -> Fit {
        match self {
            Kind::Transfer | Kind::Stopper => Fit::Rollers,
            Kind::Pusher => Fit::Straight,
        }
    }
}

/// The gap between the drawn rollers of a `length` m roller conveyor nearest `at`: the view centres
/// `floor((length − 2R) / pitch) + 1` rollers on its length (`roller.ts` `layoutRollers`).
pub fn roller_gap(length: Real, at: Real) -> Real {
    let n = (((length - 2.0 * ROLLER_R) / ROLLER_PITCH).floor() as i32 + 1).max(2);
    let first = length / 2.0 - (n - 1) as Real / 2.0 * ROLLER_PITCH + ROLLER_PITCH / 2.0;
    let k = ((at - first) / ROLLER_PITCH)
        .round()
        .clamp(0.0, (n - 2) as Real);
    first + k * ROLLER_PITCH
}

/// Why `host` cannot carry a `name` that fits `fit` by its shape, if it cannot: the generators it fits, level and
/// unskewed.
pub(super) fn shape_problem(
    project: &Project,
    host: &Belt,
    fit: Fit,
    name: &str,
) -> Option<String> {
    let inst = project.scene.instances.iter().find(|i| i.id == host.id);
    let inst = inst.expect("an instance of the layout");
    let def = definition_of(project, &host.id);
    let generator = def.generator.as_str();
    let problem = match fit {
        Fit::Rollers => (generator != "twinova.roller/1").then_some("a roller conveyor"),
        Fit::Straight => (!matches!(generator, "twinova.roller/1" | "twinova.belt/1"))
            .then_some("a straight conveyor"),
    };
    if let Some(conveyor) = problem {
        return Some(format!("which is not {conveyor}"));
    }
    if param(inst, def, "incline") != 0.0 || host.skew != 0.0 {
        return Some(format!(
            "which is not level and square: a {name} fits a level, unskewed conveyor"
        ));
    }
    None
}

/// Why `host` cannot carry a `kind`, if it cannot: its shape, and for a device with a side exit, a guard across it.
fn host_problem(project: &Project, host: &Belt, kind: Kind, out_right: bool) -> Option<String> {
    if let Some(problem) = shape_problem(project, host, kind.fit(), kind.name()) {
        return Some(problem);
    }
    if matches!(kind, Kind::Stopper) {
        return None;
    }
    let (side, name) = if out_right { (1, "right") } else { (0, "left") };
    let device = match kind {
        Kind::Transfer => "transfer",
        _ => "pusher",
    };
    host.guards[side]
        .then(|| format!("whose {name} guard closes the {device}'s exit: turn {name}_guard off"))
}

/// The generators of the mounted devices `host` fits by its shape; Build also checks a side exit's guard.
pub(crate) fn fitting(project: &Project, host: &Belt) -> Vec<&'static str> {
    [
        (Kind::Transfer, "twinova.chain_transfer/1"),
        (Kind::Stopper, "twinova.stopper/1"),
        (Kind::Pusher, "twinova.pusher/1"),
    ]
    .into_iter()
    .filter(|(kind, _)| shape_problem(project, host, kind.fit(), kind.name()).is_none())
    .map(|(_, generator)| generator)
    .collect()
}

/// The host and the position along it of a mounted device.
pub(super) fn mount<'a>(project: &'a Project, device: &str) -> Result<(&'a str, f64), String> {
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

/// A mounted device, laid out on its host.
pub(crate) enum Device {
    Transfer(Transfer),
    Stopper(Stopper),
    Pusher(Pusher),
}

/// The device `inst` is, on its host in `out`, whose conveyors are laid out; none if its family is not a mounted device.
pub(crate) fn device(
    project: &Project,
    inst: &Instance,
    def: &Definition,
    out: &Layout,
) -> Result<Option<Device>, String> {
    let kind = match def.generator.as_str() {
        "twinova.chain_transfer/1" => Kind::Transfer,
        "twinova.stopper/1" => Kind::Stopper,
        "twinova.pusher/1" => Kind::Pusher,
        _ => return Ok(None),
    };
    let real = |name| param(inst, def, name) as Real;
    let (host, at) = mount(project, &inst.id)?;
    let out_right = real("out_right") != 0.0;
    let problem = match out.belt(host) {
        Some(k) => host_problem(project, &out.belts[k], kind, out_right),
        None => Some("which is not a conveyor".into()),
    };
    if let Some(problem) = problem {
        return Err(format!("{} is mounted on {host}, {problem}", inst.id));
    }
    let host = out.belt(host).expect("checked");
    let (id, at) = (inst.id.clone(), at as Real);
    Ok(Some(match kind {
        Kind::Transfer => Device::Transfer(Transfer {
            id,
            host,
            at,
            along: (param(inst, def, "chains") * param(inst, def, "distance")) as Real,
            across: real("lane_length"),
            speed: real("speed"),
            out_right,
            closed_exit: flow_peer(project, (&inst.id, "out")).is_none(),
        }),
        Kind::Stopper => {
            let b = &out.belts[host];
            Device::Stopper(Stopper {
                id,
                host,
                at: roller_gap(b.length, at),
                width: b.width - BLADE_CLEARANCE,
                raise: real("raise_height"),
            })
        }
        Kind::Pusher => Device::Pusher(Pusher {
            id,
            host,
            at,
            stroke: real("stroke"),
            out_right,
        }),
    }))
}

/// Adds a device to `out`: a pusher also opens its side's guard on its host.
pub(crate) fn add(out: &mut Layout, device: Device) {
    match device {
        Device::Transfer(t) => out.transfers.push(t),
        Device::Stopper(s) => out.stoppers.push(s),
        Device::Pusher(p) => {
            out.belts[p.host].openings.push(super::Opening {
                right: !p.out_right,
                centre: p.at,
                length: PUSHER_OPENING,
            });
            out.pushers.push(p);
        }
    }
}

/// Adds the mounted devices of `instances` to `out`, whose conveyors are laid out.
pub(super) fn devices(
    project: &Project,
    instances: &[(&Instance, &Definition)],
    out: &mut Layout,
) -> Result<(), String> {
    for (inst, def) in instances {
        if let Some(device) = device(project, inst, def, out)? {
            add(out, device);
        }
    }
    Ok(())
}
