# Twinova roadmap

Each milestone below is broken into phases. A phase closes only with its acceptance met and a dated verification file in
`docs/superpowers/verification/` (CLAUDE.md, "Definition of Done"). The order follows dependencies: no PLC link before
the engine is deterministic, no component library before the component model exists.

References in brackets point to the research in `docs/research/` (in Italian): `06` is the best-of matrix, `05` the
engine study, `08` the Rust inventory, `09`/`10` the IEC 61131-3 runtime work.

Status: ✅ done · 🔶 partly done (gaps listed) · ⬜ not started.

---

## M0 — Research ✅

| Phase | Deliverable | Status |
|---|---|---|
| 0.1 | Feature map of Visual Components 5.1, 583 rows from the public help [01–04] | ✅ |
| 0.2 | How simulation engines are built, and how they sync with a PLC [05] | ✅ |
| 0.3 | Benchmark of 13 commercial simulators and a best-of matrix [06, benchmark/] | ✅ |
| 0.4 | Open Industry Project assessed, with measured tests and benchmark [07] | ✅ |
| 0.5 | Rust ecosystem and IEC 61131-3 runtime inventory, plus the ST-without-compilation probe [08–10] | ✅ |

## M1 — Walking skeleton 🔶

One belt conveyor in 3D, driven by Structured Text, in a Tauri desktop app.
Spec `2026-11-07-tauri-shell-design.md`.

| Phase | Deliverable | Acceptance | Status |
|---|---|---|---|
| 1.1 | Tauri 2 + Svelte 5 shell, Aura tokens, en/it | `svelte-check` clean, app starts | ✅ |
| 1.2 | Closed-form belt, ST on truST's VM, lockstep engine | 18 Rust tests incl. the 60 s scenarios | ✅ |
| 1.3 | E2E under Xvfb | screenshots; pacing 10.04 s virtual in 10.06 s wall | ✅ |
| 1.4 | **DoD gaps**: coverage ≥85 %, mutation ≥70 % on `scene`/`engine`, Windows run | figures in a verification file | 🔶 coverage 93.6 % Rust / 100 % TS, mutation 62/62; Windows compiles, tests do not start yet |

## M2 — Engineering foundation 🔶

The gate set of chetto1983/ModbusTool, ported in `2026-11-07-engineering-foundation` (plan and verification).

The tooling that every later phase relies on.

| Phase | Deliverable | Acceptance | Status |
|---|---|---|---|
| 2.1 | CI on GitHub Actions, Linux and Windows: fmt, clippy `-D warnings`, tests, svelte-check, Vitest, build | green on `main` | 🔶 Linux green; Windows test binary needs the app manifest |
| 2.2 | Coverage (`cargo llvm-cov`, `vitest --coverage`) and mutation (`cargo mutants`) gates | thresholds enforced in CI | 🔶 coverage enforced; `mutation.yml` runs on `main` only, not run yet |
| 2.3 | 600-LOC check, oxlint and prettier, dead-code and duplication checks (the ModbusTool set) | zero warnings | ✅ |
| 2.4 | Release bundles: MSI/NSIS, AppImage/deb, dmg, plus `CHANGELOG.md` | installers built by CI, sizes recorded | ⬜ |
| 2.5 | **Runtime footprint**: replace trust-runtime `full` with only the encoder and the VM (path A′ in [10]) | binary size before/after measured; ST behaviour unchanged (same tests) | ⬜ |

## M3 — Deterministic engine core ⬜

The hybrid engine of [05 §6]: discrete events with analytic motion, and fixed steps only where they are needed.

| Phase | Deliverable | Acceptance |
|---|---|---|
| 3.1 | Spike: nexosim 1.0 versus our own scheduler, on the M1 scene [08] | decision recorded with measurements |
| 3.2 | Event core: integer ns time, `(t, priority, seq)` queue, same-instant iteration with a cap | property tests for ordering; replay fingerprint test |
| 3.3 | Analytic conveyor segments with exact sensor edges (JaamSim scheme [05, r1 §4]) | edges exact to the ns; benchmark against the fixed step (OIP baseline [07]) |
| 3.4 | PLC scan as a periodic event, with the scan time per program | same results as M1 at 10 ms; scan time configurable |
| 3.5 | Run modes: Run (wall clock × scale, `OutOfSync` beyond a tolerance), as fast as possible, Step, Reset, snapshots | the lag is reported, never hidden; save and restore a state |

## M4 — Component model and scene editor ⬜

The core of the product [02a, 06 #1–2].

| Phase | Deliverable | Acceptance |
|---|---|---|
| 4.1 | Spec of the component model: node tree, joints, properties, typed signals, behaviours, interfaces | spec approved |
| 4.2 | Open scene and component format: JSON plus glTF geometry, versioned, with migrations | round-trip tests; a corpus of saved scenes opens |
| 4.3 | Behaviours written as ST function blocks bound to component signals | a component's logic edited and rebuilt in the app |
| 4.4 | Library v1: belt, roller and curved conveyors, diverter, stopper, pusher, turntable, sensors (diffuse, through-beam, colour, barcode), creator and sink | each component has an E2E scene |
| 4.5 | Editor: catalogue, drag and drop, snapping by interfaces, move/rotate, signal wiring, undo | build a two-conveyor line with a diverter by hand |

## M5 — PLC in the loop ⬜

The gap the commercial tools leave [06 §3, 03].

| Phase | Deliverable | Acceptance |
|---|---|---|
| 5.1 | I/O mapping UI: drag and drop tags, import tag tables (TIA, PLCopen), export the mapping (the Factory I/O idea [06 #11]) | mapping saved with the scene |
| 5.2 | **PLCSIM Advanced lockstep** (`TimespanSynchronized_*`, `StartProcessing`, `OnSyncPointReached`; native C++ API, Windows) | measured on S7-1500 and S7-1200 G2 (V8): step size, real-time factor, repeatability |
| 5.3 | Free-run drivers: S7 PUT/GET, Modbus TCP, OPC UA client; jitter measured | measured on PLCSIM Advanced and a real CPU |
| 5.4 | S7CommPlus symbolic access to optimised DBs [06 #9] | read/write of a DB tag on a real CPU and on PLCSIM |
| 5.5 | OPC UA server exposing the scene to an HMI or SCADA [06 #8] | browsed by an independent client |
| 5.6 | Encoder and HSC emulation for fast machines [05 §4] | the method chosen after measurement on PLCSIM |

## M6 — Education ⬜

What makes it "powerful for schools" (the user, 2026-11-07).

| Phase | Deliverable | Acceptance |
|---|---|---|
| 6.1 | Ladder editor producing ST (truST approach [09]) | the default program written in LD runs the same |
| 6.2 | Fault injection as saved scenarios, with an instructor mode behind a password [06 #12] | a teacher sets faults the student cannot see |
| 6.3 | Graded exercises: scene, goal, automatic check of the result | 10 exercises from start/stop to sorting |
| 6.4 | Classroom distribution: offline installers, auto-update, no account | installed on a clean Windows machine |
| 6.5 | Teacher and student guide (en/it) | reviewed by a teacher |

## M7 — Material flow and analysis ⬜

What the discrete-event tools do [06 #14–17, 22].

| Phase | Deliverable | Acceptance |
|---|---|---|
| 7.1 | Product types, creators with distributions, seeded random streams per component | replications repeat bit for bit |
| 7.2 | Statistics: throughput, utilisation, states, OEE, with warm-up | figures checked against a hand calculation |
| 7.3 | Headless experiments: scenarios × replications, CSV/Parquet export | runs from the CLI without UI |
| 7.4 | Logistics components: buffers, AGV on a path network, simple racks | E2E scene with an AGV |

## M8 — Kinematics and robots ⬜

| Phase | Deliverable | Acceptance |
|---|---|---|
| 8.1 | Joints and servo controllers driven by signals; gantry, pick and place | a gantry cycle driven by ST |
| 8.2 | Six-axis robot with analytic IK (rs-opw-kinematics [08]) and joint/linear moves | reachability and cycle time on a reference cell |
| 8.3 | Physics islands with Rapier for falls and tipping only [05 §6] | the deterministic replay stays green |
| 8.4 | Interface to virtual robot controllers, instead of reimplementing them [06 #16] | one controller connected |

## M9 — Interop and ecosystem ⬜

| Phase | Deliverable | Acceptance |
|---|---|---|
| 9.1 | CAD import: glTF native, STEP via a compatible library (truck-stepio first [08]) | a real machine STEP imported |
| 9.2 | Component packages: versioned, signed, a public registry [06 #20] | install a package from the registry |
| 9.3 | Engine as a service: REST and CLI, Docker [06 #18] | a scene run in CI from the CLI |
| 9.4 | Browser edition: ST interpreter in wasm (path A′/B in [10]) | the M1 scene runs in a browser tab |
