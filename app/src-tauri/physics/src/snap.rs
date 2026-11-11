//! The editor's joins (spec 2026-11-11-snapping-design.md, "Joins"), shared by the app and the agent: a part posed so
//! that one of its ends faces another's with a gap, a spur posed square onto a straight's flank, and the joins a move
//! breaks. Poses are composed in f64 from each instance's own frames (`interfaces.rs`), so a join is exact to the
//! format's precision. Each works on a copy and changes the project only when it succeeds.

use std::collections::BTreeMap;

use glamx::{DPose3, DQuat, DVec3};
use twinova_project::Project;
use twinova_project::edit::{set_flow, set_merge};
use twinova_project::model::{Connection, ParamValue, Side};

use crate::interfaces::{Direction, End, Interfaces, Rail, interfaces, world};
use crate::layout::{merge_holds, merge_problem};

/// How far apart two joined faces may be: OIP's contact tolerance (`GUARD_CONTACT_TOLERANCE`, `src/core/types/base.ts`
/// at `0572868`), and the spec's limit on a gap.
pub const GAP_MAX: f64 = 0.15;
/// How far off one axis two joined faces may be, in metres and in degrees, rounding aside.
const OFF_AXIS: f64 = 0.001;
const OFF_ANGLE: f64 = 0.1;

/// Poses `id` so that its `end` faces `peer`'s `peer_end` across `gap` metres, on one axis. A conveyor also takes the
/// height that puts its end on the other's surface; a source or a sink stands on the floor. Two flow ports of opposite
/// directions are joined by a flow, freeing what each had; a turntable's rim takes none. Returns the notice of a
/// changed height.
pub fn join(
    project: &mut Project,
    (id, end): (&str, &str),
    (peer, peer_end): (&str, &str),
    gap: f64,
) -> Result<Option<String>, String> {
    if !(0.0..=GAP_MAX).contains(&gap) {
        return Err(format!("a gap of {gap} m is outside 0 to {GAP_MAX} m"));
    }
    if id == peer {
        return Err(format!("{id} cannot join itself"));
    }
    let all = interfaces(project);
    let (mine, theirs) = (end_of(&all, id, end)?, end_of(&all, peer, peer_end)?);
    if mine.on.is_some() {
        return Err(format!(
            "{id}.{end} is where a mounted device leaves its host: join the other part to it"
        ));
    }
    let flow = match (mine.direction, theirs.direction) {
        (Direction::Out, Direction::In) | (Direction::In, Direction::Out) => true,
        (Direction::Either, _) | (_, Direction::Either) => false,
        (Direction::In, Direction::In) => {
            return Err(format!(
                "{id}.{end} and {peer}.{peer_end} both take products in"
            ));
        }
        (Direction::Out, Direction::Out) => {
            return Err(format!(
                "{id}.{end} and {peer}.{peer_end} both send products out"
            ));
        }
    };
    let target = world(project, peer, theirs);
    let facing = target.rotation * DVec3::X;
    let mut next = project.clone();
    let notice = pose_onto(
        &mut next,
        id,
        end,
        target.translation + facing * gap,
        facing,
    )?;
    if flow {
        set_flow(&mut next, id, end, Some((peer, peer_end)))?;
    }
    moved(&mut next, id);
    *project = next;
    Ok(notice)
}

/// Poses spur `spur` square onto `host`'s flank, its exit face `gap` metres off it, `at` along the host's run on its
/// `side`, at the host's height, and merges it there (decision 5). Returns the notice of a changed height.
pub fn merge(
    project: &mut Project,
    spur: &str,
    (host, at, side): (&str, f64, Side),
    gap: f64,
) -> Result<Option<String>, String> {
    if !(0.0..=GAP_MAX).contains(&gap) {
        return Err(format!("a gap of {gap} m is outside 0 to {GAP_MAX} m"));
    }
    if spur == host {
        return Err(format!("{spur} cannot merge onto itself"));
    }
    let all = interfaces(project);
    let h = all
        .get(host)
        .and_then(|i| i.host.as_ref())
        .ok_or_else(|| format!("{host} is not a conveyor"))?;
    let Rail::Line { start, length } = h.rail.clone() else {
        return Err(format!("{host} is a curve: a spur merges onto a straight"));
    };
    if !h.spurs {
        return Err(format!(
            "{host} takes no spurs: a spur merges onto a level, unskewed straight"
        ));
    }
    if !(0.0..=length).contains(&at) {
        return Err(format!("{at} m is outside {host}'s run of {length} m"));
    }
    if all.get(spur).is_none_or(|i| i.host.is_none()) {
        return Err(format!("{spur} is not a conveyor"));
    }
    let sign = if side == Side::Right { 1.0 } else { -1.0 };
    let base = place_of(project, host) * start;
    let flank = base.transform_point(DVec3::new(at, 0.0, sign * h.width / 2.0));
    let outward = base.rotation * DVec3::new(0.0, 0.0, sign);
    let mut next = project.clone();
    let notice = pose_onto(&mut next, spur, "out", flank + outward * gap, outward)?;
    set_merge(&mut next, spur, Some((host, at, side)))?;
    moved(&mut next, spur);
    if let Some(problem) = merge_problem(&next) {
        return Err(problem);
    }
    *project = next;
    Ok(notice)
}

/// Removes the joins of `id` that no longer hold after it moved or turned (decision 3): a flow between two conveyors
/// whose faces are more than `GAP_MAX` apart or off one axis, and a merge whose spur no longer meets its flank.
pub fn moved(project: &mut Project, id: &str) {
    let all = interfaces(project);
    let conveyor = |id: &str| all.get(id).is_some_and(|i| i.host.is_some());
    let holds = |c: &Connection| match c {
        Connection::Flow { from, to }
            if (from.0 == id || to.0 == id) && conveyor(&from.0) && conveyor(&to.0) =>
        {
            match (end_of(&all, &from.0, &from.1), end_of(&all, &to.0, &to.1)) {
                (Ok(a), Ok(b)) => meet(&world(project, &from.0, a), &world(project, &to.0, b)),
                _ => false,
            }
        }
        Connection::Merge {
            from,
            host,
            at,
            side,
        } if from.0 == id || host.0 == id => merge_holds(project, &from.0, &host.0, *at, *side),
        _ => true,
    };
    let kept: Vec<Connection> = (project.scene.connections.iter())
        .filter(|c| holds(c))
        .cloned()
        .collect();
    project.scene.connections = kept;
}

/// Whether the faces `a` and `b` are joined: facing each other on one axis, `b` from on `a` to `GAP_MAX` off it.
fn meet(a: &DPose3, b: &DPose3) -> bool {
    let out = a.rotation * DVec3::X;
    let d = b.translation - a.translation;
    let along = d.dot(out);
    let off = (d - out * along).length();
    let angle = (b.rotation * DVec3::X).angle_between(-out).to_degrees();
    (-OFF_AXIS..=GAP_MAX + OFF_AXIS).contains(&along) && off <= OFF_AXIS && angle <= OFF_ANGLE
}

fn end_of<'a>(
    all: &'a BTreeMap<String, Interfaces>,
    id: &str,
    name: &str,
) -> Result<&'a End, String> {
    let ends = &all.get(id).ok_or_else(|| format!("no instance {id}"))?.ends;
    (ends.iter().find(|e| e.name == name)).ok_or_else(|| format!("{id} has no end {name}"))
}

fn place_of(project: &Project, id: &str) -> DPose3 {
    crate::interfaces::place(instance(project, id))
}

/// The heading about +Y of a direction, in radians: a turn by it takes x to that direction in plan.
fn heading(v: DVec3) -> f64 {
    (-v.z).atan2(v.x)
}

/// Turns `id` about +Y and moves it so that its `end` stands at `point` facing back along `facing`; a conveyor's
/// height then puts that end at `point`'s height, and a source or a sink stands on the floor. Returns the notice of a
/// changed height.
fn pose_onto(
    project: &mut Project,
    id: &str,
    end: &str,
    point: DVec3,
    facing: DVec3,
) -> Result<Option<String>, String> {
    let all = interfaces(project);
    let local = end_of(&all, id, end)?.frame;
    let turn = DQuat::from_rotation_y(heading(-facing) - heading(local.rotation * DVec3::X));
    // q and -q are one rotation: the one with w >= 0 keeps a saved scene readable ([0, 0, 0, 1], not [0, 0, 0, -1]).
    let turn = if turn.w < 0.0 { -turn } else { turn };
    let (y, notice) = if all[id].host.is_some() {
        let y = instance(project, id).position[1];
        (y, raise(project, id, point.y - (y + local.translation.y))?)
    } else {
        (0.0, None)
    };
    // A conveyor's height moves its ends straight up, so the end is found again at its new height.
    let local = end_of(&interfaces(project), id, end)?.frame;
    let offset = turn * local.translation;
    let inst = (project.scene.instances.iter_mut())
        .find(|i| i.id == id)
        .expect("an instance");
    inst.position = [point.x - offset.x, y, point.z - offset.z];
    inst.rotation = [turn.x, turn.y, turn.z, turn.w];
    Ok(notice)
}

/// Raises conveyor `id`'s running surface by `rise` metres, within its height's range, to the micrometre: the ends'
/// f32 frames would otherwise take a height a hair past its range, or change one that should not. Returns the notice of
/// a changed height.
fn raise(project: &mut Project, id: &str, rise: f64) -> Result<Option<String>, String> {
    let inst = instance(project, id);
    let def = &project.definitions[&inst.definition];
    let Some(&ParamValue::Real(current)) = inst.parameter(def, "height") else {
        return Err(format!("{id} has no height to match"));
    };
    let height = ((current + rise) * 1e6).round() / 1e6;
    if height == current {
        return Ok(None);
    }
    let range = def.parameters.iter().find(|p| p.name == "height");
    let (min, max) = range.map_or((None, None), |p| (p.min, p.max));
    if min.is_some_and(|m| height < m) || max.is_some_and(|m| height > m) {
        return Err(format!(
            "{id} would stand {height:.3} m high, outside its height's range of {} to {} m",
            min.unwrap_or(f64::NEG_INFINITY),
            max.unwrap_or(f64::INFINITY)
        ));
    }
    let inst = (project.scene.instances.iter_mut())
        .find(|i| i.id == id)
        .expect("an instance");
    inst.parameters
        .insert("height".into(), ParamValue::Real(height));
    Ok(Some(format!(
        "{id}'s height is now {height:.3} m, to meet it"
    )))
}

fn instance<'a>(project: &'a Project, id: &str) -> &'a twinova_project::model::Instance {
    let inst = project.scene.instances.iter().find(|i| i.id == id);
    inst.expect("an instance")
}

#[cfg(test)]
#[path = "snap_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "snap_rebuild_tests.rs"]
mod rebuild_tests;

#[cfg(test)]
#[path = "snap_merge_tests.rs"]
mod merge_tests;
