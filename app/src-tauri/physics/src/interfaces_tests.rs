use super::*;
use crate::world::tests::fixture;
use twinova_project::model::{Connection, ParamValue};

const FIXTURES: [&str; 6] = [
    "demo_m3",
    "two_belts_diverter",
    "curve_line",
    "sort_line",
    "code_sort_line",
    "merge_line",
];

fn near(a: DVec3, b: [f64; 3]) -> bool {
    (a - DVec3::from(b)).length() < 1e-5
}

/// A frame's x, out of its instance.
fn out(frame: &DPose3) -> DVec3 {
    frame.rotation * DVec3::X
}

fn end_of<'a>(all: &'a BTreeMap<String, Interfaces>, id: &str, name: &str) -> &'a End {
    let ends = &all[id].ends;
    (ends.iter().find(|e| e.name == name)).unwrap_or_else(|| panic!("{id}.{name} in {ends:?}"))
}

/// Where `id`'s end `name` is in the world.
fn at(project: &Project, id: &str, name: &str) -> DPose3 {
    world(project, id, end_of(&interfaces(project), id, name))
}

/// Two ends meet face to face: the same point, opposite outward directions.
fn meet(a: &DPose3, b: &DPose3) -> bool {
    (a.translation - b.translation).length() < 1e-6 && (out(a) + out(b)).length() < 1e-6
}

/// The fixtures were laid out by a script, independently of this code: wherever a flow joins two conveyors, the
/// upstream one's `out` meets the downstream one's `in` within a micrometre (in f64, from frames each computed with
/// its instance at the origin), straights, rollers, curves and the turntable's start heading.
#[test]
fn every_conveyor_flow_in_the_fixtures_joins_ends_that_meet() {
    let mut joins = 0;
    for name in FIXTURES {
        let project = fixture(name);
        let all = interfaces(&project);
        for c in &project.scene.connections {
            let Connection::Flow { from, to } = c else {
                continue;
            };
            if all[&from.0].host.is_none() || all[&to.0].host.is_none() {
                continue;
            }
            let (a, b) = (end_of(&all, &from.0, &from.1), end_of(&all, &to.0, &to.1));
            let (wa, wb) = (world(&project, &from.0, a), world(&project, &to.0, b));
            assert!(meet(&wa, &wb), "{name}: {wa:?} and {wb:?}");
            assert_eq!((a.direction, b.direction), (Direction::Out, Direction::In));
            joins += 1;
        }
    }
    assert_eq!(joins, 12);
}

#[test]
fn ends_are_in_their_own_instances_frame() {
    let project = fixture("two_belts_diverter");
    let all = interfaces(&project);
    let r_in = end_of(&all, "roller_r", "in");
    assert!(near(r_in.frame.translation, [0.0, 0.8, 0.0]) && r_in.on.is_none());
    assert!(near(
        at(&project, "roller_r", "in").translation,
        [3.085, 0.8, 0.0]
    ));
}

#[test]
fn a_belt_ends_at_its_drum_faces_and_a_roller_conveyor_at_its_frame() {
    let project = fixture("two_belts_diverter");
    let (a_in, a_out) = (at(&project, "belt_a", "in"), at(&project, "belt_a", "out"));
    assert!(near(a_in.translation, [-0.085, 0.8, 0.0]));
    assert!(near(out(&a_in), [-1.0, 0.0, 0.0]));
    assert!(near(a_out.translation, [3.085, 0.8, 0.0]));
    assert!(near(out(&a_out), [1.0, 0.0, 0.0]));
    assert!(near(
        at(&project, "roller_r", "out").translation,
        [5.085, 0.8, 0.0]
    ));
    assert_eq!(interfaces(&project)["belt_a"].ends.len(), 2);
}

/// An inclined belt pivots about the centre of its surface (OIP `tiltDeg`): its exit face lies (L/2 + face) along the
/// slope from there, and faces up the slope.
#[test]
fn an_inclined_belt_ends_on_its_slope() {
    let mut project = fixture("two_belts_diverter");
    let belt_b = (project.scene.instances.iter_mut())
        .find(|i| i.id == "belt_b")
        .unwrap();
    belt_b
        .parameters
        .insert("incline".into(), ParamValue::Real(10.0));
    let x0 = belt_b.position[0];
    let (sin, cos) = 10.0_f64.to_radians().sin_cos();
    let reach = 1.0 + 0.085;
    let exit = at(&project, "belt_b", "out");
    assert!(
        near(
            exit.translation,
            [x0 + 1.0 + reach * cos, 0.8 + reach * sin, 0.0]
        ),
        "{exit:?}"
    );
    assert!(near(out(&exit), [cos, sin, 0.0]));
    let entry = at(&project, "belt_b", "in");
    assert!(near(
        entry.translation,
        [x0 + 1.0 - reach * cos, 0.8 - reach * sin, 0.0]
    ));
}

/// The sort line's belt C starts at the turntable's left rim, with no flow (the deck turns to it).
#[test]
fn a_turntable_has_rims_across_its_lane_that_take_either_way() {
    let project = fixture("sort_line");
    let all = interfaces(&project);
    let (left, right) = (
        at(&project, "turntable", "left"),
        at(&project, "turntable", "right"),
    );
    assert!(meet(&left, &at(&project, "belt_c", "in")));
    assert!(near(right.translation, [4.535, 0.8, 0.45]));
    assert!(near(out(&right), [0.0, 0.0, 1.0]) && near(out(&left), [0.0, 0.0, -1.0]));
    let directions = (
        end_of(&all, "turntable", "left").direction,
        end_of(&all, "turntable", "right").direction,
    );
    assert_eq!(directions, (Direction::Either, Direction::Either));
    assert_eq!(all["turntable"].ends.len(), 4);
}

/// A source's end and a sink's stand at their own pose, on the floor, facing along the flow.
#[test]
fn a_source_and_a_sink_end_where_they_stand() {
    let project = fixture("demo_m3");
    let all = interfaces(&project);
    let source = at(&project, "source", "out");
    assert!(near(source.translation, [-0.5, 0.0, 0.0]) && near(out(&source), [1.0, 0.0, 0.0]));
    assert_eq!(end_of(&all, "source", "out").direction, Direction::Out);
    let sink = at(&project, "sink", "in");
    assert!(near(sink.translation, [3.585, 0.0, 0.0]) && near(out(&sink), [-1.0, 0.0, 0.0]));
    assert_eq!(end_of(&all, "sink", "in").direction, Direction::In);
    assert!(all["source"].host.is_none() && all["sensor_a"].ends.is_empty());
}

/// A chain transfer's side exit is at its lane's end, a pusher's at its host's far edge; both to their `out_right`
/// side, in their host's frame.
#[test]
fn a_transfer_and_a_pusher_end_at_their_side_exits() {
    let project = fixture("two_belts_diverter");
    let transfer = end_of(&interfaces(&project), "chain_transfer", "out").clone();
    assert_eq!(transfer.on.as_deref(), Some("roller_r"));
    let transfer = world(&project, "chain_transfer", &transfer);
    assert!(near(transfer.translation, [3.085 + 1.0, 0.8, -0.3]));
    assert!(near(out(&transfer), [0.0, 0.0, -1.0]));
    let project = fixture("sort_line");
    let pusher = at(&project, "pusher", "out");
    assert!(near(pusher.translation, [5.07 + 1.5, 0.8, -0.3]));
    assert!(near(out(&pusher), [0.0, 0.0, -1.0]));
    let mut right = fixture("sort_line");
    let p = (right.scene.instances.iter_mut())
        .find(|i| i.id == "pusher")
        .unwrap();
    p.parameters
        .insert("out_right".into(), ParamValue::Bool(true));
    // Build refuses a pusher whose exit belt B's right guard closes, so it has no end until the guard is off.
    assert!(interfaces(&right)["pusher"].ends.is_empty());
    let b = (right.scene.instances.iter_mut())
        .find(|i| i.id == "belt_b")
        .unwrap();
    b.parameters
        .insert("right_guard".into(), ParamValue::Bool(false));
    let pusher = at(&right, "pusher", "out");
    assert!(near(pusher.translation, [6.57, 0.8, 0.3]));
    assert!(near(out(&pusher), [0.0, 0.0, 1.0]));
}

/// A device that is not mounted has no side exit, and the rest of the line keeps its interfaces.
#[test]
fn a_device_not_mounted_has_no_end_and_spoils_nothing() {
    let mut project = fixture("two_belts_diverter");
    project
        .scene
        .connections
        .retain(|c| !matches!(c, Connection::Mount { device, .. } if device.0 == "chain_transfer"));
    let all = interfaces(&project);
    assert!(all["chain_transfer"].ends.is_empty());
    assert_eq!(all["belt_b"].ends.len(), 2);
    assert_eq!(all.len(), project.scene.instances.len());
}

#[test]
fn a_straight_rail_runs_along_the_run_and_a_curve_rail_on_the_arc() {
    let project = fixture("curve_line");
    let all = interfaces(&project);
    let Some(Host {
        rail: Rail::Line { start, length },
        ..
    }) = &all["belt_b"].host
    else {
        panic!("{:?}", all["belt_b"].host)
    };
    assert!(near(start.translation, [0.0, 0.8, 0.0]) && *length == 2.0);
    assert!(near(out(start), [1.0, 0.0, 0.0]));
    let Some(Host {
        rail:
            Rail::Arc {
                centre,
                start,
                sweep,
                right,
            },
        ..
    }) = &all["curve_a"].host
    else {
        panic!("{:?}", all["curve_a"].host)
    };
    // Curve A (inner radius 0.6, width 0.6, 90° to the left) starts its arc after its 85 mm entry stub.
    assert!(near(*start, [0.085, 0.8, 0.0]));
    assert!(near(*centre, [0.085, 0.8, -0.9]));
    assert!((*sweep - std::f64::consts::FRAC_PI_2).abs() < 1e-6 && !right);
}

#[test]
fn a_rail_accepts_the_devices_and_the_spurs_its_shape_fits() {
    let sensors = SENSORS.to_vec();
    let with = |extra: &[&'static str]| [sensors.clone(), extra.to_vec()].concat();
    let host = |all: &BTreeMap<String, Interfaces>, id: &str| all[id].host.clone().unwrap();
    let all = interfaces(&fixture("two_belts_diverter"));
    assert_eq!(
        host(&all, "belt_a").accepts,
        with(&[SCANNER, "twinova.pusher/1"])
    );
    assert_eq!(
        host(&all, "roller_r").accepts,
        with(&[
            SCANNER,
            "twinova.chain_transfer/1",
            "twinova.stopper/1",
            "twinova.pusher/1"
        ])
    );
    assert!(host(&all, "belt_a").spurs && host(&all, "roller_r").spurs);
    let all = interfaces(&fixture("curve_line"));
    assert_eq!(host(&all, "curve_a").accepts, sensors);
    assert!(!host(&all, "curve_a").spurs);
    let all = interfaces(&fixture("sort_line"));
    assert_eq!(host(&all, "turntable").accepts, with(&[SCANNER]));
    assert!(!host(&all, "turntable").spurs);
    let mut tilted = fixture("two_belts_diverter");
    let r = (tilted.scene.instances.iter_mut())
        .find(|i| i.id == "roller_r")
        .unwrap();
    r.parameters.insert("skew".into(), ParamValue::Real(2.0));
    let all = interfaces(&tilted);
    assert_eq!(host(&all, "roller_r").accepts, with(&[SCANNER]));
    assert!(!host(&all, "roller_r").spurs);
}
