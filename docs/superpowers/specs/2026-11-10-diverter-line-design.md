# Spec: the right-angle diverter and the first complete line (ROADMAP 4.4a)

Date: 2026-11-10. Status: **draft**, for the user's approval.

ROADMAP 4.4 is split into sub-phases (user, 2026-11-10), each with its own spec, E2E and merge:

| Sub-phase | Scope |
|---|---|
| **4.4a** | the diverter and M4.1's acceptance line |
| 4.4b | roller and curved conveyors |
| 4.4c | stopper, pusher, turntable |
| 4.4d | through-beam, colour and barcode sensors, with product attributes |

4.4a covers the line `source → belt A → diverter → belt B → normal sink`, with the diverter's second output to a
reject sink (M4.1, "First acceptance scene").

## Decisions of the user (2026-11-10)

1. **Line first.** 4.4a ships the diverter and the M4.1 line end to end; the other families follow in 4.4b–d.
2. **The diverter is a right-angle (pop-up) transfer, as OIP's chain transfer.**
   - Straight through to `out_a`, or 90° off the side to `out_b`.
   - It works by switching the deck's surface velocity. This replaces the swing flap in the 4.3 fixture.
3. **Geometry is generated from the parameters,** as OIP generates every one of these families. Nothing from OIP is
   copied: the logic is re-expressed in Twinova's generators. 4.4 has no GLB and no Draco decoder.

## What OIP does (read at `1e7488c`, MIT; no code copied)

`examples/parts/chain-transfer/behavior.ts`, `rig.json` and `src/sim/chainTransferBehavior.test.ts`:

- **Shape.**
  - One hidden conveying deck, on a prismatic `lift` joint along +Y with a range of 0 to 0.095 m.
  - The deck moves, so it is built kinematic.
  - The lift ramps linearly over `raiseTime`, 0.15 s by default.
- **Motion.**
  - Lowered, the deck's surface velocity runs forward.
  - Raised, it runs at 90°, to −Z in the part's frame.
  - Speed is `speed`, 2 m/s by default.
- **Signals.** A BOOL "popup" command. There is **no end-position feedback**.

## Measured: the 90° transfer in Rapier (2026-11-10)

**Setup.**
- A throwaway worktree, tests in `twinova-physics` in release.
- 4.3's world: carton 0.3 m rounded 0.02, belt deck 0.17 m, friction 0.7/0.5 multiplied, step 1 ms.
- Belt A is 3 m at 0.5 m/s, then a 1.0 × 0.6 m deck, then belt B.
- The deck is a belt whose surface velocity is turned from +X to −Z at a trigger. A 2 m belt at 90° takes the carton
  off the deck's side.

| Case | Time to cross the deck sideways | Forward drift after the turn | Tilt on the deck / at the seam | Lift | Yaw |
|---|---|---|---|---|---|
| Straight through, no turn | — | — | — | 1.0 mm (spawn) | 0.005° |
| Turn with the carton centred (x = 3.5) | 882 ms to leave the deck at 0.5 m/s | 2.6 cm | 0.02° / 0.05° | 0.1 mm | 0.00° |
| Stopped for 300 ms, then turned | +300 ms | 1.8 cm | 0.02° / 0.05° | 0.1 mm | 0.00° |
| Turned while the carton straddles belt A and the deck (x = 3.0) | 967 ms | 6.9 cm | 0.02° / 0.03° | 0.1 mm | 0.01° |
| Turned while it straddles the deck and belt B (x = 3.85) | 918 ms | 4.7 cm | — | — | 0.02° |
| Side speed 1.0 m/s | 518 ms | 4.1 cm | — | — | 0.00° |

- **The drift** is the carton's forward momentum, braked by friction: v²/(2μg) = 0.25/(2·0.7·9.81) = 1.8 cm.
- **The turn alone gives a square, flat transfer.** A physical lift of the deck is not needed for it.

What this does not prove:
- other carton sizes, masses or speeds;
- a carton landing off-centre on a narrow receiving belt;
- a deck mounted inside a roller conveyor, which is OIP's real case. That arrives with the rollers in 4.4b;
- Windows. The fingerprint test in CI covers that once the diverter is in the world.

## The diverter family (`twinova.diverter@1`, amended in place: format 1 is not released)

| Parameter | Type, unit | Default | Range | Meaning |
|---|---|---|---|---|
| `length` | REAL, m | 1.0 | 0.3–5 | deck along the flow |
| `width` | REAL, m | 0.6 | 0.1–3 | deck across the flow |
| `height` | REAL, m | 0.8 | 0.1–3 | top of the deck, as a belt's `height` |
| `speed` | REAL, m/s | 0.5 | 0–3 | deck surface speed, straight and sideways |
| `out_b_right` | BOOL | FALSE | — | `out_b` on the right of the flow (+Z) instead of the left (−Z) |
| `switch_time` | TIME | 300 ms | ≥ 0 | change-over, owned by the behaviour block |
| `default_output` | DINT | 0 | 0–1 | the deck's direction at start |

- **Signals:** they stay as in 4.3. `divert` (PLC output, BOOL) asks for `out_b`. `in_position` (PLC input, BOOL)
  reports that the deck runs where `divert` asks.
- **Behaviour:** the 4.3 block `DiverterBehaviour` stays unchanged. Its native variable is `position` (DINT): 0
  runs to `out_a`, 1 to `out_b`, −1 during a change-over.
- **Native model** (in the physics world), every millisecond, from `position`:

  | `position` | The deck's surface velocity |
  |---|---|
  | 0 | forward (+X) at `speed` |
  | 1 | sideways, towards `out_b`, at `speed` |
  | −1 | zero: the deck holds what it carries, as a real transfer does while its chains rise |

  - The deck always runs; it has no motor signal, like OIP's chain transfer.
- **No physical lift.** The measurement shows the switch alone gives a square transfer. A moving body is the first
  thing 4.4c's stopper and pusher need, and it is designed there. The view draws the deck raised while `position` is
  1.
- **A diverter needs its behaviour.** A definition without a behaviour block fails Build: "twinova.diverter needs its
  behaviour". The family has no native fallback, unlike the belt, because its change-over is the block's.

## The physics world

- **Two kinds of conveying surface:** belts, and diverter decks. Each is one fixed deck with a surface velocity, as
  today.
- **Exits.** Each surface has its exits:
  - a belt has its end;
  - a deck has its end (`out_a`) and its side (`out_b`).
  - A sink attaches to an exit and collects a carton when the carton's front crosses that exit's edge, as at a belt's
    end today.
  - An unconnected exit is closed by a wall, so no product is lost (M4.1).
  - A blocked sink puts a wall at its exit.
- **A flow connection between two surfaces is physical.** The carton passes by contact and friction, as between two
  belts in 4.3. The next surface must touch the exit edge: an entry at a deck's side is placed at that edge.
- **Order** follows instance ids, as in 4.3. The diverter's position is applied in step 3 of the 1 ms loop
  (`line.rs`), like a belt's motor.
- **Sensors** mount on belts only, as in 4.3. Mounting on a deck is not needed by the line.

## The view

Geometry is generated from the parameters, in TypeScript, with three.js primitives:

| Family | What is drawn |
|---|---|
| Belt | frame, end drums, belt surface, legs to the floor |
| Diverter | frame, deck with chain strands, the deck raised when `position` is 1, and an arrow on the active output |
| Sensor | photo-eye body on its host's side, and the beam when it sees a carton |
| Source | entry marker |
| Sink | bin |

- One module per family. Each generator returns its meshes from the parameters, and a vitest asserts its bounding
  box against the parameters.
- The snapshot gains each diverter's `position`; the view draws from snapshots only, as today.

## The line fixture and its program

- `two_belts_diverter` is moved onto the new family:
  - the reject sink sits at the deck's side;
  - the program routes by the photocell before the deck. Every third carton goes to reject, so the rule is
    deterministic and countable.
  - The line program waits for `in_position` before releasing a carton onto the deck. That interlock is the reason
    the signal exists.
- `demo_m3` stays as it is.

## Acceptance of 4.4a

Each item below is machine-checked.

1. **The line runs in the app** (E2E, Linux and Windows):
   - cartons reach both sinks, in the ratio the program sets;
   - the I/O panel shows `divert` and `in_position`;
   - the deck is drawn raised while it diverts.
2. **No product is lost or duplicated** (line test): at any time, created = delivered to the two sinks + on the
   line. This holds while the normal sink is blocked; there, the accumulation is finite, because the source waits for
   a free entry.
3. **Routing follows the command** (line test): only cartons that reach the deck after `divert` and `in_position`
   both read TRUE go to `out_b`.
4. **Determinism:**
   - the same inputs give the same states under Step, ×1 and as fast as possible (engine test);
   - the physics fingerprint with a diverter is equal on Linux and Windows (CI).
5. **Performance:** the release soak of the line against 4.3a's demo figures, reported with the line's size.
6. **Gates:** coverage ≥ 85 %, no surviving mutant in the changed Rust files, Stryker ≥ 70 % on the new view
   helpers, and a verification file.

## What this will not prove

- **Building the line by hand.** Assembly, snapping and wiring belong to the editor, 4.5.
- **A surface's height against its neighbour's.** It is not checked: the fixture sets equal heights.
- **Products other than the 0.3 m carton.** Different shapes and attributes come in 4.4d.
- **A real PLC.** That is M5.
