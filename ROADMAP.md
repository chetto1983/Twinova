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

## M1 — Walking skeleton ✅

One belt conveyor in 3D, driven by Structured Text, in a Tauri desktop app.
Spec `2026-11-07-tauri-shell-design.md`.

| Phase | Deliverable | Acceptance | Status |
|---|---|---|---|
| 1.1 | Tauri 2 + Svelte 5 shell, Aura tokens, en/it | `svelte-check` clean, app starts | ✅ |
| 1.2 | Closed-form belt, ST on truST's VM, lockstep engine | 18 Rust tests incl. the 60 s scenarios | ✅ |
| 1.3 | E2E under Xvfb | screenshots; pacing 10.04 s virtual in 10.06 s wall | ✅ |
| 1.4 | **DoD gaps**: coverage ≥85 %, mutation ≥70 % on `scene`/`engine`, Windows run | figures in a verification file | ✅ coverage 93.6 % Rust / 100 % TS, mutation 62/62, E2E 6/6 on Linux and Windows |

## M2 — Engineering foundation 🔶

The gate set of chetto1983/ModbusTool, ported in `2026-11-07-engineering-foundation` (plan and verification).

The tooling that every later phase relies on.

| Phase | Deliverable | Acceptance | Status |
|---|---|---|---|
| 2.1 | CI on GitHub Actions, Linux and Windows: fmt, clippy `-D warnings`, tests, svelte-check, Vitest, build | green on `main` | ✅ green on `main` (run 42385068785, `fc0d725`); Windows through the E2E; prose-only changes skip CI (Aura scheme) |
| 2.2 | Coverage (`cargo llvm-cov`, `vitest --coverage`) and mutation (`cargo mutants`) gates | thresholds enforced in CI | ✅ coverage gates in CI; first `mutation.yml` run on `main` (42385068809): cargo mutants 62/62, Stryker 27/27 |
| 2.3 | 600-LOC check, oxlint and prettier, dead-code and duplication checks (the ModbusTool set) | zero warnings | ✅ |
| 2.4 | Release bundles as in ModbusTool: MSI, deb and AppImage, a tag-driven `release.yml`, plus `CHANGELOG.md`. Not now (user, 2026-11-07): dmg, NSIS, signing, the updater (6.4) | installers built by CI, sizes recorded | ✅ MSI 4.55 MB, deb 4.14 MB, AppImage 82.8 MB (run 42387823991); `release.yml` ready, no tag yet |
| 2.5 | **Runtime footprint**: replace trust-runtime `full` with only the encoder and the VM (path A′ in [10]) | binary size before/after measured; ST behaviour unchanged (same tests); `trust-wasm-analysis` kept for 4.7 | ⬜ |
| 2.6 | **truST upstream policy.** Execution (VM and encoder): deleted upstream on 2026-10-24, so it is forked once into a crate of ours, with attribution, and maintained here; upstream fixes are ported by hand. Language frontend (`trust-syntax`, `trust-hir`, `trust-ide`, `trust-wasm-analysis`): followed upstream by moving its git `rev` in a PR, after reading the commits that touch those crates, monthly. None of these crates is on crates.io, so nothing updates automatically | the policy in CONTRIBUTING or CLAUDE.md; the first frontend bump done through it, with CI green | ⬜ |

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
| 4.6 | **ST in VS Code**: the scene's programs are `.st` files in a project folder, rebuilt on save; VS Code with truST's `trust-lsp` extension (MIT OR Apache-2.0) is the external editor | edit in VS Code, save, the running scene picks up the new program |
| 4.7 | **In-app ST editor** for who has no VS Code (classrooms, the browser edition): Monaco with diagnostics, hover and completion from truST's `BrowserAnalysisEngine` (`trust-wasm-analysis`, already in the dependency tree) through Tauri commands | the same diagnostics as `trust-lsp` on the default program and on a broken one; Monaco measured on WebKitGTK |
| 4.8 | Spike: truST's runtime control endpoint on Twinova's simulated PLC, so VS Code's Live Values and debugger attach to the scene | decision recorded: protocol inventory, and whether it fits the virtual-time harness |

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
| 6.1 | Ladder editor producing ST (truST approach [09]). First a spike on reusing the Ladder webview of truST's VS Code extension (React, `*.ladder.json`, PLCopen LD interop) | the default program written in LD runs the same |
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
