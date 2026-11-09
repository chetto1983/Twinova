# Spec: an analytic belt with exact sensor edges (ROADMAP 3.3)

> **Superseded 2026-11-09** (user): belts are simulated with physics (Rapier), as Open Industry Project does. See spec
> `2026-11-09-component-behaviour-design.md`, "Transport with physics", and research 15. This spec is kept as history.

Date: 2026-11-08. Status: **approved** (user, 2026-11-08), decision 3 amended after measurement (below).

M1's belt (`scene.rs`) is stepped by the PLC scan: every 10 ms it moves every part by v·dt, and the photocell is
sampled after the step. So an edge is only known to within a scan: at 0.5 m/s that is 5 mm, and at 2 m/s with a 1 ms
step it is 2 mm (research note 07, OIP's fixed step). ROADMAP 3.3 asks for edges exact to the nanosecond, and for a
benchmark against the fixed step.

## The scheme (research r1 §4, JaamSim `EntityConveyor`, Apache-2.0, ideas only)

- **Lazy state.** Each part keeps its front at the time of the last update. Positions are brought up to date only
  when an event happens or someone asks, never per step.
- **One pending event per belt**, the earliest of these candidates, each computed in closed form:
  1. the next part arrival: every 2 s, once the entry is free (M1's rule);
  2. the entry becoming free, while an arrival is due;
  3. the photocell's next edge: a front reaching 2.70 m (rising), or a rear passing it (falling);
  4. the head part reaching the end: it leaves if the outlet is free, or stops there if the outlet is blocked;
  5. a moving part touching the stopped part ahead (accumulation).
- **A change of motor or outlet** brings the positions up to date, cancels the pending event and recomputes it.
- **Snapshots** read the positions at the bench's current time.

## Decisions

1. **Integer units.** Positions in nanometres (`i64`; the belt is 3 000 000 000 nm) and time in nanoseconds, the
   bench's unit.
   - Speed is in nm/s, and a distance over Δt is v·Δt / 10⁹, computed in `i128`.
   - An edge falls at the first nanosecond at which its condition holds, i.e. the time is rounded up.
   - Integer arithmetic gives the same result on every platform, and "exact to the ns" means just that.
2. **The PLC sees the photocell through its process image.** The belt sends the photocell to the controller at the
   exact edge time. The controller keeps the last value and uses it at its next scan, as a real CPU reads its inputs
   at the start of a cycle.
   - A pulse shorter than a scan can therefore be missed, as on a real PLC.
   - The UI shows the exact edge times.
   - When an edge falls exactly on a scan instant, the scan runs first, because the controller is registered first
     and same-time events follow the scheduler key (spec 2026-11-07-des-core-design.md, E). The controller therefore
     sees that edge at the following scan.
3. **Causal outputs, and a new reference.** *Amended 2026-11-08.*
   - **The bug in M1.** M1 applied the motor decided at scan k to the interval *before* that scan, which is not
     causal. Here the controller's `motor` applies from its scan on, as a CPU's output image does.
   - **The prediction was wrong.** The prediction was that events would move one scan later than M1 ("M1 + 10 ms").
     The measurement says otherwise.
   - **What the measurement shows.** In the 60 s scenarios the events fall at their closed-form times, which is
     10 ms *earlier* than M1:

     | Event | M1 | Closed form |
     |---|---|---|
     | First photocell edge | 4 820 ms | 4 810 ms |
     | First delivery | 5 420 ms | 5 410 ms |
     | Jam | 6 820 ms | 6 810 ms |

     M1's floating-point sum of 0.005 m per scan lands at 2.6999… m on the scan of the edge, so M1 saw each edge one
     scan late. The causal shift does not show in these scenarios, because the motor only changes once there, after
     the belt has stopped.
   - **What did not change.** The jam still trips 2 000 ms after the photocell is seen covered, and there are 28
     deliveries in 60 s, as in M1.
   - **The reference.** The pinned per-scan fingerprints were re-pinned on the closed-form belt. A test checks the
     closed-form times directly, as an independent reference.

## Acceptance

- **Exact edges.** Property test: for random arrival patterns and motor changes, every photocell edge time equals the
  closed-form crossing time, rounded up to the ns.
- **Closed-form times in the app.** Over the two 60 s scenarios, the photocell edge (4.81 s), the first delivery
  (5.41 s) and the jam (6.81 s) fall at their closed-form times, and every scan is pinned (decision 3).
- **Benchmark against the fixed step**, on the same scene and on 1 000 parts:
  - edge error: analytic against stepping at 10 ms and 1 ms;
  - wall time per simulated hour;
  - the number of events.
- **The usual gates**: mutation ≥ 70 % on the belt (CLAUDE.md, "conveyor kinematics"); coverage ≥ 85 %; the E2E
  unchanged and green.

## Benchmark (2026-11-08, `examples/belt_benchmark.rs`)

One simulated hour, motor on, outlet free, release build, best of 3 over two launches:

| Belt | Edges | Steps or events | Time | Edge error | Arrival drift over the hour |
|---|---|---|---|---|---|
| Closed form | 1 798 | 7 196 events | 1.2–2.0 ms | 0, exact to the ns | 0 |
| Fixed step 10 ms (M1) | 1 798 | 360 000 steps | 3.2–4.4 ms | always +10 ms | 0 |
| Fixed step 1 ms | 1 797 | 3 600 000 steps | 30–41 ms | 0 | +1 796 ms |

- **At 10 ms the error is systematic.** M1's floating-point sum lands just short of the photocell on the step of every
  edge.
- **At 1 ms the edges are sampled on time, but the arrival timer drifts.** It accumulates `+= 0.001`, so 2.0 s is
  reached after 2 001 steps: about 1 ms late per part, and one part fewer in the hour. A finer fixed step moves the
  error somewhere else rather than removing it.

## What this will not prove

- Several belts or transfers between them: one belt only. Components are M4.
- Variable speed or acceleration: the motor is on or off at 0.5 m/s, as in M1.
- That the PLC sees edges any sooner than the next scan: by decision 2 it does not.
