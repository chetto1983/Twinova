# Spec: roller conveyor, chain transfer and the first complete line (ROADMAP 4.4a)

Date: 2026-11-10. Status: **draft**, for the user's approval.

ROADMAP 4.4 is split into sub-phases, each with its own spec, E2E and merge (user, 2026-11-10):

| Sub-phase | Families |
|---|---|
| **4.4a** | straight roller conveyor, chain transfer (the two-output diverter), M4.1's acceptance line |
| 4.4b | curved conveyors (belt and roller), and the roller options 4.4a leaves out |
| 4.4c | stopper, pusher, turntable |
| 4.4d | through-beam, colour and barcode sensors, with product attributes |

## Decisions of the user (2026-11-10)

1. **Line first,** in sub-phases.
2. **The diverter is OIP's chain transfer** (right-angle pop-up transfer), fitted inside a roller conveyor, as in
   OIP. 4.4a therefore also brings the straight roller conveyor.
3. **Copy OIP, then translate** ("copiamo e traduciamo in Rust il TS: meno lavoro, più effort subito"). OIP is MIT,
   and CLAUDE.md's clean-room rule bars copying only from commercial tools.
   - **The 3D geometry is copied** in TypeScript. It is the same language and the same three.js (0.187) as
     Twinova's view.
   - **The physics and the native models are translated** into Rust, function by function, with OIP's constants
     and names. Each one cites its OIP file and commit.
   - **The device logic stays in ST** (decision of 2026-11-07). OIP's `step(ctx)` is re-expressed as the family's
     behaviour block.
   - **OIP workarounds for Jolt are kept only if Rapier needs them.** Its ghost-contact gates and its
     friction-coupling patch are translated only where a Rapier measurement shows the need (CLAUDE.md, "baseline
     or drop"; research 15 already dropped the seam gate this way).

## Provenance and what is copied (OIP `1e7488c`)

| OIP source | Lines | In Twinova |
|---|---|---|
| `src/core/geometry/{geometryBuilder,geometryShapes,geometryCsg,meshDescription}.ts`, `src/core/filletCorner.ts` | 1 372 | copied to `app/src/lib/oip/geometry/`; they import only each other (closure measured 2026-11-10) |
| `examples/parts/roller-conveyor/part.js` `geometry()`, `_shared/{rollerBed,rollerFrame,legs}.js` | about 500 | copied as the roller conveyor's view generator |
| `examples/parts/chain-transfer/part.js` `geometry()` | 266 | copied as the chain transfer's view generator |
| `examples/parts/belt-conveyor/part.js` `geometry()`, `_shared/legs.js` | 697 + 67 | copied as the belt's view generator, split below 600 lines |
| `examples/parts/diffuse-sensor/part.js`, `_shared/sensorPhotoeye.js` | 241 | copied as the photo-eye's view generator |
| `public/textures/{ChainLink.webp, RollerMetal.webp, BeltFabric.jpg, BeltFabricArrows.jpg}` | — | copied, first-party per OIP's `THIRD_PARTY_NOTICES.md` |
| `src/sim/conveyorContacts.ts` (surface velocity per contact) | 369 | translated, without the gates (above) |
| `src/sim/deckDrive.ts` (`setSurfaceVelocity`, `setSurfaceSpeed`) | 77 | translated |
| `examples/parts/chain-transfer/behavior.ts` `step()` | 30 | re-expressed as the ST block |
| `src/sim/rigDriver.ts` (`MoveKinematic` of a moving link) | the part that drives one prismatic link | translated onto Rapier's position-based kinematic body |

**Not copied:**
- OIP's part SDK (`partApi.ts` 1 700 lines, `partPackage.ts` 1 303, `rigPose.ts` 1 392). Its import closure is
  11–17 k lines, over Twinova's 600-line cap and outside its model.
- Each copied generator is adapted to Twinova's parameters (for example `frameHeight` against `height`), not to OIP's
  `definePart`.

**Obligations:**
- OIP's copyright and MIT notice go in `THIRD_PARTY_NOTICES.md`, with the commit.
- A header in each copied file names its source path.
- The copied files pass Twinova's gates like any other code: svelte-check, oxlint, knip, jscpd, the 600-line cap,
  coverage and Stryker on the helpers. A file that cannot is split or adapted, not excluded.

## What OIP does (read at `1e7488c`)

**Roller conveyor** (`examples/parts/roller-conveyor/`):
- The rollers are drawn only. Physics is one hidden conveying deck, `[length, 0.04, width]`, its top at the bed's
  surface.
- Carton grip is 0.42 by default.
- Params: `length` 4, `width` 1.524, `frameHeight` 0.17, `speed` 1.0 m/s, `accelTime`/`decelTime` 0.2 s, `friction`
  0.42, `skew`, `tiltDeg`, guards.

**Chain transfer** (`examples/parts/chain-transfer/`):
- Placement. It snaps onto a roller bed (`snap.track`). Its deck `[chains·distance, 0.06, laneLength]` sits 3 cm
  below the rollers (`targetLocalY −0.03`).
- Lift. A prismatic `lift` of 0.095 m, so the deck tops out 6.5 cm above the rollers. It ramps linearly over
  `raiseTime`, 0.15 s by default.
- Surface velocity. It follows the command at once: raised runs at 90° (−Z), lowered runs forward.
- Params: `chains` 5, `distance` 0.33 m, `laneLength` 2 m, `speed` 2 m/s, `raised`, `raiseTime`.
- Signals: a BOOL `popupTag` only, with no end-position feedback.
- OIP's own note on the lift: at the top "the deck stops and the load does not, carrying on for v²/2g. The default
  throws cargo ~20 mm".

## Measured in Rapier (2026-11-10, throwaway worktree, release)

**Setup.**
- The 4.3 world: carton 0.3 m rounded 0.02, deck 0.17 m, step 1 ms.
- Belt A, 3 m at 0.5 m/s.
- A 2 m roller host: a static deck, grip 0.42.
- A chain deck inside it: 0.66 × 0.06 × 0.6, kinematic position-based, 3 cm below, lifted 0.095 m in `raiseTime`.
- A 2 m side belt at 90°.

| Case | Lift done, carton x | To the side belt | Peak height | Tilt | Yaw |
|---|---|---|---|---|---|
| Side belt at the raised height (host + 0.065) | 4.045 | 2 525 ms | +20 mm above the raised rest | 0.66° | 0.01° |
| Side belt at the host height | 4.045 | 2 541 ms | +20 mm | 16.3° (it tips off the raised deck's edge, a 6.5 cm drop) | 0.26° |
| `raiseTime` 0.5 s | 4.121 | 2 660 ms | +1.8 mm | 16.3° (same drop) | 0.30° |
| Raised with the carton's front on the deck edge (x = 3.8) | 3.844 | 2 542 ms | +20 mm | 16.4° | 0.21° |
| Chain grip 0.7 instead of 0.35 | 4.031 | 2 478 ms | +20 mm | 16.3° | 0.39° |

- **The hop is OIP's v²/2g:** 0.095/0.15 = 0.63 m/s gives 20 mm, as OIP's comment says.
- **The transfer itself is square:** yaw ≤ 0.4°.
- **A side conveyor must be at the raised height,** or the carton tips 16° over the deck's edge. A sink at the side
  edge collects before any drop.

An earlier probe the same day turned a standalone deck's velocity without a lift (yaw 0.00°, tilt 0.05°). It is
superseded by decision 2 and kept as evidence that the velocity switch alone is square.

What this does not prove:
- other carton sizes or masses, or a packed row of cartons on the lift;
- the visible rollers' look against the physics deck, since they are drawn only;
- Windows: the fingerprint test in CI covers it.

## The families

### `twinova.roller@1`: straight roller conveyor

| Parameter | Type, unit | Default | From OIP |
|---|---|---|---|
| `length` | REAL, m | 2.0 | `length` (OIP default 4) |
| `width` | REAL, m | 0.6 | `width` (OIP 1.524); 0.6 matches the line's belts |
| `height` | REAL, m | 0.8 | top of the rollers, as the belt's `height` (OIP: `standHeight`) |
| `frame_height` | REAL, m | 0.17 | `frameHeight` |
| `speed` | REAL, m/s | 0.5 | `speed` |
| `friction` | REAL | 0.42 | `friction` (carton grip) |

- **Signals and behaviour:** the belt's. `run` (PLC output) and `running` (PLC input), with a block `RollerBehaviour`
  equal to the belt's (`motor`, `start_delay`).
- **Physics:** one static deck with OIP's grip, its top at `height`.
- **Not in 4.4a:** `accelTime`/`decelTime`, `skew`, `tiltDeg`, guards, and the speed, velocity, pulse and encoder
  tags. They go to 4.4b, with the curves, which share them.

### `twinova.chain_transfer@1`: chain transfer (replaces `twinova.diverter`)

| Parameter | Type, unit | Default | From OIP |
|---|---|---|---|
| `chains` | DINT | 2 | `chains` (OIP 5) |
| `distance` | REAL, m | 0.33 | `distance` |
| `lane_length` | REAL, m | 0.6 | `laneLength`, set to the host's width |
| `speed` | REAL, m/s | 0.5 | `speed` (OIP 2) |
| `raised` | BOOL | FALSE | `raised` (start state) |
| `raise_time` | TIME | 150 ms | `raiseTime` |
| `out_right` | BOOL | FALSE | sign of OIP's negative `speed`: carries to +Z instead of −Z |

- **Mounting:** on a roller conveyor, with a mount connection at a position along it, as a sensor mounts on a belt.
- **Flow:** its `out` port is the side exit. The straight exit is the host's `out`.
- **Signals:**
  - `popup` (PLC output, BOOL): raise and divert. This is OIP's `popupTag`.
  - `raised_fb` (PLC input, BOOL): the lift is at the top. **Not in OIP**, which has no feedback; real transfers have
    end-of-stroke switches. It is how the line program waits out `raise_time`, the interlock OIP's own help text asks
    of a program.
- **Behaviour block `ChainTransferBehaviour`** re-expresses OIP's `step()`:
  - `popup` drives `raised` at once;
  - `lift` (REAL, m) ramps towards 0.095 or 0 at 0.095/`raise_time` per second;
  - `raised_fb` is set when `lift` reaches 0.095.
  - Its native variables are `lift : REAL` and `raised : BOOL`.
- **Native model:** the deck is a kinematic position-based body at the host's height − 0.03 + `lift`. Its surface
  velocity is sideways at `speed` while `raised`, otherwise forward at the host's speed.
- **Without its block,** Build fails: "twinova.chain_transfer needs its behaviour".

`twinova.diverter` and its fixture's swing flap are removed (format 1 is not released, so it is amended in place).
4.3's diverter block test moves to the chain transfer.

## The physics world

- **Surfaces.** Belts and rollers are static decks. The chain deck is kinematic. Each carries a surface velocity
  through the contact hook, as today.
- **A kinematic deck is moved before each step** with `set_next_kinematic_translation`, Rapier's documented
  mechanism (`rigid_body.rs`: "Rapier computes the velocity needed to reach this position smoothly"). It does not
  collide with the static decks: Rapier's default `ActiveCollisionTypes` is DYNAMIC_DYNAMIC | DYNAMIC_KINEMATIC |
  DYNAMIC_FIXED (`collider_components.rs`), with no kinematic–fixed pair.
- **Exits:**
  - a conveyor's end;
  - the chain transfer's side, over the length of its deck.
  - A sink attaches to an exit and collects when the carton's front crosses it.
  - An unconnected exit is closed by a wall. A blocked sink puts a wall at its exit (M4.1: no product is lost).
- **The loop.** The chain deck is driven at step 3 of the 1 ms loop (`line.rs`), like a belt's motor. The order
  follows instance ids.

## The view

- The copied generators produce OIP's mesh descriptions.
- One small adapter turns a description into a three.js `BufferGeometry`. It is new code, with vitest tests.
- The view draws the chain deck at its `lift` from the snapshot. The snapshot gains each chain transfer's `lift`.
- A vitest per family checks its bounding box against the parameters. The source and the sink stay simple markers,
  as OIP's spawner and despawner are.

## The line fixture and its program

`two_belts_diverter` becomes:

```
source → belt_a → roller_r (with chain_transfer at its centre) → belt_b → sink_ok
                                       └→ side exit → sink_reject
```

- `sensor_a` sits at the transfer, on the roller. Two photocells can't share a host here (4.4a's sensors mount on
  belts or rollers).
- The program sends every third carton to reject, waits for `raised_fb` before it counts the carton as diverted, and
  lowers after the carton leaves.

## Acceptance of 4.4a

1. **The line runs in the app** (E2E, Linux and Windows):
   - cartons reach both sinks in the program's ratio;
   - the I/O panel shows `popup` and `raised_fb`;
   - the chain deck is drawn at its lift.
2. **No product is lost or duplicated** (line test): created = delivered + on the line, at all times. This holds with
   the normal sink blocked too, where accumulation is finite.
3. **Routing follows the command** (line test): only cartons on the deck when it is raised leave by the side.
4. **Determinism:**
   - Step, ×1 and as fast as possible give the same states (engine test);
   - the fingerprint with a chain transfer is equal on Linux and Windows (CI).
5. **Fidelity, translated from OIP's `physicsFidelity.test.ts`:**
   - surface speed on a roller bed;
   - the lift's hop at v²/2g;
   - a square transfer: yaw < 1°.
6. **Provenance:** `THIRD_PARTY_NOTICES.md` carries OIP's notice. Every copied or translated file names its source.
7. **Performance and gates:**
   - the release soak against 4.3a's figures;
   - coverage ≥ 85 %;
   - no surviving mutant in the changed Rust files;
   - Stryker ≥ 70 % on the new view helpers;
   - a verification file.

## What this will not prove

- **Building the line by hand.** That is the editor, 4.5.
- **The roller options left to 4.4b,** and curves.
- **Products other than the 0.3 m carton** (4.4d).
- **That a neighbour's height is checked.** The fixture sets it.
- **A real PLC** (M5).
