//! Curves (spec 2026-11-10-curves-belt-options-design.md, acceptance 1): a carton turns with the curve and leaves
//! square, on the centreline and level; the arc field's sign holds whichever collider the deck is.

use super::tests::{belt, fixture, run, source};
use super::*;
use crate::layout::layout;
use crate::layout::{Arc, BELT_GRIP, Belt, Sensor, Sink};

/// OIP's roller grip (`CONVEYOR_FRICTION` 0.42 on a roller surface).
const ROLLER_GRIP: Real = 0.42;

pub(super) fn curve(pose: Pose, right: bool, speed: Real, grip: Real) -> Belt {
    Belt {
        pose,
        speed,
        grip,
        arc: Some(Arc {
            inner_radius: 0.6,
            sweep: std::f32::consts::FRAC_PI_2,
            right,
        }),
        ..belt("curve", 0.0, 0.0, speed, false)
    }
}

/// Belt A (2 m), a 90° curve, belt B (2 m) from the curve's exit into a sink, one carton, every surface at `speed`.
fn turn(right: bool, speed: Real, grip: Real, guards: bool) -> World {
    let mut c = curve(
        Pose::from_translation(Vector::new(2.0, 0.0, 0.0)),
        right,
        speed,
        grip,
    );
    c.guards = [guards; 2];
    let exit = c.pose * c.arc.unwrap().end(c.width);
    let mut a = belt("a", 0.0, 2.0, speed, false);
    let mut b = Belt {
        pose: exit,
        ..belt("b", 0.0, 2.0, speed, false)
    };
    a.guards = [guards; 2];
    b.guards = [guards; 2];
    let mut w = World::new(Layout {
        belts: vec![a, c, b],
        sources: vec![source(0, 1_000_000, 1_000_000_000_000, 1)],
        sinks: vec![Sink {
            id: "sink".into(),
            exit: Exit::End(2),
        }],
        ..Layout::default()
    });
    for k in 0..3 {
        w.set_belt_running(k, true);
    }
    w
}

/// The carton's pose in belt B's frame once it is `at` along B: its offset across B, its yaw against B's flow
/// (degrees, the box's nearest face), and its tilt from upright (degrees).
fn leaving(w: &mut World, at: Real) -> (Real, Real, Real) {
    let frame = w.layout.belts[2].pose;
    for _ in 0..20_000 {
        w.step();
        let handle = w.products[0].body;
        let body = &w.world.bodies[handle];
        let p = frame.inverse_transform_point(body.translation());
        if p.x >= at {
            let local = frame.rotation.inverse() * *body.rotation();
            let x = local * Vector::X;
            let yaw = x.z.atan2(x.x).to_degrees();
            let yaw = (yaw + 45.0).rem_euclid(90.0) - 45.0;
            let tilt = (local * Vector::Y).angle_between(Vector::Y).to_degrees();
            return (p.z, yaw, tilt);
        }
    }
    panic!("the carton did not reach {at} m along belt B");
}

/// OIP's fidelity bar for a curve (spec, acceptance 1): yaw within 3° of the turn, off the centreline ≤ 5 mm, tilt
/// < 1°, for belt and roller grip, turning left and right.
#[test]
fn a_carton_turns_with_a_curve_and_leaves_square() {
    for (grip, right) in [
        (BELT_GRIP, false),
        (BELT_GRIP, true),
        (ROLLER_GRIP, false),
        (ROLLER_GRIP, true),
    ] {
        let mut w = turn(right, 0.5, grip, false);
        let (across, yaw, tilt) = leaving(&mut w, 1.0);
        let case = format!("grip {grip}, right {right}");
        assert!(
            across.abs() <= 0.005,
            "{case}: {across} m off the centreline"
        );
        assert!(yaw.abs() <= 3.0, "{case}: {yaw}° off square");
        assert!(tilt < 1.0, "{case}: tilted {tilt}°");
    }
}

/// Spec, acceptance 2: the product stays on a curve with guards at 1 m/s, and the sink takes it.
#[test]
fn a_carton_stays_on_a_guarded_curve_at_one_metre_a_second() {
    let mut w = turn(false, 1.0, BELT_GRIP, true);
    run(&mut w, 8000);
    assert_eq!((w.delivered(0), w.created()), (1, 1));
}

/// A sink at a curve's own exit takes a carton when it reaches the stub's tip; a closed exit holds it there.
#[test]
fn a_curve_hands_over_at_its_exit_stub() {
    let line = |closed_exit: bool, sinks: Vec<Sink>| {
        let mut c = curve(Pose::IDENTITY, false, 0.5, BELT_GRIP);
        c.closed_exit = closed_exit;
        let mut w = World::new(Layout {
            belts: vec![c],
            sources: vec![source(0, 1_000_000, 1_000_000_000_000, 1)],
            sinks,
            ..Layout::default()
        });
        w.set_belt_running(0, true);
        w
    };
    let sink = Sink {
        id: "sink".into(),
        exit: Exit::End(0),
    };
    let mut open = line(false, vec![sink]);
    run(&mut open, 6000);
    assert_eq!(open.delivered(0), 1);
    let mut closed = line(true, vec![]);
    run(&mut closed, 6000);
    let exit = closed.layout.belts[0].arc.unwrap().end(0.6);
    let p = exit.inverse_transform_point(Vector::from(closed.cartons()[0].position));
    assert!(
        p.x < 0.0 && p.x > -CARTON_HALF - 0.02,
        "held at the wall: {p:?}"
    );
}

/// The arc field carries a carton round the centre the same way whether the deck is collider 1 or 2, and the other
/// way when the curve turns right (Review focus).
#[test]
fn the_arc_field_turns_the_same_way_whichever_collider_it_is() {
    let carried = |deck_first: bool, omega: Real| {
        let mut world = PhysicsWorld::default();
        world.integration_parameters.dt = STEP_S;
        world.integration_parameters.friction_model = FrictionModel::Coulomb;
        let deck = || {
            ColliderBuilder::cuboid(2.0, 0.05, 2.0)
                .translation(Vector::new(0.0, 0.75, 0.0))
                .friction(BELT_GRIP / CARTON_FRICTION)
                .friction_combine_rule(CoefficientCombineRule::Multiply)
                .user_data(1)
                .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS)
        };
        if deck_first {
            world.insert_collider(deck(), None);
        }
        let carton = world.insert_body(
            RigidBodyBuilder::dynamic()
                .translation(Vector::new(0.0, 0.95, 1.0))
                .can_sleep(false),
        );
        world.insert_collider(
            ColliderBuilder::cuboid(0.15, 0.15, 0.15).friction(CARTON_FRICTION),
            Some(carton),
        );
        if !deck_first {
            world.insert_collider(deck(), None);
        }
        let hooks = Surfaces {
            surface: vec![Drive::Turn {
                centre: Vector::ZERO,
                omega: Vector::new(0.0, omega, 0.0),
            }],
        };
        for _ in 0..1000 {
            world.step_with_events(&hooks, &());
        }
        let body = &world.bodies[carton];
        (body.translation(), body.linvel(), body.angvel().y)
    };
    for deck_first in [true, false] {
        for omega in [0.5, -0.5] {
            // The carton rides the field rigidly: at its point p, v = ω × (p − c), and it spins at ω.
            let (p, v, spin) = carried(deck_first, omega);
            let field = Vector::new(0.0, omega, 0.0).cross(Vector::new(p.x, 0.0, p.z));
            assert!(
                (v - field).length() < 0.01,
                "{deck_first} {omega}: {v:?} against {field:?}"
            );
            assert!((spin - omega).abs() < 0.01, "{deck_first} {omega}: {spin}");
        }
    }
}

/// A sensor on a curve looks across the arc at its angle from the entry: a carton square on the centreline there
/// crosses its beam, one a carton's length further along or back does not; and a carton the curve carries is seen.
#[test]
fn a_sensor_on_a_curve_sees_a_carton_at_its_angle() {
    let at = std::f32::consts::FRAC_PI_4;
    let world = || {
        World::new(Layout {
            belts: vec![curve(Pose::IDENTITY, true, 0.5, BELT_GRIP)],
            sources: vec![source(0, 1_000_000, 1_000_000_000_000, 1)],
            sensors: vec![Sensor {
                id: "s".into(),
                belt: 0,
                at,
                range: 0.7,
            }],
            ..Layout::default()
        })
    };
    let arc = curve(Pose::IDENTITY, true, 0.5, BELT_GRIP).arc.unwrap();
    for (shift, seen) in [(0.0, true), (0.2, false), (-0.2, false)] {
        let mut w = world();
        let frame = arc.frame(0.6, at + shift / 0.9);
        let body =
            w.world
                .insert_body(RigidBodyBuilder::dynamic().pose(
                    frame * Pose::from_translation(Vector::new(0.0, 0.8 + CARTON_HALF, 0.0)),
                ));
        w.world.insert_collider(
            ColliderBuilder::cuboid(CARTON_HALF, CARTON_HALF, CARTON_HALF),
            Some(body),
        );
        w.world.step_with_events(&w.hooks, &());
        assert_eq!(w.sensor(0), seen, "{shift} m along the arc");
    }
    let mut w = world();
    w.set_belt_running(0, true);
    let mut on = 0;
    for _ in 0..5000 {
        w.step();
        on += usize::from(w.sensor(0));
    }
    // 0.3 m of carton, and a little more across a radial beam, at 0.5 m/s.
    assert!((600..800).contains(&on), "seen for {on} ms");
}

#[test]
fn the_curve_line_lays_out_its_curves_and_runs() {
    let l = layout(&fixture("curve_line")).unwrap();
    let ids: Vec<(&str, bool)> = l
        .belts
        .iter()
        .map(|b| (b.id.as_str(), b.arc.is_some_and(|a| a.right)))
        .collect();
    assert_eq!(
        ids,
        [
            ("belt_a", false),
            ("belt_b", false),
            ("belt_c", false),
            ("curve_a", false),
            ("curve_b", true)
        ]
    );
    let curve_a = &l.belts[3];
    assert_eq!((curve_a.grip, l.belts[4].grip), (0.7, 0.42));
    assert_eq!(curve_a.guards, [true; 2]);
    let end = curve_a.pose * curve_a.arc.unwrap().end(curve_a.width);
    assert!(
        (end.translation - l.belts[1].pose.translation).length() < 1e-4,
        "curve A hands over to belt B"
    );
    let mut w = World::new(l);
    for k in 0..5 {
        w.set_belt_running(k, true);
    }
    run(&mut w, 30_000);
    // The source makes one carton every 2 s from 10 ms; the path is 3 + 1.58 + 2 + 1.58 + 2 m at 0.5 m/s.
    assert_eq!(w.created(), w.delivered(0) + w.cartons().len() as u64);
    assert!(w.delivered(0) >= 5, "{}", w.delivered(0));
}

#[test]
fn a_line_with_curves_gives_equal_bits_on_every_platform() {
    let fingerprint = || {
        let mut w = World::new(layout(&fixture("curve_line")).unwrap());
        for k in 0..5 {
            w.set_belt_running(k, true);
        }
        run(&mut w, 15_000);
        w.fingerprint()
    };
    let first = fingerprint();
    assert_eq!(first, fingerprint());
    // Pinned on Linux x86-64 (2026-11-10, Coulomb friction, one solver iteration), equal in dev and release; the
    // windows-2022 CI job runs this test too.
    // Re-pinned 2026-11-11 for 4.4d: the fingerprint covers each product's size, colour, code and read mark, and the
    // carton's mass is set (2.7 kg) instead of following from its rounded core (1.76 kg).
    assert_eq!(first, 0x94df_f910_2f56_7273, "{first:#018x}");
}
