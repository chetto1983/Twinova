//! OIP's turntable (spec 2026-11-11-stopper-pusher-turntable-design.md, acceptance 3): its lane's field turns a
//! carton with the deck, which then runs off along the new heading.

use super::tests::{belt, fixture, run, source};
use super::*;
use crate::layout::{Sink, TURNTABLE_LANE, layout};

const ROLLER_GRIP: Real = 0.42;
/// The deck's centre.
const CENTRE: Vector = Vector::new(2.45, 0.0, 0.0);

/// OIP's slew, as the turntable's block steps it each millisecond: up to `top` °/s over 0.5 s, braking to land on
/// the target.
struct Slew {
    angle: Real,
    rate: Real,
    top: Real,
}

impl Slew {
    /// One millisecond towards `target` (degrees), with the deck at its new angle and rate set in `w`'s turntable.
    fn step(&mut self, w: &mut World, target: Real) {
        let d = target - self.angle;
        let change = self.top / 0.5 * 0.001;
        let goal =
            if self.rate * d >= 0.0 && d.abs() > self.rate * self.rate / (self.top / 0.5 * 2.0) {
                self.top * d.signum()
            } else {
                0.0
            };
        self.rate = if self.rate < goal {
            (self.rate + change).min(goal)
        } else {
            (self.rate - change).max(goal)
        };
        if self.rate * d > 0.0 && d.abs() <= self.rate.abs() * 0.001 {
            (self.angle, self.rate) = (target, 0.0);
        } else {
            self.angle += self.rate * 0.001;
        }
        w.set_turntable(0, self.angle.to_radians(), self.rate.to_radians());
        w.step();
    }
}

/// Belt A (2 m) into a 0.6 m turntable, belt C off its left at 90° into a sink; one carton; every surface at 0.5.
fn cell() -> World {
    let lane = Belt {
        pose: Pose::from_translation(CENTRE - Vector::new(0.45, 0.0, 0.0)),
        length: TURNTABLE_LANE * 0.6,
        grip: ROLLER_GRIP,
        turntable: true,
        ..belt("t", 0.0, 0.0, 0.5, false)
    };
    let off = Belt {
        pose: Pose::from_parts(
            CENTRE - Vector::new(0.0, 0.0, 0.45),
            Rotation::from_rotation_y(std::f32::consts::FRAC_PI_2),
        ),
        ..belt("c", 0.0, 2.0, 0.5, false)
    };
    let mut w = World::new(Layout {
        belts: vec![belt("a", 0.0, 2.0, 0.5, false), lane, off],
        turntables: vec![1],
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

/// The carton's plan position and heading.
fn plan(w: &World) -> (Vector, Rotation) {
    let body = &w.world.bodies[w.cartons[0].1];
    (body.translation(), *body.rotation())
}

/// Runs the cell until the carton stands at the deck's centre, the lane stopped.
fn centred(w: &mut World) {
    while w.cartons().first().is_none_or(|c| c.position[0] < CENTRE.x) {
        w.step();
    }
    w.set_belt_speed(1, 0.0);
    run(w, 500);
}

#[test]
fn a_carton_turns_with_the_deck_and_runs_off_along_its_new_heading() {
    let mut w = cell();
    centred(&mut w);
    let (p0, r0) = plan(&w);
    let mut slew = Slew {
        angle: 0.0,
        rate: 0.0,
        top: 20.0,
    };
    let mut ms = 0;
    while slew.angle != 90.0 || slew.rate != 0.0 {
        slew.step(&mut w, 90.0);
        ms += 1;
    }
    // OIP's profile: 5° up in 0.5 s, 80° at 20°/s, 5° down, 5.0 s if continuous. Stepped, the braking starts 0.01°
    // late and OIP's step lands the deck once what is left fits in one step, at 0.76°/s: 4 983 ms (spec, amended in
    // Task 2).
    assert_eq!(ms, 4983, "turned in {ms} ms");
    let quarter = Rotation::from_rotation_y(std::f32::consts::FRAC_PI_2);
    let (p, r) = plan(&w);
    let carried = CENTRE + quarter * (p0 - CENTRE);
    let off = Vector::new(p.x - carried.x, 0.0, p.z - carried.z).length();
    assert!(off <= 0.002, "{off} m off its carried place");
    let turned = (r * r0.inverse() * Vector::X)
        .angle_between(quarter * Vector::X)
        .to_degrees();
    assert!(turned <= 0.5, "{turned}° off the deck's turn");
    assert!((w.turntable_angle(0) - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    w.set_belt_speed(1, 0.5);
    run(&mut w, 300);
    let v = w.world.bodies[w.cartons[0].1].linvel();
    assert!(
        (v.z + 0.5).abs() < 0.005 && v.x.abs() < 0.005,
        "along the new heading: {v:?}"
    );
    run(&mut w, 6000);
    assert_eq!(
        w.delivered(0),
        1,
        "off the deck onto belt C and into its sink"
    );
}

/// A field about the centre at the rate the deck turns, whatever the lane's speed; settled, the flow along the angle.
#[test]
fn the_lane_turns_about_the_deck_or_runs_along_its_angle() {
    let mut w = cell();
    let lane = w.surfaces[1].clone();
    assert_eq!(lane.len(), 3, "the lane and its two caps");
    w.set_turntable(0, 0.3, 0.2);
    for &(index, _) in &lane {
        let Drive::Turn { centre, omega } = w.hooks.surface[index] else {
            panic!("turning")
        };
        assert!(
            (centre - CENTRE).length() < 1e-6
                && (omega - Vector::new(0.0, 0.2, 0.0)).length() < 1e-6
        );
    }
    w.set_turntable(0, std::f32::consts::FRAC_PI_2, 0.0);
    w.set_belt_speed(1, 0.4);
    for &(index, _) in &lane {
        let Drive::Linear(v) = w.hooks.surface[index] else {
            panic!("settled")
        };
        assert!((v - Vector::new(0.0, 0.0, -0.4)).length() < 1e-6, "{v:?}");
    }
    assert_eq!((w.turntables(), w.turntable_id(0)), (1, "t"));
}

#[test]
fn the_sort_line_lays_its_turntable_out_as_a_lane_from_rim_to_rim() {
    let l = layout(&fixture("sort_line")).unwrap();
    let k = l.belt("turntable").unwrap();
    assert_eq!(l.turntables, [k]);
    let t = &l.belts[k];
    assert!(t.turntable && !t.closed_exit && t.guards == [false; 2]);
    assert!((t.length - 0.9).abs() < 1e-6 && (t.width - 0.6).abs() < 1e-6 && t.grip == 0.42);
    assert!((t.pose.translation - Vector::new(4.0, 0.0, 0.0)).length() < 1e-6);
    let s = l.sensors.iter().find(|s| s.id == "s_turn").unwrap();
    assert_eq!((s.belt, s.at), (k, 0.6));
    assert!(!l.belts[l.belt("roller_a").unwrap()].closed_exit);
}

#[test]
fn a_turntable_gives_equal_bits_on_every_platform() {
    let fingerprint = || {
        let mut w = cell();
        centred(&mut w);
        let mut slew = Slew {
            angle: 0.0,
            rate: 0.0,
            top: 90.0,
        };
        for _ in 0..1500 {
            slew.step(&mut w, 90.0);
        }
        w.set_belt_speed(1, 0.5);
        run(&mut w, 500);
        w.fingerprint()
    };
    let first = fingerprint();
    assert_eq!(first, fingerprint());
    // Pinned on Linux x86-64 (2026-11-11), equal in dev and release; the windows-2022 CI job runs this test too.
    assert_eq!(first, 0x1109_870f_76d5_c9c8, "{first:#018x}");
}
