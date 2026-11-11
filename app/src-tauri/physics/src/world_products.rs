//! The products and the sources that make them (spec 2026-11-11-sensors-products-design.md). A product is a rounded box
//! with its own size, mass, colour and code; a source draws them from its seed, so the same seed makes the same
//! products on every platform.
//!
//! Translated from Open Industry Project at `1e7488c` (MIT; THIRD_PARTY_NOTICES.md): mulberry32 (`src/core/rng.ts`),
//! the box spawner's draws (`examples/parts/box-spawner/behavior.ts`: one draw for colour and code together, then the
//! size) and the cargo's edge rounding (`src/sim/bodyFactory.ts` `makeBoxShape`). OIP's spawn jitter is not
//! translated: a Twinova source runs at a fixed interval.

use rapier3d::prelude::*;

use super::World;
use crate::layout::Products;
use crate::surfaces::CARTON_FRICTION;

/// OIP's random-size range, full extents, in the order it draws them: length, height, width (`SIZE_MIN`, `SIZE_MAX`).
const SIZE_MIN: [f64; 3] = [0.4, 0.3, 0.3];
const SIZE_MAX: [f64; 3] = [0.8, 0.5, 0.5];
/// OIP's cargo edge rounding: at most 2 cm, and 0.4 of the smallest half extent.
const ROUNDING: Real = 0.02;
const ROUNDING_SHARE: Real = 0.4;

/// mulberry32, a 32-bit state stepped by a Weyl sequence and mixed: a float in [0, 1) per call.
pub(super) struct Mulberry32(u32);

impl Mulberry32 {
    pub(super) fn new(seed: u32) -> Self {
        Self(seed)
    }

    pub(super) fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x6d2b_79f5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        f64::from(t ^ (t >> 14)) / 4_294_967_296.0
    }
}

/// A product on the line: its body and what its source gave it.
pub(super) struct Product {
    pub(super) id: u64,
    pub(super) body: RigidBodyHandle,
    pub(super) half: Vector,
    pub(super) color: Option<[u8; 3]>,
    pub(super) code: Option<i32>,
    /// A barcode scanner has read it (OIP's `scanned` mark).
    pub(super) read: bool,
}

/// A product drawn and not yet made: it waits for a free entry with the size it was drawn with.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct Drawn {
    pub(super) half: Vector,
    pub(super) color: Option<[u8; 3]>,
    pub(super) code: Option<i32>,
}

pub(super) struct SourceState {
    pub(super) next_ns: i64,
    pub(super) made: i64,
    rng: Mulberry32,
    /// How far the colour and code lists have been walked.
    walked: usize,
    pending: Option<Drawn>,
}

impl SourceState {
    pub(super) fn new(first_ns: i64, seed: u32) -> Self {
        Self {
            next_ns: first_ns,
            made: 0,
            rng: Mulberry32::new(seed),
            walked: 0,
            pending: None,
        }
    }

    /// The next product, in OIP's draw order: one draw for its colour and code together, in random order, or the next
    /// of each list; then its length, height and width, for a random size.
    pub(super) fn draw(&mut self, p: &Products) -> Drawn {
        let (colors, codes) = (p.colors.len(), p.codes.len());
        let (ci, vi) = if colors == 0 && codes == 0 {
            (0, 0)
        } else if p.random_order {
            let r = self.rng.next();
            ((r * colors as f64) as usize, (r * codes as f64) as usize)
        } else {
            let n = self.walked;
            self.walked += 1;
            (n % colors.max(1), n % codes.max(1))
        };
        let size = if p.random_size {
            let [length, height, width] =
                [0, 1, 2].map(|k| SIZE_MIN[k] + self.rng.next() * (SIZE_MAX[k] - SIZE_MIN[k]));
            Vector::new(length as Real, width as Real, height as Real)
        } else {
            p.size
        };
        Drawn {
            // Along the flow, up, across: the belt frame's x, y and z.
            half: Vector::new(size.x, size.z, size.y) / 2.0,
            color: p.colors.get(ci).copied(),
            code: p.codes.get(vi).copied(),
        }
    }
}

impl World {
    /// Adds a product at the source's belt entry when it is due and the entry is free; one that waited for a free
    /// entry starts the next interval when it lands, as M3's arrivals did.
    pub(super) fn feed(&mut self, source: usize) {
        let s = &self.layout.sources[source];
        let state = &mut self.sources[source];
        if self.t_ns < state.next_ns || (s.max_count > 0 && state.made >= s.max_count) {
            return;
        }
        let drawn = state
            .pending
            .take()
            .unwrap_or_else(|| state.draw(&s.products));
        let b = &self.layout.belts[s.belt];
        let half = drawn.half;
        let at = b.pose
            * Pose::from_translation(Vector::new(half.x + 0.001, b.height + half.y + 0.001, 0.0));
        if self
            .world
            .intersect_shape(at, &Cuboid::new(half), QueryFilter::only_dynamic())
            .next()
            .is_some()
        {
            self.sources[source].pending = Some(drawn);
            return;
        }
        let body = self.world.insert_body(
            // Continuous collision, as OIP's LinearCast: a fast or dropped product cannot tunnel through a wall.
            RigidBodyBuilder::dynamic()
                .pose(at)
                .can_sleep(false)
                .ccd_enabled(true),
        );
        let rounding = ROUNDING.min(ROUNDING_SHARE * half.min_element());
        let core = half - Vector::splat(rounding);
        self.world.insert_collider(
            ColliderBuilder::round_cuboid(core.x, core.y, core.z, rounding)
                .mass(s.products.mass)
                .restitution(0.0)
                .friction(CARTON_FRICTION)
                .friction_combine_rule(CoefficientCombineRule::Multiply),
            Some(body),
        );
        self.products.push(Product {
            id: self.next_id,
            body,
            half,
            color: drawn.color,
            code: drawn.code,
            read: false,
        });
        self.next_id += 1;
        let state = &mut self.sources[source];
        state.made += 1;
        state.next_ns = self.t_ns + s.interval_ns;
    }

    /// How far a product reaches from its centre along `axis`, a unit vector in the world: its box's support.
    pub(super) fn reach(&self, product: &Product, axis: Vector) -> Real {
        let rotation = self.world.bodies[product.body].rotation();
        [Vector::X, Vector::Y, Vector::Z]
            .into_iter()
            .zip(product.half.to_array())
            .map(|(local, half)| (rotation * local).dot(axis).abs() * half)
            .sum()
    }
}

#[cfg(test)]
#[path = "world_products_tests.rs"]
mod tests;
