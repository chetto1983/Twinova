# Verification: one solver iteration per millisecond, and I/O addresses resolved at Build (ROADMAP 4.4b2, 2026-11-10)

Spec: `2026-11-10-solver-substeps-io-links-design.md` (approved). Plan: `2026-11-10-solver-substeps-io-links.md`.
Machine: the cloud container with 4 cores, Xvfb with Mesa llvmpipe, as in 4.4b1. CI: PR #34.

## What changed

- **The physics solves with one iteration per step** (`num_solver_iterations = 1`; Rapier's default is 4).
- **The line copies I/O through addresses parsed at Build:** the controller's `Port`, the field's mirrors.

## The acceptance

| # | Criterion | Evidence | Result |
|---|---|---|---|
| 1 | The 4.3–4.4b1 tests pass unchanged, but for the three fingerprints | `cargo test --workspace`: app 88, physics 55, project 48. No test edited but the three pinned values | ✅ |
| 1 | The fingerprints equal in dev, in release and on windows-2022 | two belts `0x2674dda1319b763a`, transfer line `0x2c224abfa5e90d6c`, curve line `0xaba061e54b1dfbe2`: equal in dev and release locally; windows-2022 on CI (PR #34) | ✅ |
| 2 | Measured in release, before and after | below | ✅ |
| 3 | Gates; E2E on Linux and Windows | below; Windows on CI | ✅ |

## Measured in release

**A simulated millisecond, in µs** (`Line::step_ms` over 30 s of line after 5 s, three rounds):

| | Transfer line | Curve line |
|---|---|---|
| 4.4b1 | 49.8–50.6 | 77.7–80.0 |
| One solver iteration (Task 1) | 38.0–39.4 | 58.6–61.0 |
| Addresses resolved at Build (Task 2) | **34.7–36.9** | **54.3–58.1** |
| Since 4.4b1 | −28 % | −29 % |

The physics step alone, with Rapier's counters: 25.4 → 15.3 µs and 54.9 → 34.0 µs (the spec has the breakdown and the
fidelity figures at 4, 2 and 1 iterations).

**Soak of the release build** (`xvfb-run node e2e/soak.mjs 45 <fixture>`), the engine process:

| Phase | Transfer line: 4.4b1 → now | Curve line: 4.4b1 → now |
|---|---|---|
| Idle | 0.0 %, 170 MB → 0.0 %, 170 MB | 0.0 %, 174 MB → 0.0 %, 176 MB |
| Run ×1 | 10.9 % → 8.8 % | 13.6 % → 10.7 % |
| As fast as possible | 73.6 % → 73.8 % | 73.6 % → 73.0 % |
| Paused | 0.1 % → 0.0 % | 0.0 % → 0.0 % |
| **Virtual time in 45 s as fast as possible** | **488 s → 741 s (10.9× → 16.5×)** | **359 s → 510 s (8.0× → 11.3×)** |

- **Memory is flat** in every phase: 171 MB and 177 MB.
- Since 4.4b, as fast as possible went from 4.1× to 16.5× on the transfer line and from 2.7× to 11.3× on the curve
  line.

## Gates

- **Rust:** fmt, clippy `-D warnings`, `cargo test --workspace` (app 88, physics 55, project 48), in dev; the physics
  also in release.
- **cargo mutants** on Task 2's functions (`Port`, the controller's reads and writes, the field's mirrors, the line's
  build and step): 28 mutants, 16 caught, 9 unviable, and the 3 timeouts that stop the line's time. No survivor.
  Task 1 is one constant, held by the three fingerprints.
- **Frontend:** unchanged.
- **E2E, Linux local:** `run.mjs` 11/11, `project.mjs` 42/42, `line.mjs` 4/4, `curve.mjs` 3/3.

## Found by measuring

- **Two solver iterations are worse than one or four on one test:** a carton entering a stopped belt is held 2.6 mm
  past its band. One iteration holds it at 3.195 m, four at 3.192 m (band 3.0–3.2 m).
- **The guards cost nothing measurable;** a curve's collision detection is its chords, two or three under each
  carton, as OIP builds it.
- **The field wrote a command at any address it was given,** into an unused corner of its image. It now refuses an
  address that is not one of its commands or feedbacks.

## What this does not prove

- **Products other than the 0.3 m carton, stacked or piled cartons, a line more crowded than a dozen cartons:** one
  solver iteration is validated on the 4.3–4.4b acceptance scenes only.
- **The roller curve's margin:** its yaw out of the curve was read at 2.78° with four iterations and 2.91° with one,
  against a bound of 3°, on one side each.
- **Windows' timing:** CI runs the tests and the E2E there, not the soak.
- **What is left:** the field's ST (truST's VM, about 0.35 µs a statement) and the physics are now about equal
  shares of a millisecond; neither is touched further here.
