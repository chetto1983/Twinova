//! OIP's diverter, a pusher (spec 2026-11-11-stopper-pusher-turntable-design.md, acceptance 2): it sweeps a passing
//! carton across its host onto a takeaway, upright and square, or into a sink at its side.

use super::tests::{belt, fixture, source};
use super::*;
use crate::layout::{Opening, PUSHER_OPENING, Pusher, Sink, layout};
use twinova_project::model::{Connection, ParamValue};

/// OIP's `divertTime`, 0.25 s, as the pusher's block slews it: one millisecond of stroke per step.
const PUSH_MS: usize = 250;

/// A 4 m belt at 0.5 m/s with a pusher at 2 m on its right, pushing left; a takeaway belt running left from beside it
/// when `takeaway`; one carton.
fn line(takeaway: bool, sinks: Vec<Sink>) -> World {
    let mut host = belt("h", 0.0, 4.0, 0.5, true);
    host.guards = [false, true];
    host.openings = vec![Opening {
        right: true,
        centre: 2.0,
        length: PUSHER_OPENING,
    }];
    let mut belts = vec![host];
    if takeaway {
        let mut t = belt("t", 0.0, 3.0, 0.5, true);
        t.pose = Pose::from_parts(
            Vector::new(2.0, 0.0, -0.32),
            Rotation::from_rotation_y(std::f32::consts::FRAC_PI_2),
        );
        t.guards = [true; 2];
        belts.push(t);
    }
    let mut w = World::new(Layout {
        pushers: vec![Pusher {
            id: "p".into(),
            host: 0,
            at: 2.0,
            stroke: 0.75,
            out_right: false,
        }],
        belts,
        sources: vec![source(0, 1_000_000, 1_000_000_000, 1)],
        sinks,
        ..Layout::default()
    });
    for k in 0..w.belts() {
        w.set_belt_running(k, true);
    }
    w
}

/// Runs until the carton's centre passes the pusher, then sweeps it as the PLC does: extend until the end of the
/// stroke, then back; `then` more milliseconds after. Returns the carton's worst tilt.
fn push(w: &mut World, then: usize) -> Real {
    while w.cartons().first().is_none_or(|c| c.position[0] < 2.0) {
        w.step();
    }
    let mut worst: Real = 0.0;
    for ms in 1..=(2 * PUSH_MS + then) {
        let travel = if ms <= PUSH_MS {
            ms
        } else {
            (2 * PUSH_MS).saturating_sub(ms)
        };
        w.set_pusher(0, travel as Real / PUSH_MS as Real);
        w.step();
        if let Some(&(_, h)) = w.cartons.first() {
            let r = *w.world.bodies[h].rotation();
            worst = worst.max((r * Vector::Y).angle_between(Vector::Y).to_degrees());
        }
    }
    worst
}

#[test]
fn a_pushed_carton_rides_a_takeaway_away_upright_and_square() {
    let mut w = line(true, vec![]);
    let worst = push(&mut w, 2000);
    let body = &w.world.bodies[w.cartons[0].1];
    let (p, v, r) = (body.translation(), body.linvel(), *body.rotation());
    assert!(worst < 5.0, "tilted up to {worst}°");
    let tilt = (r * Vector::Y).angle_between(Vector::Y).to_degrees();
    assert!(tilt < 1.0, "at rest tilted {tilt}°");
    let x = r * Vector::X;
    let yaw = (x.z.atan2(x.x).to_degrees() + 45.0).rem_euclid(90.0) - 45.0;
    assert!(yaw.abs() <= 3.0, "{yaw}° off square");
    assert!(
        (p.x - 2.0).abs() < 0.15 && p.z < -1.0,
        "on the takeaway: {p:?}"
    );
    assert!(
        (v.z + 0.5).abs() < 0.01 && v.x.abs() < 0.01,
        "carried: {v:?}"
    );
    assert_eq!(w.pusher_reach(0), 0.0);
    assert_eq!((w.pushers(), w.pusher_id(0)), (1, "p"));
}

/// A sink at the pusher's side takes what it pushes off; blocked, or with nothing at that side, there is no wall,
/// which the paddle's stroke would cross, so the carton falls beside the conveyor.
#[test]
fn a_sink_at_the_side_takes_a_pushed_carton_and_nothing_there_lets_it_fall() {
    let sink = || Sink {
        id: "reject".into(),
        exit: Exit::Push(0),
    };
    let mut open = line(false, vec![sink()]);
    push(&mut open, 0);
    assert_eq!((open.delivered(0), open.cartons().len()), (1, 0));
    let mut blocked = line(false, vec![sink()]);
    blocked.set_sink_blocked(0, true);
    assert!(blocked.sink_blocked(0));
    push(&mut blocked, 1500);
    assert_eq!(blocked.delivered(0), 0);
    let mut nowhere = line(false, vec![]);
    push(&mut nowhere, 1500);
    for w in [blocked, nowhere] {
        let p = w.cartons()[0].position;
        assert!(
            p[1] < 0.5 && p[2] < -0.3,
            "on the floor beside the conveyor: {p:?}"
        );
    }
}

/// The paddle at rest: OIP's box beside the host, its face 1 cm outside the frame, from 0.047 above the surface.
#[test]
fn the_paddle_rests_beside_its_host() {
    let mut w = line(false, vec![]);
    w.step();
    let body = &w.world.bodies[w.devices.pushers[0].body];
    let b = w.world.colliders[body.colliders()[0]].compute_aabb();
    assert!((b.mins.z - (0.3 + 0.01 + 0.0105)).abs() < 1e-5, "{b:?}");
    assert!(
        (b.mins.y - 0.847).abs() < 1e-5 && (b.maxs.y - 1.247).abs() < 1e-5,
        "{b:?}"
    );
    assert!((b.maxs.x - b.mins.x - 0.47).abs() < 1e-5 && (b.maxs.z - b.mins.z - 0.5).abs() < 1e-5);
}

fn set(project: &mut twinova_project::Project, instance: &str, name: &str, value: ParamValue) {
    let inst = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == instance);
    inst.unwrap().parameters.insert(name.into(), value);
}

/// The sort line without its turntable: the pusher on belt B opens its right guard and feeds the reject sink.
fn sort_line() -> twinova_project::Project {
    let mut project = fixture("sort_line");
    let gone = ["turntable", "s_turn"];
    project
        .scene
        .instances
        .retain(|i| !gone.contains(&i.id.as_str()));
    project.scene.connections.retain(|c| match c {
        Connection::Flow { from, to } => {
            !gone.contains(&from.0.as_str()) && !gone.contains(&to.0.as_str())
        }
        Connection::Mount { device, .. } => !gone.contains(&device.0.as_str()),
    });
    project
}

#[test]
fn a_pusher_fits_a_level_square_straight_conveyor_open_on_its_exit_side() {
    let l = layout(&sort_line()).unwrap();
    let b = l.belt("belt_b").unwrap();
    assert_eq!(
        l.pushers,
        [Pusher {
            id: "pusher".into(),
            host: b,
            at: 1.5,
            stroke: 0.75,
            out_right: false,
        }]
    );
    assert_eq!(
        l.belts[b].openings,
        [Opening {
            right: true,
            centre: 1.5,
            length: 0.75
        }]
    );
    let reject = l.sinks.iter().find(|s| s.id == "sink_reject").unwrap();
    assert_eq!(reject.exit, Exit::Push(0));
    let cases = [
        (
            "incline",
            ParamValue::Real(5.0),
            "which is not level and square: a pusher fits a level, unskewed conveyor",
        ),
        (
            "left_guard",
            ParamValue::Bool(true),
            "whose left guard closes the pusher's exit: turn left_guard off",
        ),
    ];
    for (name, value, problem) in cases {
        let mut project = sort_line();
        set(&mut project, "belt_b", name, value);
        assert_eq!(
            layout(&project).unwrap_err(),
            format!("pusher is mounted on belt_b, {problem}")
        );
    }
    let mut project = sort_line();
    for c in &mut project.scene.connections {
        if let Connection::Mount { host, device, .. } = c
            && device.0 == "pusher"
        {
            host.0 = "source".into();
        }
    }
    assert_eq!(
        layout(&project).unwrap_err(),
        "pusher is mounted on source, which is not a conveyor"
    );
    let mut project = fixture("sort_line");
    for c in &mut project.scene.connections {
        if let Connection::Mount { host, device, .. } = c
            && device.0 == "pusher"
        {
            host.0 = "turntable".into();
        }
    }
    assert_eq!(
        layout(&project).unwrap_err(),
        "pusher is mounted on turntable, which is not a straight conveyor"
    );
}

/// The pusher's side exit takes a carton as soon as its face crosses the host's edge on the exit side, along the
/// paddle and a carton either way of it (0.235 + 0.15 m), on the host's surface; not before, nor further along.
#[test]
fn the_side_exit_reaches_along_the_paddle_and_past_the_hosts_edge() {
    let mut w = line(false, vec![]);
    let mut at = |x: Real, z: Real, y: Real| {
        let body = w
            .world
            .insert_body(RigidBodyBuilder::dynamic().translation(Vector::new(x, y, z)));
        w.reached(Exit::Push(0), body)
    };
    let on = 0.8 + CARTON_HALF;
    let crossed = -(0.3 - CARTON_HALF) - 0.001;
    assert!(at(2.0, crossed, on), "its face just past the edge");
    assert!(!at(2.0, crossed + 0.002, on), "its face just inside");
    assert!(
        at(2.0 + 0.38, crossed, on) && at(2.0 - 0.38, crossed, on),
        "along the paddle and a carton"
    );
    assert!(
        !at(2.0 + 0.39, crossed, on) && !at(2.0 - 0.39, crossed, on),
        "further along"
    );
    assert!(!at(2.0, crossed, 0.3), "fallen to the floor");
    assert!(!at(2.0, -crossed, on), "past the other edge");
}

/// The world reports each device it has, none where it has none, and where a pusher's paddle stands.
#[test]
fn the_world_counts_its_devices_and_reports_a_pushers_reach() {
    let empty = World::new(Layout::default());
    assert_eq!(
        (empty.stoppers(), empty.pushers(), empty.turntables()),
        (0, 0, 0)
    );
    let mut w = line(false, vec![]);
    w.set_pusher(0, 0.4);
    assert_eq!(w.pusher_reach(0), 0.4);
}
