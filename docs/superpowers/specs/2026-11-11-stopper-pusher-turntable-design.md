# Spec: stopper, pusher and turntable (ROADMAP 4.4c)

Date: 2026-11-11. Status: **approved** (user, 2026-11-11, with both recommended decisions).

4.4a split ROADMAP 4.4 into sub-phases; 4.4c is the stopper, the pusher and the turntable (user, 2026-11-11: "il passo
naturale per chiudere la 4.4"). The method is 4.4a's and 4.4b's:
- OIP's geometry is copied in TypeScript;
- its physics is translated into Rust;
- its `step()` is re-expressed in ST;
- its Jolt workarounds are kept only where a Rapier measurement needs them.

## Provenance (OIP `1e7488c`, MIT)

| OIP source | Lines | In Twinova |
|---|---|---|
| `examples/parts/blade-stop/part.js` `geometry()` | 220 | copied as the stopper's view generator |
| `examples/parts/diverter/behavior.ts` `geometry()` | 271 | copied as the pusher's view generator |
| `scripts/build-diverter-textures.mjs` output `public/textures/{DiverterLabel,DiverterMetal}.webp` | 24.7 + 50.8 KB | copied, first-party per OIP's `THIRD_PARTY_NOTICES.md` (lines 61–66) |
| `examples/parts/turntable-conveyor/part.js` `geometry()`, `_shared/rollerBed.js` (its radius, link and chord-clipped lengths) | 574 + 153 | copied as the turntable's view generator, split below 600 lines |
| `src/core/filletCorner.ts`, `geometryBuilder.ts` `fillet` | — | the builder's `fillet`, left out in 4.4a, is copied: the pusher's hose uses it |
| the three `step()` | 17 + 23 + 61 | re-expressed as ST blocks |
| the three `rig.json` colliders and joints | — | translated into Rust (below) |

- **No model is copied.** All three parts are procedural at `1e7488c`.
- **The diverter's old GLB is left out.** `public/part-assets/d1eb5b7c…a086a.glb` (Draco, 184 KB) is kept by OIP
  only for old projects, and the current rig does not use it. ROADMAP 4.4's Draco question therefore does not arise.
- **The pusher's label carries no name or logo.** It is a yellow hazard sticker reading "KEEP CLEAR / MOVING PARTS",
  drawn by OIP's own script.

## What OIP does (read at `1e7488c`)

**Blade stop** (`blade-stop/`):
- **Placement.** It snaps into a level, straight roller bed. The blade sits in the roller gap nearest the drop, square
  to the flow, and is as wide as the bed less 3 cm.
- **Collider.** A hidden kinematic box, 0.03 thick and `raiseHeight` + 0.15 tall. Raised, it spans 0.15 below the
  surface to `raiseHeight` above it.
- **Stroke.** A prismatic `raise` of `raiseHeight` + 0.05 (lowered, its top is 5 cm below the surface), slewed
  linearly over a fixed 0.15 s.
- **Params:** `width`, `active`, `raiseHeight` 0.04 (0.02–0.5), `airPressureHeight` (a mount offset, view only), and
  a BOOL tag.
- **Feedback:** none.

**Diverter, a pusher** (`diverter/`):
- **Placement.** It sits on a guard rail of the host, its paddle face 1 cm outside the rail. OIP cuts a 0.75 m opening
  in that guard, mesh and collider; the far side is not opened.
- **Collider.** One hidden kinematic box, 0.47 along the flow × 0.40 tall × 0.50 deep, from 0.047 to 0.447 above the
  surface, with friction 0.6.
- **Stroke.** A prismatic `ext3` of `divertDistance` (0.75 m), slewed linearly over `divertTime` (0.25 s, so 3 m/s),
  out and back.
- **Params:** `diverted`, `divertTime`, `divertDistance`, `standHeight`, and a BOOL tag.
- **Feedback:** none to the PLC. A green lamp lights at home and a red one otherwise.

**Turntable conveyor** (`turntable-conveyor/`):
- **Lane.** Three static boxes form a cross: 1.5w × w, plus two caps reaching the rim. They carry one surface. While
  settled its velocity runs along the deck's angle. While turning it is a rotating field ω × (p − c), the curves' own.
- **Deck.** A kinematic disc, 72 mm under the lane. Cargo does not touch it.
- **Hold.** The deck **grips** its load for the whole turn: the carton is made kinematic and teleported. OIP's reason:
  in Jolt, friction alone left it 0.7–4° off, and on a bare plate the load sometimes stopped following.
- **Slew.**
  - Shortest path to the target angle, at most `rotationSpeedDeg` (20°/s), accelerating over 0.5 s.
  - It brakes to land on the target: 90° takes 5.0 s.
  - Enable off stops it in 0.3 s; it resumes when enable returns.
  - The lane conveys only once settled.
- **Params:** `width` 1.524, `standHeight`, `speed` 1 m/s, `deckAngleDeg`, `rotationSpeedDeg`.
- **Tags:** angle (REAL), speed (REAL), enable (BOOL), done (BOOL).
- **OIP's own bug:** the deck starts at 0°, not at `deckAngleDeg`.

## Measured in Rapier (2026-11-11, throwaway worktree, release)

Setup: the 4.4b2 world (Coulomb, one solver iteration, 1 ms), the 0.3 m carton, a 0.6 m roller host with grip 0.42,
and the colliders above as kinematic position-based bodies. Probe: a throwaway test module (`probe44c.rs`), not kept
in the repository; the tests of Tasks 1–3 measure the same again.

**Stopper, blade raised at x = 2, a 4 m roller and three cartons:**

| Host speed | Raise | Front against the blade | Residual v | Max tilt | Max rise |
|---|---|---|---|---|---|
| 0.5 m/s | 0.04 | −0.3 mm | 0 | 1.55° | 3.5 mm |
| 1.0 | 0.04 | −0.6 mm | 0 | 6.55° | 14 mm |
| 2.0 | 0.04 | −0.3 mm | 0 | 30° | 48 mm |
| 0.5 | 0.15 | −0.2 mm | 0 | 0.25° | 1 mm |
| 1.0 | 0.15 | −0.2 mm | 0 | 0.07° | 1 mm |

- **It stops a queue dead:** the three cartons stand touching (1.835, 1.536, 1.236).
- **Released, the front carton is at the host's speed 209 ms after the command** (amended in Task 1, 2026-11-11).
  The blade's top passes the surface 67 ms into OIP's 0.15 s stroke of 0.09 m, then the rollers bring the carton to
  0.5 m/s at μg in 121 ms. The probe's first figures (157–538 ms) lowered the blade 0.24 m in 150 ms, 2.7 times OIP's
  stroke, and are withdrawn.
- **OIP's default raise of 4 cm is low for this carton above 0.5 m/s.** The impact tips it over the blade's top. OIP's
  own demo raises 0.15.
- **A blade raised under a carton throws it** (max rise 0.115 m, then on its side). OIP does the same; keeping the
  blade down is the PLC's job.

**Pusher (OIP's paddle, 0.75 m stroke) beside a 0.6 m host at 0.5 m/s, fired when the carton's centre passes it:**

| Takeaway | Stroke time | Result | Max tilt | Yaw at rest |
|---|---|---|---|---|
| Perpendicular belt, its entry beside the host (2 cm gap) | 0.25 s (3 m/s) | carried away at 0.5 m/s, 6.8 cm off its centre | 2.0° | 0.4° |
| Same | 0.5 s | same, 6.9 cm | 0.1° | 0.1° |
| Same, host at 1.0 m/s | 0.25 s | same, 16 cm off: 1 cm over its edge without guards | 2.6° | 0.2° |
| Parallel lane, far guard | 0.25 s | against the guard, carried | 6.1° | 2.0° |
| Parallel lane, no far guard | 0.25 s | **thrown 2.6 m to the floor** | — | — |
| Parallel lane, far guard | 0.12 s (6 m/s) | **knocked on its side** | 91° | — |

- **Late or early firing (±0.1 m) clears the deck the same.**
- **Paddle friction 0.6 against 0 changes the yaw by under 3°.**
- **What leaves at 3 m/s needs something to stop it.** Either a takeaway that runs the same way, or a guard.

**Turntable, OIP's static cross and its field only, with no grip and no deck body:**

| Carton at rest, from the centre | Top rate | Turn of 90° | Yaw after | Off its carried place | Then along the new heading |
|---|---|---|---|---|---|
| 0 | 20°/s | 4 983 ms | 89.999° | 0.0 mm | 0.4999 m/s |
| (0.05, 0.03) m | 20°/s | 4 983 ms | 89.999° | 0.1 mm | 0.5000 |
| 0.3 m | 20°/s | 4 983 ms | 89.998° | 0.3 mm | 0.5000 |
| (0.3, 0.2) m | 60°/s | 1 998 ms | 89.994° | 0.5 mm | 0.5000 |
| 0.3 m | 180°/s | 1 001 ms | 90.008° | 0.7 mm | 0.5000 |

- **Rapier's Coulomb field carries the load rigidly**, as the curves showed (4.4b), and OIP's grip patches Jolt.
  "Baseline or drop": the grip is not translated, and neither is the deck body, which cargo never touches.

What these probes do not prove:
- other cartons, or a row of cartons on a turntable or against a pusher;
- the views against the colliders;
- Windows (the fingerprint test in CI covers it).

## The families

### `twinova.stopper@1`

| Parameter | Type, unit | Default | From OIP |
|---|---|---|---|
| `raise_height` | REAL, m | 0.04 | `raiseHeight` (0.02–0.5) |
| `raise_time` | TIME | 150 ms | OIP's fixed `RAISE_SECONDS` |

- **Mounting:** on a roller conveyor at a position along it, which snaps to the nearest gap between the drawn rollers.
  The host must be straight, level and unskewed. The blade is the host's width less 3 cm.
- **Signals:**
  - `stop`: PLC output, TRUE raises the blade (OIP's tag).
  - `raised_fb`: PLC input, the top of the stroke. This is Twinova's addition, as on the chain transfer.
- **Behaviour** `StopperBehaviour`: `lift` slews linearly over `raise_time`, as the chain transfer's does. The native
  model poses the blade at `lift`.
- **Physics:**
  - OIP's collider: a kinematic box, 0.03 × (`raise_height` + 0.15) × the blade's width.
  - Its carton grip is √(0.6 × 0.5) = 0.548: OIP's ground friction against its box's, combined as Jolt does by their
    geometric mean (amended in Task 1; the probe used 0.3).
  - OIP's `baseCollider` is not translated: it lies under the surface, where no carton reaches.
- **Not taken:** `airPressureHeight`, which is a view offset; the view uses OIP's default.

### `twinova.pusher@1`

| Parameter | Type, unit | Default | From OIP |
|---|---|---|---|
| `stroke` | REAL, m | 0.75 | `divertDistance` (0.1–1.5) |
| `push_time` | TIME | 250 ms | `divertTime` (50 ms–2 s) |
| `out_right` | BOOL | FALSE | which side it pushes to, as the chain transfer's: mounted on the host's right, it pushes left |

- **Mounting:** on a straight, level conveyor (belt or rollers) at a position along it, its paddle face 1 cm outside
  the host's frame.
  - **The push side's guard opens 0.75 m around the pusher, mesh and collider, as in OIP (decision 2).**
  - The exit side's guard must be off. As on the chain transfer, Build refuses otherwise and names the fix.
- **Flow:** its `out` port is the side exit, to a sink (a chute) or to a conveyor.
  - With neither, or with its sink blocked, that side has no wall (amended in Task 1). The paddle's stroke crosses that
    edge, and a wall there crushed the carton against the paddle (measured). What it pushes there falls beside the
    conveyor.
- **Signals:**
  - `extend`: PLC output (OIP's tag).
  - `extended_fb` and `retracted_fb`: PLC inputs, the two ends of the stroke. These are Twinova's additions;
    `retracted_fb` is OIP's green lamp.
- **Behaviour** `PusherBehaviour`: `reach` slews linearly over `push_time`. The native model poses the paddle at
  `reach × stroke`.
- **Physics:** OIP's collider, a kinematic box 0.47 × 0.40 × 0.50, with the stopper's carton grip of 0.548.
- **View:** OIP's machine, with its lamps from the snapshot's reach.

### `twinova.turntable@1`

| Parameter | Type, unit | Default | From OIP |
|---|---|---|---|
| `width` | REAL, m | 0.6 | `width` (OIP 1.524; 0.6 matches the line); the deck's diameter is 1.5 × width |
| `height` | REAL, m | 0.8 | `standHeight`: the lane's top |
| `speed` | REAL, m/s | 0.5 | `speed` |
| `angle` | REAL, ° | 0 | `deckAngleDeg`: the start angle, and the target while no PLC writes one |
| `rotation_speed` | REAL, °/s | 20 | `rotationSpeedDeg` (1–360) |
| `friction` | REAL | 0.42 | the lane is rollers |
| the conveyor drive's parameters | | | as `twinova.roller@1` (decision 1) |

- **Physics:**
  - The lane is a conveyor in the layout: OIP's static cross is its deck, at the turntable's centre.
  - While the deck turns, the lane is the curves' rotating field at the turn's rate. Settled, it runs along the
    deck's angle at the drive's speed.
  - There is no grip and no deck body (measured above).
  - Its ends have no wall, because the deck turns them (OIP has none).
- **Signals:**
  - `target_angle`: PLC output, REAL, °; optional, falling back to `angle`.
  - `enable`: PLC output, optional, falling back to TRUE.
  - `in_position`: PLC input (OIP's done).
  - The lane's signals are a conveyor's (decision 1).
- **Behaviour** `TurntableBehaviour`:
  - OIP's slew, millisecond by millisecond: shortest path, the ramp over 0.5 s, braking onto the target, a 0.3 s stop
    when `enable` drops.
  - The lane's drive, as every conveyor's.
  - Its outputs: the angle, the rate and `surface_speed`. The lane's drive ramps to 0 when the deck leaves its
    position, and to `speed_ref` once it is back, as every conveyor's drive does with `run` (amended in Task 3). With
    OIP's ramp of 0 the lane stops at once, as OIP's does. While the deck turns, the native model drives the lane by
    the rotating field whatever its speed.
  - Two fixes on OIP. The deck starts at `angle`, not at 0. A reversal mid-turn ramps through 0 instead of
    reversing within one step.
- **Sensors:** a diffuse sensor may mount on a turntable. Its beam is fixed, across the lane in the turntable's start
  frame, as OIP's eye stands on the floor beside the slewing deck.
- **Flow:** ports `in` and `out` along the start frame. The physical handover at any angle is by contact, as between
  conveyors.

## Decisions of the user (2026-11-11)

1. **The turntable's lane is a conveyor's** (chosen over OIP's REAL speed tag). It is driven as every Twinova
   conveyor: `run`, an optional `speed_ref`, `running`, and the native velocity, encoder and pulse, through the ST
   drive. Its ramp parameters default to 0, like OIP's lane, which has none.
2. **The pusher's own guard opens around it, as in OIP** (chosen over requiring it off). The guard is split around
   0.75 m, view and collider, on straight conveyors.

## Acceptance

1. **Stopper.** At 0.5 m/s with OIP's defaults:
   - a queue of three cartons stands against the blade, the front one within 1 mm of it, at rest, tilted < 2°;
   - lowered, the front carton runs on at the host's speed within 0.22 s of the command (amended in Task 1: 67 ms of
     stroke to the surface and 121 ms at μg; measured 209 ms);
   - `raised_fb` follows the stroke.
2. **Pusher.** At OIP's 0.25 s, a carton passing at 0.5 m/s is pushed onto a perpendicular takeaway and carried away
   upright (tilt at rest < 1°, max < 5°) and square (yaw ≤ 3°). `extended_fb` and `retracted_fb` follow the stroke.
3. **Turntable:**
   - a carton turned 90° at 20°/s leaves its place by ≤ 2 mm and its yaw by ≤ 0.5°, and runs out along the new
     heading at the lane's speed;
   - 90° at 20°/s takes 4 983 ms (amended in Task 2). OIP's profile is 5.0 s if continuous; stepped every
     millisecond, its braking starts 0.01° late, and its step lands the deck once what is left fits in one step, at
     0.76°/s;
   - `in_position` is TRUE only when settled;
   - enable off stops it within 3°.
4. **The line** `sort_line`, run in the app on Linux and Windows:

   ```
   source → roller_a (stopper) → turntable ─(0°)→ belt_b (pusher → reject) → sink_b
                                            └(90°)→ belt_c → sink_c
   ```

   - The PLC holds each carton at the stopper until the turntable is home and empty.
   - It turns every second carton to `belt_c`, and pushes every second straight one to `reject`.
   - No carton is lost: created = delivered + on the line, every millisecond.
   - The split is the program's.
   - The fingerprint is equal across run modes and platforms.
5. **The gates.** Rust: fmt, clippy and tests, coverage ≥ 85 %, no surviving mutant. Frontend: svelte-check, oxlint,
   knip, jscpd, coverage, Stryker ≥ 70 %. The 600-line cap. The E2E on Linux and Windows. A soak.

## What this does not prove

- Products other than the carton, pallets (4.4d).
- A turntable chained to another, or fed at an angle other than its four ends.
- A stopper on a belt (OIP's fits a roller bed only), or a pusher on a curve.
- Pneumatics: the strokes are OIP's linear slews, with no cushioning or air pressure.
