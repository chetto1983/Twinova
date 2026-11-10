# Verification: component behaviours on the physical line (ROADMAP 4.3, 2026-11-09)

Spec: `2026-11-09-component-behaviour-design.md`. Plan: `2026-11-09-component-behaviour.md`, Task 7. The acceptance of
4.3 is "a component's logic edited and rebuilt in the app". Revision: `35695aa` (PR #26, merged as `275074b`).

## The acceptance, through the real app

`npm run e2e` (tauri-driver), debug build. Linux runs locally under Xvfb with software GL. Windows runs on the
windows-2022 runner, CI run 42705042616.

| Scenario (`e2e/run.mjs`) | Linux, local | Windows, CI |
|---|---|---|
| `demo_m3` builds at start. Start latches `belt_a.run`, and the belt's behaviour reports `belt_a.running` | ✅ | ✅ |
| Cartons ride the belt and reach the sink; virtual time follows the wall clock in Run (Windows: 5.02 s virtual in 5.02 s wall) | ✅ | ✅ |
| Blocked sink: the held carton covers the photocell, trips the jam and stops the belt | ✅ | ✅ |
| Save state and Restore; Run at ×2 (Windows: 8.00 s virtual in 4.04 s wall); Italian applies without a reload | ✅ | ✅ |
| **The belt's behaviour, `twinova.belt@1` tab, edited to `PT := T#500ms`, then Build**: `run` at 0.01 s, `running` at 0.51 s | ✅ | ✅ |
| **A broken behaviour**: Build fails with its file and line, and the running line keeps running on the last good build | ✅ | ✅ |
| Total | 11/11 | 11/11 |

`e2e/project.mjs` (4.2's corpus, now on the new engine): Linux 42/42, Windows 40/40 (no file modes on Windows).

The physics fingerprint `0x7ee1e6ce7ddf4a1f` is the same on Linux and on windows-2022 (`project-windows` job,
`cargo test -p twinova-physics`). Rust and frontend gates were green on the PR, and file-size was within the cap.
The `cargo mutants` run on `main` for this merge is run 42709307511.

## Soak of the release build, against M3

`npx tauri build --no-bundle && xvfb-run node e2e/soak.mjs 45`: 45 s per phase. Same machine as M3: the cloud
container, 4 cores, Xvfb with Mesa llvmpipe. M3's figures are the 60 s run of `0fad8c7` (`2026-11-08-cpu-ram-soak.md`).

| Phase | twinova CPU | WebKitWebProcess CPU | RSS twinova | RSS WebKitWebProcess | M3: twinova CPU, RSS |
|---|---|---|---|---|---|
| Idle | 0.0 % | 0.0 % | 176 → 175 MB | 434 → 431 MB | 0.0 %, 173 MB |
| Run ×1, line running | 9.1 % | 287 % | 176 → 176 MB | 441 → 452 MB (peak 468) | 7.2 %, 173 MB |
| As fast as possible | 44.5 % | 176 % | 176 → 176 MB | 452 → 453 MB | 44.4 %, 173 MB |
| Paused | 0.0 % | 0.3 % | 177 → 176 MB | 451 → 451 MB | 0.0 %, 173 MB |

WebKitNetworkProcess: 0 %, 48 MB, in every phase.

- **Memory is flat**: 175–177 MB for the app, 3 MB above M3, with Rapier and two truST runtimes.
- **Run ×1 costs 9.1 % of one core**, against 7.2 %. Each 10 ms scan now runs 10 physics steps and 10 field cycles.
- **As fast as possible: 734.7 s of virtual time in 45 s, 16× real time.** M3 reached 640× at the same duty cycle:
  25 ms of stepping every 33 ms (`commands.rs`, `FAST_BUDGET` and `FRAME`). This still clears M3's criterion of more
  than 10×, but it is **40× slower than M3**.

## Replay cost of a saved state

`cargo run --release --example replay_cost` (demo, line started):

| Build | 60 s virtual | 600 s virtual | M3, 600 s |
|---|---|---|---|
| release (`opt-level = "s"`, as shipped) | run 1.49 s, restore 1.44 s | run 14.1 s, restore 14.2 s | restore 0.34 s |
| same, `opt-level = 3` (measured only, not adopted) | 1.08 s | run 10.5 s, restore 11.1 s | |

On one core, the engine runs 42× real time. A restore replays from zero, so restoring one hour of virtual time takes
about 85 s, and an 8 h shift about 11 min, by extrapolation. In M3 these were 2.3 s and 19 s.

## Where the time goes

Profile: `valgrind --tool=callgrind` on `replay_cost 2`, a release build with line tables. Total 1.02 G instructions;
11 % of that is the one-off build of the two runtimes. Of the stepping:

| Part | Share of the total | What it is |
|---|---|---|
| Rapier `PhysicsPipeline::step` | 33 % | 1 000 steps per virtual second; most of it is the contact solver |
| truST `TestHarness::get_output` | 31 % | `line.rs` reads each belt's `motor` every millisecond by path (`Field::native`). Each read parses the path as an ST expression again (`parse.rs`, `parse_debug_expression`, 13 %), then lowers and evaluates it |
| truST VM cycle | 17 % | the field's 1 ms cycles and the controller's 10 ms scans |

`opt-level = 3` gains 1.35×, so it is not the cause. The read by path costs as much as the physics. truST
`6405915` already has two reads that skip the parser:

- `get_direct_output(address)`, which the field already uses for feedback;
- `get_access(name)`, which reads a `VAR_ACCESS`.

Either would mean changing the generated field program. That is a design change, so it is proposed to the user rather
than made here (CLAUDE.md, "stop before bespoke").

## What this does not prove

- **Speed on larger lines.** Only the demo is measured: one belt, one sensor, a source and a sink. Physics cost grows
  with the number of cartons in contact, and the field cost with the number of instances. No scaling curve was
  measured.
- **Physical fidelity against a reference.** The carton recipe is OIP's (research 15), and the measured outcomes match
  M3's: deliveries in 20 s, and the jam between 6.7 and 7.0 s. There is no comparison with a real conveyor.
- **The diverter.** Its behaviour block runs in the field, but its family has no physical model yet (4.4). A project
  with a diverter is refused at Build with that message.
- **A real PLC.** The controller is truST in-process; PLC in the loop is M5.
- **Long restores.** The restore cost above is an extrapolation from 600 s.
