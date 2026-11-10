# Spec: the Run loop's pace and the drive feedback as a native model (ROADMAP 4.4b1)

Date: 2026-11-10. Status: **approved** (user, 2026-11-10: "proceed with all the optimisations", both items, all three
signals).

## Why

4.4b's soak ran the transfer line as fast as possible at 4.1× real time, against 6.9× in 4.4a. The user asked whether
it can do better.

## Measured (2026-11-10, release, throwaway worktree on `0c7127c`)

Where a simulated millisecond goes, in µs (`Line::step_ms` timed section by section, 30 s of line after 5 s):

| Section | Transfer line | Curve line |
|---|---|---|
| Field cycle (ST) | 34.3 (54 %) | 49.5 (47 %) |
| Physics step | 23.2 (37 %) | 48.3 (46 %) |
| I/O copies between the controller and the field | 4.8 | 6.2 |
| Controller scan | 0.8 | 0.6 |
| **Total** | **63.4** | **105.0** |

- **truST's VM** (`6405915`) costs about 0.35 µs per ST statement and 0.7 µs per block call: 4 empty blocks 3.2 µs a
  cycle, 10 REAL assignments in each 17.6 µs, a TON call in each 8.6 µs. Its tier-1 specialised executor
  (`TRUST_VM_TIER1_SPECIALIZED_EXECUTOR`) changes nothing measurable.
- **Without the velocity, the encoder and the pulse in the drive block,** the field cycle falls to 21.5 µs on the
  transfer line (total 49.8, −21 %) and to 28.6 µs on the curve line (total 83.6, −20 %).
- **The Run loop** scans for 25 ms and then sleeps a whole 33 ms frame, so as fast as possible uses 43 % of the wall
  time and publishes 17 snapshots a second. Sleeping only the rest of the frame: 349 s of virtual time in 45 s on the
  transfer line (7.8×), the engine at 65 % of a core instead of 36 %, memory flat.
- A snapshot with its JSON costs 5–10 µs a frame.

What this does not measure: a machine with one or two cores, Windows, the physics step's own breakdown.

## Decisions

1. **The Run loop sleeps what is left of its frame.** A tick takes one 33 ms frame, of which as fast as possible
   scans for 25 ms: about 30 snapshots a second, as spec 2026-11-08-run-modes-design.md decided, and the 8 ms left
   for the UI that `FAST_BUDGET`'s comment already describes.
2. **`velocity`, `encoder` and `pulse` are written by the conveyor's native model,** not by its ST block.
   - They are OIP's `publishDrive` (`_shared/conveyorDrive.js`), translated into Rust: the velocity is
     speed / `speed_at_30` × 30; the encoder is the surface's travel, which the world already integrates in f64; the
     pulse is TRUE for one millisecond each `pulse_pitch` of travel, either way.
   - They read the speed the surface ran at during the millisecond just stepped, so the encoder is the physics' own
     travel (OIP: "integrated on sim time, so it cannot drift against the physics").
   - The block keeps what is logic: the ramp to `speed_ref`, `running`, and the native `surface_speed`. It no longer
     declares the three signals or `pulse_pitch`.
   - A signal a native model writes is not placed in the field's image: bound, the line writes it to the controller;
     unbound, nothing is done. This holds for a conveyor without a behaviour too.
   - *Cost:* the three signals can no longer be changed in the block's ST. This amends 4.4b's decision 5, which put
     the whole drive in ST.
3. **Left as they are:** the I/O copies (4.8–6.2 µs, strings parsed every millisecond) and the physics step.

## Acceptance

1. The 4.4b drive tests hold on the native model: `velocity` 30 at `speed_at_30`, 15 at half, 0 stopped; the encoder
   equal to the world's travel; one pulse per `pulse_pitch`. In the app, curve A reads 30 and its travel
   (`e2e/curve.mjs`).
2. The physics fingerprints are unchanged.
3. Measured in release: µs per simulated millisecond before and after, and the soak of both lines.
4. Gates as in 4.4b; E2E on Linux and Windows.
