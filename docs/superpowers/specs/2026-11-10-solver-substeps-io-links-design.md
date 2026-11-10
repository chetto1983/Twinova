# Spec: one solver iteration per millisecond, and I/O addresses resolved at Build (ROADMAP 4.4b2)

Date: 2026-11-10. Status: **approved** (user, 2026-11-10: one solver iteration, after the measurements below; the I/O
copies with "all the optimisations").

## Why

After 4.4b1 a simulated millisecond costs 50 µs on the transfer line and 79 µs on the curve line, of which the physics
step is 47–60 % and the I/O copies 5–6 µs. The user pointed at both.

## Measured (2026-11-10, release, throwaway worktree on `905dc3d`)

**The physics step**, with Rapier's own counters (its `profiler` feature), 30 s of line after 20 s, µs per step:

| | Transfer line | Curve line |
|---|---|---|
| Cartons, contact pairs, contacts (mean) | 6.7, 8.6, 29.4 | 9.9, 33.6, 75.7 |
| Collision detection (narrow phase) | 4.7 (4.3) | 13.5 (13.0) |
| Solver (velocity resolution) | 18.6 (17.7) | 37.8 (36.5) |
| CCD, islands, the rest | < 1 | < 1 |
| **Rapier's step** | **24.3** | **52.5** |

- **The solver is 70–75 % of the step.** `num_solver_iterations` is 4, Rapier's default, chosen for steps of 16 ms;
  Twinova steps every millisecond.

| `num_solver_iterations` | Transfer line, µs | Curve line, µs | Tests |
|---|---|---|---|
| 4 (today) | 25.4 | 54.9 | all pass |
| 2 | 19.6 | 43.3 | the three fingerprints, and `a_carton_passes_the_seam_without_lifting`: held on a stopped belt 2.6 mm past its band |
| **1** | **15.3 (−40 %)** | **34.0 (−38 %)** | the three fingerprints only; the app's 86 tests pass |

- **Fidelity at 1 against 4:**

| Figure | 4 iterations | 1 iteration | Bound |
|---|---|---|---|
| Belt curve, left: off the centreline, yaw | 0.84 mm, 1.41° | 0.73 mm, 1.04° | 5 mm, 3° |
| Belt curve, right | −0.25 mm, −1.08° | −0.38 mm, −1.00° | 5 mm, 3° |
| Roller curve | 1.30 mm, 2.78° (left) | −1.60 mm, −2.91° (right) | 5 mm, 3° |
| Guarded belt curve at 1 m/s | 1.86 mm, 1.93° | 0.76 mm, 1.00° | 5 mm, 3° |
| Carry speed at 0.5 m/s | 0.49998 | 0.50000 | ± 0.005 |
| Lift at a seam onto a stopped belt | 0.075 mm | 0.091 mm | 0.5 mm |
| Front of a carton held on that seam | 3.192 m | 3.195 m | 3.0–3.2 m |

- **The guards cost nothing measurable** (the curve line's step is 34.1 µs with them and 33.7 µs without). A curve's
  narrow phase is its chords: a carton lies on two or three of them (27.5 deck pairs for 9.9 cartons), as OIP builds a
  curve.
- **The I/O copies** parse their address strings every millisecond: the natives' writes 1.7–2.2 µs, the commands to
  the field 1.5–2.0 µs (`mirror` parses, formats and parses again), the feedback back 1.5–1.7 µs.

What this does not measure: the roller curve's left and right cases at both settings (the probe's output was
interleaved, one of each was read), products other than the carton, stacked or piled cartons, a line more crowded
than a dozen cartons.

## Decisions

1. **The world solves with one iteration per step** (`num_solver_iterations = 1`). The step is already a millisecond;
   the 4.3–4.4b acceptance tests pass unchanged. Every fingerprint is pinned again, a changed number by design, as
   with `Coulomb` in 4.4b, and every run's numbers change slightly.
2. **Addresses are resolved at Build.** The line keeps, parsed, the controller address of each native input the
   program declares, and of each command and feedback it copies; the field keeps the mirror of each of its commands
   and feedbacks. The copies' results are the same to the bit.
3. **Left as they are:** a curve's chords (OIP's construction), and truST's VM.

## Acceptance

1. The 4.3–4.4b1 tests pass unchanged, but for the three fingerprints, pinned again and equal in dev, in release and
   on windows-2022.
2. Measured in release: µs per simulated millisecond before and after each decision, and the soak of both lines.
3. Gates as in 4.4b; E2E on Linux and Windows.
