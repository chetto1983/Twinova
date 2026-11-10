//! The line's physics (spec 2026-11-09-component-behaviour-design.md, "Transport with physics"; research 15 and 16):
//! products are rigid bodies in Rapier, and a belt is a fixed box whose contacts get a tangent velocity, Rapier's
//! documented mechanism for conveyor belts (`SolverContact::tangent_velocity`). Nothing hands a product from one belt
//! to the next: contacts and friction do, as in Open Industry Project and NX MCD. One step is 1 ms of virtual time.
//!
//! Translated from Open Industry Project at `1e7488c` (MIT, Copyright (c) 2024-present Automation Standard LLC and
//! Contributors; THIRD_PARTY_NOTICES.md): a surface's linear velocity per contact (`src/sim/conveyorContacts.ts`,
//! `src/sim/deckDrive.ts` `setSurfaceVelocity`), and a moving link as a kinematic body moved before each step
//! (`src/sim/rigDriver.ts`, `MoveKinematic`), here for the chain transfer's lift. OIP's ghost-contact gates and its
//! friction coupling are not translated: they patch Jolt, and Rapier passes these tests without them (research 15).

use rapier3d::prelude::*;
use serde::Serialize;

use crate::layout::{Exit, Layout};

pub const STEP_NS: i64 = 1_000_000;
const STEP_S: Real = 0.001;
/// A standard carton's friction. Rapier has no geometric mean, so pairs multiply, and a surface's collider friction
/// is its carton grip divided by this (OIP: surfaceFrictionFromCarton).
const CARTON_FRICTION: Real = 0.5;
/// The products of 4.3: M3's 0.3 m carton.
pub const CARTON_HALF: Real = 0.15;
const CARTON_DENSITY: Real = 100.0;
/// One solid deck per belt, as deep as OIP's frame (CONVEYOR_BODY_THICKNESS): a thin slab over a housing leaves a
/// notch cargo catches on at junctions.
const BELT_THICKNESS: Real = 0.17;
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

/// Surface velocities for the contact hook, belts first and then chain transfers; a surface's collider carries its
/// index + 1 in its user data.
///
/// No seam gate. OIP filters shallow sideways contacts at junctions (conveyorContacts.ts), and the Rapier probe with
/// sharp boxes and a thin deck needed it (research 15). With OIP's own carton and deck (rounded edges, one deep solid
/// deck), the tests pass the same with and without it, so it was dropped ("baseline or drop", 2026-11-09). Measured
/// on straight belts butted end to end at 1 ms; curves and coarser steps may need it again.
struct Surfaces {
    surface: Vec<Vector>,
}

/// A surface's deck: its grip against a carton, and its index for the contact hook.
fn surface(half: Vector, grip: Real, index: usize) -> ColliderBuilder {
    ColliderBuilder::cuboid(half.x, half.y, half.z)
        .friction(grip / CARTON_FRICTION)
        .friction_combine_rule(CoefficientCombineRule::Multiply)
        .user_data(index as u128 + 1)
        .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS)
}

/// The surface of a contact pair, and whether it is collider 2: the tangent velocity is that of collider 2 relative to
/// collider 1, so a belt as collider 2 gives the opposite sign (measured with the probe, research 15).
fn surface_of_pair(user1: u128, user2: u128) -> Option<(usize, bool)> {
    match (user1, user2) {
        (0, 0) => None,
        (belt, 0) => Some((belt as usize - 1, false)),
        (_, belt) => Some((belt as usize - 1, true)),
    }
}

impl PhysicsHooks for Surfaces {
    fn modify_solver_contacts(&self, ctx: &mut ContactModificationContext) {
        let pair = surface_of_pair(
            ctx.colliders[ctx.collider1].user_data,
            ctx.colliders[ctx.collider2].user_data,
        );
        if let (Some((belt, second)), Some(m)) = (pair, ctx.rigid_mut()) {
            let v = if second {
                -self.surface[belt]
            } else {
                self.surface[belt]
            };
            for c in m.solver_contacts.iter_mut() {
                c.tangent_velocity = v;
            }
        }
    }
}

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
    /// Each chain transfer's deck, a kinematic body, and its lift as last set.
    decks: Vec<RigidBodyHandle>,
    lifts: Vec<Real>,
}

/// The wall across an exit: past a conveyor's end, or along a chain transfer's side over its deck.
fn exit_wall(layout: &Layout, exit: Exit) -> ColliderBuilder {
    let (pose, at, half) = match exit {
        Exit::End(k) => {
            let b = &layout.belts[k];
            let at = Vector::new(b.length + 0.01, b.height + WALL_HEIGHT / 2.0, 0.0);
            (
                b.pose,
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

/// How far along its host, from the deck's centre, a chain transfer's side exit reaches: the deck, and a carton that
/// only overlaps it, as a chute beside the transfer catches what the chains drag off (measured 2026-11-10: with the
/// deck's own span, a queue standing over a raised transfer spilled the cartons it half carried onto the floor).
fn side_reach(t: &crate::layout::Transfer) -> Real {
    t.along / 2.0 + CARTON_HALF
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
        for (i, belt) in layout.belts.iter().enumerate() {
            let centre = Vector::new(belt.length / 2.0, belt.height - BELT_THICKNESS / 2.0, 0.0);
            world.insert_collider(
                surface(
                    Vector::new(belt.length, BELT_THICKNESS, belt.width) / 2.0,
                    belt.grip,
                    i,
                )
                .position(belt.pose * Pose::from_translation(centre)),
                None,
            );
            if belt.closed_exit {
                world.insert_collider(exit_wall(&layout, Exit::End(i)), None);
            }
            hooks.surface.push(Vector::ZERO);
        }
        let mut decks = Vec::new();
        for (k, t) in layout.transfers.iter().enumerate() {
            let host = layout.belts[t.host].pose;
            let deck = world.insert_body(
                RigidBodyBuilder::kinematic_position_based()
                    .pose(host * Pose::from_translation(deck_centre(&layout, k, 0.0))),
            );
            let half = Vector::new(t.along, TRANSFER_DECK, t.across) / 2.0;
            world.insert_collider(
                surface(half, CHAIN_GRIP, layout.belts.len() + k),
                Some(deck),
            );
            if t.closed_exit {
                world.insert_collider(exit_wall(&layout, Exit::Side(k)), None);
            }
            hooks.surface.push(Vector::ZERO);
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
        self.hooks.surface[self.layout.belts.len() + transfer] = host.rotation * local;
        self.lifts[transfer] = lift;
        let at = host * Pose::from_translation(deck_centre(&self.layout, transfer, lift));
        self.world.bodies[self.decks[transfer]].set_next_kinematic_translation(at.translation);
    }

    /// A belt's motor: its surface runs at the belt's speed while on.
    pub fn set_belt_running(&mut self, belt: usize, on: bool) {
        let b = &self.layout.belts[belt];
        let speed = if on { b.speed } else { 0.0 };
        self.hooks.surface[belt] = b.pose.rotation * Vector::new(speed, 0.0, 0.0);
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
                let (b, p) = (&self.layout.belts[belt], self.local(belt, carton));
                p.x + CARTON_HALF >= b.length
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
        let origin =
            b.pose
                .transform_point(Vector::new(s.at, b.height + BEAM_HEIGHT, -b.width / 2.0));
        let dir = b.pose.rotation * Vector::Z;
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
