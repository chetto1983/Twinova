# Verification: event core on nexosim (2026-11-08)

Plan: `docs/superpowers/plans/2026-11-08-des-core-integration.md` (ROADMAP 3.2). Spec:
`docs/superpowers/specs/2026-11-07-des-core-design.md`. PR #7.

The app's engine now runs on a nexosim bench on one thread:
- nexosim comes from the user's fork `chetto1983/nexosim` at `480e7247`, with the FIFO executor patch;
- the controller and the belt are models;
- the truST harness and the scene live in `Model::Env`;
- UI inputs reach the bench as events.

## Acceptance

| Criterion | Result |
|---|---|
| The tests still green | 30 Rust tests pass: the 27 of M1, adapted only by `.unwrap()` where the API now returns `Result`, plus the 3 below |
| M1's behaviour, scan by scan | the fingerprints of both 60 s scenarios, pinned on the old engine before the change (task 2), are reproduced exactly |
| Replay | the same scripted inputs (Start, block, Stop, free, Start over 30 s) give the same states twice |
| Order of simultaneous events | property test: random models and times; the order is time, then registration. It fails without the patch: `[(3, 1), (3, 0)]` |
| E2E on the real app | CI run 42459841117 on `da4a075`: green on `ubuntu-24.04` and `windows-2022`; all 7 jobs green |
| Coverage | `cargo llvm-cov`: 93.75 % of lines (bench.rs 97.3 %, engine.rs 100 %) |
| Mutation ≥ 70 % on the engine | `cargo mutants` on `scene.rs`, `engine.rs`, `bench.rs`: 79 caught, 1 timeout, 4 unviable, 0 missed (80 of 80 viable) |

## Found and fixed

- **A non-`Send` future.** truST's `Plc` is not `Sync`: it holds a `Cell<Option<Instant>>`, checked with a
  compile-time assertion. So a reference to it held across an `.await` made the method an invalid nexosim input. The
  controller's state is now built before awaiting.
- **Two surviving mutants** in a helper that turned a nexosim error into text. No test can provoke that error. The
  helper duplicated a closure already in use, so it was removed.
- **A runtime fault of the ST program** used to be returned by `Plc::scan`. In a model it would have panicked the
  executor. The controller now reports it and stops scanning, and `Engine::step` returns it, as before. The command
  test of a fault that stops Run still passes.

## What this does not prove

- That the upstream PR is accepted: it is not open yet. The branch is on the fork; opening it needs the user, because
  this session cannot reach `asynchronics/nexosim`.
- The PLC state in snapshots: `Model::Env` is not saved by nexosim (3.5).
- A local E2E run: the E2E ran in CI only, on both platforms.
- Run pacing under load: `catch_up` keeps M1's per-scan loop, and no overload was provoked.
