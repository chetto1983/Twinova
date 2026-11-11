//! `merge` and the joins `moved` keeps and drops (spec 2026-11-11-snapping-design.md, decisions 3 and 5).

use glamx::{DPose3, DQuat, DVec3};
use twinova_project::Project;
use twinova_project::edit;
use twinova_project::model::{Connection, ParamValue, Side};

use super::{join, merge, moved};
use crate::interfaces::{Host, Rail, interfaces, place, world};
use crate::world::tests::fixture;

fn yawed(degrees: f64) -> [f64; 4] {
    let q = DQuat::from_rotation_y(degrees.to_radians());
    [q.x, q.y, q.z, q.w]
}

fn exit(p: &Project, id: &str) -> DPose3 {
    let all = interfaces(p);
    let end = all[id].ends.iter().find(|e| e.name == "out").unwrap();
    world(p, id, end)
}

fn merges(p: &Project) -> Vec<(String, String, f64, Side)> {
    (p.scene.connections.iter())
        .filter_map(|c| match c {
            Connection::Merge {
                from,
                host,
                at,
                side,
            } => Some((from.0.clone(), host.0.clone(), *at, *side)),
            _ => None,
        })
        .collect()
}

/// A belt turned 30° away from the origin, and two spurs scattered elsewhere.
fn line() -> (Project, String, String, String) {
    let mut p = fixture("demo_m3");
    p.scene.instances.clear();
    p.scene.connections.clear();
    let host = edit::place(&mut p, "twinova.belt@1", [1.0, 0.0, 2.0], yawed(30.0)).unwrap();
    let a = edit::place(&mut p, "twinova.belt@1", [9.0, 0.0, 9.0], yawed(200.0)).unwrap();
    let b = edit::place(&mut p, "twinova.roller@1", [-9.0, 0.0, 9.0], yawed(-70.0)).unwrap();
    (p, host, a, b)
}

/// The spur's exit face stands `gap` off the host's flank at `at` on its `side`, square to it, at its surface height.
#[test]
fn a_spur_merges_square_onto_either_flank_with_a_gap() {
    let (mut p, host, a, b) = line();
    merge(&mut p, &a, (&host, 1.2, Side::Left), 0.04).unwrap();
    merge(&mut p, &b, (&host, 2.0, Side::Right), 0.0).unwrap();
    let all = interfaces(&p);
    let Some(Host {
        rail: Rail::Line { start, .. },
        width,
        ..
    }) = &all[&host].host
    else {
        panic!()
    };
    let base = place(p.scene.instances.iter().find(|i| i.id == host).unwrap()) * *start;
    for (spur, at, sign, gap) in [(&a, 1.2, -1.0, 0.04), (&b, 2.0, 1.0, 0.0)] {
        let outward = base.rotation * DVec3::new(0.0, 0.0, sign);
        let flank = base.transform_point(DVec3::new(at, 0.0, sign * width / 2.0));
        let face = exit(&p, spur);
        assert!(
            (face.translation - (flank + outward * gap)).length() < 1e-6,
            "{spur}: {face:?}"
        );
        assert!(
            ((face.rotation * DVec3::X) + outward).length() < 1e-6,
            "{spur} faces the flank"
        );
    }
    assert_eq!(
        merges(&p),
        [
            (a.clone(), host.clone(), 1.2, Side::Left),
            (b.clone(), host.clone(), 2.0, Side::Right)
        ]
    );
}

#[test]
fn a_merge_is_refused_with_its_reason() {
    let (mut p, host, a, b) = line();
    let curve = edit::place(&mut p, "twinova.belt_curve@1", [5.0, 0.0, -5.0], yawed(0.0)).unwrap();
    let tilted = edit::place(&mut p, "twinova.roller@1", [0.0, 0.0, -9.0], yawed(0.0)).unwrap();
    p.scene
        .instances
        .iter_mut()
        .find(|i| i.id == tilted)
        .unwrap()
        .parameters
        .insert("incline".into(), ParamValue::Real(5.0));
    let source = edit::place(&mut p, "twinova.source@1", [3.0, 0.0, 3.0], yawed(0.0)).unwrap();
    merge(&mut p, &a, (&host, 1.0, Side::Left), 0.0).unwrap();
    let before = p.clone();
    /// A spur, where it merges, the gap, and the refusal.
    type Refusal<'a> = (&'a str, (&'a str, f64, Side), f64, &'a str);
    let refusals: [Refusal; 9] = [
        (
            &b,
            (&host, 1.0, Side::Left),
            -0.01,
            "a gap of -0.01 m is outside 0 to 0.15 m",
        ),
        (
            &b,
            (&host, 1.0, Side::Left),
            0.16,
            "a gap of 0.16 m is outside",
        ),
        (
            &host,
            (&host, 1.0, Side::Left),
            0.0,
            "cannot merge onto itself",
        ),
        (
            &b,
            ("ghost", 1.0, Side::Left),
            0.0,
            "ghost is not a conveyor",
        ),
        (&b, (&source, 1.0, Side::Left), 0.0, "is not a conveyor"),
        (
            &b,
            (&curve, 1.0, Side::Left),
            0.0,
            "is a curve: a spur merges onto a straight",
        ),
        (&b, (&tilted, 1.0, Side::Left), 0.0, "takes no spurs"),
        (&b, (&host, 3.5, Side::Left), 0.0, "3.5 m is outside"),
        (&source, (&host, 2.0, Side::Right), 0.0, "is not a conveyor"),
    ];
    for (spur, on, gap, message) in refusals {
        let error = merge(&mut p, spur, on, gap).unwrap_err();
        assert!(error.contains(message), "{error}");
    }
    let error = merge(&mut p, &b, (&host, 1.3, Side::Left), 0.0).unwrap_err();
    assert!(error.contains("over"), "spurs on one side overlap: {error}");
    assert_eq!(p, before, "a refused merge changes nothing");
}

/// `moved` checks only the joins of what moved: a spur or a host that moved off loses its merge, the rest keeps theirs,
/// even a join that another, unchecked move broke.
#[test]
fn a_move_drops_only_the_moved_parts_broken_joins() {
    let shifted = |id: &str, check: &str| {
        let mut p = fixture("merge_line");
        p.scene
            .instances
            .iter_mut()
            .find(|i| i.id == id)
            .unwrap()
            .position[0] += 0.5;
        moved(&mut p, check);
        merges(&p).len()
    };
    assert_eq!(shifted("belt_a", "belt_a"), 0, "the host moved off");
    assert_eq!(shifted("belt_s", "belt_s"), 0, "the spur moved off");
    assert_eq!(shifted("belt_a", "sink"), 1, "the sink did not move");
    assert_eq!(shifted("sink", "sink"), 1);
    let mut p = fixture("two_belts_diverter");
    p.scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_b")
        .unwrap()
        .position[2] += 1.0;
    moved(&mut p, "belt_a");
    let flow = Connection::Flow {
        from: ("roller_r".into(), "out".into()),
        to: ("belt_b".into(), "in".into()),
    };
    assert!(
        p.scene.connections.contains(&flow),
        "belt A's move checks only belt A's joins"
    );
}

/// A matched height is set to the micrometre, so it reaches its range's ends and no further.
#[test]
fn a_matched_height_reaches_the_ends_of_its_range_and_no_further() {
    for (height, incline, joins) in [(0.0, 0.0, true), (3.0, 0.0, true), (0.8, -30.0, false)] {
        let (mut p, host, a, _) = line();
        let set = |p: &mut Project, id: &str, name: &str, v: f64| {
            let inst = p.scene.instances.iter_mut().find(|i| i.id == id).unwrap();
            inst.parameters.insert(name.into(), ParamValue::Real(v));
        };
        set(&mut p, &host, "height", height);
        set(&mut p, &a, "incline", incline);
        set(&mut p, &a, "length", 10.0);
        let result = join(&mut p, (&a, "in"), (&host, "out"), 0.0);
        assert_eq!(result.is_ok(), joins, "{height} {incline}: {result:?}");
        if joins {
            let inst = p.scene.instances.iter().find(|i| i.id == a).unwrap();
            assert_eq!(inst.parameters["height"], ParamValue::Real(height));
        } else {
            assert!(result.unwrap_err().contains("would stand -1.74"));
        }
    }
}
