# Verification: the Run loop's pace and the drive feedback as a native model (ROADMAP 4.4b1, 2026-11-10)

Spec: `2026-11-10-run-pace-native-feedback-design.md` (approved). Plan: `2026-11-10-run-pace-native-feedback.md`.
Machine: the cloud container with 4 cores, Xvfb with Mesa llvmpipe, as in 4.4b. CI: PR #33.

## What changed

- **The Run loop** sleeps what is left of its 33 ms frame, not a whole frame after its work.
- **A conveyor's `velocity`, `encoder` and `pulse`** are its native model's (`drive.rs`, OIP's `publishDrive`
  translated into Rust), written to the controller after each physics step. The encoder is the world's own travel of
  the surface. The ST block keeps the ramp, `running` and `surface_speed`.

## The acceptance

| # | Criterion | Evidence | Result |
|---|---|---|---|
| 1 | The 4.4b drive figures hold on the native model | `drive_tests.rs`: 30 at `speed_at_30`, 15 at half, 0 stopped, signed in reverse; one pulse per inch over a ramp and 10 s, either way. `line_drive_tests.rs`: the encoder equals the world's travel to the bit; the pulse counted by the controller is one per inch; the velocity reads 15 half way up a 1 s ramp; a conveyor without a behaviour reports too. `e2e/curve.mjs`: curve A reads 30 and its travel in the app | ✅ |
| 2 | The physics fingerprints are unchanged | `0x106e94227f30a7d7`, `0x88a2f689560c76e4`, `0xfb7e9104c1a6c781`, as in 4.4b | ✅ |
| 3 | Measured in release, before and after | below | ✅ |
| 4 | Gates; E2E on Linux and Windows | below; Windows on CI (PR #33) | ✅ |

## Measured in release

**A simulated millisecond, in µs** (`Line::step_ms` timed section by section in a throwaway worktree, 30 s of line
after 5 s, two rounds):

| Section | Transfer line: before → after | Curve line: before → after |
|---|---|---|
| Field cycle (ST) | 34.3 → 20.5–20.8 | 49.5 → 24.8–25.3 |
| Physics step | 23.2 → 23.5–23.8 | 48.3 → 46.5–48.0 |
| I/O copies and native writes | 4.8 → 4.8 | 6.2 → 5.6–5.9 |
| Controller scan | 0.8 → 0.8 | 0.6 → 0.5 |
| **Total** | **63.4 → 49.8–50.6 (−21 %)** | **105.0 → 77.7–80.0 (−25 %)** |

**Soak of the release build** (`xvfb-run node e2e/soak.mjs 45 <fixture>`), the engine process:

| Phase | Transfer line: 4.4b → now | Curve line: 4.4b → now |
|---|---|---|
| Idle | 0.0 %, 176 MB → 0.0 %, 170 MB | 0.0 %, 176 MB → 0.0 %, 174 MB |
| Run ×1 | 11.6 % → 10.9 % | 16.0 % → 13.6 % |
| As fast as possible | 35.8 % → 73.6 % | 35.5 % → 73.6 % |
| Paused | 0.0 % → 0.1 % | 0.0 % → 0.0 % |
| **Virtual time in 45 s as fast as possible** | **184 s → 488 s (4.1× → 10.9×)** | **123 s → 359 s (2.7× → 8.0×)** |

- **Memory is flat** in every phase: 171 MB and 175 MB.
- **As fast as possible now uses three quarters of a core** (25 ms of scans in a 33 ms frame), where it used a third.
  That is the cost of the pace.
- **The frame between snapshots** is 33 ms, against 58 ms before (`run_as_fast_as_possible_publishes_a_snapshot_a_frame`,
  which fails on the old sleep).
- The web process's CPU ranges from 213 % to 281 % across these runs and 4.4b's, under llvmpipe's software rendering.
  It is not attributed.

## Gates

- **Rust:** fmt, clippy `-D warnings`, `cargo test --workspace` (app 86, physics 55, project 48).
- **cargo mutants** on the changed code, no survivor:
  - `drive.rs`, the field's `block_signals` and placement, and the line's step: 53 mutants, 46 caught, 4 unviable,
    and 3 timeouts, which are the mutants that stop the line's time;
  - the world's conveyor speed: 6, 4 caught and 2 unviable.
- **Frontend:** unchanged.
- **E2E, Linux local:** `run.mjs` 11/11, `project.mjs` 42/42, `line.mjs` 4/4, `curve.mjs` 3/3.

## Tests moved or changed

- **Moved with the same figures** (the approved change): the two 4.4b field tests that read the velocity, the encoder
  and the pulse from the ST block are now `drive_tests.rs` and `line_drive_tests.rs`.
- **Mechanical:** the two tests that locate a broken line in the belt's block expect line 30; the demo's generated
  `VAR_CONFIG` and the placement test list only the block's signals; a private output address is now tested on the
  chain transfer.

## What this does not prove

- **A machine with one or two cores,** where three quarters of a core for the engine leaves less for the view.
- **Windows' timing:** CI runs the tests and the E2E there, not the soak.
- **A PLC program that counts every pulse:** a pulse lasts one millisecond and the controller scans every 10 ms, as in
  4.4b and in OIP.
- **The remaining costs,** not touched: the physics step (47–60 % of a millisecond now), and the I/O copies, whose
  addresses are parsed every millisecond (5–6 µs).
