# Twinova roadmap

Each milestone below is broken into phases. A phase closes only with its acceptance met and a dated verification file in
`docs/superpowers/verification/` (CLAUDE.md, "Definition of Done"). The order follows dependencies: no PLC link before
the engine is deterministic, no component library before the component model exists.

References in brackets point to the research in `docs/research/` (in Italian): `06` is the best-of matrix, `05` the
engine study, `08` the Rust inventory, `09`/`10` the IEC 61131-3 runtime work.

Status: ✅ done · 🔶 partly done (gaps listed) · ⏸ measured and deferred · ⬜ not started.

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
| 2.1 | CI on GitHub Actions, Linux and Windows: fmt, clippy `-D warnings`, tests, svelte-check, Vitest, build | green on `main` | ✅ green on `main` (run 42385068785, `fc0d725`); Windows through the E2E; prose-only changes skip CI (Aura scheme). Since 2026-11-08 the jobs run in parallel and a pull request takes about 4 min instead of 14 (`verification/2026-11-08-ci-speed.md`) |
| 2.2 | Coverage (`cargo llvm-cov`, `vitest --coverage`) and mutation (`cargo mutants`) gates | thresholds enforced in CI | ✅ coverage gates in CI; first `mutation.yml` run on `main` (42385068809): cargo mutants 62/62, Stryker 27/27. The cargo mutants gate fails on any survivor since PR #11: exit code 3 (timeouts) had hidden 7 of them |
| 2.3 | 600-LOC check, oxlint and prettier, dead-code and duplication checks (the ModbusTool set) | zero warnings | ✅ |
| 2.4 | Release bundles as in ModbusTool: MSI, deb and AppImage, a tag-driven `release.yml`, plus `CHANGELOG.md`. Not now (user, 2026-11-07): dmg, NSIS, signing, the updater (6.4) | installers built by CI, sizes recorded | ✅ MSI 4.55 MB, deb 4.14 MB, AppImage 82.8 MB (run 42387823991); `release.yml` ready, no tag yet. Since 2026-11-08 the installers are built on pushes to `main` only (user), and the bundler's tools are cached and fetched with retries |
| 2.5 | **Runtime footprint**: evaluate reducing trust-runtime `full` to the encoder and VM (path A′ in [10]) | measured cost/benefit and execution decision; preserve ST behaviour; frontend work stays in 4.7 | ✅ decision closed in M3: retain `full` at `6405915`. The original acceptance (replace `full`, binary size before/after) was not met: the phase was redefined as an evaluation on 2026-11-08, because the measurement showed little binary saving against a ≥45,700-line fork. B has no comparative measurements. No runtime replacement was implemented. See `2026-11-08-m3-closure-design.md` for evidence and reopening criteria |
| 2.6 | **truST upstream policy.** Execution (VM and encoder): deleted upstream on 2026-10-24, so it stays pinned at `6405915` and is ours to maintain (forked or replaced when 2.5's A′/B choice is made); upstream fixes are ported by hand. Language frontend (`trust-syntax`, `trust-hir`, `trust-ide`, `trust-wasm-analysis`): followed upstream by moving its git `rev` in a PR, after reading the commits that touch those crates, monthly. None of these crates is on crates.io, so nothing updates automatically | the policy in CONTRIBUTING or CLAUDE.md; the first frontend bump done through it, with CI green | ✅ policy written (spec `2026-11-07-trust-upstream-policy-design`, CLAUDE.md). The first frontend bump came with 4.7: `trust-wasm-analysis` and the three crates it brings at upstream v0.39.0 (`5eb7f8c`), after reading the 148 commits that touch them; the runtime stays at `6405915` |
| 2.7 | **Twinova for coding agents** (user, 2026-11-11, before 4.4d): the engine served over MCP on stdio and HTTP, with full scene editing | an MCP client runs and edits a line on both transports, with the engine's own fingerprint | ✅ the engine moved to `twinova-engine` (no Tauri; Windows CI tests it now) with Build shared by the app and the agent; `twinova-agent` serves 23 tools on rmcp (stdio, and streamable HTTP on axum at `127.0.0.1:7650/mcp`), one engine per HTTP session. rmcp's own client runs the sort line on both transports to the engine's fingerprint, and extends it with a belt, builds, runs, saves and reopens it the same; `.mcp.json` registers it for agents (`verification/2026-11-11-agent-server.md`) |

## M3 — Deterministic engine core ✅

The hybrid engine of [05 §6]: discrete events with analytic motion, and fixed steps only where they are needed.

| Phase | Deliverable | Acceptance | Status |
|---|---|---|---|
| 3.1 | Spike: nexosim 1.0 versus our own scheduler, on the M1 scene [08] | decision recorded with measurements | ✅ nexosim on one thread, executor made FIFO (spec `2026-11-07-des-core-design`). The M1 scene on nexosim matches M1's engine scan by scan over 60 s. FIFO patch submitted as [upstream PR #189](https://github.com/asynchronics/nexosim/pull/189); acceptance pending, fork pin retained |
| 3.2 | Event core on nexosim (decided in 3.1): the app's lockstep engine replaced by a nexosim bench on one thread, with the FIFO executor patch; PLC and scene as models; UI inputs as events, snapshots as queries | the 27 tests and the E2E still green; property test on the order of simultaneous events; replay fingerprint test; mutation ≥70 % on the engine | ✅ on the user's nexosim fork with the FIFO patch; M1 reproduced scan by scan; replay and ordering property tests; mutation 80/80; E2E on Linux and Windows (run 42459841117) |
| 3.3 | Analytic conveyor segments with exact sensor edges (JaamSim scheme [05, r1 §4]) | edges exact to the ns; benchmark against the fixed step (OIP baseline [07]) | ✅ closed-form belt, edges exact to the ns; events at their closed-form times (10 ms before M1, whose float sum was late); one hour in 1.2–2.0 ms against 3.2–4.4 ms at a 10 ms step (spec `2026-11-08-analytic-belt-design`) |
| 3.4 | PLC scan as a periodic event, with the scan time per program | same results as M1 at 10 ms; scan time configurable | ✅ the period comes from the program's IEC `TASK` intervals (their greatest common divisor), 10 ms without one; the default program's pinned scans are unchanged (spec `2026-11-08-scan-period-design`) |
| 3.5 | Run modes: Run (wall clock × scale, `OutOfSync` beyond a tolerance), as fast as possible, Step, Reset, snapshots | the lag is reported, never hidden; save and restore a state | ✅ speed ×0.25–×10 and as fast as possible (548 × real time in release), lag reported; snapshots by replay, exact, one hour restored in 2.33 s (release); verification `2026-11-08-run-modes.md` |

Closed 2026-11-08: retain the measured pinned truST runtime; neither extraction A′ nor a new interpreter B is
justified for M3. The FIFO contribution is submitted upstream; maintainer acceptance remains external, and the
tested fork pin stays in use. [Closure decision](docs/superpowers/specs/2026-11-08-m3-closure-design.md) and
[acceptance evidence](docs/superpowers/verification/2026-11-08-m3-closure.md).

Handoff to M4: measure multi-component scaling and investigate the webview memory growth recorded in the soak.
Snapshots currently use one in-memory replay slot, for the same program. External PLC connectivity remains M5.

## M4 — Component model and scene editor 🔶

The core of the product [02a, 06 #1–2].

Started 2026-11-08: catalogue-first direction, source research and live Open Industry Project exploration.
The [M4.1 model](docs/superpowers/specs/2026-11-08-component-model-design.md) is approved, and the
[M4.2 planning draft](docs/superpowers/plans/2026-11-08-component-format.md) is recorded.
The project format (4.2), component behaviours on a physical line (4.3), the first complete line with OIP's roller conveyor and chain transfer (4.4a), curves, the belt's look, guards, incline and the conveyor drive (4.4b), the stopper, pusher and turntable (4.4c), and products with attributes and four new sensors (4.4d) are implemented and
accepted through the real app.

| Phase | Deliverable | Acceptance | Status |
|---|---|---|---|
| 4.1 | Spec of the component model: node tree, joints, properties, typed signals, behaviours, interfaces | spec approved | ✅ spec approved (user, 2026-11-08): `2026-11-08-component-model-design`; signals bind to IEC direct addresses, per instance through `VAR_CONFIG`, measured on the pinned truST (`verification/2026-11-08-component-model.md`) |
| 4.2 | Open scene and component format: JSON plus glTF geometry, versioned, with migrations | round-trip tests; a corpus of saved scenes opens | ✅ the `twinova-project` crate: a folder with a hashed manifest written last, a save marker and rollback, and a byte-reproducible zip archive. Open, save, export and import are in the app; `twinova --project=<folder>` opens a folder at start. The corpus opens, saves and reopens through the real app on Linux and Windows, and its 12 invalid fixtures are refused with their diagnostics (`verification/2026-11-09-project-format.md`). Only format 1 exists, so there is no migration yet; GLB assets are stored, not drawn (4.4) |
| 4.3 | Behaviours written as ST function blocks bound to component signals | a component's logic edited and rebuilt in the app | ✅ each definition's behaviour is an ST function block in `components/<key>.st`, run per instance in a 1 ms field runtime bound by `VAR_CONFIG`. The line program works by address, and transport is Rapier physics with OIP's carton and deck. Edited in the app's tab and rebuilt: a 500 ms start delay shows up in `running`, and a broken block fails Build with its file and line while the line keeps running, on Linux and Windows (`verification/2026-11-09-component-behaviour.md`). As fast as possible fell from 640× to 16× real time; the profile puts half of it in Rapier and in truST's read by path |
| 4.3a | A native model reads its block's variables through the block resolved at Build, not by path | the line's results unchanged; time measured before and after | ✅ the read by path re-parsed `F.<instance>.<name>` every millisecond, 31 % of a run. Replay of 600 s: 14.5–15.2 s → 10.3 s; as fast as possible: 16× → 22× real time. Physics is now 48 % of a run (`verification/2026-11-09-field-native-reads.md`) |
| 4.4 | Library v1: belt, roller and curved conveyors, diverter, stopper, pusher, turntable, sensors (diffuse, through-beam, colour, barcode), creator and sink. Models: Open Industry Project's GLBs are first-party and MIT (its `THIRD_PARTY_NOTICES.md`, read at `0572868`), so they may be used with its copyright and licence in Twinova's `THIRD_PARTY_NOTICES.md`, after checking each model for its name or logo. The diverter has one; belts, rollers, curves and sensors are generated by OIP's code (MIT), whose logic is re-expressed in Twinova's generators. Its part assets are Draco-compressed with WebP textures, so the Draco decoder's own licence is read before it ships | each component has an E2E scene; each model taken from OIP has a test of its size, its origin and its axes against the family's generator |
| 4.4a | First part of 4.4 (user: line first, OIP copied and translated): straight roller conveyor and OIP's chain transfer (the two-output diverter), with M4.1's acceptance line | the line runs in the app; no carton lost; routing by the PLC; determinism across run modes and platforms | ✅ OIP's geometry kernel and generators copied (roller, chain transfer, photo eye), its surface velocity and kinematic lift translated into Rust, its `step()` in ST. Every third carton to reject in the app on Linux and Windows; conservation every millisecond; the line's fingerprint equal on windows-2022. As fast as possible runs the line at 6.9× real time (`verification/2026-11-10-diverter-line.md`). The belt's look, curves and roller options move to 4.4b |
| 4.4b | Second part of 4.4 (user: all four items 4.4a left): curved belt and roller conveyors, the belt's look, side guards and incline, the drive ramp, roller skew, and the speed and encoder signals | a carton turns with a curve and leaves square; a line straight → curve → straight runs in the app; guards, incline and skew after OIP's fidelity suite; the drive's ramp, encoder and velocity | ✅ Coulomb friction world-wide (measured: the default holds a load's yaw); OIP's curve decks and rotating field, guards, incline and skew translated into Rust; its drive in ST with optional signals that fall back to a parameter; its belt, curved belt and curved roller drawn. A carton turns 90° ± 3° within 5 mm; the curve line runs in the app on Linux and Windows; its fingerprint equal across platforms. Breakaway is 1–1.8° under atan(μ) in Rapier (acceptance amended, user). As fast as possible runs the transfer line at 4.1× real time (the drive in ST costs a third of the field) (`verification/2026-11-10-curves-belt-options.md`) |
| 4.4b1 | The Run loop's pace, and a conveyor's velocity, encoder and pulse as its native model (user: "can we do better?" after 4.4b's soak) | the 4.4b drive figures unchanged; time measured before and after | ✅ the loop slept a whole frame after its work (43 % of the time, 17 snapshots a second) and the three signals cost a third of the field's cycle in ST. A simulated millisecond: 63 → 50 µs on the transfer line, 105 → 79 µs on the curve line. As fast as possible: 4.1× → 10.9× and 2.7× → 8.0× real time, memory flat (`verification/2026-11-10-run-pace-native-feedback.md`) |
| 4.4b2 | One solver iteration per millisecond, and I/O addresses resolved at Build (user, after 4.4b1's profile: the physics was 47–60 % of a millisecond) | the 4.3–4.4b1 tests unchanged but for the fingerprints, pinned again; time measured before and after | ✅ Rapier's solver, at its default of 4 iterations for 16 ms steps, was 70–75 % of the physics step: with one, 25 → 15 µs and 55 → 34 µs, the curves' fidelity the same or better. Addresses parsed once instead of every millisecond: 3 µs more. A simulated millisecond: 50 → 36 µs on the transfer line, 79 → 56 µs on the curve line. As fast as possible: 10.9× → 16.5× and 8.0× → 11.3× real time (`verification/2026-11-10-solver-substeps-io-links.md`) |
| 4.4c | Third part of 4.4 (user: "the natural step to close 4.4"): OIP's blade stop, diverter (a pusher) and turntable conveyor, with a sorting line | a queue held dead at the stopper and released at speed; a pushed carton upright and square on a takeaway; a carton turned 90° in place and run off along the new heading; the sort line routes as its program says and loses none, in the app on Linux and Windows | ✅ the blade and the paddle are kinematic bodies posed from their ST blocks, the turntable a conveyor whose lane's field turns with its deck: OIP's grip and deck body are not translated, the field alone turns a carton within 0.01° (measured). The queue stands within 1 mm of the blade, the front carton at speed 209 ms after release; a pushed carton lands within 2.6° of upright; 90° at 20°/s takes 4 983 ms, OIP's stepped profile. The sort line routes 8 cartons a minute, every route taken, the same under Step, ×1 and as fast as possible (`verification/2026-11-11-stopper-pusher-turntable.md`) |
| 4.4d | Fourth part of 4.4 (user: in order, after 2.7): products with size, mass, colour and code, and the through-beam, colour, laser and barcode sensors | OIP's product sizes through every family; seeded sources; each sensor's reading; a line sorting by code and colour in the app on Linux and Windows, driven by the agent too | ✅ list parameters in the format; the source draws OIP's sizes, colours and codes from a seed (mulberry32, OIP's draw order); the four sensors native, as the diffuse one (user). The carton weighed 1.76 kg (parry gives a round cuboid its core's mass): the mass is set. OIP's 0.6 m and 0.8 m boxes pass every family, within 3° on every curve but the belt → roller-curve handover (5.4–7.0°, physical). The code and colour line sends every carton to the sink of its colour, the same under Step, ×1 and as fast as possible (`verification/2026-11-11-sensors-products.md`). Pallets, totes and the light curtain stay out |
| 4.5 | Editor: catalogue, drag and drop, snapping by interfaces, move/rotate, signal wiring, undo | build a two-conveyor line with a diverter by hand | ✅ in two parts, 4.5a and 4.5b |
| 4.5a | First part of 4.5 (user: split as 4.4): the component library, placing from a catalogue, moving and turning with a gizmo, editing in the inspector, undo, a new project | a line built by hand in the app from an empty project builds and runs; undo and redo give the same scene; save and reopen give the same project | ✅ the 15 families embedded as one library the fixtures copy; the agent's edits moved to the format crate and shared with the app, with whole-project undo (100 steps, the ST sources kept); a catalogue, an editable inspector, picking and three's TransformControls in the view, Delete and Ctrl+Z/Y. From an empty project a source, two belts, a sink and a photocell are placed, connected, mounted and bound by hand, built and run to the sink, undone to empty and redone, saved and reopened: 10/10 on Linux, and on Windows in CI (`verification/2026-11-11-editor-basics.md`). Snapping by interfaces is 4.5b |
| 4.5b | Second part of 4.5: snapping by interfaces (conveyor end to end with a gap, devices onto rails, sources and sinks onto ends, a spur onto a flank) | the ROADMAP's line with a diverter built by hand with drag and snap only; the fixtures rebuilt by joins equal the originals | ✅ ends are the parts' real faces, as in OIP (a belt's drum, 85 mm past its run, now in the physics too); a join takes a gap the user sets, real in the physics (measured: a product crosses a gap up to about half its length); a new `merge` connection opens the host's guard. Joins in Rust (`physics::snap`, shared with the agent), candidates in the view with OIP's thresholds. The five fixtures rebuilt from scattered parts by joins alone land within 8.5e-7 m; `two_belts_diverter` built by hand in the app with drag and snap only, built, run to both sinks and saved equal to the fixture; a spur merged in the app delivers: 10/10 on Linux, Windows in CI (`verification/2026-11-11-snapping.md`) |
| 4.6 | **ST in VS Code**: the scene's programs are `.st` files in a project folder, rebuilt on save; VS Code with truST's `trust-lsp` extension (MIT OR Apache-2.0) is the external editor | edit in VS Code, save, the running scene picks up the new program | ✅ PR #41, on `main`: the line program goes online (a warm restart, RETAIN kept) and a behaviour rebuilds; Twinova writes `trust-lsp.toml` and the field's configuration for the Marketplace extension (0.24.75, 0.38.0 and 0.39.0 give the same diagnostics; three false E202 per block remain). E2E on Linux and windows-2022 in CI, and on a Windows 11 VM against the installed MSI (5/5), with VS Code and the extension 0.39.0; the MCP agent's `change_program` and `reload_sources` on the same folder (10/10). Spec `2026-11-12-st-vscode-design`, verification `2026-11-12-st-vscode` |
| 4.7 | **In-app ST editor** for who has no VS Code (classrooms): Monaco with diagnostics, hover and completion from truST's `BrowserAnalysisEngine` (`trust-wasm-analysis`, already in the dependency tree) through Tauri commands | the same diagnostics as `trust-lsp` on the default program and on a broken one; Monaco measured on WebKitGTK; the frontend `rev` moved as the 2.6 policy says | ✅ Monaco 0.57.0 with its own `st` language; truST's diagnostics before Build, hover and completion through Tauri commands, from `trust-wasm-analysis` at upstream v0.39.0 (`5eb7f8c`). On the 6 valid fixtures and a broken program the editor lists what `trust-lsp` 0.39.0 does, less the false E202 of the blocks (7/7). Release build against `main`: CPU unchanged, JS 816 → 4,199 KB, web process +70 MB. Spec `2026-11-12-st-editor-design`, verification `2026-11-12-st-editor` |
| 4.8 | **Online monitoring in the editor** (research [17]): each line's values beside the code, TRUE/FALSE on conditions, an FB instance chosen to watch; modify and force from the code, as TIA Portal and CODESYS do. From the pinned runtime's `DebugControl` (reached from the harness), not from VS Code (user, 2026-11-12) | values read at the end of a scan equal the Watch pane's; a forced input holds across scans; monitoring cost measured against plain Run | ✅ Monitoring (Ctrl+T) draws each visible variable's value after the scan and TRUE/FALSE on conditions; a variables table above the code lists, edits and monitors the declarations (rename through truST); a program's variable, an instance's member or an I/O address is written or forced, the forces listed in the Watch pane and replayed by Restore. truST's debug control attaches at the first force: a run as fast as possible slows by 13 %, monitoring alone costs the reads. Spec `2026-11-13-online-monitoring-design`, verification `2026-11-13-online-monitoring` |
| 4.8a | **The dock maximized** (user, 2026-11-13: the PLC pane needs room; first a maximize button, then a window of its own): the dock, whichever tab is open, over the scene and both side panels, the toolbar and the status bar kept | dock over the workspace, Monaco follows its size, monitoring goes on, the height restored | ✅ A button in the dock's bar; the scene keeps drawing behind it, unchanged. Spec `2026-11-13-maximized-dock-design`, verification `2026-11-13-maximized-dock` |
| 4.8b | **The PLC pane in a window of its own**, e.g. on a second monitor | measured first: one owner of the program's text for two windows (today webview state), capabilities for a second window, tauri-driver across windows | ⬜ |
| 4.9 | **Breakpoints as HOLD** (research [17]): a breakpoint in the editor's gutter stops the program, its timers and the simulated plant, as an S7-1500 in HOLD; Run, Run to cursor, step over/into/out and the call stack | measured first: what the runtime's cycle does at a breakpoint inside the deterministic engine; replay unchanged by a stop and resume | ⬜ |
| 4.10 | **Completion as in PLC IDEs, and navigation** (research [17]): an FB or function call inserted with all its parameters (`:=` inputs, `=>` outputs), optional ones reducible; declare an undeclared variable; go to definition, references, rename | E2E on each; whether truST already provides call templates measured before any code | ⬜ |

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
| 8.1 | Joints and servo controllers driven by signals; gantry, pick and place. Axes are commanded from ST with the PLCopen Motion Control Part 1 single-axis blocks (`MC_Power`, `MC_MoveAbsolute`/`Relative`/`Additive`/`Velocity`, `MC_Halt`, `MC_Stop`, `MC_Reset`, `MC_Read*`) and the PLCopen axis state diagram, as S7-1500 technology objects expose them. Reference: RTmotion ([leijian001/plcopen_motion_control](https://github.com/leijian001/plcopen_motion_control), Intel, Apache-2.0, C++17, last commit 2021-09), for its blocks, state machine, S-curve planners and tests, read as a reference rather than linked; jerk-limited trajectories through the `ruckig` crate (MIT, [08]) | a gantry cycle driven by ST through `MC_` blocks; every transition of the PLCopen axis state diagram tested, including errors and `MC_Reset` |
| 8.2 | Six-axis robot with analytic IK (rs-opw-kinematics [08]) and joint/linear moves | reachability and cycle time on a reference cell |
| 8.3 | Physics islands with Rapier for falls and tipping only [05 §6] | the deterministic replay stays green |
| 8.4 | Interface to virtual robot controllers, instead of reimplementing them [06 #16] | one controller connected |

## M9 — Interop and ecosystem ⬜

| Phase | Deliverable | Acceptance |
|---|---|---|
| 9.1 | CAD import: glTF native, STEP via a compatible library (truck-stepio first [08]) | a real machine STEP imported |
| 9.2 | Component packages: versioned, signed, a public registry [06 #20] | install a package from the registry |
| 9.3 | Engine as a service: REST and CLI, Docker [06 #18] | a scene run in CI from the CLI |

The browser edition (wasm) that stood here as 9.4 is dropped (user, 2026-11-07): Twinova is a Tauri desktop app, and its
engine runs natively. No phase is chosen or weighed for wasm.
