//! The contact hook that drives the decks: each deck's collider carries its surface's index + 1 in its user data, and
//! every contact on it gets the surface's velocity as its tangent velocity, Rapier's documented mechanism for conveyor
//! belts (`SolverContact::tangent_velocity`).
//!
//! Translated from Open Industry Project at `1e7488c` (MIT; THIRD_PARTY_NOTICES.md), `src/sim/conveyorContacts.ts`: a
//! straight surface's linear velocity, and a curve's rotating field, whose velocity at a contact point p is
//! ω × (p − centre). Rapier gives the point: inside `modify_solver_contacts`, `SolverContact::anchor1` "always holds
//! the fresh world-space point" (rapier3d 0.36.1, `geometry/contact_pair.rs`).
//!
//! No seam gate. OIP filters shallow sideways contacts at junctions (conveyorContacts.ts), and the Rapier probe with
//! sharp boxes and a thin deck needed it (research 15). With OIP's own carton and deck (rounded edges, one deep solid
//! deck), the tests pass the same with and without it, so it was dropped ("baseline or drop", 2026-11-09). Measured
//! on straight belts butted end to end at 1 ms, and on a 90° curve's chords (spec
//! 2026-11-10-curves-belt-options-design.md: their joints lift a carton by nothing measurable).

use rapier3d::prelude::*;

/// A standard carton's friction. Rapier has no geometric mean, so pairs multiply, and a surface's collider friction
/// is its carton grip divided by this (OIP: surfaceFrictionFromCarton).
pub(crate) const CARTON_FRICTION: Real = 0.5;

/// How a surface moves what it carries.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Drive {
    Linear(Vector),
    /// About `centre`, at `omega` (rad/s, about the world axis it points along).
    Turn {
        centre: Vector,
        omega: Vector,
    },
}

/// The surfaces' drives, by index: conveyors' decks first, then chain transfers.
pub(crate) struct Surfaces {
    pub(crate) surface: Vec<Drive>,
}

/// A surface's deck: its grip against a carton, and its index for the contact hook.
pub(crate) fn surface(half: Vector, grip: Real, index: usize) -> ColliderBuilder {
    ColliderBuilder::cuboid(half.x, half.y, half.z)
        .friction(grip / CARTON_FRICTION)
        .friction_combine_rule(CoefficientCombineRule::Multiply)
        .user_data(index as u128 + 1)
        .active_hooks(ActiveHooks::MODIFY_SOLVER_CONTACTS)
}

/// The surface of a contact pair, and whether it is collider 2: the tangent velocity is that of collider 2 relative to
/// collider 1, so a belt as collider 2 gives the opposite sign (measured with the probe, research 15).
pub(crate) fn surface_of_pair(user1: u128, user2: u128) -> Option<(usize, bool)> {
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
        let (Some((belt, second)), Some(m)) = (pair, ctx.rigid_mut()) else {
            return;
        };
        let sign = |v: Vector| if second { -v } else { v };
        match self.surface[belt] {
            Drive::Linear(v) => {
                let v = sign(v);
                for c in m.solver_contacts.iter_mut() {
                    c.tangent_velocity = v;
                }
            }
            Drive::Turn { centre, omega } => {
                for c in m.solver_contacts.iter_mut() {
                    c.tangent_velocity = sign(omega.cross(c.anchor1 - centre));
                }
            }
        }
    }
}
