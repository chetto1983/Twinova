# Spec: the discrete-event core, nexosim against our own queue (ROADMAP 3.1)

Date: 2026-11-07. Status: **decided: nexosim, on one thread, with its single-threaded executor made FIFO**
(user, 2026-11-07); the patch is proposed upstream.

ROADMAP 3.1 asks whether Twinova's event core should be nexosim 1.0 or a scheduler of our own. The answer is measured
against the requirements of research note 05 §6 and CLAUDE.md:
- integer time;
- a `(t, priority, seq)` order;
- same-instant iteration with a cap;
- a wall clock that reports `OutOfSync`;
- save and restore;
- determinism, with simultaneous-event ordering that is specified and tested.

Twinova is a Tauri desktop app and its engine runs natively, so wasm is not a requirement (user, 2026-11-07). Note 05
listed it for a browser edition that has been dropped.

Prototype: `prototypes/des-probe` (nexosim `=1.0.0`, Rust 1.99.0, 4-core cloud container).

## What nexosim 1.0 is (read from the installed source)

- **Licence**: MIT OR Apache-2.0. Rust 1.88 or later.
- **Model**: an actor model. Models are `#[Model]` structs with typed `Output` ports, connected through mailboxes,
  and they self-schedule with `Context::schedule_event`. A custom executor runs them on `SimInit::with_num_threads(n)`
  threads; `SimInit::new()` takes `num_cpus::get()` (`sim_init.rs:58`).
- **Time**: `MonotonicTime` is `tai_time::TaiTime`, whole seconds in an `i64` plus nanoseconds in a `u32`. That is
  integer, nanosecond time.
- **Order**: the documentation promises *causal messaging* only (`lib.rs`, "Message ordering guarantees"):
  - A→B messages keep their order;
  - it is transitive;
  - same-time events from the global scheduler keep their relative order.

  Two messages sent to B by different models in the same time slice "may be processed in any order". There is no
  priority.
- **Wall clock**: `SimInit::with_clock(clock, ticker)` and `with_clock_tolerance(d)`. A lag beyond the tolerance is
  `ExecutionError::OutOfSync(lag)`.
- **Snapshots**: `Simulation::save(writer)` and `SimInit::restore(reader)`, with models serialized through serde.
- **Same-instant loops**: no cap. `with_timeout` bounds a step in wall time.

## Measurements (prototype, 2026-11-07)

### A. Order of simultaneous events

Eight emitter models tick together every 1 ms for 1 s, each sending its id to one collector. 20 runs per case. The
"skewed work" case gives emitter 0 the most busy work and emitter 7 none, so that threads finish in different orders.

| Case | Distinct orders in 20 runs | Order at each instant |
|---|---|---|
| nexosim, 4 threads, no work | 1 | 0, 1, …, 7 |
| nexosim, 4 threads, skewed work | 1 | 0, 1, …, 7 |
| nexosim, 1 thread, no work | 1 | **7, 6, …, 0** |
| nexosim, 1 thread, skewed work | 1 | **7, 6, …, 0** |
| nexosim, 2, 3 or 8 threads | — | 0, 1, …, 7 |
| own queue | 1 | 0, 1, …, 7 (by construction) |

For a fixed thread count the order was stable, but **it depends on the thread count**: one thread reverses it.
`SimInit::new()` takes the number of logical CPUs, so the same scene can give a different sequence on a 1-core
virtual machine and on a multi-core PC. This is allowed by nexosim's contract, which leaves the order unspecified. Pinning one thread everywhere would
make it consistent, but it would then rest on behaviour the documentation does not promise.

### B. Throughput

1 000 periodic emitters at 1 ms, 1 s simulated, which is 1 000 000 events with no messages between models. Five
launches, each the best of 3:

| Engine | Time | Events/s |
|---|---|---|
| nexosim, 1 thread | 388–559 ms | 1.8–2.6 M |
| nexosim, 4 threads | 736–921 ms | 1.1–1.4 M |
| own queue | 48–74 ms | 13.5–21.0 M |

The own queue does much less work: no async dispatch, no mailboxes, no ports. It is an upper bound for a plain
scheduler, not a like-for-like model. Both engines are far above what the M1 scene needs: 100 PLC scans per second
and a few edges per part.

### C. wasm32 (not a requirement)

Measured before wasm was dropped: `cargo build --release --target wasm32-unknown-unknown` builds with no change in
7.8 s, and the probe is 455 KB. It weighs nothing in the decision.

### D. Save and restore

Run to 0.5 s, `save`, rebuild the bench, `restore`, run to 1 s. Compared with one uninterrupted run, with 1 and with
4 threads: the same 8 000 events in the same order, from a 210-byte state.

### E. The executor patch (decided after A–D)

Where the order is decided, read in nexosim's `main` (`0579f95`, 2026-10-22):
- **The scheduler queue is already totally ordered.** Its key is `(MonotonicTime, origin model id)`
  (`simulation/scheduler.rs:249`), and equal keys come out FIFO (`util/priority_queue.rs`). The model id is the
  registration index (`simulation.rs:1307`); the global scheduler's id is `usize::MAX - 1` (`simulation.rs:168`), so
  its events come last.
- **The executor then reverses it.** `handle_scheduler_step` spawns one task per origin, in key order
  (`simulation.rs:1461`). The single-threaded executor keeps its work queue in a `Vec` and takes the last task first,
  with `pop()` (`executor/st_executor.rs`), so it runs them LIFO.
- **What the maintainers have said** (issue #34, PRs #62 and #122): full determinism needs a single thread; in
  multi-threaded execution "the relative order can never be guaranteed whatever we do". The order by origin was added
  for determinism. Nobody had raised the LIFO work queue.

The patch makes the work queue a `VecDeque` used FIFO. It adds a test and documents the single-threaded order: 4 files,
+74/−6. It is in `prototypes/des-probe/nexosim-fifo-st-executor.patch`, signed by the user.

| Check | Result |
|---|---|
| nexosim's own suite, `cargo test --all-features` | 220 passed, 4 ignored, before and after |
| New test, five models with same-time events on one thread | passes; with the old LIFO queue it fails with `[4, 3, 2, 1, 0]` |
| `cargo fmt --check`, clippy on the touched files, wasm build (their CI) | clean |
| Probe A with the patch | 0, 1, …, 7 with 1, 2, 3, 4 and 8 threads; 1 distinct order in 20 runs, also with skewed work |
| Probe D with the patch | save and restore still exact |
| Throughput, 1 thread, no messages (B) | 476–491 ms before, 462–526 ms after: within noise |
| Throughput, 1 thread, each event sends a message to one collector | 848–913 ms before, **963–1 050 ms after: about 10–15 % slower** |
| Throughput, 4 threads, with messages | 1 214–1 345 ms either way (the multi-threaded executor is untouched) |

Not run: Miri, which is in their CI, because no nightly toolchain is installed here.

## Requirements against the two candidates

| Requirement (05 §6, CLAUDE.md) | nexosim 1.0 | Own queue |
|---|---|---|
| Integer ns time | yes | yes (`u64` ns) |
| Specified `(t, prio, seq)` order for simultaneous events | causal order only, no priority; it depended on the thread count. **With the patch on one thread**: by scheduler key, tested and documented (E) | yes, by construction |
| Same order on every machine | only by pinning 1 thread (an undocumented order) | yes |
| Same-instant iteration with a cap and an error | no cap; timeout on native only | to write (a counter) |
| Wall clock with `OutOfSync` | yes | to write; M1's `catch_up` already reports the lag |
| Save and restore | yes, exact (D) | to write (serde of the state) |
| Throughput for Twinova's scenes | ample | ample |
| Code to own | none | a queue of about 40 lines plus clock, snapshots and cap |
| Extra | gRPC server, port and model architecture | — |

## Decision

**nexosim, run on one thread, with the FIFO executor patch** (user, 2026-11-07). The user rejected our own queue as a
poor choice in the long run, and chose to act on nexosim's kernel instead.
- The order of simultaneous events, which the PLC lockstep needs, then follows the scheduler key: time, then origin
  model in registration order, then the global scheduler. The new test pins it.
- The patch goes upstream as a pull request. Until it is merged, Twinova depends on the patched commit
  (`[patch.crates-io]` to a fork). If the maintainers refuse it, that fork is ours to keep in step.
- The price: about 10–15 % throughput on one thread when models exchange many messages. Twinova's scenes need far
  less than either rate.

## What this does not prove

- That the maintainers accept the patch, or that the order stays as it is in later nexosim releases without it.
- That FIFO is never slower on other workloads: only the two throughput cases above were run.
- The M1 scene itself: ROADMAP 3.1 asks for the spike "on the M1 scene". These measurements use synthetic
  periodic emitters, which isolate the core properties (order, throughput, snapshots), but no conveyor ran.
- That nexosim's order is fixed for a given thread count on other machines: 20 runs on one machine are not a proof.
- Throughput with real models: the probe has no messages in B and no conveyor kinematics.
- The cost of writing the clock, the cap and the snapshots ourselves: estimated, not measured.
