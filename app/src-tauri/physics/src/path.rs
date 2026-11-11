//! A conveyor's deck and guards as boxes, straight or curved.
//!
//! Translated from Open Industry Project at `1e7488c` (MIT; THIRD_PARTY_NOTICES.md):
//! - a curve's deck: chords of at most `MAX_SEGMENT_DEG` and a straight stub of `ARC_END_EXT` at each end
//!   (`src/core/geometry/collisionShapes.ts` `sdkCurvedDeckSubs`), driven by one rotating surface field;
//! - the guards: their inner face at the frame's wall, `GUARD_HEIGHT` above the surface, as thick as
//!   `GUARD_COLLISION_THICKNESS`, with no friction (`src/core/types/base.ts`; `src/sim/rigBuild.ts`: "a guard is
//!   friction 0").
//!
//! A curve's origin is its entry, on the centreline: it runs along local +x for the entry stub, turns about a centre
//! `inner_radius + width / 2` to its left (−z) or right (+z), and leaves along the exit stub (spec
//! 2026-11-10-curves-belt-options-design.md, amended in Task 2).

use rapier3d::prelude::{Pose, Real, Rotation, Vector};

use crate::layout::{Arc, Belt};

/// OIP `ARC_END_EXT`: half the 0.17 m belt height, the end drum's radius.
pub const STUB: Real = 0.085;
/// OIP `MAX_SEGMENT_DEG`.
const MAX_SEGMENT: Real = 10.0 * std::f32::consts::PI / 180.0;
/// How far each chord overlaps the next (OIP: `chord / 2 + 0.01`).
const CHORD_OVERLAP: Real = 0.01;
/// A turntable's caps (OIP `turntable-conveyor/part.js` `laneCap`): each `CAP_DEPTH` widths deep plus a centimetre
/// each side, centred `CAP_CENTRE` widths across, so it overlaps the lane by 2 cm and ends at the rim, 0.75 widths out.
const CAP_DEPTH: Real = 0.25;
const CAP_CENTRE: Real = 0.625;
const CAP_OVERLAP: Real = 0.01;
/// One solid deck per conveyor, as deep as OIP's frame (CONVEYOR_BODY_THICKNESS): a thin slab over a housing leaves a
/// notch cargo catches on at junctions.
pub const DECK_THICKNESS: Real = 0.17;
pub const GUARD_HEIGHT: Real = 0.15;
const GUARD_THICKNESS: Real = 0.05;
const FRAME_WALL: Real = 0.01;

/// How a piece of deck moves its load: along a direction, or round its curve's centre.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Motion {
    /// The piece's local +x, in the world.
    Along(Rotation),
    /// About `centre`, in the world, at `rate` × the centreline's speed (rad/s per m/s, along the axis it turns about).
    Turn { centre: Vector, rate: Vector },
}

/// A box in the world: its pose and its half extents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Piece {
    pub pose: Pose,
    pub half: Vector,
}

impl Arc {
    /// +1 for a curve to the left (−z), −1 to the right.
    fn turn(&self) -> Real {
        if self.right { -1.0 } else { 1.0 }
    }

    /// The centreline's radius.
    pub fn radius(&self, width: Real) -> Real {
        self.inner_radius + width / 2.0
    }

    /// The curve's frame on the centreline, `angle` into the arc, in the curve's frame: x along the flow, z to its
    /// right.
    pub fn frame(&self, width: Real, angle: Real) -> Pose {
        let (t, r) = (self.turn(), self.radius(width));
        let (sin, cos) = angle.sin_cos();
        Pose::from_parts(
            Vector::new(STUB + r * sin, 0.0, -t * r * (1.0 - cos)),
            Rotation::from_rotation_y(t * angle),
        )
    }

    /// The exit's frame: the tip of the exit stub.
    pub fn end(&self, width: Real) -> Pose {
        self.frame(width, self.sweep) * Pose::from_translation(Vector::new(STUB, 0.0, 0.0))
    }

    /// The centre the arc turns about, in the curve's frame.
    pub(crate) fn centre(&self, width: Real) -> Vector {
        Vector::new(STUB, 0.0, -self.turn() * self.radius(width))
    }

    /// The angular velocity about the curve's up axis, per m/s of the centreline.
    fn rate(&self, width: Real) -> Real {
        self.turn() / self.radius(width)
    }

    /// The chords' bounds, in angle along the arc.
    fn chords(&self) -> impl Iterator<Item = (Real, Real)> {
        let n = (self.sweep / MAX_SEGMENT).ceil().max(1.0) as usize;
        let step = self.sweep / n as Real;
        (0..n).map(move |i| (i as Real * step, (i + 1) as Real * step))
    }

    /// A box along the chord from `a` to `b` (angles along the arc), `offset` to the right of the centreline.
    fn chord(&self, width: Real, (a, b): (Real, Real), offset: Real) -> (Pose, Real) {
        let at = |angle| {
            (self.frame(width, angle) * Pose::from_translation(Vector::new(0.0, 0.0, offset)))
                .translation
        };
        let (pa, pb) = (at(a), at(b));
        let pose = Pose::from_parts(
            (pa + pb) / 2.0,
            Rotation::from_rotation_y(self.turn() * (a + b) / 2.0),
        );
        (pose, pa.distance(pb))
    }
}

/// A box `half` large whose centre is `at` in `frame`.
fn placed(frame: Pose, at: Vector, half: Vector) -> Piece {
    Piece {
        pose: frame * Pose::from_translation(at),
        half,
    }
}

/// A conveyor's deck, and how each box moves its load: one box for a straight conveyor; for a turntable's lane, that
/// box and OIP's two caps that reach the rim across it, a static cross (`turntable-conveyor/part.js`: a rotating
/// rectangle would sweep into its neighbours); a curve's entry stub, chords and exit stub.
pub fn deck(belt: &Belt) -> Vec<(Piece, Motion)> {
    let (pose, w) = (belt.pose, belt.width);
    let y = belt.height - DECK_THICKNESS / 2.0;
    let Some(arc) = belt.arc else {
        let half = Vector::new(belt.length + 2.0 * belt.face, DECK_THICKNESS, w) / 2.0;
        let centre = Vector::new(belt.length / 2.0, y, 0.0);
        let along = Motion::Along(pose.rotation);
        let mut out = vec![(placed(pose, centre, half), along)];
        if belt.turntable {
            let cap = Vector::new(w, DECK_THICKNESS, CAP_DEPTH * w + 2.0 * CAP_OVERLAP) / 2.0;
            for side in [-1.0, 1.0] {
                let at = centre + Vector::new(0.0, 0.0, side * (CAP_CENTRE * w - CAP_OVERLAP));
                out.push((placed(pose, at, cap), along));
            }
        }
        return out;
    };
    let stub = Vector::new(STUB, DECK_THICKNESS, w) / 2.0;
    let end = pose * arc.end(w);
    let turn = Motion::Turn {
        centre: pose.transform_point(arc.centre(w)),
        rate: pose.rotation * Vector::new(0.0, arc.rate(w), 0.0),
    };
    let entry = placed(pose, Vector::new(STUB / 2.0, y, 0.0), stub);
    let mut out = vec![(entry, Motion::Along(pose.rotation))];
    for bounds in arc.chords() {
        let (at, chord) = arc.chord(w, bounds, 0.0);
        let half = Vector::new(chord / 2.0 + CHORD_OVERLAP, DECK_THICKNESS / 2.0, w / 2.0);
        out.push((placed(pose * at, Vector::new(0.0, y, 0.0), half), turn));
    }
    let exit = placed(end, Vector::new(-STUB / 2.0, y, 0.0), stub);
    out.push((exit, Motion::Along(end.rotation)));
    out
}

/// A conveyor's guards, left (−z) then right, those it has: walls from the deck's bottom to `GUARD_HEIGHT` above its
/// surface, their inner face at the frame's wall, open where a pusher stands beside a straight one. On a curve, chords whose ends lie on the arc, as OIP sweeps its
/// guard along a polyline (`examples/parts/_shared/guardArc.js`): a 10° chord bows into the lane by up to 4.7 mm on
/// the default curve's outer guard.
pub fn guards(belt: &Belt) -> Vec<Piece> {
    let (pose, w) = (belt.pose, belt.width);
    let y = belt.height + (GUARD_HEIGHT - DECK_THICKNESS) / 2.0;
    let thick = (GUARD_HEIGHT + DECK_THICKNESS) / 2.0;
    let wall = |length: Real| Vector::new(length / 2.0, thick, GUARD_THICKNESS / 2.0);
    let mut out = Vec::new();
    for (side, right) in [(-1.0, false), (1.0, true)] {
        if !belt.guards[usize::from(right)] {
            continue;
        }
        let z = side * (w / 2.0 + FRAME_WALL + GUARD_THICKNESS / 2.0);
        let Some(arc) = belt.arc else {
            for (from, to) in spans(belt, right) {
                out.push(placed(
                    pose,
                    Vector::new((from + to) / 2.0, y, z),
                    wall(to - from),
                ));
            }
            continue;
        };
        out.push(placed(pose, Vector::new(STUB / 2.0, y, z), wall(STUB)));
        for bounds in arc.chords() {
            let (at, chord) = arc.chord(w, bounds, z);
            out.push(placed(
                pose * at,
                Vector::new(0.0, y, 0.0),
                wall(chord + CHORD_OVERLAP),
            ));
        }
        let end = pose * arc.end(w);
        out.push(placed(end, Vector::new(-STUB / 2.0, y, z), wall(STUB)));
    }
    out
}

/// The stretches of a straight conveyor's guard, left or right, from face to face, between the openings the pushers
/// beside it cut.
fn spans(belt: &Belt, right: bool) -> Vec<(Real, Real)> {
    let mut cuts: Vec<(Real, Real)> = (belt.openings.iter())
        .filter(|o| o.right == right)
        .map(|o| (o.centre - o.length / 2.0, o.centre + o.length / 2.0))
        .collect();
    cuts.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut out = Vec::new();
    let (mut from, to) = (-belt.face, belt.length + belt.face);
    for (start, end) in cuts {
        if start > from {
            out.push((from, start.min(to)));
        }
        from = from.max(end);
    }
    if from < to {
        out.push((from, to));
    }
    out
}

/// A curve's centreline length: both stubs and the arc.
pub fn arc_length(arc: &Arc, width: Real) -> Real {
    2.0 * STUB + arc.radius(width) * arc.sweep
}

#[cfg(test)]
#[path = "path_tests.rs"]
mod tests;
