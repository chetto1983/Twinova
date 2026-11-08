# Verification: closed-form belt and scan period (2026-11-08)

Specs: `2026-11-08-analytic-belt-design.md` (ROADMAP 3.3) and `2026-11-08-scan-period-design.md` (ROADMAP 3.4). PR #8,
CI run 42463081110 on `4b2013b`: all seven jobs green, the E2E of the real app on `ubuntu-24.04` and `windows-2022`
included.

## 3.3: closed-form belt

| Criterion | Result |
|---|---|
| Edges exact to the ns | property test: the photocell only changes at handled instants, and one ns earlier it had the other value |
| Closed-form times | the first edge at 4.81 s, deliveries at 5.41 s + 2 s·n, a blocked belt full nose to tail; in the app, the photocell at 4 810 ms, the first delivery at 5 410 ms, the jam at 6 810 ms |
| Against M1 | events 10 ms earlier, explained by M1's floating-point sum, which saw each edge one scan late; the jam still 2 s after the photocell is seen; 28 deliveries in 60 s |
| Benchmark, one simulated hour | closed form 7 196 events in 1.2–2.0 ms, against 360 000 steps in 3.2–4.4 ms at 10 ms (edges always 10 ms late) and 3 600 000 steps in 30–41 ms at 1 ms (arrivals drift 1.8 s) |
| Coverage | CI gate at 85 % passed (`rust` job) |

## 3.4: scan period

| Criterion | Result |
|---|---|
| Same results as M1 at 10 ms | the default program's pinned per-scan fingerprints are unchanged |
| Configurable | `T#5ms` gives 5 ms, `T#4ms` with `T#6ms` gives 2 ms, `SINGLE` is refused, no configuration gives 10 ms; a 5 ms program steps 5 ms at a time |
| truST behaviour relied on | measured on the pin: `tasks()` lists the intervals, the contract names resolve under a configuration, a cycle runs only the due tasks |

## What this does not prove

- **Mutation on `belt.rs`**: not run locally (user's request). `mutation.yml` measures it on `main` after the merge.
- **Several belts, variable speed, `SINGLE` tasks**: none of these is implemented yet.
- **The scan period in the UI**: it is not shown yet.
