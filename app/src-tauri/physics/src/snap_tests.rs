use super::*;
use crate::interfaces::place;
use crate::world::tests::fixture;
use twinova_project::edit;
use twinova_project::history::History;
use twinova_project::model::Quat;

/// An empty project to place library families in (the M3 demo, emptied).
fn empty() -> Project {
    let mut p = fixture("demo_m3");
    p.scene.instances.clear();
    p.scene.connections.clear();
    p
}

fn yaw(degrees: f64) -> Quat {
    let q = DQuat::from_rotation_y(degrees.to_radians());
    [q.x, q.y, q.z, q.w]
}

/// Places a family somewhere out of the way, turned, so a join has to move and turn it.
fn scattered(p: &mut Project, key: &str) -> String {
    let n = p.scene.instances.len() as f64;
    edit::place(p, key, [7.0 + n, 0.0, -5.0 - n], yaw(37.0 * (n + 1.0))).unwrap()
}

fn set(p: &mut Project, id: &str, name: &str, value: ParamValue) {
    let inst = p.scene.instances.iter_mut().find(|i| i.id == id).unwrap();
    inst.parameters.insert(name.into(), value);
}

fn at(p: &Project, id: &str, name: &str) -> DPose3 {
    let all = interfaces(p);
    world(p, id, end_of(&all, id, name).unwrap())
}

fn flows(p: &Project) -> Vec<String> {
    (p.scene.connections.iter())
        .filter_map(|c| match c {
            Connection::Flow { from, to } => {
                Some(format!("{}.{} {}.{}", from.0, from.1, to.0, to.1))
            }
            _ => None,
        })
        .collect()
}

fn pose(p: &Project, id: &str) -> DPose3 {
    place(p.scene.instances.iter().find(|i| i.id == id).unwrap())
}

/// Faces `a` and `b` stand `gap` apart, on one axis, facing each other.
fn faced(a: &DPose3, b: &DPose3, gap: f64) {
    let out = a.rotation * DVec3::X;
    let d = b.translation - a.translation;
    assert!((d - out * gap).length() < 1e-6, "{a:?} {b:?}");
    assert!(
        ((b.rotation * DVec3::X) + out).length() < 1e-6,
        "{a:?} {b:?}"
    );
}

#[test]
fn a_conveyor_joins_another_end_to_end_with_a_flow() {
    let mut p = empty();
    let belt = edit::place(&mut p, "twinova.belt@1", [0.0; 3], [0.0, 0.0, 0.0, 1.0]).unwrap();
    let roller = scattered(&mut p, "twinova.roller@1");
    let notice = join(&mut p, (&roller, "in"), (&belt, "out"), 0.0).unwrap();
    assert_eq!(notice, None, "equal heights");
    let r = pose(&p, &roller);
    assert!(
        (r.translation - DVec3::new(3.085, 0.0, 0.0)).length() < 1e-6,
        "{r:?}"
    );
    assert!(r.rotation.angle_between(DQuat::IDENTITY) < 1e-6);
    assert_eq!(flows(&p), [format!("{belt}.out {roller}.in")]);
    // A belt after the roller conveyor stands its drum face, 85 mm, past the roller conveyor's frame end.
    let next = scattered(&mut p, "twinova.belt@1");
    join(&mut p, (&next, "in"), (&roller, "out"), 0.0).unwrap();
    assert!((pose(&p, &next).translation - DVec3::new(5.17, 0.0, 0.0)).length() < 1e-6);
    // Joined the other way round, the upstream part moves: its exit onto the other's entry.
    let first = scattered(&mut p, "twinova.belt@1");
    join(&mut p, (&first, "out"), (&belt, "in"), 0.0).unwrap();
    assert!((pose(&p, &first).translation - DVec3::new(-3.17, 0.0, 0.0)).length() < 1e-6);
    assert!(flows(&p).contains(&format!("{first}.out {belt}.in")));
}

#[test]
fn a_join_leaves_the_gap_asked_between_the_faces() {
    for gap in [0.0, 0.02, GAP_MAX] {
        let mut p = empty();
        let a = edit::place(&mut p, "twinova.belt@1", [1.0, 0.0, 2.0], yaw(30.0)).unwrap();
        let b = scattered(&mut p, "twinova.belt@1");
        join(&mut p, (&b, "in"), (&a, "out"), gap).unwrap();
        faced(&at(&p, &a, "out"), &at(&p, &b, "in"), gap);
    }
    let mut p = empty();
    let a = edit::place(&mut p, "twinova.belt@1", [0.0; 3], [0.0, 0.0, 0.0, 1.0]).unwrap();
    let b = scattered(&mut p, "twinova.belt@1");
    let before = p.clone();
    for gap in [-0.001, 0.151] {
        let error = join(&mut p, (&b, "in"), (&a, "out"), gap).unwrap_err();
        assert!(error.contains("is outside 0 to 0.15 m"), "{error}");
    }
    assert_eq!(p, before, "a refused join changes nothing");
}

#[test]
fn curves_turn_the_line_left_and_right() {
    for (right, heading) in [(false, 90.0), (true, -90.0)] {
        let mut p = empty();
        let a = edit::place(&mut p, "twinova.belt@1", [0.0; 3], [0.0, 0.0, 0.0, 1.0]).unwrap();
        let curve = scattered(&mut p, "twinova.belt_curve@1");
        set(&mut p, &curve, "turn_right", ParamValue::Bool(right));
        join(&mut p, (&curve, "in"), (&a, "out"), 0.0).unwrap();
        let b = scattered(&mut p, "twinova.roller@1");
        join(&mut p, (&b, "in"), (&curve, "out"), 0.0).unwrap();
        faced(&at(&p, &curve, "out"), &at(&p, &b, "in"), 0.0);
        let turned = pose(&p, &b).rotation * DVec3::X;
        assert!(
            (turned - DQuat::from_rotation_y(f64::to_radians(heading)) * DVec3::X).length() < 1e-6
        );
        assert_eq!(flows(&p).len(), 2);
    }
}

/// A turntable takes a flow at its start heading; a conveyor at its rim takes none, as the sort line's belt C.
#[test]
fn a_turntable_joins_by_its_lane_with_a_flow_and_by_its_rims_without() {
    let mut p = empty();
    let t = edit::place(
        &mut p,
        "twinova.turntable@1",
        [2.0, 0.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    )
    .unwrap();
    let a = scattered(&mut p, "twinova.belt@1");
    join(&mut p, (&a, "out"), (&t, "in"), 0.0).unwrap();
    let c = scattered(&mut p, "twinova.belt@1");
    join(&mut p, (&c, "in"), (&t, "left"), 0.0).unwrap();
    faced(&at(&p, &t, "left"), &at(&p, &c, "in"), 0.0);
    assert_eq!(flows(&p), [format!("{a}.out {t}.in")]);
    // The turntable itself moved onto a belt by a rim, a quarter turn from its lane: it turns to face it.
    let d = edit::place(&mut p, "twinova.belt@1", [0.0, 0.0, 9.0], yaw(-20.0)).unwrap();
    join(&mut p, (&t, "right"), (&d, "out"), 0.0).unwrap();
    faced(&at(&p, &d, "out"), &at(&p, &t, "right"), 0.0);
}

#[test]
fn sources_and_sinks_stand_on_the_floor_at_the_ends_they_join() {
    let mut p = fixture("two_belts_diverter");
    for (id, end, peer, peer_end) in [
        ("source", "out", "belt_a", "in"),
        ("sink_ok", "in", "belt_b", "out"),
        ("sink_reject", "in", "chain_transfer", "out"),
    ] {
        let inst = p.scene.instances.iter_mut().find(|i| i.id == id).unwrap();
        (inst.position, inst.rotation) = ([9.0, 0.0, 9.0], yaw(123.0));
        edit::set_flow(&mut p, id, end, None).unwrap();
        join(&mut p, (id, end), (peer, peer_end), 0.0).unwrap();
        let (mine, theirs) = (at(&p, id, end), at(&p, peer, peer_end));
        assert!(mine.translation.y.abs() < 1e-6, "{id} on the floor");
        let floor = |v: DVec3| DVec3::new(v.x, 0.0, v.z);
        assert!(
            (floor(mine.translation) - floor(theirs.translation)).length() < 1e-6,
            "{id}"
        );
        assert!(
            ((mine.rotation * DVec3::X) + theirs.rotation * DVec3::X).length() < 1e-6,
            "{id}"
        );
    }
    let sorted = |mut v: Vec<String>| {
        v.sort();
        v
    };
    assert_eq!(
        sorted(flows(&p)),
        sorted(flows(&fixture("two_belts_diverter")))
    );
}

#[test]
fn the_moved_conveyor_takes_the_height_that_meets_the_other() {
    let mut p = empty();
    let a = edit::place(&mut p, "twinova.belt@1", [0.0; 3], [0.0, 0.0, 0.0, 1.0]).unwrap();
    set(&mut p, &a, "height", ParamValue::Real(0.95));
    let b = scattered(&mut p, "twinova.belt@1");
    let notice = join(&mut p, (&b, "in"), (&a, "out"), 0.0).unwrap();
    assert_eq!(
        notice.as_deref(),
        Some(format!("{b}'s height is now 0.950 m, to meet it").as_str())
    );
    faced(&at(&p, &a, "out"), &at(&p, &b, "in"), 0.0);
    // An inclined belt's entry is below its height: its height is raised until the entry meets the surface.
    let c = scattered(&mut p, "twinova.belt@1");
    set(&mut p, &c, "incline", ParamValue::Real(10.0));
    join(&mut p, (&c, "in"), (&b, "out"), 0.02).unwrap();
    let (out, entry) = (at(&p, &b, "out"), at(&p, &c, "in"));
    assert!((entry.translation.y - out.translation.y).abs() < 1e-6);
    let rise = (1.5 + 0.085) * 10.0_f64.to_radians().sin();
    let inst = p.scene.instances.iter().find(|i| i.id == c).unwrap();
    assert!(
        matches!(inst.parameters["height"], ParamValue::Real(h) if (h - 0.95 - rise).abs() < 1e-6)
    );
}

#[test]
fn a_join_is_refused_with_its_reason() {
    let mut p = fixture("two_belts_diverter");
    let long = scattered(&mut p, "twinova.belt@1");
    set(&mut p, &long, "length", ParamValue::Real(10.0));
    set(&mut p, &long, "incline", ParamValue::Real(30.0));
    let before = p.clone();
    let refusals = [
        (
            ("belt_b", "in"),
            ("roller_r", "in"),
            "belt_b.in and roller_r.in both take products in",
        ),
        (
            ("belt_b", "out"),
            ("roller_r", "out"),
            "belt_b.out and roller_r.out both send products out",
        ),
        (
            ("belt_b", "in"),
            ("belt_b", "out"),
            "belt_b cannot join itself",
        ),
        (
            ("belt_b", "side"),
            ("roller_r", "out"),
            "belt_b has no end side",
        ),
        (("ghost", "in"), ("roller_r", "out"), "no instance ghost"),
        (
            ("chain_transfer", "out"),
            ("sink_ok", "in"),
            "join the other part to it",
        ),
        (
            (long.as_str(), "in"),
            ("belt_a", "out"),
            "outside its height's range of 0 to 3 m",
        ),
    ];
    for (mine, theirs, message) in refusals {
        let error = join(&mut p, mine, theirs, 0.0).unwrap_err();
        assert!(error.contains(message), "{error}");
    }
    assert_eq!(p, before);
}

/// A move keeps the flows whose faces still meet within the gap's reach and drops the others (decision 3).
#[test]
fn a_move_keeps_the_joins_that_hold_and_drops_the_others() {
    let shift = |dx: f64, dz: f64, turn: f64| {
        let mut p = fixture("two_belts_diverter");
        let inst = p
            .scene
            .instances
            .iter_mut()
            .find(|i| i.id == "belt_b")
            .unwrap();
        inst.position[0] += dx;
        inst.position[2] += dz;
        inst.rotation = yaw(turn);
        moved(&mut p, "belt_b");
        flows(&p).contains(&"roller_r.out belt_b.in".to_string())
    };
    assert!(shift(0.0, 0.0, 0.0) && shift(0.1, 0.0, 0.0) && shift(0.15, 0.0, 0.0));
    assert!(!shift(0.152, 0.0, 0.0), "beyond the gap's reach");
    assert!(!shift(-0.002, 0.0, 0.0), "into the other");
    assert!(!shift(0.0, 0.002, 0.0), "off the axis");
    assert!(!shift(0.0, 0.0, 0.2), "turned");
    // The sink's flow is not a join between conveyors: it stays wherever the belt goes.
    let mut p = fixture("two_belts_diverter");
    p.scene
        .instances
        .iter_mut()
        .find(|i| i.id == "belt_b")
        .unwrap()
        .position[2] += 1.0;
    moved(&mut p, "belt_b");
    assert!(flows(&p).contains(&"belt_b.out sink_ok.in".to_string()));
}

#[test]
fn one_undo_takes_back_the_whole_join() {
    let mut p = empty();
    let a = edit::place(&mut p, "twinova.belt@1", [0.0; 3], [0.0, 0.0, 0.0, 1.0]).unwrap();
    set(&mut p, &a, "height", ParamValue::Real(0.9));
    let b = scattered(&mut p, "twinova.belt@1");
    let before = p.clone();
    let mut history = History::default();
    history
        .apply(&mut p, |p| join(p, (&b, "in"), (&a, "out"), 0.0))
        .unwrap();
    assert_ne!(p, before);
    assert!(history.undo(&mut p));
    assert_eq!(p.scene, before.scene);
}
