//! OIP's blade stop (spec 2026-11-11-stopper-pusher-turntable-design.md, acceptance 1): a raised blade holds a queue
//! dead, and lowered it lets the front carton run on at the host's speed.

use super::tests::{belt, fixture, run, source};
use super::*;
use crate::layout::{Stopper, layout, roller_gap};
use twinova_project::Project;
use twinova_project::model::{Connection, ParamValue};

const ROLLER_GRIP: Real = 0.42;
/// OIP's fixed 0.15 s, as the stopper's block slews it: one millisecond of stroke per step.
const RAISE_MS: usize = 150;

/// A 4 m roller conveyor at `speed` with a stopper at its middle gap raising `raise` m, and three cartons a second
/// apart; the blade is up from the start.
fn queue(speed: Real, raise: Real) -> World {
    let mut w = World::new(Layout {
        belts: vec![Belt {
            grip: ROLLER_GRIP,
            ..belt("r", 0.0, 4.0, speed, true)
        }],
        stoppers: vec![Stopper {
            id: "s".into(),
            host: 0,
            at: roller_gap(4.0, 2.0),
            width: 0.57,
            raise,
        }],
        sources: vec![source(0, 1_000_000, 1_000_000_000, 3)],
        ..Layout::default()
    });
    w.set_belt_running(0, true);
    w.set_stopper(0, 1.0);
    w
}

fn tilt(w: &World, k: usize) -> Real {
    let r = *w.world.bodies[w.products[k].body].rotation();
    (r * Vector::Y).angle_between(Vector::Y).to_degrees()
}

#[test]
fn a_raised_blade_holds_a_queue_dead() {
    let mut w = queue(0.5, 0.04);
    let mut worst: Real = 0.0;
    for _ in 0..8000 {
        w.step();
        for k in 0..w.products.len() {
            worst = worst.max(tilt(&w, k));
        }
    }
    let face = roller_gap(4.0, 2.0) - 0.015;
    let x: Vec<Real> = w.cartons().iter().map(|c| c.position[0]).collect();
    assert_eq!(x.len(), 3);
    assert!(
        (face - (x[0] + CARTON_HALF)).abs() <= 0.001,
        "front at {x:?}, face {face}"
    );
    for pair in x.windows(2) {
        assert!(
            (pair[0] - pair[1] - 2.0 * CARTON_HALF).abs() < 0.005,
            "a queue: {x:?}"
        );
    }
    let v = w.world.bodies[w.products[0].body].linvel();
    assert!(v.length() < 1e-3, "at rest: {v:?}");
    assert!(worst < 2.0, "tilted {worst}°");
}

/// Spec, acceptance 1 (amended in Task 1): the blade's top passes the surface 67 ms into the stroke, then the rollers
/// bring the carton to speed at μg in 121 ms.
#[test]
fn a_lowered_blade_lets_the_front_carton_run_on_at_the_hosts_speed() {
    let mut w = queue(0.5, 0.04);
    run(&mut w, 8000);
    for ms in 1..=1000 {
        let lift = 1.0 - (ms as Real / RAISE_MS as Real).min(1.0);
        w.set_stopper(0, lift);
        w.step();
        let v = w.world.bodies[w.products[0].body].linvel();
        if (v.x - 0.5).abs() < 0.005 {
            assert!(ms <= 220, "at speed after {ms} ms");
            return;
        }
    }
    panic!("the front carton did not run on");
}

/// OIP's stroke: up, the blade reaches from 0.15 m under the surface to its raise above it; down, its top is 5 cm
/// under the surface; across the host less 3 cm, 3 cm thick, in the gap.
#[test]
fn the_blade_spans_oips_stroke() {
    let mut w = queue(0.5, 0.04);
    let bounds = |w: &World| {
        let body = &w.world.bodies[w.devices.stoppers[0].body];
        w.world.colliders[body.colliders()[0]].compute_aabb()
    };
    w.step();
    let up = bounds(&w);
    assert!(
        (up.maxs.y - 0.84).abs() < 1e-5 && (up.mins.y - 0.65).abs() < 1e-5,
        "{up:?}"
    );
    assert!((up.maxs.z - up.mins.z - 0.57).abs() < 1e-5);
    assert!((up.maxs.x - up.mins.x - 0.03).abs() < 1e-5);
    assert!(((up.maxs.x + up.mins.x) / 2.0 - roller_gap(4.0, 2.0)).abs() < 1e-5);
    assert_eq!(w.stopper_lift(0), 1.0);
    w.set_stopper(0, 0.0);
    w.step();
    assert!((bounds(&w).maxs.y - 0.75).abs() < 1e-5, "{:?}", bounds(&w));
    assert_eq!((w.stoppers(), w.stopper_id(0)), (1, "s"));
}

/// The gap between the view's rollers nearest a mount: 27 rollers on 2 m from 0.025, 53 on 4 m with one at the centre,
/// 3 on 0.2 m (OIP's pitch 0.075, radius 0.024), clamped to the first and the last gap; a tie goes downstream. A
/// conveyor too short for two rollers has its gap at its middle.
#[test]
fn a_stopper_snaps_to_the_gap_between_rollers_nearest_its_mount() {
    for (length, at, gap) in [
        (2.0, 1.64, 1.6375),
        (2.0, 1.6, 1.6375),
        (2.0, 0.0, 0.0625),
        (2.0, 2.0, 1.9375),
        (4.0, 2.0, 2.0375),
        (4.0, 1.99, 1.9625),
        (0.2, 0.1, 0.1375),
        (0.2, 0.05, 0.0625),
        (0.1, 0.0, 0.05),
    ] {
        let snapped = roller_gap(length, at);
        assert!(
            (snapped - gap).abs() < 1e-5,
            "{length} m at {at}: {snapped}"
        );
    }
}

/// The sort line, without its turntable and the photocell on it: the stopper is laid out on roller A.
fn sort_line() -> Project {
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

fn set(project: &mut Project, instance: &str, name: &str, value: ParamValue) {
    let inst = project
        .scene
        .instances
        .iter_mut()
        .find(|i| i.id == instance);
    inst.unwrap().parameters.insert(name.into(), value);
}

#[test]
fn a_stopper_fits_a_level_square_roller_conveyor_only() {
    let l = layout(&sort_line()).unwrap();
    assert_eq!(
        l.stoppers,
        [Stopper {
            id: "stopper".into(),
            host: l.belt("roller_a").unwrap(),
            at: 1.6375,
            width: 0.6 - 0.03,
            raise: 0.04,
        }]
    );
    for (name, value) in [("incline", 5.0), ("skew", 5.0)] {
        let mut project = sort_line();
        set(&mut project, "roller_a", name, ParamValue::Real(value));
        assert_eq!(
            layout(&project).unwrap_err(),
            "stopper is mounted on roller_a, which is not level and square: a stopper fits a level, unskewed conveyor"
        );
    }
    let mut project = sort_line();
    for c in &mut project.scene.connections {
        if let Connection::Mount { host, device, .. } = c
            && device.0 == "stopper"
        {
            host.0 = "belt_in".into();
        }
    }
    assert_eq!(
        layout(&project).unwrap_err(),
        "stopper is mounted on belt_in, which is not a roller conveyor"
    );
}
