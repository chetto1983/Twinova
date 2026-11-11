//! The devices that move in the line: OIP's chain transfer, blade stop and pusher, each a kinematic body moved before
//! each step (OIP `src/sim/rigDriver.ts`, `MoveKinematic`; Rapier's position-based kinematic body), and their side
//! exits; and OIP's turntable, whose lane stays still while its flow turns. A kinematic body does not collide with the
//! fixed decks and guards it passes through (Rapier's default `ActiveCollisionTypes`), only with the cartons.
//!
//! The turntable carries its load by its surface alone: while the deck turns, the lane is the curves' rotating field
//! about its centre at the deck's rate; settled, its flow runs along the deck's angle (OIP `turntable-conveyor/part.js`
//! `step`). OIP's deck body and its grip are not translated: cargo never touches the deck, and in Rapier the field
//! alone turns a carton 90° within 0.01° (spec, measured).
//!
//! Translated from Open Industry Project at `1e7488c` (MIT; THIRD_PARTY_NOTICES.md): the colliders of
//! `examples/parts/{chain-transfer,blade-stop,diverter}/rig.json`, as the spec 2026-11-11-stopper-pusher-turntable-design.md
//! measured them in Rapier.

use rapier3d::prelude::*;

use super::{WALL_HEIGHT, World, along, side};
use crate::layout::{Exit, Layout, Pusher, Stopper, Transfer};
use crate::path::Motion;
use crate::surfaces::{CARTON_FRICTION, Drive, Surfaces, surface};

/// The chain transfer (OIP `examples/parts/chain-transfer/`): its pop-up stroke (`RAISE`), its deck's depth (`rig.json`
/// deck 0.06), and how far under the host's rollers it is fitted (`snap.track.targetLocalY` −0.03).
pub const TRANSFER_STROKE: Real = 0.095;
const TRANSFER_DECK: Real = 0.06;
pub(super) const TRANSFER_BELOW: Real = 0.03;
/// OIP's chains grip as rollers (`conveyor.surface: "roller"`, `CONVEYOR_FRICTION`).
const CHAIN_GRIP: Real = 0.35;
/// The blade stop (OIP `blade-stop/part.js`): the blade's thickness, how far it reaches under the surface when up
/// (`BLADE_BOTTOM`), and how far under the surface its top rests when down (`LOWERED_TOP`).
const BLADE_THICKNESS: Real = 0.03;
const BLADE_BELOW: Real = 0.15;
const BLADE_LOWERED: Real = 0.05;
/// The pusher's paddle collider (OIP `diverter/rig.json` link `blade`): along the host, high and deep, its bottom above
/// the surface, and its face's clearance outside the host's frame (`mount.outwardOffset` 0.05 less the plate's 0.0395).
const PADDLE: Vector = Vector::new(0.47, 0.40, 0.50);
const PADDLE_LIFT: Real = 0.047;
const PADDLE_CLEARANCE: Real = 0.0105;
/// The frame's wall outside a conveyor's width (path.rs `FRAME_WALL`).
const FRAME_WALL: Real = 0.01;
/// A blade or a paddle grips a carton as OIP's ground 0.6 (`bodyFactory.ts` `GROUND_FRICTION`) against its box's 0.5
/// (`base.ts` `BOX_FRICTION`), combined as Jolt does by default, by their geometric mean: √0.3.
const STEEL_GRIP: Real = 0.547_722_6;

/// A device's kinematic body, and its position as last set: 0 at rest to 1 at the end of its stroke.
pub(super) struct Moving {
    pub(super) body: RigidBodyHandle,
    at: Real,
}

/// A turntable's deck: its lane, the centre it turns about, and its angle (radians, counter-clockwise seen from above,
/// from its start heading) and rate (rad/s) as last set.
pub(super) struct Spin {
    lane: usize,
    centre: Vector,
    angle: Real,
    rate: Real,
}

impl Spin {
    /// The drive of a piece of its lane that runs along `rotation`, at `speed` along the deck's heading.
    fn drive(&self, rotation: Rotation, speed: Real) -> Drive {
        if self.rate != 0.0 {
            return Drive::Turn {
                centre: self.centre,
                omega: rotation * Vector::new(0.0, self.rate, 0.0),
            };
        }
        Drive::Linear(
            rotation * Rotation::from_rotation_y(self.angle) * Vector::new(speed, 0.0, 0.0),
        )
    }
}

/// Each device's body, by layout index; a chain transfer's surface too; each turntable's deck.
pub(super) struct Devices {
    transfers: Vec<(usize, Moving)>,
    pub(super) stoppers: Vec<Moving>,
    pub(super) pushers: Vec<Moving>,
    spins: Vec<Spin>,
}

/// A side exit: along `host` at `at`, as far each way as `span`, past the edge `edge` from its centreline, to its
/// right or its left.
#[derive(Clone, Copy)]
pub(super) struct Side {
    pub host: usize,
    pub at: Real,
    pub span: Real,
    pub edge: Real,
    pub right: bool,
}

/// The side exit of a chain transfer or a pusher: the deck's span or the paddle's. A product counts while it overlaps it
/// (`World::reached`), as a chute beside the transfer catches what the chains drag off (measured 2026-11-10: with the
/// deck's own span, a queue standing over a raised transfer spilled the cartons it half carried onto the floor).
pub(super) fn side_exit(layout: &Layout, exit: Exit) -> Option<Side> {
    match exit {
        Exit::End(_) => None,
        Exit::Side(k) => {
            let t = &layout.transfers[k];
            Some(Side {
                host: t.host,
                at: t.at,
                span: t.along / 2.0,
                edge: t.across / 2.0,
                right: t.out_right,
            })
        }
        Exit::Push(k) => {
            let p = &layout.pushers[k];
            Some(Side {
                host: p.host,
                at: p.at,
                span: PADDLE.x / 2.0,
                edge: layout.belts[p.host].width / 2.0,
                right: p.out_right,
            })
        }
    }
}

/// How far a side exit's wall runs past its span each way: M3's carton's half, as it did before products had sizes.
const SIDE_WALL_MARGIN: Real = 0.15;

/// The wall along a side exit that feeds nothing.
pub(super) fn side_wall(layout: &Layout, s: Side) -> ColliderBuilder {
    let b = &layout.belts[s.host];
    let at = Vector::new(
        s.at,
        b.height + WALL_HEIGHT / 2.0,
        side(s.right) * (s.edge + 0.01),
    );
    ColliderBuilder::cuboid(s.span + SIDE_WALL_MARGIN, WALL_HEIGHT / 2.0, 0.01)
        .position(b.pose * Pose::from_translation(at))
}

/// A chain transfer's deck centre in its host's frame, at `lift` (0 down, 1 up) of its stroke.
pub(super) fn deck_centre(layout: &Layout, transfer: usize, lift: Real) -> Vector {
    let t = &layout.transfers[transfer];
    let top = layout.belts[t.host].height - TRANSFER_BELOW + TRANSFER_STROKE * lift;
    Vector::new(t.at, top - TRANSFER_DECK / 2.0, 0.0)
}

/// A stopper's blade centre in its host's frame, at `lift` of its stroke: up, it reaches from `BLADE_BELOW` under the
/// surface to `raise` above it; down, its top is `BLADE_LOWERED` under the surface.
fn blade_centre(layout: &Layout, s: &Stopper, lift: Real) -> Vector {
    let surface = layout.belts[s.host].height;
    let stroke = s.raise + BLADE_LOWERED;
    let up = surface - BLADE_BELOW + (s.raise + BLADE_BELOW) / 2.0;
    Vector::new(s.at, up - (1.0 - lift) * stroke, 0.0)
}

/// A pusher's paddle centre in its host's frame, at `reach` of its stroke: at rest its face is just outside the
/// frame on the side opposite its exit.
fn paddle_centre(layout: &Layout, p: &Pusher, reach: Real) -> Vector {
    let b = &layout.belts[p.host];
    let rest = b.width / 2.0 + FRAME_WALL + PADDLE_CLEARANCE + PADDLE.z / 2.0;
    let y = b.height + PADDLE_LIFT + PADDLE.y / 2.0;
    Vector::new(p.at, y, side(p.out_right) * (reach * p.stroke - rest))
}

/// A steel box that grips a carton as `STEEL_GRIP`, on a kinematic body.
fn steel(half: Vector) -> ColliderBuilder {
    ColliderBuilder::cuboid(half.x, half.y, half.z)
        .friction(STEEL_GRIP / CARTON_FRICTION)
        .friction_combine_rule(CoefficientCombineRule::Multiply)
}

fn kinematic(
    world: &mut PhysicsWorld,
    host: Pose,
    at: Vector,
    collider: ColliderBuilder,
) -> Moving {
    let body = world.insert_body(
        RigidBodyBuilder::kinematic_position_based().pose(host * Pose::from_translation(at)),
    );
    world.insert_collider(collider, Some(body));
    Moving { body, at: 0.0 }
}

impl Devices {
    /// Every device's body at rest, and the walls along the side exits that feed nothing.
    pub(super) fn new(world: &mut PhysicsWorld, hooks: &mut Surfaces, layout: &Layout) -> Self {
        let pose = |host: usize| layout.belts[host].pose;
        let mut transfers = Vec::new();
        for (k, t) in layout.transfers.iter().enumerate() {
            let half = Vector::new(t.along, TRANSFER_DECK, t.across) / 2.0;
            let index = hooks.surface.len();
            let deck = surface(half, CHAIN_GRIP, index);
            transfers.push((
                index,
                kinematic(world, pose(t.host), deck_centre(layout, k, 0.0), deck),
            ));
            hooks.surface.push(Drive::Linear(Vector::ZERO));
        }
        let stoppers = (layout.stoppers.iter())
            .map(|s| {
                let half = Vector::new(BLADE_THICKNESS, s.raise + BLADE_BELOW, s.width) / 2.0;
                kinematic(
                    world,
                    pose(s.host),
                    blade_centre(layout, s, 0.0),
                    steel(half),
                )
            })
            .collect();
        let pushers = (layout.pushers.iter())
            .map(|p| {
                kinematic(
                    world,
                    pose(p.host),
                    paddle_centre(layout, p, 0.0),
                    steel(PADDLE / 2.0),
                )
            })
            .collect();
        for (k, _) in layout
            .transfers
            .iter()
            .enumerate()
            .filter(|(_, t)| t.closed_exit)
        {
            let wall = side_wall(
                layout,
                side_exit(layout, Exit::Side(k)).expect("a side exit"),
            );
            world.insert_collider(wall, None);
        }
        let spins = (layout.turntables.iter())
            .map(|&lane| {
                let b = &layout.belts[lane];
                let centre = b
                    .pose
                    .transform_point(Vector::new(b.length / 2.0, 0.0, 0.0));
                Spin {
                    lane,
                    centre,
                    angle: 0.0,
                    rate: 0.0,
                }
            })
            .collect();
        Self {
            transfers,
            stoppers,
            pushers,
            spins,
        }
    }

    /// The drive of a piece of `belt`'s deck that moves its load by `motion` at `speed`: a turntable's lane turns with
    /// its deck.
    pub(super) fn drive(&self, belt: usize, motion: Motion, speed: Real, skew: Real) -> Drive {
        let spin = self.spins.iter().find(|s| s.lane == belt);
        match (motion, spin) {
            (Motion::Along(rotation), Some(spin)) => spin.drive(rotation, speed),
            (Motion::Along(rotation), None) => Drive::Linear(rotation * along(speed, skew)),
            (Motion::Turn { centre, rate }, _) => Drive::Turn {
                centre,
                omega: rate * speed,
            },
        }
    }
}

/// Moves `moving` to `at` of its stroke, `centre` in its host's frame, by the next step.
fn pose_at(world: &mut PhysicsWorld, moving: &mut Moving, host: Pose, centre: Vector, at: Real) {
    moving.at = at;
    let to = host * Pose::from_translation(centre);
    world.bodies[moving.body].set_next_kinematic_translation(to.translation);
}

impl World {
    pub fn transfers(&self) -> usize {
        self.layout.transfers.len()
    }

    pub fn transfer_id(&self, transfer: usize) -> &str {
        &self.layout.transfers[transfer].id
    }

    pub fn transfer_lift(&self, transfer: usize) -> Real {
        self.devices.transfers[transfer].1.at
    }

    /// A chain transfer's deck at `lift` of its stroke, carrying sideways while `raised` and forward otherwise, at its
    /// speed (OIP `chain-transfer/behavior.ts` `step`). The deck reaches that height at the next step.
    pub fn set_transfer(&mut self, transfer: usize, lift: Real, raised: bool) {
        let t: &Transfer = &self.layout.transfers[transfer];
        let host = self.layout.belts[t.host].pose;
        let local = if raised {
            Vector::new(0.0, 0.0, side(t.out_right) * t.speed)
        } else {
            Vector::new(t.speed, 0.0, 0.0)
        };
        let (index, moving) = &mut self.devices.transfers[transfer];
        self.hooks.surface[*index] = Drive::Linear(host.rotation * local);
        let centre = deck_centre(&self.layout, transfer, lift);
        pose_at(&mut self.world, moving, host, centre, lift);
    }

    pub fn stoppers(&self) -> usize {
        self.layout.stoppers.len()
    }

    pub fn stopper_id(&self, stopper: usize) -> &str {
        &self.layout.stoppers[stopper].id
    }

    pub fn stopper_lift(&self, stopper: usize) -> Real {
        self.devices.stoppers[stopper].at
    }

    /// A stopper's blade at `lift` of its stroke, 0 down and 1 up (OIP `blade-stop/part.js` `step`), by the next step.
    pub fn set_stopper(&mut self, stopper: usize, lift: Real) {
        let s = &self.layout.stoppers[stopper];
        let (host, centre) = (
            self.layout.belts[s.host].pose,
            blade_centre(&self.layout, s, lift),
        );
        pose_at(
            &mut self.world,
            &mut self.devices.stoppers[stopper],
            host,
            centre,
            lift,
        );
    }

    pub fn pushers(&self) -> usize {
        self.layout.pushers.len()
    }

    pub fn pusher_id(&self, pusher: usize) -> &str {
        &self.layout.pushers[pusher].id
    }

    pub fn pusher_reach(&self, pusher: usize) -> Real {
        self.devices.pushers[pusher].at
    }

    /// A pusher's paddle at `reach` of its stroke, 0 home and 1 out (OIP `diverter/behavior.ts` `step`), by the next
    /// step.
    pub fn set_pusher(&mut self, pusher: usize, reach: Real) {
        let p = &self.layout.pushers[pusher];
        let (host, centre) = (
            self.layout.belts[p.host].pose,
            paddle_centre(&self.layout, p, reach),
        );
        pose_at(
            &mut self.world,
            &mut self.devices.pushers[pusher],
            host,
            centre,
            reach,
        );
    }

    pub fn turntables(&self) -> usize {
        self.devices.spins.len()
    }

    pub fn turntable_id(&self, turntable: usize) -> &str {
        &self.layout.belts[self.devices.spins[turntable].lane].id
    }

    /// A turntable's deck angle as last set, radians counter-clockwise seen from above.
    pub fn turntable_angle(&self, turntable: usize) -> Real {
        self.devices.spins[turntable].angle
    }

    /// A turntable's deck at `angle` (radians, counter-clockwise seen from above, from its start heading), turning at
    /// `rate` (rad/s): its lane turns its load with it while `rate` is not 0, and runs along `angle` otherwise.
    pub fn set_turntable(&mut self, turntable: usize, angle: Real, rate: Real) {
        let spin = &mut self.devices.spins[turntable];
        (spin.angle, spin.rate) = (angle, rate);
        let lane = spin.lane;
        self.drive(lane);
    }
}
