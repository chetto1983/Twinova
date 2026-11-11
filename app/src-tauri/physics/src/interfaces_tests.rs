use super::*;
use crate::world::tests::fixture;
use twinova_project::model::{Connection, ParamValue};

const FIXTURES: [&str; 5] = [
    "demo_m3",
    "two_belts_diverter",
    "curve_line",
    "sort_line",
    "code_sort_line",
];

fn near(a: Vector, b: Vector) -> bool {
    (a - b).length() < 1e-5
}

/// A frame's x, out of its instance.
fn out(frame: &Pose) -> Vector {
    frame.rotation * Vector::X
}

fn end_of<'a>(all: &'a BTreeMap<String, Interfaces>, id: &str, name: &str) -> &'a End {
    let ends = &all[id].ends;
    (ends.iter().find(|e| e.name == name)).unwrap_or_else(|| panic!("{id}.{name} in {ends:?}"))
}

/// Two ends meet face to face: the same point, opposite outward directions.
fn meet(a: &End, b: &End) -> bool {
    near(a.frame.translation, b.frame.translation) && near(out(&a.frame), -out(&b.frame))
}

/// The fixtures were laid out by a script, independently of this code: wherever a flow joins two conveyors, the
/// upstream one's `out` meets the downstream one's `in`, straights, rollers, curves and the turntable's start heading.
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
            assert!(meet(a, b), "{name}: {a:?} and {b:?}");
            assert_eq!((a.direction, b.direction), (Direction::Out, Direction::In));
            joins += 1;
        }
    }
    assert_eq!(joins, 12);
}

#[test]
fn a_belt_ends_at_its_drum_faces_and_a_roller_conveyor_at_its_frame() {
    let all = interfaces(&fixture("two_belts_diverter"));
    let (a_in, a_out) = (end_of(&all, "belt_a", "in"), end_of(&all, "belt_a", "out"));
    assert!(near(a_in.frame.translation, Vector::new(-0.085, 0.8, 0.0)));
    assert!(near(out(&a_in.frame), Vector::NEG_X));
    assert!(near(a_out.frame.translation, Vector::new(3.085, 0.8, 0.0)));
    assert!(near(out(&a_out.frame), Vector::X));
    let r_out = end_of(&all, "roller_r", "out");
    assert!(near(r_out.frame.translation, Vector::new(5.085, 0.8, 0.0)));
    assert_eq!(all["belt_a"].ends.len(), 2);
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
    let x0 = belt_b.position[0] as Real;
    let all = interfaces(&project);
    let (sin, cos) = 10.0_f32.to_radians().sin_cos();
    let reach = 1.0 + 0.085;
    let exit = end_of(&all, "belt_b", "out");
    assert!(
        near(
            exit.frame.translation,
            Vector::new(x0 + 1.0 + reach * cos, 0.8 + reach * sin, 0.0)
        ),
        "{exit:?}"
    );
    assert!(near(out(&exit.frame), Vector::new(cos, sin, 0.0)));
    let entry = end_of(&all, "belt_b", "in");
    assert!(near(
        entry.frame.translation,
        Vector::new(x0 + 1.0 - reach * cos, 0.8 - reach * sin, 0.0)
    ));
}

/// The sort line's belt C starts at the turntable's left rim, with no flow (the deck turns to it).
#[test]
fn a_turntable_has_rims_across_its_lane_that_take_either_way() {
    let all = interfaces(&fixture("sort_line"));
    let (left, right) = (
        end_of(&all, "turntable", "left"),
        end_of(&all, "turntable", "right"),
    );
    assert!(meet(left, end_of(&all, "belt_c", "in")));
    assert!(near(right.frame.translation, Vector::new(4.535, 0.8, 0.45)));
    assert!(near(out(&right.frame), Vector::Z) && near(out(&left.frame), Vector::NEG_Z));
    assert_eq!(
        (left.direction, right.direction),
        (Direction::Either, Direction::Either)
    );
    assert_eq!(all["turntable"].ends.len(), 4);
}

/// A source's end and a sink's stand at their own pose, on the floor, facing along the flow.
#[test]
fn a_source_and_a_sink_end_where_they_stand() {
    let all = interfaces(&fixture("demo_m3"));
    let source = end_of(&all, "source", "out");
    assert!(near(source.frame.translation, Vector::new(-0.5, 0.0, 0.0)));
    assert!(near(out(&source.frame), Vector::X) && source.direction == Direction::Out);
    let sink = end_of(&all, "sink", "in");
    assert!(near(sink.frame.translation, Vector::new(3.585, 0.0, 0.0)));
    assert!(near(out(&sink.frame), Vector::NEG_X) && sink.direction == Direction::In);
    assert!(all["source"].host.is_none() && all["sensor_a"].ends.is_empty());
}

/// A chain transfer's side exit is at its lane's end, a pusher's at its host's far edge; both to their `out_right` side.
#[test]
fn a_transfer_and_a_pusher_end_at_their_side_exits() {
    let all = interfaces(&fixture("two_belts_diverter"));
    let transfer = end_of(&all, "chain_transfer", "out");
    assert!(near(
        transfer.frame.translation,
        Vector::new(3.085 + 1.0, 0.8, -0.3)
    ));
    assert!(near(out(&transfer.frame), Vector::NEG_Z));
    let all = interfaces(&fixture("sort_line"));
    let pusher = end_of(&all, "pusher", "out");
    assert!(near(
        pusher.frame.translation,
        Vector::new(5.07 + 1.5, 0.8, -0.3)
    ));
    assert!(near(out(&pusher.frame), Vector::NEG_Z));
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
    let all = interfaces(&right);
    let pusher = end_of(&all, "pusher", "out");
    assert!(near(pusher.frame.translation, Vector::new(6.57, 0.8, 0.3)));
    assert!(near(out(&pusher.frame), Vector::Z));
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
    let all = interfaces(&fixture("curve_line"));
    let Some(Host {
        rail: Rail::Line { start, length },
        ..
    }) = &all["belt_a"].host
    else {
        panic!("{:?}", all["belt_a"].host)
    };
    assert!(near(start.translation, Vector::new(0.0, 0.8, 0.0)) && *length == 3.0);
    assert!(near(out(start), Vector::X));
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
    let entry = end_of(&all, "curve_a", "in").frame.translation;
    assert!(near(*start, entry + Vector::new(0.085, 0.0, 0.0)));
    assert!(near(*centre, *start + Vector::new(0.0, 0.0, -0.9)));
    assert!((*sweep - std::f32::consts::FRAC_PI_2).abs() < 1e-6 && !right);
}

#[test]
fn a_rail_accepts_the_devices_its_shape_fits() {
    let sensors = SENSORS.to_vec();
    let with = |extra: &[&'static str]| [sensors.clone(), extra.to_vec()].concat();
    let accepts =
        |all: &BTreeMap<String, Interfaces>, id: &str| all[id].host.clone().unwrap().accepts;
    let all = interfaces(&fixture("two_belts_diverter"));
    assert_eq!(
        accepts(&all, "belt_a"),
        with(&[SCANNER, "twinova.pusher/1"])
    );
    assert_eq!(
        accepts(&all, "roller_r"),
        with(&[
            SCANNER,
            "twinova.chain_transfer/1",
            "twinova.stopper/1",
            "twinova.pusher/1"
        ])
    );
    let all = interfaces(&fixture("curve_line"));
    assert_eq!(accepts(&all, "curve_a"), sensors);
    let all = interfaces(&fixture("sort_line"));
    assert_eq!(accepts(&all, "turntable"), with(&[SCANNER]));
    let mut tilted = fixture("two_belts_diverter");
    let r = (tilted.scene.instances.iter_mut())
        .find(|i| i.id == "roller_r")
        .unwrap();
    r.parameters.insert("skew".into(), ParamValue::Real(2.0));
    assert_eq!(accepts(&interfaces(&tilted), "roller_r"), with(&[SCANNER]));
}
