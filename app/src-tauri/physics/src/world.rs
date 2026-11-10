//! The line's physics (spec 2026-11-09-component-behaviour-design.md, "Transport with physics"; research 15 and 16):
//! products are rigid bodies in Rapier, and a conveyor is fixed boxes whose contacts get a surface velocity
//! (`surfaces.rs`), straight or curved, with its guards (`path.rs`). Nothing hands a product from one conveyor to the
//! next: contacts and friction do, as in Open Industry Project and NX MCD. One step is 1 ms of virtual time.
//!
//! Translated from Open Industry Project at `1e7488c` (MIT, Copyright (c) 2024-present Automation Standard LLC and
//! Contributors; THIRD_PARTY_NOTICES.md): a deck's surface velocity (`src/sim/deckDrive.ts` `setSurfaceVelocity`,
//! and a skewed roller's in `examples/parts/roller-conveyor/behavior.ts`), and a moving link as a kinematic body moved
//! before each step (`src/sim/rigDriver.ts`, `MoveKinematic`), here for the chain transfer's lift. OIP's ghost-contact
//! gates and its friction coupling are not translated: they patch Jolt, and Rapier passes these tests without them
//! (research 15).

use rapier3d::prelude::*;
use serde::Serialize;

use crate::layout::{Belt, Exit, Layout};
use crate::path::{Motion, Piece, deck, guards};
#[cfg(test)]
use crate::surfaces::surface_of_pair;
use crate::surfaces::{CARTON_FRICTION, Drive, Surfaces, surface};

pub const STEP_NS: i64 = 1_000_000;
const STEP_S: Real = 0.001;
/// The products of 4.3: M3's 0.3 m carton.
pub const CARTON_HALF: Real = 0.15;
const CARTON_DENSITY: Real = 100.0;
/// OIP's cargo edge rounding (bodyFactory.ts makeBoxShape): min(0.02 m, 0.4 × the smallest half extent).
const CARTON_ROUNDING: Real = 0.02;
const WALL_HEIGHT: Real = 0.4;
/// The beam of a diffuse sensor, above the running surface.
const BEAM_HEIGHT: Real = 0.05;
/// The chain transfer (OIP `examples/parts/chain-transfer/`): its pop-up stroke (`RAISE`), its deck's depth (`rig.json`
/// deck 0.06), and how far under the host's rollers it is fitted (`snap.track.targetLocalY` −0.03).
pub const TRANSFER_STROKE: Real = 0.095;
const TRANSFER_DECK: Real = 0.06;
const TRANSFER_BELOW: Real = 0.03;
/// OIP's chains grip as rollers (`conveyor.surface: "roller"`, `CONVEYOR_FRICTION`).
const CHAIN_GRIP: Real = 0.35;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Carton {
    pub id: u64,
    pub position: [f32; 3],
    pub rotation: [f32; 4],
}

struct SourceState {
    next_ns: i64,
    made: i64,
}

pub struct World {
    world: PhysicsWorld,
    hooks: Surfaces,
    layout: Layout,
    cartons: Vec<(u64, RigidBodyHandle)>,
    next_id: u64,
    t_ns: i64,
    sources: Vec<SourceState>,
    delivered: Vec<u64>,
    /// The wall each blocked sink puts at its exit.
    sink_walls: Vec<Option<ColliderHandle>>,
    /// Each conveyor's surfaces, by index into the hook's, and how each moves its load.
    surfaces: Vec<Vec<(usize, Motion)>>,
    /// Each chain transfer's surface, its deck, a kinematic body, and its lift as last set.
    transfer_surfaces: Vec<usize>,
    decks: Vec<RigidBodyHandle>,
    lifts: Vec<Real>,
}

/// The wall across an exit: past a conveyor's end, or along a chain transfer's side over its deck.
fn exit_wall(layout: &Layout, exit: Exit) -> ColliderBuilder {
    let (pose, at, half) = match exit {
        Exit::End(k) => {
            let b = &layout.belts[k];
            let (frame, length) = exit_frame(b);
            let at = Vector::new(length + 0.01, b.height + WALL_HEIGHT / 2.0, 0.0);
            (
                frame,
                at,
                Vector::new(0.01, WALL_HEIGHT / 2.0, b.width / 2.0),
            )
        }
        Exit::Side(k) => {
            let t = &layout.transfers[k];
            let b = &layout.belts[t.host];
            let z = side(t.out_right) * (t.across / 2.0 + 0.01);
            let at = Vector::new(t.at, b.height + WALL_HEIGHT / 2.0, z);
            (
                b.pose,
                at,
                Vector::new(side_reach(t), WALL_HEIGHT / 2.0, 0.01),
            )
        }
    };
    ColliderBuilder::cuboid(half.x, half.y, half.z).position(pose * Pose::from_translation(at))
}

/// A frame along which a conveyor's exit lies, and how far: a straight conveyor's own, at its length; a curve's exit
/// stub tip.
fn exit_frame(b: &Belt) -> (Pose, Real) {
    match b.arc {
        None => (b.pose, b.length),
        Some(arc) => (b.pose * arc.end(b.width), 0.0),
    }
}

/// How far along its host, from the deck's centre, a chain transfer's side exit reaches: the deck, and a carton that
/// only overlaps it, as a chute beside the transfer catches what the chains drag off (measured 2026-11-10: with the
/// deck's own span, a queue standing over a raised transfer spilled the cartons it half carried onto the floor).
fn side_reach(t: &crate::layout::Transfer) -> Real {
    t.along / 2.0 + CARTON_HALF
}

/// A deck's surface velocity in its own frame. Skewed rollers drive along their normal, the flow turned by the skew at
/// speed / cos(skew), so the throughput stays `speed` while the load drifts sideways at speed · tan(skew) (OIP
/// `roller-conveyor/behavior.ts`).
fn along(speed: Real, skew: Real) -> Vector {
    if skew == 0.0 {
        return Vector::new(speed, 0.0, 0.0);
    }
    let (sin, cos) = skew.sin_cos();
    let magnitude = speed / cos;
    Vector::new(cos * magnitude, 0.0, -sin * magnitude)
}

/// The sign of a chain transfer's side along its host's z.
fn side(right: bool) -> Real {
    if right { 1.0 } else { -1.0 }
}

/// A chain transfer's deck centre in its host's frame, at `lift` (0 down, 1 up) of its stroke.
fn deck_centre(layout: &Layout, transfer: usize, lift: Real) -> Vector {
    let t = &layout.transfers[transfer];
    let top = layout.belts[t.host].height - TRANSFER_BELOW + TRANSFER_STROKE * lift;
    Vector::new(t.at, top - TRANSFER_DECK / 2.0, 0.0)
}

impl World {
    pub fn new(layout: Layout) -> Self {
        let mut world = PhysicsWorld::default();
        world.integration_parameters.dt = STEP_S;
        // Coulomb friction: one constraint per contact point, each with its own tangent velocity, so a curve turns its
        // load with it. Rapier's default (Simplified) adds a twist constraint with no target velocity that holds the
        // load's yaw (spec 2026-11-10-curves-belt-options-design.md, measured).
        world.integration_parameters.friction_model = FrictionModel::Coulomb;
        world.insert_collider(
            // The floor, for what falls off a belt.
            ColliderBuilder::cuboid(500.0, 0.05, 500.0)
                .translation(Vector::new(0.0, -0.05, 0.0))
                .friction(CARTON_FRICTION),
            None,
        );
        let mut hooks = Surfaces {
            surface: Vec::new(),
        };
        let mut surfaces = Vec::new();
        for (i, belt) in layout.belts.iter().enumerate() {
            let mut own = Vec::new();
            for (Piece { pose, half }, motion) in deck(belt) {
                let index = hooks.surface.len();
                world.insert_collider(surface(half, belt.grip, index).position(pose), None);
                hooks.surface.push(Drive::Linear(Vector::ZERO));
                own.push((index, motion));
            }
            surfaces.push(own);
            if belt.closed_exit {
                world.insert_collider(exit_wall(&layout, Exit::End(i)), None);
            }
            for Piece { pose, half } in guards(belt) {
                // Cargo slides along a guard (OIP: "a guard is friction 0").
                let wall = ColliderBuilder::cuboid(half.x, half.y, half.z).friction(0.0);
                world.insert_collider(wall.position(pose), None);
            }
        }
        let mut transfer_surfaces = Vec::new();
        let mut decks = Vec::new();
        for (k, t) in layout.transfers.iter().enumerate() {
            let host = layout.belts[t.host].pose;
            let deck = world.insert_body(
                RigidBodyBuilder::kinematic_position_based()
                    .pose(host * Pose::from_translation(deck_centre(&layout, k, 0.0))),
            );
            let half = Vector::new(t.along, TRANSFER_DECK, t.across) / 2.0;
            let index = hooks.surface.len();
            world.insert_collider(surface(half, CHAIN_GRIP, index), Some(deck));
            if t.closed_exit {
                world.insert_collider(exit_wall(&layout, Exit::Side(k)), None);
            }
            hooks.surface.push(Drive::Linear(Vector::ZERO));
            transfer_surfaces.push(index);
            decks.push(deck);
        }
        let sources = layout
            .sources
            .iter()
            .map(|s| SourceState {
                next_ns: s.first_ns,
                made: 0,
            })
            .collect();
        let sinks = layout.sinks.len();
        Self {
            world,
            hooks,
            layout,
            cartons: Vec::new(),
            next_id: 1,
            t_ns: 0,
            sources,
            delivered: vec![0; sinks],
            sink_walls: vec![None; sinks],
            lifts: vec![0.0; decks.len()],
            surfaces,
            transfer_surfaces,
            decks,
        }
    }

    pub fn transfers(&self) -> usize {
        self.layout.transfers.len()
    }

    pub fn transfer_id(&self, transfer: usize) -> &str {
        &self.layout.transfers[transfer].id
    }

    pub fn transfer_lift(&self, transfer: usize) -> Real {
        self.lifts[transfer]
    }

    /// A chain transfer's deck at `lift` of its stroke, carrying sideways while `raised` and forward otherwise, at its
    /// speed (OIP `chain-transfer/behavior.ts` `step`). The deck reaches that height at the next step.
    pub fn set_transfer(&mut self, transfer: usize, lift: Real, raised: bool) {
        let t = &self.layout.transfers[transfer];
        let host = self.layout.belts[t.host].pose;
        let local = if raised {
            Vector::new(0.0, 0.0, side(t.out_right) * t.speed)
        } else {
            Vector::new(t.speed, 0.0, 0.0)
        };
        self.hooks.surface[self.transfer_surfaces[transfer]] = Drive::Linear(host.rotation * local);
        self.lifts[transfer] = lift;
        let at = host * Pose::from_translation(deck_centre(&self.layout, transfer, lift));
        self.world.bodies[self.decks[transfer]].set_next_kinematic_translation(at.translation);
    }

    /// A belt's motor: its surface runs at the belt's speed while on.
    pub fn set_belt_running(&mut self, belt: usize, on: bool) {
        let speed = if on {
            self.layout.belts[belt].speed
        } else {
            0.0
        };
        self.set_belt_speed(belt, speed);
    }

    /// A conveyor's surface speed, m/s, along its flow (a curve's at its centreline); negative runs it backwards.
    pub fn set_belt_speed(&mut self, belt: usize, speed: Real) {
        let b = &self.layout.belts[belt];
        for &(index, motion) in &self.surfaces[belt] {
            self.hooks.surface[index] = match motion {
                Motion::Along(rotation) => Drive::Linear(rotation * along(speed, b.skew)),
                Motion::Turn { centre, rate } => Drive::Turn {
                    centre,
                    omega: rate * speed,
                },
            };
        }
    }

    /// A blocked sink holds products at its exit, against a wall (M3's "outlet blocked").
    pub fn set_sink_blocked(&mut self, sink: usize, blocked: bool) {
        match (self.sink_walls[sink], blocked) {
            (None, true) => {
                let wall = exit_wall(&self.layout, self.layout.sinks[sink].exit);
                self.sink_walls[sink] = Some(self.world.insert_collider(wall, None));
            }
            (Some(handle), false) => {
                self.world.remove_collider(handle);
                self.sink_walls[sink] = None;
            }
            _ => {}
        }
    }

    pub fn sinks(&self) -> usize {
        self.layout.sinks.len()
    }

    pub fn sink_id(&self, sink: usize) -> &str {
        &self.layout.sinks[sink].id
    }

    pub fn sink_blocked(&self, sink: usize) -> bool {
        self.sink_walls[sink].is_some()
    }

    pub fn delivered(&self, sink: usize) -> u64 {
        self.delivered[sink]
    }

    /// The cartons the sources have made: delivered plus those on the line, when none is lost.
    pub fn created(&self) -> u64 {
        self.sources.iter().map(|s| s.made as u64).sum()
    }

    /// One millisecond: the physics step, then the sinks take what reached them and the sources add what is due.
    pub fn step(&mut self) {
        self.world.step_with_events(&self.hooks, &());
        self.t_ns += STEP_NS;
        for sink in 0..self.layout.sinks.len() {
            if self.sink_walls[sink].is_none() {
                self.collect(sink);
            }
        }
        for source in 0..self.layout.sources.len() {
            self.feed(source);
        }
    }

    /// The carton's position in a belt's frame.
    fn local(&self, belt: usize, handle: RigidBodyHandle) -> Vector {
        self.layout.belts[belt]
            .pose
            .inverse_transform_point(self.world.bodies[handle].translation())
    }

    /// Removes the cartons whose front reached the sink's exit, as M3 delivered: past a conveyor's end, or past a
    /// chain transfer's side along its deck.
    fn collect(&mut self, sink: usize) {
        let exit = self.layout.sinks[sink].exit;
        let reached: Vec<usize> = (0..self.cartons.len())
            .filter(|&k| self.reached(exit, self.cartons[k].1))
            .collect();
        for k in reached.into_iter().rev() {
            let (_, handle) = self.cartons.remove(k);
            self.world.remove_body(handle);
            self.delivered[sink] += 1;
        }
    }

    fn reached(&self, exit: Exit, carton: RigidBodyHandle) -> bool {
        match exit {
            Exit::End(belt) => {
                let b = &self.layout.belts[belt];
                let (frame, length) = exit_frame(b);
                let p = frame.inverse_transform_point(self.world.bodies[carton].translation());
                p.x + CARTON_HALF >= length
                    && p.z.abs() <= b.width / 2.0
                    && p.y >= b.height - CARTON_HALF
            }
            Exit::Side(transfer) => {
                let t = &self.layout.transfers[transfer];
                let (b, p) = (&self.layout.belts[t.host], self.local(t.host, carton));
                side(t.out_right) * p.z + CARTON_HALF >= t.across / 2.0
                    && (p.x - t.at).abs() <= side_reach(t)
                    && p.y >= b.height - CARTON_HALF
            }
        }
    }

    /// Adds a carton at the source's belt entry when it is due and the entry is free; a carton that waited for a
    /// free entry starts the next interval when it lands, as M3's arrivals did.
    fn feed(&mut self, source: usize) {
        let s = &self.layout.sources[source];
        let state = &self.sources[source];
        if self.t_ns < state.next_ns || (s.max_count > 0 && state.made >= s.max_count) {
            return;
        }
        let b = &self.layout.belts[s.belt];
        let at = b.pose
            * Pose::from_translation(Vector::new(
                CARTON_HALF + 0.001,
                b.height + CARTON_HALF + 0.001,
                0.0,
            ));
        let probe = Cuboid::new(Vector::splat(CARTON_HALF));
        if self
            .world
            .intersect_shape(at, &probe, QueryFilter::only_dynamic())
            .next()
            .is_some()
        {
            return;
        }
        let handle = self
            .world
            // Continuous collision, as OIP's LinearCast: a fast or dropped carton cannot tunnel through a wall.
            .insert_body(
                RigidBodyBuilder::dynamic()
                    .pose(at)
                    .can_sleep(false)
                    .ccd_enabled(true),
            );
        let core = CARTON_HALF - CARTON_ROUNDING;
        self.world.insert_collider(
            ColliderBuilder::round_cuboid(core, core, core, CARTON_ROUNDING)
                .density(CARTON_DENSITY)
                .restitution(0.0)
                .friction(CARTON_FRICTION)
                .friction_combine_rule(CoefficientCombineRule::Multiply),
            Some(handle),
        );
        self.cartons.push((self.next_id, handle));
        self.next_id += 1;
        let state = &mut self.sources[source];
        state.made += 1;
        state.next_ns = self.t_ns + s.interval_ns;
    }

    /// Whether a carton crosses a diffuse sensor's beam: across its belt, at its mount, above the running surface.
    pub fn sensor(&self, sensor: usize) -> bool {
        let s = &self.layout.sensors[sensor];
        let b = &self.layout.belts[s.belt];
        let (frame, at) = match b.arc {
            None => (b.pose, s.at),
            Some(arc) => (b.pose * arc.frame(b.width, s.at), 0.0),
        };
        let origin = frame.transform_point(Vector::new(at, b.height + BEAM_HEIGHT, -b.width / 2.0));
        let dir = frame.rotation * Vector::Z;
        self.world
            .cast_ray(
                &Ray::new(origin, dir),
                s.range,
                true,
                QueryFilter::only_dynamic(),
            )
            .is_some()
    }

    /// The cartons, by id, for the view and the tests.
    pub fn cartons(&self) -> Vec<Carton> {
        self.cartons
            .iter()
            .map(|(id, h)| {
                let body = &self.world.bodies[*h];
                let (t, r) = (body.translation(), body.rotation());
                Carton {
                    id: *id,
                    position: [t.x, t.y, t.z],
                    rotation: [r.x, r.y, r.z, r.w],
                }
            })
            .collect()
    }

    /// The bits of every carton's pose and velocity: equal for equal runs, on every platform (enhanced-determinism).
    pub fn fingerprint(&self) -> u64 {
        self.cartons
            .iter()
            .flat_map(|(id, h)| {
                let b = &self.world.bodies[*h];
                let (t, r, v) = (b.translation(), b.rotation(), b.linvel());
                [t.x, t.y, t.z, r.x, r.y, r.z, r.w, v.x, v.y, v.z]
                    .map(|f| u64::from(f.to_bits()))
                    .into_iter()
                    .chain([*id])
            })
            .fold(0xcbf2_9ce4_8422_2325, |h, x| {
                (h ^ x).wrapping_mul(0x100_0000_01b3)
            })
    }
}

#[cfg(test)]
#[path = "world_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "world_transfer_tests.rs"]
mod transfer_tests;

#[cfg(test)]
#[path = "world_curve_tests.rs"]
mod curve_tests;

#[cfg(test)]
#[path = "world_options_tests.rs"]
mod options_tests;
