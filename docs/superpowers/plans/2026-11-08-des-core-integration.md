# Plan: event core on nexosim (ROADMAP 3.2)

## Goal

Replace the app's hand-written lockstep (`engine.rs`: inputs, PLC time, one cycle, motor, scene, every 10 ms) with a
nexosim bench on one thread, without changing what the user sees: the same scene, the same program, the same run modes
(Run, Pause, Step, Reset) and the same snapshots.

## Architecture

From the 3.1 prototype (`prototypes/des-probe/src/m1.rs`), which matches M1's engine scan by scan over 60 s:
- `PlcModel`: `Env` = truST's `Plc`, built by a `ProtoModel`. It scans every 10 ms (self-scheduled) and sends `motor`.
  Inputs: `sensor`, and `press(start, stop)`, which is held for the next scan, as `Engine::press` does today.
- `SceneModel`: `Env` = `Scene`. Input `motor` steps the belt by one scan and sends `sensor`. Input `outlet_blocked`
  comes from the UI.
- The UI talks to the bench only through registered endpoints:
  - Start, Stop and the outlet toggle are `EventSource`s, sent with `Simulation::process_event`;
  - the snapshot is a `QuerySource` answered by both models and merged, read with `process_query`.
- Run paces the bench with `step_until(origin + wall time)`, capped per tick as `MAX_SCANS_PER_TICK` does today, and
  reports the lag. `OutOfSync` and the clock are 3.5.
- One thread (`SimInit::with_num_threads(1)`), with the FIFO executor patch, so that same-time events follow the
  scheduler key: time, then origin model in registration order, then the global scheduler (spec 3.1, section E).

## Tech Stack

nexosim 1.0 with the FIFO executor patch, from the user's fork until the upstream PR is merged
(`[patch.crates-io]`). serde for the models. truST at the pin, unchanged.

## Spec

`docs/superpowers/specs/2026-11-07-des-core-design.md` (decision and measurements). The tauri-shell spec still defines
the scene, the contract variables and the run modes.

## Global Constraints

- CLAUDE.md:
  - determinism is tested (a replay fingerprint);
  - simultaneous-event ordering is specified and tested, with a property test;
  - coverage ≥ 85 %; mutation ≥ 70 % on the engine;
  - files ≤ 600 lines.
- No change visible to the user: the 27 Rust tests and the E2E stay green, unchanged.
- `Model::Env` is not saved by nexosim: snapshots of the PLC state stay out of scope (3.5).

## Review Focus

- The order of one step must stay M1's:
  1. inputs;
  2. PLC cycle;
  3. motor;
  4. scene;
  5. sensor.

  A one-scan offset is only visible with the outlet blocked (3.1, negative control).
- Error paths: a truST runtime fault inside a model must still stop Run and reach the UI as `engine-error`.

## Decisions taken while writing this plan

- The prototype's per-scan fingerprint comparison becomes the regression test: the M1 engine's 60 s results are pinned
  as constants before it is removed, so the test survives the removal.
- A runtime fault cannot `expect` inside a model: it panics the executor. The model records the error and the bench
  surfaces it on the next query, or nexosim's `ExecutionError::Panic` is mapped to the same message. Which one is chosen
  by reading nexosim's panic path, in task 2.
- **Blocker**: the dependency on the patched nexosim needs the user's fork (`chetto1983/nexosim`) for CI to fetch it.
  Task 1 waits for it.

## File structure

```
app/src-tauri/Cargo.toml          nexosim (patched, from the fork), serde
app/src-tauri/src/bench.rs        the two models and their prototypes
app/src-tauri/src/engine.rs       Engine on top of the bench: same public API
app/src-tauri/src/engine_tests.rs replay fingerprint, ordering property test, pinned M1 results
app/src-tauri/.cargo/mutants.toml engine.rs and bench.rs in scope
```

## Tasks

- [ ] **1. Dependency.** The fork has the patch; `[patch.crates-io] nexosim = { git = <fork>, rev = <commit> }`.
  Verify: `cargo tree -i nexosim` shows the fork; the 27 tests still pass.
- [x] **2. Pin M1's results.** Before touching the engine, record the per-scan fingerprints of the two 60 s scenarios
  as a test against the current engine. Verify: green on the current engine.
- [ ] **3. Bench.** `bench.rs` with the models, ported from the prototype, with error handling instead of `expect`.
  Verify: the pinned results from task 2 are reproduced; a runtime-fault test reaches the error.
- [ ] **4. Engine on the bench.** `engine.rs` keeps its public API (`new`, `press`, `set_outlet_blocked`, `step`,
  `catch_up`, `snapshot`, `time_ms`); `commands.rs` unchanged. Verify: 27 tests, `cargo clippy -D warnings`, E2E 6/6.
- [ ] **5. Determinism tests.** Replay fingerprint (two runs, the same inputs, identical snapshots), and a property test
  of the order of simultaneous events with random model counts and times. Verify: green; `cargo mutants` ≥ 70 % on
  `engine.rs` and `bench.rs`; `cargo llvm-cov` ≥ 85 %.
- [ ] **6. Docs.** Verification file, ROADMAP 3.2, CHANGELOG (no user-visible change: an "Internal" note only if the
  format has one).
