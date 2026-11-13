# Verification: online monitoring, forcing and the variables table (ROADMAP 4.8, 2026-11-13)

- **Spec:** `2026-11-13-online-monitoring-design.md`. Approved, decisions 1 to 11, amended with Task 2's measurements
  and the first E2E run's.
- **Plan:** `2026-11-13-online-monitoring.md`.
- **Machine:** the cloud container with 4 cores, Linux, WebKitGTK under `xvfb-run -s "-screen 0 1440x900x24"`.
- **CI:** pull request #45. The whole test suites, coverage and mutation run there (CLAUDE.md, user 2026-11-13).

## What runs

- **Engine.**
  - `watch.rs` reads paths and qualified conditions from the PLC's or the field's harness after a scan, with no debug
    control.
  - It turns a user's text into a value of the variable's type.
  - It lists an instance's members from the runtime's storage.
- **`controller_monitor.rs`.**
  - Attaches truST's `DebugControl` at the first write or force.
  - Forces and releases a program's variable, an instance's member on that instance, or an I/O address. A variable
    declared at an address forces the address too.
  - Writes a variable once.
- **`Engine`.** Logs `Write`, `Force` and `Release` as inputs.
- **App.**
  - `monitor.rs` turns the visible lines into watches (trust-ide's inline values, the syntax tree's fields and
    conditions) and adds the variables table's rows.
  - `declarations.rs` reads a POU's declarations and edits them; renames go through trust-ide.
- **Webview.**
  - Monitoring toggle and Ctrl+T, values in the code, the variables table, the write/force bar, and the Watch pane's
    Forces list with its address forcing.

## The acceptance

| # | Criterion | Evidence | Result |
|---|---|---|---|
| 1 | End-of-scan values equal the Watch pane's | `engine_monitor_tests.rs`, `the_watched_values_equal_the_program_s`. `e2e/monitor.mjs`: the table's `belt_b_run` and `detected_a` equal the Watch pane's `belt_b.run` and `sensor_a.detected` | ✅ |
| 2 | Conditions | the same test: `Main.count MOD 3 = 0` against `count` read; the E2E: line 39 draws TRUE or FALSE with its colour | ✅ |
| 3 | Forcing | engine: a forced variable holds for 1,000 scans, its output too, belt B stands; a forced input holds against the plant and comes back on release; a member holds on its instance. E2E: the same through the table and the Watch pane | ✅ |
| 4 | Determinism | `monitoring_leaves_the_line_s_trajectory` (fingerprint and snapshot equal while read at every scan); `a_restored_state_replays_the_forces` (500 snapshots equal after restore) | ✅ |
| 5 | Cost | below | ✅ |
| 6 | Gates | below | ✅ locally; CI on #45 |

### Acceptance 5: the release build, the diverter, 30 s per phase

`npm run soak`:
- with Monitoring on (`SOAK_MONITOR=1`);
- with `detected_b` also forced through the table (`SOAK_FORCE=detected_b`), which attaches the debug control. The
  program does not use `detected_b`, so the line runs as without the force.

| | Off | Monitoring | Monitoring + a force |
|---|---|---|---|
| Idle: WebKit CPU | 0.1 % | 3.3 % (the 500 ms reads) | 6.5 % (the bar open too) |
| Run ×1: twinova CPU | 11.0 % | 11.6 % | 12.7 % |
| As fast as possible: virtual time in 30 s | 359.5 s | 367.6 s | 312.9 s (−13 %) |
| Paused: WebKit CPU | 1.6 % | 2.1 % | 2.9 % |
| twinova RSS | 179–180 MB | 179–180 MB | 180–181 MB |

- **Monitoring alone costs the reads only.** The debug control's cost appears with the first force, as the engine
  measured: 3 to 6 times a scan, 1 % to 15 % of a run.

### Measurements that changed the design

All recorded in the spec's amendments:
- reads need no debug control;
- releasing a forced input needs nothing written;
- truST drops a force named by a dotted path and holds one given to the member's instance;
- a forced variable left its output TRUE, so its address is forced too;
- Monaco 0.57 draws no `after` text on an empty range;
- truST's parser has no `Condition` node;
- the parser attaches a comment above a declaration to its name.

### The gates

- **Rust:**
  - fmt, `clippy --workspace --all-targets -D warnings`;
  - the touched crates' tests: engine 108, app 70.
- **Frontend:**
  - svelte-check and oxlint: 0 warnings;
  - prettier, knip and jscpd: clean;
  - vitest: 213.
- **E2E:** 119/119 on Linux, 12 files, `monitor.mjs` 6/6 among them.
- **Coverage and mutation:** CI on pull request #45 (`ci.yml`, `mutation.yml`).
- **Mutation's timeout raised from 60 to 180 s (user, 2026-11-13).**
  - The engine's tests took 41-47 s per mutant on one shard's runner and more than 60 s on the other two (runs
    43403329446 and 43405902568).
  - There, 42 engine mutants timed out, and the gate counts a timeout as caught. 13 of them were survivors on the
    faster runner; the tests that kill them came with this pull request.

## What this does not prove

- **Values as each line ran, breakpoints, stepping:** ROADMAP 4.9.
- **Forcing inside a block.** A block's tab is read-only.
- **A non-BOOL variable at an address.** It is forced as a variable only: its image holds bits of another kind
  (a DINT at `%ID` travels as a DWORD).
- **Monitoring a real or emulated PLC.**
- **Windows,** until CI's windows-2022 E2E.
