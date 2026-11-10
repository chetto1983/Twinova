# Plan: curves, the belt's look, guards and incline, drive and tags (ROADMAP 4.4b)

## Goal

Curved belt and curved roller conveyors run and turn their load. Belts, rollers and curves have guards; belts and
rollers can incline. Their drive ramps, and they can report speed and travel to the PLC. The belt and the curves are
drawn as OIP draws them.

## Architecture

- **Physics (`twinova-physics`):**
  - Coulomb friction world-wide;
  - a curve is chord decks of at most 10° with a rotating surface field (ω × (p − centre), from each contact's
    `anchor1`) and two straight stubs;
  - guards are walls along a conveyor's sides;
  - incline tilts a deck;
  - skew turns a roller deck's surface velocity.
- **Format:**
  - the two curve families;
  - the new parameters of belts and rollers;
  - `optional` on a definition's signal;
  - a curve line fixture.
- **Field and line:**
  - the drive block ramps `velocity : REAL` (m/s), which the line applies to the surface;
  - `speed_ref`, `velocity`, `encoder` and `pulse` are optional signals;
  - an unbound optional signal gets a field-private `%M` address.
- **View:**
  - OIP's curved belt, curved roller and straight belt generators, copied, with `sweep`, `fillet` and `arc` back in
    the kernel;
  - the belt's fabric scrolls with its speed;
  - guards follow their parameters.

## Tech Stack

Rust (Rapier 0.36.1, `enhanced-determinism`, Coulomb friction; truST `6405915`), Tauri 2, Svelte 5, three.js 0.187;
cargo test, vitest, Stryker, cargo mutants; E2E with tauri-driver on Linux and Windows.

## Spec

`docs/superpowers/specs/2026-11-10-curves-belt-options-design.md` (approved 2026-11-10).

## Global Constraints

- **Provenance as in 4.4a:** each copied or translated file names its OIP source at `1e7488c`, and the notices list
  them.
- **The 4.3/4.4a tests pass unchanged after Task 1.** A test that does not is reported to the user before it is
  edited.
- **Gates:**
  - 600 LOC per file;
  - coverage ≥ 85 %;
  - zero surviving mutants;
  - Stryker ≥ 70 % on the view helpers;
  - knip and jscpd clean.
- **The app works at every commit.**
- **Fixtures are written by independent scripts.**

## Review Focus

- The arc velocity's sign for a contact on either collider, and for a right-hand curve.
- Guards on a curve: on the inside and the outside of the arc.
- An optional signal's private address must never collide with a bound one.
- The ramp's exact timing in field cycles.

## Decisions taken while writing this plan

- **Task 1 changes behaviour before anything else.** The demo's photocell moves to 2.75 m (user): at 2.70 it sat on
  the held carton's back face to ±0.04 mm, and Coulomb tipped it. The transfer test measures the carry at steady state
  (user).

## Tasks

### Task 1 — Coulomb friction

- [ ] World-wide Coulomb.
- [ ] Re-run every test. Report the demo jam and the transfer's peak speed, and fix them as the user decided.
- [ ] Re-pin the fingerprints, equal in dev and release.
- [ ] Verify: Rust gates; windows-2022 in CI.

### Task 2 — Format: curves, options, optional signals

- [ ] Tests first:
  - the curve definitions and the new parameters validate;
  - an optional signal may stay unbound; a required one may not.
- [ ] Fixtures: the curve families; the belt and roller parameters; a line `belt → curve → belt` with guards.
- [ ] Verify: crate tests and corpus.

### Task 3 — Physics: curves, guards, incline, skew

- [ ] Tests first, after OIP's fidelity suite:
  - a curve turns its load 90° ± 3°, ≤ 5 mm off the centreline;
  - a guard holds a carton a skewed roller drifts to it, at s·tan(skew);
  - breakaway on an incline within 1° of atan(μ);
  - the curve line's fingerprint.
- [ ] Implement in `layout.rs` and `world.rs`, splitting by concern under 600 LOC.
- [ ] Verify: mutants on the physics.

### Task 4 — Drive and signals

- [ ] Tests first:
  - the ramp reaches rated speed in `accel_time` and stops in `decel_time`;
  - the encoder and the pulse follow travel;
  - `velocity` reads 30 at `speed_at_30`;
  - a bound `speed_ref` sets the speed;
  - an unbound optional signal builds.
- [ ] Implement the drive block in the fixtures (script), the field's private addresses, and `line.rs`.
- [ ] Verify: mutants on `field.rs` and `line.rs`.

### Task 5 — View

- [ ] Copy OIP's straight belt, curved belt and curved roller generators, and the kernel's `sweep`, `fillet` and
  `arc`; draw the guards.
- [ ] Tests: bounds against OIP's figures; the fabric's scroll from the snapshot.
- [ ] Verify: frontend gates, Stryker.

### Task 6 — Acceptance

- [ ] E2E of the curve line on Linux and Windows.
- [ ] The release soak.
- [ ] Verification file; ROADMAP 4.4b; CHANGELOG.
