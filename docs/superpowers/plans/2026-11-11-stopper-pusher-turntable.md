# Plan: stopper, pusher and turntable (ROADMAP 4.4c)

## Goal

Three new families, each copied and translated from Open Industry Project at `1e7488c` (MIT):
- `twinova.stopper@1` (OIP's blade stop);
- `twinova.pusher@1` (OIP's diverter);
- `twinova.turntable@1` (OIP's turntable conveyor).

They are proved on a sorting line run in the real app.

## Architecture

- **Physics (Rust, `twinova-physics`):**
  - The stopper's blade and the pusher's paddle are kinematic position-based bodies, posed before each step, as the
    chain transfer's deck.
  - The turntable's lane is a conveyor of the layout: OIP's static cross is its deck. Its surface is either the
    curves' rotating field at the deck's rate, or a linear velocity along the deck's angle.
  - The pusher opens its own side's guard.
- **Logic (ST):** three behaviour blocks: `StopperBehaviour`, `PusherBehaviour` and `TurntableBehaviour`. The
  turntable's block holds OIP's slew and the conveyor drive.
- **App:**
  - The field reads each block's native outputs. The line poses the bodies and the lane.
  - The snapshot carries each moving device's position for the view.
- **View:** OIP's three generators, copied. A part tagged as moving hangs in a child group posed from the snapshot.

## Tech Stack

Rust 2024, Rapier 0.36.1 (`enhanced-determinism`), truST `6405915`, Tauri 2, Svelte 5, three.js 0.187, vitest,
Stryker, cargo-mutants, WebdriverIO with tauri-driver.

## Spec

`docs/superpowers/specs/2026-11-11-stopper-pusher-turntable-design.md` (approved 2026-11-11, decisions 1 and 2 as
recommended).

## Global Constraints

- **Files:**
  - no file over 600 lines;
  - `world.rs` (511) and `layout.rs` (361) are split by concern before they grow;
  - every copied file carries its OIP path and commit;
  - `THIRD_PARTY_NOTICES.md` (both) lists the new copies and textures.
- **Determinism:**
  - the existing fingerprints stay pinned: two belts `0x2674dda1319b763a`, transfer line `0x2c224abfa5e90d6c`, curve
    line `0xaba061e54b1dfbe2`;
  - a new fingerprint is pinned for `sort_line`.
- **Fixtures:** written by an independent script in the scratchpad (`migrate_fixtures_4_4c.py`), never by the Rust
  encoder.
- **Gates:**
  - Rust: fmt, clippy `-D warnings`, tests, coverage ≥ 85 %, no surviving mutant on the changed physics and field
    code.
  - Frontend: svelte-check, oxlint, prettier, knip, jscpd, i18n, coverage, Stryker ≥ 70 % on the changed helpers.
  - `scripts/check-file-size.sh`.

## Review Focus

- **The blade and the paddle.** They pass through the host's fixed deck and guards; kinematic against fixed does not
  collide in Rapier by default. Nothing else may collide with them except cartons.
- **The turntable's field.** Its sign while turning must match the deck's turn in the view. Settled, its heading must
  be the deck's angle.
- **The ST slew.** It must land on the target to the millisecond, and a reversal must ramp through zero.
- **The PLC program.** It must singulate before the stopper: a blade raised under the next carton throws it
  (measured).

## Decisions taken while writing this plan

1. **The turntable's angle.** It is positive counter-clockwise seen from above: about +Y, carrying +x towards −z, to
   the lane's left. OIP's sign is the same: `R_y`, with `(fx c + fz s)` for vx.
2. **The turntable's frame.** Its instance's origin is the deck's centre at the floor, as OIP's. In the layout its
   lane is a conveyor whose entry lies 0.75 × width behind the centre.
3. **The stopper's mount position snaps to the gap between the drawn rollers nearest it.** It uses the roller view's
   pitch, 0.075, and its count rule (`roller.ts` `layoutRollers`), in Rust and in TS. The same table of cases tests
   both.
4. **An unconnected pusher `out`, or a blocked sink there, has no wall** (amended in Task 1). The plan first gave it a
   wall, as an unconnected chain transfer's. The paddle's stroke crosses that edge, and the wall crushed the carton
   against the paddle.
5. **`sort_line`'s turntable turns at 90°/s, not OIP's 20°/s**, so the run is short. Acceptance 3 is proved at 20°/s
   in the field's and the world's tests.
6. **`sort_line` singulates on an infeed belt before the stopper's roller,** because the probe showed that a blade
   raised under a queued carton throws it.
7. **The snapshot's `transfers` becomes `actuators`, `{ id, value }`.** The value is a transfer's or a stopper's lift,
   a pusher's reach (0–1) or a turntable's angle in degrees, and the view knows which from the family. This is
   refactor on touch: one list instead of four.

## File structure

| File | Change |
|---|---|
| `app/src-tauri/physics/src/layout.rs` | conveyors stay; mounted devices move to `layout_devices.rs` (transfer, stopper, pusher), the turntable to the conveyors |
| `app/src-tauri/physics/src/layout_devices.rs` | new: mount resolution, host checks, `Stopper`, `Pusher` |
| `app/src-tauri/physics/src/path.rs` | the turntable's cross deck; guard openings on straight conveyors |
| `app/src-tauri/physics/src/world.rs` | conveyors, sources, sinks, sensors; devices move out |
| `app/src-tauri/physics/src/world_devices.rs` | new: kinematic devices (transfer, stopper, pusher) and the turntable's surface |
| `app/src-tauri/physics/src/world_{stopper,pusher,turntable}_tests.rs` | new |
| `app/src-tauri/src/field.rs` | natives of the three families; the turntable's drive feedback is native |
| `app/src-tauri/src/line.rs` | devices posed from their natives |
| `app/src-tauri/src/engine.rs` | `actuators` in the snapshot |
| `app/src-tauri/src/line_devices_tests.rs`, `field_devices_tests.rs` | new |
| `app/src-tauri/project/tests/fixtures/valid/sort_line/` | new, written by the script |
| `app/src/lib/oip/geometry/{builder.ts, fillet.ts}` | `fillet` copied |
| `app/src/lib/oip/parts/{stopper.ts, pusher.ts, turntable.ts, turntableBed.ts}` | copied generators |
| `app/src/lib/oip/parts/{part.ts, roller.ts, belt.ts}` | the moving tag; guard openings |
| `app/src/lib/{families.ts, families_devices.ts, materials.ts, snapshot.ts, ProjectScene.svelte}` | drawing and posing the three families |
| `app/public/textures/{DiverterLabel,DiverterMetal}.webp` | copied |
| `app/e2e/sort.mjs` | new |
| docs | verification file, ROADMAP 4.4c row, CHANGELOG, THIRD_PARTY_NOTICES |

## Tasks

### Task 1: the stopper and the pusher in the physics

- [x] Split `layout.rs`: mounted devices to `layout_devices.rs`, behaviour unchanged. `cargo test -p twinova-physics`
  is green with the fingerprints unchanged.
- [x] Split `world.rs`: the chain transfer's body and exit to `world_devices.rs`. Tests are green, the fingerprints
  unchanged.
- [x] Tests first (`world_stopper_tests.rs`, spec acceptance 1):
  - three cartons queue against a raised blade at 0.5 m/s (front within 1 mm, at rest, tilt < 2°);
  - lowered, the front carton is at speed within 0.22 s (spec, amended);
  - layout refuses a stopper on a belt, a curve, an inclined or a skewed roller, and names the fix;
  - the mount snaps to the gap, on a table of cases.
- [x] Implement `Stopper` and `World::set_stopper(k, lift)`: OIP's blade 0.03 × (raise + 0.15) × (width − 0.03),
  grip √(0.6 × 0.5). The stroke is raise + 0.05, so lowered its top is 5 cm under the surface.
- [x] Tests first (`world_pusher_tests.rs`, spec acceptance 2):
  - at 0.25 s a passing carton goes to a perpendicular takeaway upright (at rest < 1°, max < 5°) and square (≤ 3°);
  - a sink on `out` collects it past the host's edge; with `out` unconnected or its sink blocked the carton falls
    beside the conveyor;
  - the guard on its own side has a 0.75 m opening, collider pieces asserted in `path_tests.rs`;
  - layout refuses a pusher on a curve, an inclined or a skewed host, or with a guard on its exit side.
- [x] Implement `Pusher`, `Exit::Push` and `World::set_pusher(k, reach)`: OIP's paddle 0.47 × 0.40 × 0.50, its face
  1 cm outside the frame, 0.047 above the surface, the same grip.
- [x] Gate: `cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test
  --workspace`; `cargo mutants` on the changed physics files; `bash scripts/check-file-size.sh`.
- [x] Commit `feat(physics): OIP's blade stop and pusher as kinematic bodies`.

### Task 2: the turntable in the physics

- [x] Tests first (`world_turntable_tests.rs`, spec acceptance 3):
  - a carton at rest on the lane, turned 90° at 20°/s with OIP's profile (the test drives angle and rate as the block
    will), leaves its place by ≤ 2 mm and its yaw by ≤ 0.5°;
  - settled at 90°, the lane carries it along the new heading at its speed, into a conveyor at that end;
  - a sensor on the turntable sees across the start frame;
  - the layout places the lane's entry 0.75 × width behind the centre.
- [x] Implement:
  - the conveyor's deck shape (straight, curve, turntable cross);
  - `path::deck` for the cross;
  - `World::set_turntable(k, angle, rate)`: while the rate is not 0 the lane's field turns about the centre at that
    rate; otherwise it runs along the angle at its speed.
- [x] Gate as in Task 1, and a fingerprint test for a turntable world.
- [x] Commit `feat(physics): OIP's turntable as a conveyor whose flow turns with its deck`.

### Task 3: fixtures, behaviour blocks, field and line

- [x] Script `migrate_fixtures_4_4c.py` (scratchpad), canonical JSON as in 4.4b:
  - the three definitions and their `.st` blocks;
  - the new `valid/sort_line`:
    `source → belt_in → roller_a (stopper) → turntable → belt_b (pusher → reject) → sink_b`, and at 90°
    `turntable → belt_c → sink_c`;
  - its `plc/Main.st`.
  The corpus test opens it.
- [x] Tests first (`field_devices_tests.rs`):
  - the stopper's and the pusher's strokes take their time to the millisecond, with feedback at the ends only;
  - the turntable turns 90° at 20°/s in 5.0 s ± 1 ms;
  - `in_position` is TRUE only when settled;
  - enable off stops it within 3° and it resumes;
  - a reversal mid-turn ramps through 0;
  - the lane is held at 0 while turning, and its drive is a conveyor's.
- [x] Implement the natives in `field.rs`, the devices in `line.rs`, and `actuators` in `engine.rs`. The view reads
  `actuators` in the same commit, so the app stays whole (amended in Task 3).
- [x] Tests (`line_devices_tests.rs`, spec acceptance 4):
  - `sort_line` conserves cartons every millisecond;
  - the split matches the program: every second carton to `sink_c`, every second straight one to `reject`;
  - the fingerprint is equal across Run ×1, as fast as possible and step by step, and pinned.
- [x] Gate: Rust as in Task 1; `cargo mutants` on `field.rs`, `line.rs` and `engine.rs`'s changed functions.
- [x] Commit `feat(line): stopper, pusher and turntable behaviours, natives and the sort line`.

### Task 4: the view

- [x] Copy:
  - OIP's blade stop, diverter and turntable `geometry()`;
  - `fillet`;
  - the turntable's roller bed (radius 0.04, chord-clipped);
  - the two diverter textures.
  Provenance headers, and the files split below 600 lines.
- [x] Tests (vitest):
  - each generator's bounds against OIP's `rig.json` footprint;
  - the moving tag on the blade, the paddle and the deck;
  - families place the three as the physics does (mount, gap snap, centre);
  - guard openings in the roller and belt views;
  - `actuators` posing.
- [x] Implement `families` (split if near 600), `ProjectScene.svelte`'s moving groups, and `snapshot.ts`.
- [x] Gate: `npm run check && npm run lint && npm run format:check && npm run check:i18n && npm run deadcode && npm run
  dup && npm run coverage && npm run build`; `npm run mutation` on the changed helpers.
- [x] Commit `feat(view): OIP's blade stop, pusher and turntable drawn and moving`.

### Task 5: E2E, soak and documents

- [x] `e2e/sort.mjs`, registered in `npm run e2e`:
  - the sort line builds and runs;
  - both sinks and the reject count;
  - `in_position` and the feedback in the I/O panel;
  - a screenshot.
  Linux under `xvfb-run`.
- [x] Soak: `npx tauri build --no-bundle && xvfb-run npm run soak -- 45 sort_line`.
- [x] Write the verification file `docs/superpowers/verification/2026-11-11-stopper-pusher-turntable.md`, and update
  the ROADMAP 4.4c row, `CHANGELOG.md` and `THIRD_PARTY_NOTICES.md`.
- [x] Push, open the PR, CI green on Linux and Windows, merge (user's standing instruction).
