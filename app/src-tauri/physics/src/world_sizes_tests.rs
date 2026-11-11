//! Products of OIP's sizes through every family (spec 2026-11-11-sensors-products-design.md, acceptance 1): OIP's box,
//! 0.6 × 0.4 × 0.4 m, and the random range's largest, 0.8 × 0.5 × 0.5 m, both at OIP's 10 kg, made by a source and
//! carried through curves, a turntable, a stopper, a pusher and a chain transfer into sinks.

use super::curve_tests::curve;
use super::tests::{belt, run, source};
use super::*;
use crate::layout::{
    BELT_GRIP, Products, Pusher, Sink, Source, Stopper, TURNTABLE_LANE, Transfer, roller_gap,
};

const ROLLER_GRIP: Real = 0.42;

/// (name, length, width, height).
const SIZES: [(&str, [Real; 3]); 2] = [
    ("OIP's box", [0.6, 0.4, 0.4]),
    ("the largest", [0.8, 0.5, 0.5]),
];

fn products(size: [Real; 3]) -> Products {
    Products {
        size: Vector::from_array(size),
        mass: 10.0,
        ..Products::carton()
    }
}

/// `count` products every `interval_ms`, from a source on belt 0.
fn feeding(size: [Real; 3], count: i64, interval_ms: i64) -> Source {
    Source {
        products: products(size),
        ..source(0, 1_000_000, interval_ms * 1_000_000, count)
    }
}

/// Steps until the first product's centre passes `x`, failing after 30 s of virtual time.
fn arrive(w: &mut World, x: Real) {
    while w.cartons().first().is_none_or(|c| c.position[0] < x) {
        assert!(w.t_ns < 30_000_000_000, "no product reached x {x}");
        w.step();
    }
}

fn running(layout: Layout) -> World {
    let mut w = World::new(layout);
    for k in 0..w.belts() {
        w.set_belt_running(k, true);
    }
    w
}

/// A product's yaw against `frame`'s flow (degrees, to its nearest face) and its tilt from upright (degrees).
fn attitude(w: &World, k: usize, frame: Rotation) -> (Real, Real) {
    let local = frame.inverse() * *w.world.bodies[w.products[k].body].rotation();
    let x = local * Vector::X;
    let yaw = (x.z.atan2(x.x).to_degrees() + 45.0).rem_euclid(90.0) - 45.0;
    (
        yaw,
        (local * Vector::Y).angle_between(Vector::Y).to_degrees(),
    )
}

/// Belt A, a 90° curve, belt B into a sink; A and B with `straight` grip, the curve with `bend`'s. The product's offset,
/// yaw and tilt 1.5 m along B, and whether the sink took it.
fn through_a_curve(size: [Real; 3], straight: Real, bend: Real) -> (Real, Real, Real, u64) {
    let c = curve(
        Pose::from_translation(Vector::new(2.0, 0.0, 0.0)),
        false,
        0.5,
        bend,
    );
    let exit = c.pose * c.arc.unwrap().end(c.width);
    let mut w = running(Layout {
        belts: vec![
            Belt {
                grip: straight,
                ..belt("a", 0.0, 2.0, 0.5, false)
            },
            c,
            Belt {
                pose: exit,
                grip: straight,
                ..belt("b", 0.0, 3.0, 0.5, false)
            },
        ],
        sources: vec![feeding(size, 1, 1000)],
        sinks: vec![Sink {
            id: "sink".into(),
            exit: Exit::End(2),
        }],
        ..Layout::default()
    });
    loop {
        w.step();
        let p = exit.inverse_transform_point(w.world.bodies[w.products[0].body].translation());
        if p.x >= 1.5 {
            let (yaw, tilt) = attitude(&w, 0, exit.rotation);
            run(&mut w, 4000);
            return (p.z, yaw, tilt, w.delivered(0));
        }
        assert!(w.t_ns < 30_000_000_000, "lost: {p:?}");
    }
}

/// The curve's yaw grows with the box, within 3° on a belt curve and on a roller curve between roller conveyors
/// (measured 2026-11-11: 2.1–2.4°); a roller curve fed and emptied by belts turns a long box 5.4–7.0°, its two ends on
/// grips 0.7 and 0.42, as the spec's probe found.
#[test]
fn oip_sizes_turn_with_a_curve_and_reach_the_sink() {
    for (name, size) in SIZES {
        for (straight, bend, max_yaw) in [
            (BELT_GRIP, BELT_GRIP, 3.0),
            (ROLLER_GRIP, ROLLER_GRIP, 3.0),
            (BELT_GRIP, ROLLER_GRIP, 7.5),
        ] {
            let (across, yaw, tilt, delivered) = through_a_curve(size, straight, bend);
            let case = format!("{name}, grips {straight} and {bend}");
            assert!(
                across.abs() <= 0.01,
                "{case}: {across} m off the centreline"
            );
            assert!(yaw.abs() <= max_yaw, "{case}: {yaw}°");
            assert!(tilt < 1.0, "{case}: tilted {tilt}°");
            assert_eq!(delivered, 1, "{case}");
        }
    }
}

/// A 1.0 m turntable (its lane 1.5 m) between belt A and belt C off its left: the product stops near the deck's centre,
/// is carried through 90° at 20°/s and runs off into C's sink.
#[test]
fn oip_sizes_turn_square_on_a_turntable() {
    for (name, size) in SIZES {
        let centre = Vector::new(2.75, 0.0, 0.0);
        let lane = Belt {
            pose: Pose::from_translation(centre - Vector::new(0.75, 0.0, 0.0)),
            length: TURNTABLE_LANE,
            width: 1.0,
            grip: ROLLER_GRIP,
            turntable: true,
            ..belt("t", 0.0, 0.0, 0.5, false)
        };
        let off = Belt {
            pose: Pose::from_parts(
                centre - Vector::new(0.0, 0.0, 0.75),
                Rotation::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ),
            ..belt("c", 0.0, 2.0, 0.5, false)
        };
        let mut w = running(Layout {
            belts: vec![belt("a", 0.0, 2.0, 0.5, false), lane, off],
            turntables: vec![1],
            sources: vec![feeding(size, 1, 1000)],
            sinks: vec![Sink {
                id: "sink".into(),
                exit: Exit::End(2),
            }],
            ..Layout::default()
        });
        arrive(&mut w, centre.x);
        w.set_belt_speed(1, 0.0);
        run(&mut w, 500);
        let before = w.cartons()[0].position;
        let rate: Real = 20.0;
        let mut angle: Real = 0.0;
        while angle < 90.0 {
            angle = (angle + rate * 0.001).min(90.0);
            let r = if angle < 90.0 { rate } else { 0.0 };
            w.set_turntable(0, angle.to_radians(), r.to_radians());
            w.step();
        }
        let quarter = Rotation::from_rotation_y(std::f32::consts::FRAC_PI_2);
        let (yaw, tilt) = attitude(&w, 0, quarter);
        let at = w.cartons()[0].position;
        // Where the deck's turn carries the product from where it stopped.
        let carried = centre + quarter * (Vector::new(before[0], 0.0, before[2]) - centre);
        let off = Vector::new(at[0] - carried.x, 0.0, at[2] - carried.z).length();
        assert!(off <= 0.002, "{name}: {off} m off its carried place");
        assert!(yaw.abs() <= 0.5, "{name}: {yaw}° off the turn");
        assert!(tilt < 1.0, "{name}: tilted {tilt}°");
        w.set_belt_speed(1, 0.5);
        run(&mut w, 8000);
        assert_eq!(w.delivered(0), 1, "{name}: off the deck into belt C's sink");
    }
}

/// A raised blade holds a queue of three, upright, each against the next.
#[test]
fn oip_sizes_queue_upright_at_a_stopper() {
    for (name, size) in SIZES {
        let mut w = running(Layout {
            belts: vec![Belt {
                grip: ROLLER_GRIP,
                ..belt("r", 0.0, 6.0, 0.5, true)
            }],
            stoppers: vec![Stopper {
                id: "s".into(),
                host: 0,
                at: roller_gap(6.0, 4.0),
                width: 0.57,
                raise: 0.04,
            }],
            sources: vec![feeding(size, 3, 2500)],
            ..Layout::default()
        });
        w.set_stopper(0, 1.0);
        let mut worst: Real = 0.0;
        for _ in 0..20_000 {
            w.step();
            for k in 0..w.products.len() {
                worst = worst.max(attitude(&w, k, Rotation::IDENTITY).1);
            }
        }
        let x: Vec<Real> = w.cartons().iter().map(|c| c.position[0]).collect();
        let face = roller_gap(6.0, 4.0) - 0.015;
        assert_eq!(x.len(), 3, "{name}");
        assert!(
            (face - (x[0] + size[0] / 2.0)).abs() <= 0.002,
            "{name}: front at {x:?}"
        );
        for pair in x.windows(2) {
            assert!(
                (pair[0] - pair[1] - size[0]).abs() < 0.01,
                "{name}: a queue {x:?}"
            );
        }
        assert!(worst < 2.0, "{name}: tilted {worst}°");
    }
}

/// A pusher sweeps the product onto a takeaway wide enough for its length, upright and square, and its sink takes it.
#[test]
fn oip_sizes_are_pushed_upright_onto_a_takeaway() {
    for (name, size) in SIZES {
        let mut host = belt("h", 0.0, 5.0, 0.5, true);
        host.guards = [false, true];
        let mut takeaway = belt("t", 0.0, 3.0, 0.5, false);
        takeaway.pose = Pose::from_parts(
            Vector::new(2.5, 0.0, -0.32),
            Rotation::from_rotation_y(std::f32::consts::FRAC_PI_2),
        );
        takeaway.width = size[0] + 0.2;
        takeaway.guards = [true; 2];
        let mut w = running(Layout {
            belts: vec![host, takeaway],
            pushers: vec![Pusher {
                id: "p".into(),
                host: 0,
                at: 2.5,
                stroke: 0.75,
                out_right: false,
            }],
            sources: vec![feeding(size, 1, 1000)],
            sinks: vec![Sink {
                id: "sink".into(),
                exit: Exit::End(1),
            }],
            ..Layout::default()
        });
        arrive(&mut w, 2.5);
        let mut worst: Real = 0.0;
        for ms in 1..=1000 {
            let travel = if ms <= 250 {
                ms
            } else {
                500_usize.saturating_sub(ms)
            };
            w.set_pusher(0, travel as Real / 250.0);
            w.step();
            worst = worst.max(attitude(&w, 0, Rotation::IDENTITY).1);
        }
        let (yaw, tilt) = attitude(&w, 0, Rotation::IDENTITY);
        assert!(
            worst < 5.0 && tilt < 1.0,
            "{name}: tilted {worst}° then {tilt}°"
        );
        assert!(yaw.abs() <= 3.0, "{name}: {yaw}° off square");
        run(&mut w, 8000);
        assert_eq!(
            w.delivered(0),
            1,
            "{name}: carried off into the takeaway's sink"
        );
    }
}

/// A raised chain transfer carries the product off its host's side into the sink there.
#[test]
fn oip_sizes_leave_by_a_chain_transfer() {
    for (name, size) in SIZES {
        let mut w = running(Layout {
            belts: vec![Belt {
                grip: ROLLER_GRIP,
                ..belt("r", 0.0, 4.0, 0.5, true)
            }],
            transfers: vec![Transfer {
                id: "t".into(),
                host: 0,
                at: 2.0,
                along: 0.66,
                across: 0.6,
                speed: 0.5,
                out_right: false,
                closed_exit: false,
            }],
            sources: vec![feeding(size, 1, 1000)],
            sinks: vec![Sink {
                id: "sink".into(),
                exit: Exit::Side(0),
            }],
            ..Layout::default()
        });
        arrive(&mut w, 2.0);
        w.set_belt_speed(0, 0.0);
        let mut worst: Real = 0.0;
        for ms in 1..=4000 {
            w.set_transfer(0, (ms as Real / 150.0).min(1.0), true);
            w.step();
            if !w.products.is_empty() {
                worst = worst.max(attitude(&w, 0, Rotation::IDENTITY).1);
            }
        }
        assert_eq!(w.delivered(0), 1, "{name}");
        assert!(worst < 3.0, "{name}: tilted {worst}°");
    }
}
