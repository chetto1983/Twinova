# Spec: curves, the belt's look, guards and incline, drive and tags (ROADMAP 4.4b)

Date: 2026-11-10. Status: **approved** (user, 2026-11-10): `Coulomb` friction world-wide, OIP's velocity scale (30 at `speed_at_30`).

## Scope (user, 2026-11-10)

4.4b holds all four items 4.4a left to it:
1. curved belt and curved roller conveyors;
2. the belt's look;
3. side guards and incline;
4. drive ramps, roller skew, and the speed and encoder signals.

The method is 4.4a's:
- OIP at `1e7488c` (MIT) is copied in TypeScript for the 3D view;
- it is translated into Rust for the physics;
- device logic is re-expressed in ST;
- OIP's workarounds for Jolt are kept only where a Rapier measurement shows the need.

## Measured: a 90° curve in Rapier (2026-11-10, throwaway worktree, release)

The curve is built as OIP builds it (`src/core/geometry/collisionShapes.ts` `sdkCurvedDeckSubs`):
- a deck of chord boxes of at most 10° (`MAX_SEGMENT_DEG`), 0.15 m deep (`CONVEYOR_BODY_THICKNESS`), its top at the
  surface;
- one rotating surface field over all chords: the velocity at a contact point p is ω × (p − centre), with
  ω = speed / centre radius (`conveyorContacts.ts`, arc entry). Rapier's hook gives each contact's world point
  (`SolverContact::anchor1`, "inside modify_solver_contacts this always holds the fresh world-space point");
- a straight stub of 0.085 m at each end (`ARC_END_EXT`), with linear velocity.

**Setup:**
- belt A, 2 m, then the curve's entry stub;
- a 90° left curve, inner radius 0.6, width 0.6;
- the exit stub, then belt B;
- the 0.3 m carton, at 1 ms steps.

| Friction model | Carton turned through the curve (90° expected) | Off the centreline | Tilt | Lift | Cost per step |
|---|---|---|---|---|---|
| `Simplified` (Rapier's default) | 5–15° | up to 28 mm | ≤ 0.40° | 1.0 mm (spawn) | 13.6–16.2 µs |
| **`Coulomb`** | **87–89°** | **≤ 1 mm** | **0.06°** | 1.0 mm | 16.4–18.0 µs |

The cases measured:
- belt grip at 0.5 and at 1.0 m/s;
- roller grip (0.42);
- a 0.3 m inner radius;
- a 0.17 m deck (same results).

**Why `Simplified` fails.** It solves one friction constraint per contact group plus a "twist" constraint that
"aims to eliminate angular movement in the manifold's tangent plane" (`integration_parameters.rs`). That twist
constraint has no target velocity (`contact_with_twist_friction.rs`: "TODO: add something similar to tangent
velocity to the twist"), so it holds the carton's yaw against the static deck. OIP gets the turn from an angular
surface velocity and its Jolt friction-coupling patch. In Rapier, `Coulomb` (one constraint per contact point, each
with its own tangent velocity) gives the same rigid rotation.

**No seam gate is needed:** the chords' joints lift the carton by nothing measurable.

What this does not prove:
- other carton sizes;
- a curve fed by a roller conveyor;
- `Coulomb`'s effect on the 4.4a tests and fingerprints. That is Task 1's first measurement.

## Decisions proposed

1. **The world's friction model becomes `Coulomb`.** It is the measured condition for a curve to turn its load, at
   about +20 % per physics step.
   - Every existing fingerprint is pinned again: a changed number, by design.
   - Every 4.3/4.4a physics, line and E2E test must pass unchanged; any that does not is reported, not edited.
   - The release soak is measured again.
2. **Curves (`twinova.belt_curve@1`, `twinova.roller_curve@1`):**
   - Parameters (OIP's):
     - `inner_radius`, m: 0.6 by default (OIP 0.825), 0.2–4;
     - `width`, m: 0.6;
     - `sweep`, degrees: 90, 15–180;
     - `height`, m: 0.8;
     - `speed`, m/s: 0.5, defined at the centreline as in OIP;
     - `friction`: 0.7 for the belt, 0.42 for rollers;
     - `frame_height`, m: 0.17;
     - `turn_right` (BOOL): Twinova's, since OIP turns a curve by rotating or reversing it.
   - Ports `in`, `out`, `rail`, and the signals and behaviour of a straight conveyor of the same surface.
   - A sensor may mount on a curve, at an angle along the arc (degrees from the entry).
3. **The belt's look:** OIP's `belt-conveyor` `geometry()`, straight path only. Multi-segment paths and guard openings
   stay out: no part of Twinova has them. OIP's scrolling fabric becomes a texture whose offset follows the belt's
   surface speed from the snapshot.
4. **Guards and incline:**
   - **Guards.** Belts, rollers and curves have `left_guard` and `right_guard` (BOOL, default TRUE), with OIP's
     guards: 0.15 m above the surface, 0.005 m thick, at the frame's outer face (`GUARD_HEIGHT`, `GUARD_THICKNESS`).
     They are physical walls and are drawn.
   - **Incline.** Belts and rollers have `incline` in degrees (OIP `tiltDeg`, ±30): the deck tilted about its
     centre, its surface velocity along the tilted tangent. A chain transfer fits only in a level, unskewed roller
     conveyor, which Build checks.
5. **Drive and signals,** OIP's `_shared/conveyorDrive.js`, re-expressed in ST:
   - **The ramp.** The behaviour block of a belt, roller or curve ramps the surface speed by `accel_time` and
     `decel_time` (seconds from stopped to rated speed, OIP's VFD ramp; 0 is instant). Its native variable becomes
     `velocity : REAL` (m/s) instead of `motor : BOOL`.
   - **Optional signals:**
     - `speed_ref` (PLC output, REAL m/s): when bound, it replaces `speed`;
     - `velocity` (PLC input, REAL): OIP's VFD scale, 30 at `speed_at_30`;
     - `encoder` (PLC input, REAL m): the surface travel;
     - `pulse` (PLC input, BOOL): one field cycle each `pulse_pitch` metres.
   - **A signal may be optional** (format 1 amended: `optional: true` on the definition's signal). An optional signal
     left unbound is given a field-private `%M` address at Build, so the block still compiles. Its value is then the
     block's default: an unbound `speed_ref` reads 0 and means "use `speed`".
   - **Roller skew** (OIP `skew`, ±60°): the surface velocity turned by the skew with magnitude s/cos(skew), as in
     OIP, so throughput stays s while the product drifts sideways at s·tan(skew) towards a guard.
6. **3D:**
   - OIP's curved belt and curved roller generators are copied, with the kernel operations they need (`sweep`,
     `fillet`, `arc` come back), and `_shared/guardArc.js`;
   - the belt and roller guards follow their parameters.

## Amended while writing the format (Task 2, 2026-11-10)

- **A curve's frame.** A curve's origin is its entry, on the centreline, at the surface's height like a belt's tail.
  It runs along local +x for the entry stub (0.085 m), turns about a centre at inner_radius + width / 2 to its left
  (−z) or, with `turn_right`, to its right (+z), and leaves along the exit stub. A default 90° left curve placed at the
  origin hands over at (0.985, 0, −0.985).
- **The ramps default to 0 s** (instant), not OIP's 0.5 s, so the 4.3/4.4a lines keep their timing. The other drive
  defaults are OIP's: `speed_at_30` 0.5 m/s (0.01–10), `pulse_pitch` one inch, 0.0254 m (0.00127–2.54).
- **A required signal must be bound.** The format refuses an instance that leaves a non-optional signal unbound
  ("… needs it bound to an address"). Until now only truST's compile caught it, at Build.
- **The chain transfer's exit side has no guard.** Guard openings are out of scope, so the line fixture's roller has
  `left_guard` FALSE, the side its transfer carries out on; Build will refuse a guard across a transfer's exit
  (Task 3).
- **Until Task 4, the curves' blocks are the straight conveyor's** (`motor : BOOL`), and the optional drive signals
  stay unbound in the fixtures.

## Acceptance of 4.4b

1. **Curves:**
   - a carton turns with a 90° curve (belt and rollers) and leaves square: yaw within 3° of the turn, off the
     centreline ≤ 5 mm, tilt < 1°;
   - a line straight → curve → straight runs in the app (E2E, Linux and Windows).
2. **Guards and incline:**
   - a skewed roller drifts a carton against its guard and the guard holds it (lateral speed s·tan(skew) until
     contact);
   - a carton climbs a 10° incline at the belt's speed, and does not slip back at OIP's belt grip;
   - the product stays on a curve with guards at 1 m/s.
3. **Drive:**
   - the surface reaches rated speed in `accel_time` and stops in `decel_time`, linearly;
   - the encoder reads the travel within one millisecond's travel;
   - `velocity` reads 30 at `speed_at_30`;
   - a bound `speed_ref` sets the speed.
4. **Fidelity, after OIP's `physicsFidelity.test.ts`:**
   - breakaway on an incline within 1° of atan(μ);
   - a skewed bed's throughput s and lateral speed s·tan(skew).
5. **Determinism and gates:**
   - fingerprints are equal in dev, release and on windows-2022;
   - coverage ≥ 85 %;
   - no surviving mutant;
   - Stryker ≥ 70 % on the view helpers;
   - a verification file and a release soak.

## What this will not prove

- **Spurs, merges, multi-segment belts and guard openings,** which OIP has and Twinova has no family for.
- **Pallets and totes,** or products other than the carton (4.4d).
- **OIP's imperial units** (FPM): Twinova's signals are in m/s and m.
