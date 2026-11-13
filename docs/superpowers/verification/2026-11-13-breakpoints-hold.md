# Verification: breakpoints as HOLD (ROADMAP 4.9)

Date: 2026-11-13. Spec `2026-11-13-breakpoints-hold-design.md`, plan `2026-11-13-breakpoints-hold.md`.

## What was measured

- **Before the spec** (spec, "What exists"), with a throwaway probe on the 4.4a line in release:
  - at a stop, truST's hook blocks the scanning thread, so the engine, the plant and virtual time stand;
  - stops, steps and continues leave the fingerprint equal;
  - values read from the paused snapshot take 1 to 7 µs;
  - at the pin, a breakpoint in an IF's body stopped at the IF every scan.
- **Engine:** `cargo test -p twinova-engine debugger`, 7/7, on the 4.4a line and on a program calling a user's block.
  The tests cover:
  - HOLD with virtual time standing and the fingerprint unchanged;
  - over, into, out, continue and Run to cursor;
  - the stack innermost first;
  - a line taking its nearest statement;
  - letting go;
  - a breakpoint in an IF's body stopping only when the body runs (after 5 s, count 0 then 1);
  - a breakpoint on the IF's line stopping once a scan.

  **Control:** without the fork's patch, the body test fails, stopping at line 39.
- **App:** `cargo test -p twinova`, 77/77. `debug_commands`:
  - while held, a command that needs the engine answers "The PLC is in HOLD: continue first" within 100 ms, instead
    of waiting on the main thread;
  - a Step stopped by a breakpoint returns once resumed;
  - Reset lets the line go;
  - the session's breakpoints are armed again at Run.
- **truST's own tests on the fork** (`chetto1983/trust-platform`, branch `twinova/6405915-breakpoint-body`), run in the
  fork's CI, since Twinova does not compile truST's tests (user, 2026-11-13). Run 43435374062: `debug::breakpoints`,
  27/27, and `tests/debug_stepping.rs`, 7/7, `breakpoint_only_triggers_for_taken_branch` among them.
- **E2E on Linux,** `e2e/breakpoints.mjs`, 10/10:
  - glyph margin and F9;
  - HOLD in the status bar, with time standing;
  - count at the stop;
  - F10, F5, Ctrl+F10;
  - Continue with the breakpoint off;
  - Reset while held;
  - a breakpoint in an IF's body (line 40) stops after the first carton, with count 0, then 1 after F11;
  - a line inserted above moves a breakpoint from 40 to 41, and after Build, Run stops at 41 (spec decision 8);
  - a breakpoint in a branch that never runs (`IF count < 0 THEN`) never holds the line. As fast as possible for 4 s,
    in the debug build under xvfb: 42.4 s of virtual time without the debugger, 37.0 s with the breakpoint (−13 %,
    the debug control's cost).

  Regressions checked: `monitor` 7/7, `plc_window` 6/6, `workspace` 9/9, `editor_st` 6/6, `editor` 10/10. The whole
  suite runs in CI.

### The release soak: the 4.4a line, Linux under xvfb, 30 s per phase

| | 4.8's "Off" (before) | Off, now | Monitoring and a force, now |
|---|---|---|---|
| Idle: WebKit CPU | 0.1 % | 1.1 % | 7.5 % |
| Run ×1: twinova CPU | 11.0 % | 11.0 % | 12.9 % |
| As fast as possible: virtual time in 30 s | 359.5 s | 359.0 s | 310.8 s (−13 %) |
| Paused: WebKit CPU | 1.6 % | 2.7 % | 3.5 % |

- **The status bar asks where the program is held every 300 ms.** That costs about one point of web-process CPU when
  idle or paused, and nothing measurable while running.
- **Attaching the debug control** (a force, or the first breakpoint) costs 13 % of a run as fast as possible, as 4.8
  measured.
- **A breakpoint never hit.** The 4.4a program runs every line, so the E2E adds a branch that never runs:
  - the app measured 37.0 s of virtual time against 42.4 s without the debugger (debug build, above);
  - the engine measured 0.99 s against 1.01 s for 20 s of virtual time, both with the control attached.

  So the cost is the control's, not the breakpoint's.

## Measurements that changed the design

- **The pin's overlap rule** led to porting upstream `c9635b1c` through the user's fork (decision 1).
- **Tauri runs commands without `async` on the main thread,** so `with_engine` never waits while held (decision 4).
  Step became async, so a stop inside it holds a worker.
- **A stop sends no event, since its scan holds the Run loop.** The webview asks `hold` instead (plan, decision 5). A
  forwarding thread would have kept the debug control alive and never ended.
- **Build, Reset, Restore and Pause each leave no breakpoint in the runtime,** so `AppState` keeps the session's and
  arms them at each Run and Step (plan, decision 4).
- **The fork's CI first ran no unit test:** its filter, `host::debug::breakpoints`, matched none of 4083 tests, and the
  step was green. The filter is now `debug::breakpoints::`, and a step that runs no test fails.

## What this does not prove

- **Conditions, hit counts and logpoints.** truST has them.
- **Breakpoints in blocks, and writing while held.**
- **Windows,** until CI's windows-2022 E2E.
- **A real GPU's soak.**
