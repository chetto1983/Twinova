//! The line's physics (spec 2026-11-09-component-behaviour-design.md, "Transport with physics"; research 15 and 16):
//! products are rigid bodies in Rapier, and a belt is a fixed box whose contacts get a tangent velocity, Rapier's
//! documented mechanism for conveyor belts (`SolverContact::tangent_velocity`). Nothing hands a product from one belt
//! to the next: contacts and friction do, as in Open Industry Project and NX MCD. One step is 1 ms of virtual time.

use rapier3d::prelude::*;
use serde::Serialize;

use crate::layout::{Belt, Layout};

pub const STEP_NS: i64 = 1_000_000;
const STEP_S: Real = 0.001;
/// A belt's grip against a standard carton, the figure a belt datasheet quotes (OIP: BELT_FRICTION).
const BELT_FRICTION: Real = 0.7;
/// A standard carton's friction. Rapier has no geometric mean, so pairs multiply, and a belt's collider friction is
/// its carton grip divided by this (OIP: surfaceFrictionFromCarton).
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

/// Belt surface velocities for the contact hook; a belt collider carries its index + 1 in its user data.
///
/// No seam gate. OIP filters shallow sideways contacts at junctions (conveyorContacts.ts), and the Rapier probe with
/// sharp boxes and a thin deck needed it (research 15). With OIP's own carton and deck (rounded edges, one deep solid
/// deck), the tests pass the same with and without it, so it was dropped ("baseline or drop", 2026-11-09). Measured
/// on straight belts butted end to end at 1 ms; curves and coarser steps may need it again.
struct Belts {
    surface: Vec<Vector>,
}

/// The belt of a contact pair, and whether it is collider 2: the tangent velocity is that of collider 2 relative to
/// collider 1, so a belt as collider 2 gives the opposite sign (measured with the probe, research 15).
fn belt_of_pair(user1: u128, user2: u128) -> Option<(usize, bool)> {
    match (user1, user2) {
        (0, 0) => None,
        (belt, 0) => Some((belt as usize - 1, false)),
        (_, belt) => Some((belt as usize - 1, true)),
    }
}

impl PhysicsHooks for Belts {
    fn modify_solver_contacts(&self, ctx: &mut ContactModificationContext) {
        let pair = belt_of_pair(
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
    hooks: Belts,
    layout: Layout,
    cartons: Vec<(u64, RigidBodyHandle)>,
    next_id: u64,
    t_ns: i64,
    sources: Vec<SourceState>,
    delivered: Vec<u64>,
    /// The wall each blocked sink puts at its belt's exit.
    sink_walls: Vec<Option<ColliderHandle>>,
}

fn wall(belt: &Belt) -> ColliderBuilder {
    let at = Vector::new(belt.length + 0.01, belt.height + WALL_HEIGHT / 2.0, 0.0);
    ColliderBuilder::cuboid(0.01, WALL_HEIGHT / 2.0, belt.width / 2.0)
        .position(belt.pose * Pose::from_translation(at))
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
        let mut hooks = Belts {
            surface: Vec::new(),
        };
        for (i, belt) in layout.belts.iter().enumerate() {
            let centre = Vector::new(belt.length / 2.0, belt.height - BELT_THICKNESS / 2.0, 0.0);
            world.insert_collider(
                ColliderBuilder::cuboid(belt.length / 2.0, BELT_THICKNESS / 2.0, belt.width / 2.0)
                    .position(belt.pose * Pose::from_translation(centre))
                    .friction(BELT_FRICTION / CARTON_FRICTION)
                    .friction_combine_rule(CoefficientCombineRule::Multiply)
                    .user_data(i as u128 + 1)
                    .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS),
                None,
            );
            if belt.closed_exit {
                world.insert_collider(wall(belt), None);
            }
            hooks.surface.push(Vector::ZERO);
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
        }
    }

    /// A belt's motor: its surface runs at the belt's speed while on.
    pub fn set_belt_running(&mut self, belt: usize, on: bool) {
        let b = &self.layout.belts[belt];
        let speed = if on { b.speed } else { 0.0 };
        self.hooks.surface[belt] = b.pose.rotation * Vector::new(speed, 0.0, 0.0);
    }

    /// A blocked sink holds products at its belt's exit, against a wall (M3's "outlet blocked").
    pub fn set_sink_blocked(&mut self, sink: usize, blocked: bool) {
        match (self.sink_walls[sink], blocked) {
            (None, true) => {
                let belt = &self.layout.belts[self.layout.sinks[sink].belt];
                self.sink_walls[sink] = Some(self.world.insert_collider(wall(belt), None));
            }
            (Some(handle), false) => {
                self.world.remove_collider(handle);
                self.sink_walls[sink] = None;
            }
            _ => {}
        }
    }

    pub fn sink_blocked(&self, sink: usize) -> bool {
        self.sink_walls[sink].is_some()
    }

    pub fn delivered(&self, sink: usize) -> u64 {
        self.delivered[sink]
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

    /// Removes the cartons whose front reached the exit of the sink's belt, as M3 delivered.
    fn collect(&mut self, sink: usize) {
        let belt = self.layout.sinks[sink].belt;
        let b = &self.layout.belts[belt];
        let reached: Vec<usize> = (0..self.cartons.len())
            .filter(|&k| {
                let p = self.local(belt, self.cartons[k].1);
                p.x + CARTON_HALF >= b.length
                    && p.z.abs() <= b.width / 2.0
                    && p.y >= b.height - CARTON_HALF
            })
            .collect();
        for k in reached.into_iter().rev() {
            let (_, handle) = self.cartons.remove(k);
            self.world.remove_body(handle);
            self.delivered[sink] += 1;
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
