//! Guards, skew and incline (spec 2026-11-10-curves-belt-options-design.md, acceptance 2 and 4), after OIP's
//! `physicsFidelity.test.ts`: a skewed bed's throughput s and lateral speed s·tan(skew), the guard that squares the
//! load, a climb at the belt's speed, and breakaway on an incline within 1° of atan(μ).

use super::tests::{belt, fixture, run, source};
use super::*;
use crate::layout::{BELT_GRIP, Belt, inclined, layout};
use twinova_project::model::{Connection, ParamValue};

const ROLLER_GRIP: Real = 0.42;

fn one_conveyor(b: Belt, running: bool) -> World {
    let mut w = World::new(Layout {
        belts: vec![b],
        sources: vec![source(0, 1_000_000, 1_000_000_000_000, 1)],
        ..Layout::default()
    });
    w.set_belt_running(0, running);
    w
}

/// The carton's position and velocity in the conveyor's frame.
fn local(w: &World) -> (Vector, Vector) {
    let (frame, body) = (w.layout.belts[0].pose, &w.world.bodies[w.cartons[0].1]);
    (
        frame.inverse_transform_point(body.translation()),
        frame.rotation.inverse() * body.linvel(),
    )
}

fn skewed(guards: [bool; 2]) -> World {
    one_conveyor(
        Belt {
            grip: ROLLER_GRIP,
            guards,
            skew: 10.0_f32.to_radians(),
            ..belt("r", 0.0, 4.0, 0.5, true)
        },
        true,
    )
}

/// OIP: a skewed bed keeps the throughput s while the load drifts at s·tan(skew), positive to the left, until the
/// guard squares it; the guard, without friction, holds it at the throughput.
#[test]
fn a_skewed_roller_drifts_a_carton_to_its_guard_which_holds_it() {
    let lateral = 0.5 * 10.0_f32.to_radians().tan();
    let mut w = skewed([true; 2]);
    let mut drift = None;
    for _ in 0..8000 {
        w.step();
        let (p, v) = local(&w);
        if drift.is_none() && p.z < -0.08 {
            drift = Some(v);
        }
        if p.x > 3.0 {
            break;
        }
    }
    let drift = drift.expect("it drifts left");
    assert!(
        (drift.z + lateral).abs() < 0.03 * lateral,
        "drifts at {} m/s",
        -drift.z
    );
    assert!((drift.x - 0.5).abs() < 0.01, "throughput {}", drift.x);
    let (p, v) = local(&w);
    // The guard's inner face is 0.31 m from the centreline: the carton's side lies on it.
    assert!((p.z + 0.31 - CARTON_HALF).abs() < 0.003, "held at {}", p.z);
    assert!(v.z.abs() < 0.002 && (v.x - 0.5).abs() < 0.01, "{v:?}");
    let mut open = skewed([false, true]);
    run(&mut open, 8000);
    assert!(
        local(&open).0.y < 0.5,
        "without its left guard, the carton leaves the bed"
    );
}

/// Spec, acceptance 2: a carton climbs a 10° incline at the belt's speed, and does not slip back at OIP's belt grip.
#[test]
fn a_carton_climbs_an_incline_at_the_belts_speed() {
    let incline = 10.0_f32.to_radians();
    let pose = inclined(Pose::IDENTITY, 4.0, 0.8, incline);
    let mut w = one_conveyor(
        Belt {
            pose,
            ..belt("b", 0.0, 4.0, 0.5, true)
        },
        true,
    );
    run(&mut w, 1000);
    let (start, low) = (local(&w).0.x, w.cartons()[0].position[1]);
    run(&mut w, 1000);
    let (p, v) = local(&w);
    assert!((v.x - 0.5).abs() < 0.005 && v.y.abs() < 0.005, "{v:?}");
    assert!(
        (p.y - (0.8 + CARTON_HALF)).abs() < 0.003,
        "on the slope: {p:?}"
    );
    let climbed = w.cartons()[0].position[1] - low;
    assert!(
        (climbed - (p.x - start) * incline.sin()).abs() < 0.002,
        "{climbed} m up"
    );
    // Stopped, it brakes at about (μ cos 10° − sin 10°) g = 5 m/s², over 2.5 cm, then holds.
    w.set_belt_running(0, false);
    run(&mut w, 300);
    let held = local(&w).0.x;
    run(&mut w, 1000);
    assert!(
        (local(&w).0.x - held).abs() < 0.002,
        "held on the stopped belt"
    );
}

/// OIP fidelity: a load on a stopped bed holds below atan(μ) and slides above it. Rapier gives way about 1–1.8°
/// below atan(μ), an effective μ 5.6 % low, whatever its solver iterations, warm start or contact softness (spec,
/// amended in Task 3, user): it holds 2° below and slides 1° above, for rollers and belts.
#[test]
fn a_carton_breaks_away_on_an_incline_near_atan_mu() {
    for grip in [ROLLER_GRIP, BELT_GRIP] {
        let breakaway = grip.atan().to_degrees();
        let slide = |degrees: Real| {
            // The entry is the high end, so a carton that slides runs along the bed.
            let pose = inclined(Pose::IDENTITY, 4.0, 0.8, -degrees.to_radians());
            let mut w = one_conveyor(
                Belt {
                    pose,
                    grip,
                    ..belt("r", 0.0, 4.0, 0.5, true)
                },
                false,
            );
            run(&mut w, 200);
            let start = local(&w).0.x;
            run(&mut w, 500);
            local(&w).0.x - start
        };
        let held = slide(breakaway - 2.0);
        assert!(held.abs() < 0.002, "{held} m at {}°", breakaway - 2.0);
        let slid = slide(breakaway + 1.0);
        assert!(slid > 0.05, "{slid} m at {}°", breakaway + 1.0);
    }
}

fn set(project: &mut twinova_project::Project, instance: &str, name: &str, value: ParamValue) {
    let inst = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == instance);
    inst.unwrap().parameters.insert(name.into(), value);
}

#[test]
fn a_chain_transfer_fits_only_a_level_square_roller_conveyor_open_on_its_exit_side() {
    let cases = [
        (
            "incline",
            ParamValue::Real(5.0),
            "which is not level and square",
        ),
        (
            "skew",
            ParamValue::Real(5.0),
            "which is not level and square",
        ),
        (
            "left_guard",
            ParamValue::Bool(true),
            "whose left guard closes the transfer's exit: turn left_guard off",
        ),
    ];
    for (name, value, problem) in cases {
        let mut project = fixture("two_belts_diverter");
        set(&mut project, "roller_r", name, value);
        let error = layout(&project).unwrap_err();
        assert!(
            error.starts_with("chain_transfer is mounted on roller_r, "),
            "{error}"
        );
        assert!(error.contains(problem), "{name}: {error}");
    }
    let mut project = fixture("two_belts_diverter");
    set(
        &mut project,
        "roller_r",
        "right_guard",
        ParamValue::Bool(false),
    );
    set(
        &mut project,
        "chain_transfer",
        "out_right",
        ParamValue::Bool(true),
    );
    assert!(
        layout(&project).is_ok(),
        "the right side open for a transfer to the right"
    );
}

#[test]
fn the_layout_takes_incline_skew_guards_and_a_sensor_angle_from_the_parameters() {
    let mut project = fixture("curve_line");
    set(&mut project, "belt_a", "incline", ParamValue::Real(10.0));
    set(
        &mut project,
        "belt_a",
        "left_guard",
        ParamValue::Bool(false),
    );
    for c in &mut project.scene.connections {
        if let Connection::Mount {
            host,
            device,
            position,
            ..
        } = c
            && device.0 == "sensor_c"
        {
            host.0 = "curve_b".into();
            position[0] = 45.0;
        }
    }
    let l = layout(&project).unwrap();
    let a = &l.belts[0];
    assert_eq!(a.guards, [false, true]);
    let pivot = a.pose.transform_point(Vector::new(1.5, 0.8, 0.0));
    assert!(
        (pivot - Vector::new(1.5, 0.8, 0.0)).length() < 1e-5,
        "tilted about its surface's centre: {pivot}"
    );
    let head = a.pose.transform_point(Vector::new(3.0, 0.8, 0.0));
    assert!(
        (head.y - (0.8 + 1.5 * 10.0_f32.to_radians().sin())).abs() < 1e-5,
        "head up: {head}"
    );
    let sensor = l.sensors.iter().find(|s| s.id == "sensor_c").unwrap();
    assert_eq!(l.belts[sensor.belt].id, "curve_b");
    assert!((sensor.at - std::f32::consts::FRAC_PI_4).abs() < 1e-6);
    let mut project = fixture("two_belts_diverter");
    set(&mut project, "belt_a", "skew", ParamValue::Real(-20.0));
    let l = layout(&project).unwrap();
    assert!((l.belts[0].skew + 20.0_f32.to_radians()).abs() < 1e-6);
    assert_eq!(l.belts[0].guards, [true; 2]);
    assert_eq!(
        l.belts[2].guards,
        [false, true],
        "the roller's exit side is open in the fixture"
    );
    assert_eq!(l.belts[0].grip, BELT_GRIP);
}

/// OIP's skew: the throughput stays the speed and the drift is speed · tan(skew), to the left for a positive skew.
#[test]
fn a_skewed_surface_keeps_its_throughput_and_drifts_by_the_tangent() {
    let skew = 30.0_f32.to_radians();
    let v = along(0.5, skew);
    assert!(
        (v.x - 0.5).abs() < 1e-6 && (v.z + 0.5 * skew.tan()).abs() < 1e-6,
        "{v:?}"
    );
    assert_eq!(along(0.5, 0.0), Vector::new(0.5, 0.0, 0.0));
}

/// The drive sets a conveyor's speed, signed: a negative one carries back along the flow.
#[test]
fn a_conveyor_runs_at_the_speed_its_drive_sets_either_way() {
    let mut w = one_conveyor(belt("b", 0.0, 4.0, 0.5, true), false);
    w.set_belt_speed(0, 0.3);
    run(&mut w, 1500);
    assert!((local(&w).1.x - 0.3).abs() < 0.005, "{:?}", local(&w).1);
    w.set_belt_speed(0, -0.2);
    run(&mut w, 1000);
    assert!((local(&w).1.x + 0.2).abs() < 0.005, "{:?}", local(&w).1);
}

/// The view scrolls a belt's fabric by its travel: the speed it ran at, each millisecond, either way.
#[test]
fn a_conveyor_counts_its_surface_travel() {
    let mut w = World::new(Layout {
        belts: vec![
            belt("a", 0.0, 2.0, 0.5, true),
            belt("b", 2.0, 2.0, 0.5, true),
        ],
        ..Layout::default()
    });
    assert_eq!((w.belts(), w.belt_id(1)), (2, "b"));
    w.set_belt_speed(0, 0.5);
    run(&mut w, 1000);
    w.set_belt_speed(0, -0.25);
    assert_eq!((w.belt_speed(0), w.belt_speed(1)), (-0.25, 0.0));
    run(&mut w, 400);
    assert!(
        (w.belt_travel(0) - 0.4).abs() < 1e-9,
        "{}",
        w.belt_travel(0)
    );
    assert_eq!(w.belt_travel(1), 0.0, "a stopped belt does not run");
}
