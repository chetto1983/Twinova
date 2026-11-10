use super::*;
use crate::layout::BELT_GRIP;

fn near(a: Real, b: Real) -> bool {
    (a - b).abs() < 1e-4
}

fn arc(sweep_deg: Real, right: bool) -> Arc {
    Arc {
        inner_radius: 0.6,
        sweep: sweep_deg.to_radians(),
        right,
    }
}

pub(crate) fn conveyor(arc: Option<Arc>, guards: [bool; 2]) -> Belt {
    Belt {
        id: "c".into(),
        pose: Pose::from_translation(Vector::new(1.0, 0.0, 2.0)),
        length: 3.0,
        width: 0.6,
        height: 0.8,
        speed: 0.5,
        grip: BELT_GRIP,
        closed_exit: false,
        arc,
        guards,
        skew: 0.0,
    }
}

/// The distance in plan from a curve's centre (in its frame), and the heading, of a pose.
fn polar(p: Pose, centre: Vector) -> (Real, Vector) {
    let d = p.translation - centre;
    ((d.x * d.x + d.z * d.z).sqrt(), p.rotation * Vector::X)
}

#[test]
fn a_curve_hands_over_a_stub_past_its_arc_turning_left_or_right() {
    let left = arc(90.0, false).end(0.6);
    assert!(
        near(left.translation.x, 0.985) && near(left.translation.z, -0.985),
        "{left:?}"
    );
    assert!((left.rotation * Vector::X - Vector::NEG_Z).length() < 1e-5);
    let right = arc(90.0, true).end(0.6);
    assert!(
        near(right.translation.x, 0.985) && near(right.translation.z, 0.985),
        "{right:?}"
    );
    assert!((right.rotation * Vector::X - Vector::Z).length() < 1e-5);
    let half_turn = arc(180.0, false).end(0.6);
    assert!(near(half_turn.translation.x, 0.0) && near(half_turn.translation.z, -1.8));
    assert!(near(
        arc_length(&arc(90.0, false), 0.6),
        0.17 + 0.9 * std::f32::consts::FRAC_PI_2
    ));
}

#[test]
fn a_curve_frame_runs_on_the_centreline_along_the_flow() {
    let a = arc(90.0, false);
    let centre = a.centre(0.6);
    assert!(near(centre.x, STUB) && near(centre.z, -0.9));
    for deg in [0.0, 30.0, 45.0, 90.0] {
        let (r, heading) = polar(a.frame(0.6, (deg as Real).to_radians()), centre);
        assert!(near(r, 0.9), "{deg}: {r}");
        let radial = (a.frame(0.6, (deg as Real).to_radians()).translation - centre) / r;
        assert!(heading.dot(radial).abs() < 1e-5, "{deg}: tangent");
        // The centre is on the left: the outward radial points to the right of the flow, and x × z = −y.
        assert!(
            heading.cross(radial).y < 0.0,
            "{deg}: the centre is on the left"
        );
    }
}

#[test]
fn a_straight_deck_is_one_box_along_the_conveyor() {
    let belt = conveyor(None, [true; 2]);
    let deck = deck(&belt);
    assert_eq!(deck.len(), 1);
    let (piece, motion) = deck[0];
    assert_eq!(piece.half, Vector::new(1.5, DECK_THICKNESS / 2.0, 0.3));
    assert_eq!(
        piece.pose.translation,
        Vector::new(2.5, 0.8 - DECK_THICKNESS / 2.0, 2.0)
    );
    assert_eq!(motion, Motion::Along(belt.pose.rotation));
}

/// OIP: chords of at most 10°, each `chord / 2 + 0.01` long each side, between two stubs of `ARC_END_EXT`.
#[test]
fn a_curve_deck_is_two_stubs_and_chords_of_at_most_ten_degrees() {
    for (sweep, chords) in [(90.0, 9), (15.0, 2), (180.0, 18), (95.0, 10)] {
        let belt = conveyor(Some(arc(sweep, false)), [false; 2]);
        let deck = deck(&belt);
        assert_eq!(deck.len(), chords + 2, "{sweep}");
        let step = sweep.to_radians() / chords as Real;
        let (stub, _) = deck[0];
        assert_eq!(
            stub.half,
            Vector::new(STUB / 2.0, DECK_THICKNESS / 2.0, 0.3)
        );
        let Motion::Turn { centre, rate } = deck[1].1 else {
            panic!("a chord turns");
        };
        assert!(
            near(centre.x, 1.0 + STUB) && near(centre.z, 2.0 - 0.9),
            "{centre:?}"
        );
        assert!(near(rate.y, 1.0 / 0.9) && near(rate.x, 0.0), "{rate:?}");
        for (piece, _) in &deck[1..=chords] {
            let chord = 2.0 * 0.9 * (step / 2.0).sin();
            assert!(
                near(piece.half.x, chord / 2.0 + 0.01),
                "{sweep}: {:?}",
                piece.half
            );
            let local = Pose::from_translation(belt.pose.translation).inverse() * piece.pose;
            let (r, _) = polar(local, arc(sweep, false).centre(0.6));
            assert!(near(r, 0.9 * (step / 2.0).cos()), "{sweep}: {r}");
            assert!(near(piece.pose.translation.y, 0.8 - DECK_THICKNESS / 2.0));
        }
        let (exit, motion) = deck[chords + 1];
        let end = belt.pose * arc(sweep, false).end(0.6);
        let tip = exit.pose * Pose::from_translation(Vector::new(STUB / 2.0, 0.0, 0.0));
        assert!(
            near(tip.translation.x, end.translation.x)
                && near(tip.translation.z, end.translation.z)
        );
        assert_eq!(motion, Motion::Along(end.rotation));
    }
}

#[test]
fn a_right_curve_turns_the_other_way_about_a_centre_on_its_right() {
    let belt = conveyor(Some(arc(90.0, true)), [false; 2]);
    let Motion::Turn { centre, rate } = deck(&belt)[1].1 else {
        panic!("a chord turns");
    };
    assert!(
        near(centre.z, 2.0 + 0.9) && near(rate.y, -1.0 / 0.9),
        "{centre:?} {rate:?}"
    );
}

/// OIP's guards: the inner face at the frame's wall, `width / 2 + 0.01`, 0.15 m above the surface.
#[test]
fn straight_guards_stand_at_the_frame_wall_on_the_sides_they_are_on() {
    assert!(guards(&conveyor(None, [false; 2])).is_empty());
    let both = guards(&conveyor(None, [true; 2]));
    assert_eq!(both.len(), 2);
    for (piece, side) in both.iter().zip([-1.0, 1.0]) {
        let inner = (piece.pose.translation.z - 2.0) * side - piece.half.z;
        assert!(near(inner, 0.31), "{inner}");
        assert!(near(
            piece.pose.translation.y + piece.half.y,
            0.8 + GUARD_HEIGHT
        ));
        assert!(near(
            piece.pose.translation.y - piece.half.y,
            0.8 - DECK_THICKNESS
        ));
        assert!(near(piece.half.x, 1.5) && near(piece.pose.translation.x, 2.5));
    }
    let right = guards(&conveyor(None, [false, true]));
    assert_eq!(right.len(), 1);
    assert!(right[0].pose.translation.z > 2.0);
}

/// Every guard of a curve stands at the frame wall, inside and outside the arc, to within the bow of a 10° chord of
/// the outer guard, 1.235 × (1 − cos 5°) = 4.7 mm, as OIP's guard polyline bows (`_shared/guardArc.js`).
#[test]
fn curve_guards_follow_the_arc_inside_and_outside() {
    for right in [false, true] {
        let a = arc(90.0, right);
        let belt = conveyor(Some(a), [true; 2]);
        let walls = guards(&belt);
        assert_eq!(walls.len(), 2 * (9 + 2), "{right}");
        let origin = Pose::from_translation(belt.pose.translation);
        let centre = a.centre(0.6);
        for (k, piece) in walls.iter().enumerate() {
            let local = origin.inverse() * piece.pose;
            let face = |dz: Real| local * Pose::from_translation(Vector::new(0.0, 0.0, dz));
            let inner_face = [face(-piece.half.z), face(piece.half.z)];
            let lane = inner_face
                .iter()
                .map(|f| match k % 11 {
                    0 => f.translation.z.abs(),
                    10 => {
                        let end = a.end(0.6);
                        (end.inverse() * *f).translation.z.abs()
                    }
                    _ => (polar(*f, centre).0 - 0.9).abs(),
                })
                .fold(Real::INFINITY, Real::min);
            assert!(lane >= 0.31 - 0.0048, "{right} wall {k}: {lane}");
            assert!(lane <= 0.31 + 0.003, "{right} wall {k}: {lane}");
        }
    }
}
