//! Acceptance 3 of spec 2026-11-11-snapping-design.md: the fixtures, laid out by a script before this code existed,
//! rebuilt from their instances scattered anywhere, with joins (gap 0), merges and mounts only.

use std::collections::BTreeSet;

use glamx::DQuat;
use twinova_project::Project;
use twinova_project::edit::set_mount;
use twinova_project::model::{Connection, Instance};

use super::{join, merge};
use crate::interfaces::{interfaces, place};
use crate::world::tests::fixture;

fn conveyor(p: &Project, id: &str) -> bool {
    interfaces(p)[id].host.is_some()
}

/// Rebuilds fixture `name` from `anchor`, the one conveyor left where it was, joining the others and turntable `rims`
/// (conveyors that start at a rim, with no flow) as the fixture's connections say. Returns the fixture and the rebuild.
fn rebuild(name: &str, anchor: &str, rims: &[(&str, &str)]) -> (Project, Project) {
    let original = fixture(name);
    let mut p = original.clone();
    for (k, inst) in p.scene.instances.iter_mut().enumerate() {
        if inst.id != anchor {
            let q = DQuat::from_rotation_y(0.7 * k as f64 + 0.3);
            (inst.position, inst.rotation) =
                ([5.0 - k as f64, 0.0, 3.0 + k as f64], [q.x, q.y, q.z, q.w]);
        }
    }
    p.scene.connections.clear();
    let mut placed: BTreeSet<String> = BTreeSet::from([anchor.to_string()]);
    let host_of = |id: &str| {
        original.scene.connections.iter().find_map(|c| match c {
            Connection::Mount { device, host, .. } if device.0 == id => Some(host.0.clone()),
            _ => None,
        })
    };
    for c in &original.scene.connections {
        if let Connection::Mount {
            device,
            host,
            position,
            ..
        } = c
        {
            set_mount(&mut p, &device.0, Some((&host.0, position[0]))).unwrap();
        }
    }
    let ready = |placed: &BTreeSet<String>, id: &str| {
        placed.contains(id) || host_of(id).is_some_and(|h| placed.contains(&h))
    };
    let mut pending: Vec<&Connection> = original.scene.connections.iter().collect();
    let mut rims: Vec<(&str, &str)> = rims.to_vec();
    while !pending.is_empty() || !rims.is_empty() {
        let before = pending.len() + rims.len();
        pending.retain(|c| match c {
            Connection::Flow { from, to } => {
                let (f, t) = (ready(&placed, &from.0), ready(&placed, &to.0));
                if f && !t {
                    join(&mut p, (&to.0, &to.1), (&from.0, &from.1), 0.0).unwrap();
                    placed.insert(to.0.clone());
                } else if t && !f {
                    join(&mut p, (&from.0, &from.1), (&to.0, &to.1), 0.0).unwrap();
                    placed.insert(from.0.clone());
                } else if f && t {
                    twinova_project::edit::set_flow(&mut p, &from.0, &from.1, Some((&to.0, &to.1)))
                        .unwrap();
                } else {
                    return true;
                }
                false
            }
            Connection::Merge {
                from,
                host,
                at,
                side,
            } if placed.contains(&host.0) => {
                merge(&mut p, &from.0, (&host.0, *at, *side), 0.0).unwrap();
                placed.insert(from.0.clone());
                false
            }
            Connection::Merge { .. } => true,
            Connection::Mount { .. } => false,
        });
        rims.retain(|(belt, turntable)| {
            if !placed.contains(*turntable) {
                return true;
            }
            join(&mut p, (belt, "in"), (turntable, "left"), 0.0).unwrap();
            placed.insert(belt.to_string());
            false
        });
        assert!(
            pending.len() + rims.len() < before,
            "{name}: stuck on {pending:?} {rims:?}"
        );
    }
    (original, p)
}

fn instance<'a>(p: &'a Project, id: &str) -> &'a Instance {
    p.scene.instances.iter().find(|i| i.id == id).unwrap()
}

fn connections(p: &Project) -> BTreeSet<String> {
    p.scene
        .connections
        .iter()
        .map(|c| format!("{c:?}"))
        .collect()
}

/// Every conveyor stands where the fixture has it, within 1e-6 m and 1e-6 rad, at its height; the connections are the
/// fixture's, a mount's distance included.
fn same(name: &str, anchor: &str, rims: &[(&str, &str)]) {
    let (original, rebuilt) = rebuild(name, anchor, rims);
    let mut conveyors = 0;
    for inst in &original.scene.instances {
        if !conveyor(&original, &inst.id) {
            continue;
        }
        let (a, b) = (place(inst), place(instance(&rebuilt, &inst.id)));
        let moved = (a.translation - b.translation).length();
        let turned = a.rotation.angle_between(b.rotation);
        assert!(
            moved < 1e-6 && turned < 1e-6,
            "{name}: {} off by {moved} m, {turned} rad",
            inst.id
        );
        assert_eq!(
            inst.parameters,
            instance(&rebuilt, &inst.id).parameters,
            "{name}: {}",
            inst.id
        );
        conveyors += 1;
    }
    assert!(conveyors >= 2, "{name}");
    assert_eq!(connections(&rebuilt), connections(&original), "{name}");
}

#[test]
fn the_curve_line_rebuilds_from_joins() {
    same("curve_line", "belt_a", &[]);
}

#[test]
fn the_sort_lines_rebuild_from_joins_and_a_rim() {
    same("sort_line", "belt_in", &[("belt_c", "turntable")]);
    same("code_sort_line", "belt_in", &[("belt_c", "turntable")]);
}

#[test]
fn the_line_with_a_diverter_rebuilds_from_joins_and_mounts() {
    same("two_belts_diverter", "belt_a", &[]);
}

#[test]
fn the_merge_line_rebuilds_from_a_merge() {
    same("merge_line", "belt_a", &[]);
}
