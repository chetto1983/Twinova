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

#[path = "world_devices.rs"]
mod devices;
#[cfg(test)]
use devices::TRANSFER_BELOW;
pub use devices::TRANSFER_STROKE;
use devices::{Devices, side_exit, side_wall};

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
    /// Whether each sink is blocked, and the wall it then puts at its exit.
    blocked: Vec<bool>,
    sink_walls: Vec<Option<ColliderHandle>>,
    /// Each conveyor's surfaces, by index into the hook's, and how each moves its load.
    surfaces: Vec<Vec<(usize, Motion)>>,
    /// Each conveyor's surface speed as last set, and its travel since the world began (f64, so a long run does not
    /// drift), for the view's scrolling fabric.
    speeds: Vec<Real>,
    travels: Vec<f64>,
    /// The devices that move: chain transfers, stoppers and pushers.
    devices: Devices,
}

/// The wall across an exit: past a conveyor's end, or along a chain transfer's side. A pusher's side has none: its
/// paddle's stroke crosses that edge, and a wall there would crush what it pushes (measured 2026-11-11), so what it
/// pushes to nowhere falls beside the conveyor.
fn exit_wall(layout: &Layout, exit: Exit) -> Option<ColliderBuilder> {
    let k = match exit {
        Exit::End(k) => k,
        Exit::Side(_) => return side_exit(layout, exit).map(|s| side_wall(layout, s)),
        Exit::Push(_) => return None,
    };
    let b = &layout.belts[k];
    let (frame, length) = exit_frame(b);
    let at = Vector::new(length + 0.01, b.height + WALL_HEIGHT / 2.0, 0.0);
    let half = Vector::new(0.01, WALL_HEIGHT / 2.0, b.width / 2.0);
    Some(
        ColliderBuilder::cuboid(half.x, half.y, half.z)
            .position(frame * Pose::from_translation(at)),
    )
}

/// A frame along which a conveyor's exit lies, and how far: a straight conveyor's own, at its length; a curve's exit
/// stub tip.
fn exit_frame(b: &Belt) -> (Pose, Real) {
    match b.arc {
        None => (b.pose, b.length),
        Some(arc) => (b.pose * arc.end(b.width), 0.0),
    }
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

/// The sign of a device's side along its host's z.
fn side(right: bool) -> Real {
    if right { 1.0 } else { -1.0 }
}

impl World {
    pub fn new(layout: Layout) -> Self {
        let mut world = PhysicsWorld::default();
        world.integration_parameters.dt = STEP_S;
        // Coulomb friction: one constraint per contact point, each with its own tangent velocity, so a curve turns its
        // load with it. Rapier's default (Simplified) adds a twist constraint with no target velocity that holds the
        // load's yaw (spec 2026-11-10-curves-belt-options-design.md, measured).
        world.integration_parameters.friction_model = FrictionModel::Coulomb;
        // One solver iteration per step: Rapier's default of 4 is for steps of 16 ms, and at a millisecond it is 70 %
        // of the step for no measured gain in fidelity (spec 2026-11-10-solver-substeps-io-links-design.md).
        world.integration_parameters.num_solver_iterations = 1;
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
                world.insert_collider(exit_wall(&layout, Exit::End(i)).expect("an end wall"), None);
            }
            for Piece { pose, half } in guards(belt) {
                // Cargo slides along a guard (OIP: "a guard is friction 0").
                let wall = ColliderBuilder::cuboid(half.x, half.y, half.z).friction(0.0);
                world.insert_collider(wall.position(pose), None);
            }
        }
        let devices = Devices::new(&mut world, &mut hooks, &layout);
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
            blocked: vec![false; sinks],
            sink_walls: vec![None; sinks],
            speeds: vec![0.0; surfaces.len()],
            travels: vec![0.0; surfaces.len()],
            surfaces,
            devices,
        }
    }

    pub fn belts(&self) -> usize {
        self.layout.belts.len()
    }

    pub fn belt_id(&self, belt: usize) -> &str {
        &self.layout.belts[belt].id
    }

    /// A conveyor's surface speed as last set, m/s: the speed it ran at during the last step.
    pub fn belt_speed(&self, belt: usize) -> Real {
        self.speeds[belt]
    }

    /// How far a conveyor's surface has run along its flow, m: back for a negative speed.
    pub fn belt_travel(&self, belt: usize) -> f64 {
        self.travels[belt]
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
        self.speeds[belt] = speed;
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

    /// A blocked sink takes nothing and holds products at its exit, against a wall (M3's "outlet blocked").
    pub fn set_sink_blocked(&mut self, sink: usize, blocked: bool) {
        if self.blocked[sink] == blocked {
            return;
        }
        self.blocked[sink] = blocked;
        if let Some(handle) = self.sink_walls[sink].take() {
            self.world.remove_collider(handle);
        }
        if blocked && let Some(wall) = exit_wall(&self.layout, self.layout.sinks[sink].exit) {
            self.sink_walls[sink] = Some(self.world.insert_collider(wall, None));
        }
    }

    pub fn sinks(&self) -> usize {
        self.layout.sinks.len()
    }

    pub fn sink_id(&self, sink: usize) -> &str {
        &self.layout.sinks[sink].id
    }

    pub fn sink_blocked(&self, sink: usize) -> bool {
        self.blocked[sink]
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
        for (travel, speed) in self.travels.iter_mut().zip(&self.speeds) {
            *travel += f64::from(*speed) * (STEP_NS as f64 * 1e-9);
        }
        for sink in 0..self.layout.sinks.len() {
            if !self.blocked[sink] {
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
            Exit::Side(_) | Exit::Push(_) => {
                let s = side_exit(&self.layout, exit).expect("a side exit");
                let (b, p) = (&self.layout.belts[s.host], self.local(s.host, carton));
                side(s.right) * p.z + CARTON_HALF >= s.edge
                    && (p.x - s.at).abs() <= s.reach
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

#[cfg(test)]
#[path = "world_stopper_tests.rs"]
mod stopper_tests;

#[cfg(test)]
#[path = "world_pusher_tests.rs"]
mod pusher_tests;
