//! Spurs merging onto a straight's flank (spec 2026-11-11-snapping-design.md, decision 5): the host's guard opens over
//! the spur's width where it meets the flank, as a pusher's does, and the spur's exit is open. Measured with a throwaway
//! probe (2026-11-11): through a closed guard no carton crossed; through the opening all did, upright.

use rapier3d::prelude::{Pose, Real, Vector};
use twinova_project::Project;
use twinova_project::model::{Connection, Side};

use super::devices::{Fit, shape_problem};
use super::{Belt, Layout, Opening, faces};

/// How far the spur's exit face may stand off the flank (OIP's `GUARD_CONTACT_TOLERANCE`, `src/core/types/base.ts` at
/// `0572868`), and how far before it, rounding aside.
pub const MERGE_REACH: Real = 0.15;
const MERGE_SLACK: Real = 0.001;
/// How far the spur's exit may stand from its `at` along the run, and how far from square to the flank.
const MERGE_ALONG: Real = 0.001;
const MERGE_SQUARE: Real = 0.1;

/// Opens the guards of `out`'s hosts where their spurs merge, and the spurs' exits; or why a merge cannot be.
pub(super) fn merges(project: &Project, out: &mut Layout) -> Result<(), String> {
    let mut placed: Vec<(usize, bool, Real, Real, &str)> = Vec::new();
    for c in &project.scene.connections {
        let Connection::Merge {
            from,
            host,
            at,
            side,
        } = c
        else {
            continue;
        };
        let (spur_id, host_id) = (from.0.as_str(), host.0.as_str());
        let what = format!("{spur_id} merges onto {host_id}");
        let spur = (out.belt(spur_id)).ok_or_else(|| format!("{what}, but is not a conveyor"))?;
        let host = (out.belt(host_id)).ok_or_else(|| format!("{what}, which is not a conveyor"))?;
        let right = *side == Side::Right;
        let (h, s, at) = (&out.belts[host], &out.belts[spur], *at as Real);
        if let Some(problem) = shape_problem(project, h, Fit::Straight, "spur") {
            return Err(format!("{what}, {problem}"));
        }
        if !(0.0..=h.length).contains(&at) {
            return Err(format!(
                "{what} at {at} m, outside its run of {} m",
                h.length
            ));
        }
        if let Some(problem) = meeting(h, faces(s)[1], at, right) {
            return Err(format!("{what}: {problem}"));
        }
        let (half, width) = (s.width / 2.0, s.width);
        if let Some(other) = (placed.iter())
            .find(|(k, r, a, w, _)| *k == host && *r == right && (a - at).abs() < w + half)
            .map(|p| p.4)
        {
            return Err(format!("{what} over {other}'s merge on its {side:?} side"));
        }
        placed.push((host, right, at, half, spur_id));
        out.belts[host].openings.push(Opening {
            right,
            centre: at,
            length: width,
        });
        out.belts[spur].closed_exit = false;
    }
    Ok(())
}

/// Why a spur's `exit` face does not meet host `h`'s flank at `at`, on its right or left side, if it does not: at `at`
/// along the run, from on the flank to `MERGE_REACH` off it, square to it.
fn meeting(h: &Belt, exit: Pose, at: Real, right: bool) -> Option<String> {
    let local = h.pose.inverse() * exit;
    let side: Real = if right { 1.0 } else { -1.0 };
    let p = local.translation;
    if (p.x - at).abs() > MERGE_ALONG {
        return Some(format!(
            "its exit is at {:.3} m along the run, not {at} m",
            p.x
        ));
    }
    let gap = side * p.z - h.width / 2.0;
    if !(-MERGE_SLACK..=MERGE_REACH + MERGE_SLACK).contains(&gap) {
        return Some(format!(
            "its exit is {gap:.3} m off the flank; a merge reaches {MERGE_REACH} m"
        ));
    }
    let into = local.rotation * Vector::X;
    let off = (into.angle_between(Vector::new(0.0, 0.0, -side))).to_degrees();
    if off <= MERGE_SQUARE {
        return None;
    }
    Some(format!("its exit is {off:.2}° from square to the flank"))
}

#[cfg(test)]
#[path = "layout_merge_tests.rs"]
mod tests;
