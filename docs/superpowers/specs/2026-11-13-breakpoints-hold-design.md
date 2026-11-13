# Spec: breakpoints as HOLD (ROADMAP 4.9)

Date: 2026-11-13. Status: **proposed**, decisions 1 to 10 awaiting the user's approval.

ROADMAP 4.9: "Breakpoints as HOLD (research [17]): a breakpoint in the editor's gutter stops the program, its timers
and the simulated plant, as an S7-1500 in HOLD; Run, Run to cursor, step over/into/out and the call stack". It is
accepted by:
- measured first: what the runtime's cycle does at a breakpoint inside the deterministic engine;
- replay unchanged by a stop and resume.

## What exists (inventory and measurements, 2026-11-13)

### What the two IDEs do (research 17, §3)

- **TIA Portal, S7-1500:**
  - breakpoints in SCL and STL, with Run and Run to cursor; no step over, into or out;
  - at a stop the CPU goes to **HOLD**: the program, its timers, the watchdog and the cycle-time check stop, outputs
    keep their last value, inputs are not read;
  - breakpoints are not saved in the project.
- **CODESYS:**
  - breakpoints in every editor, with a condition, a hit count or one FB instance;
  - Step Into F8, Over F10, Out Shift+F10, Run to Cursor, Call Stack;
  - the task in debug stops (`HALT ON BP`), and its I/O is not updated.

### The runtime at the pin `6405915` (read in the installed source)

- **`DebugControl`** (`host/debug/control/api/`) has:
  - `set_breakpoints_for_file`, `clear_breakpoints`;
  - `continue_run`, `step`, `step_over`, `step_out`, `pause`;
  - `is_paused`, `last_location`, `frame_locations` (the call stack), `hold_reason`;
  - `with_snapshot`, the variables as they stood at the stop;
  - a stop channel, `set_stop_sender`.

  It is `Clone` (an `Arc`), so another thread holds it.
- **`Runtime::resolve_breakpoint_location(source, file_id, line, column)`** turns a line into the nearest statement.
  `file_id` is the file's place in Build's list.
- **At a stop, the hook blocks the scanning thread.** It waits on a `Condvar` (`control/hook.rs`, `cvar.wait`) until
  another thread calls `continue_run` or a step.
- **The matching rule.** At the pin, a statement matches a breakpoint when their spans **overlap**
  (`host/debug/breakpoints.rs`, `matches_breakpoint`).
  - An IF, FOR, WHILE, REPEAT or CASE spans its body, so a breakpoint in a body stops at the enclosing statement
    every scan.
  - Its once-per-scan guard then hides the body's own stop.
  - Upstream fixed it in **`c9635b1c`** (2026-10-18), "Stop a breakpoint in a loop or IF body in the body, not at the
    enclosing statement". A statement now matches when it *starts inside* the breakpoint's span: a one-line change, with
    tests. That commit is not an ancestor of the pin.

### Twinova

- **The Run loop holds the engine's mutex across `Engine::step`** (`commands.rs`, `run`). A stop inside a scan holds it
  until the stop ends.
- **A Tauri command without `async` runs in the invoke handler, on the main thread.** Tauri-macros 2.7.4,
  `command/wrapper.rs`, `ExecutionContext::Blocking`. Twinova's engine commands are synchronous, so one that waits for
  the held mutex freezes the app.
- **The build's file ids.** `Controller::build` compiles the program's files in the project's order, so a file's id is
  its index in `project.program`.

### Measured: a throwaway probe in a worktree outside the repository, release build, the 4.4a line

The engine stepped on a thread that held a mutex, as the Run loop does; the test thread drove the debug control. A
breakpoint was set on line 40 of `Main.st`, `count := count + 1;`, inside `IF entered.Q THEN`.

| What | At the pin | With `c9635b1c`'s rule (patched in the probe only) |
|---|---|---|
| First stop | at once, t = 0, on line 39 (the IF) | t = 5.34 s, on line 40, as the first carton enters |
| Next stop after Continue | the next scan (t = 10 ms), line 39 again | t = 7.34 s, the next carton (one every 2 s), line 40 |
| Steps from the stop | line 44, 45, 49 (the IF's body skipped) | line 41, 42, 44 |
| Engine time 300 ms into a stop | unchanged; the mutex held | unchanged; the mutex held |
| `Main.count` read from the paused snapshot, on another thread | — | 0 at the stop, 1 after one step; 1-7 µs a read |
| Fingerprint after 20 s with stops, steps and continues, against no debug | equal | **equal** (`f36c3907b1e0fd36`) |

| 20 s of virtual time, release | Wall time |
|---|---|
| No debug control | 0.91 s |
| Debug control attached | 1.01 s |
| Attached, one breakpoint never hit | 0.99 s |

**What these do not show:**
- the app's commands while held, and the UI;
- a block's file (the field's runtime);
- a breakpoint in a user FB called from the program (the 4.4a program calls only standard blocks), so the call stack
  shows one frame;
- Windows.

## Decisions

1. **Port `c9635b1c`'s matching rule into the pinned runtime, as the truST policy says** (spec
   2026-11-07-trust-upstream-policy-design.md, §1: "ported by hand, as a reviewed change with a test, citing the upstream
   commit").
   - The pin can't be edited in place, and vendoring `trust-runtime` means 286,000 lines, 25 MB and 35 workspace
     dependencies.
   - So the change lives in a **fork of trust-platform under the user's account**: a branch from `6405915` with the
     one-line rule and upstream's tests.
   - Twinova points at it with `[patch."https://github.com/johannesPettersson80/trust-platform"]`. It patches both
     `trust-runtime` and `trust-runtime-core`, so both come from one source.
   - **The user creates the fork:** this session can't create repositories.
2. **A breakpoint stops the whole simulation: HOLD.**
   - The engine's step blocks in the scan, so the program, its timers, the plant and virtual time all stand. That is an
     S7-1500 in HOLD, measured above.
   - The status bar reads "HOLD at Main.st:40".
3. **Breakpoints go in the program's files only.**
   - They are set from the PLC editor's gutter (a click on the glyph margin) or F9 on the cursor's line.
   - A line with no statement takes the nearest one (`resolve_breakpoint_location`).
   - A block's tab stays read-only (4.8), and its runtime is the field's, so it has no breakpoints.
4. **The app never waits for a held engine.**
   - `AppState` keeps a clone of the debug control outside the engine's mutex.
   - Continue, the steps and Run to cursor go through it.
   - While held, every engine command answers from the paused snapshot (values, call stack) or refuses with "The PLC
     is in HOLD: continue first". It never waits for the mutex.
   - The Run loop's snapshot events stop, since its thread is in the scan.
5. **The commands are VS Code's** (the user, 4.7: "tutte le funzionalità che offre VScode"):
   - Continue F5, Step over F10, Step into F11, Step out Shift+F11, toggle breakpoint F9;
   - Run to cursor Ctrl+F10: a one-shot breakpoint on the cursor's line, then Continue;
   - all of them in the PLC pane's bar, and in the PLC window's.
   - TIA's S7-1500 has only Run and Run to cursor; CODESYS uses F8 for Step into. The keys can follow CODESYS
     instead, if the user prefers.
6. **At a stop, the values are those of the stopped statement.**
   - The code's inline values (4.8) and the variables table read the paused snapshot. This is ROADMAP 4.8's "values
     as each line ran", at the stop.
   - The current line is highlighted, and a Call Stack list shows the frames.
7. **No write or force while held** (first version).
   - A write while held lands in the middle of a scan, and the input log replays at scan boundaries, so replay would
     differ.
   - The bar says to continue first.
8. **Breakpoints belong to the session.**
   - They are not saved, as in TIA.
   - Monaco's decorations move them with their lines while editing, and each Build sets them again on the new text.
9. **Reset, Build, a Restore and Pause end a HOLD first.** They clear the breakpoints in the runtime, continue the
   scan to its end, then act. The editor keeps the breakpoints and sets them again after a Build.
10. **The debug control attaches at the first breakpoint**, as at the first force (4.8). It is measured at +10 % of
    engine time with nothing hit.

## Test plan

- **Engine (Rust), on the 4.4a line:**
  - stop at line 40 when the first carton enters;
  - steps 41, 42, 44;
  - Continue to the next carton;
  - the fingerprint equals a run without stops;
  - a breakpoint on an IF's line stops once a scan, at the IF;
  - Reset while held unblocks and resets.
- **App (Rust):** while held, the engine commands answer from the snapshot or refuse, within a time limit (no wait on
  the mutex).
- **Unit (vitest):** breakpoint decorations and keys; the HOLD status text.
- **E2E, new file `breakpoints.mjs`:**
  - F9 on line 40 and Run: HOLD shown, virtual time stands, count's inline value at the stop;
  - F10 moves the line;
  - Continue to the next stop;
  - Run to cursor;
  - remove the breakpoint and Continue: the line runs;
  - Reset while held.
- **Soak:** the release build with one breakpoint never hit, against none.

## What this does not prove

- Conditions, hit counts and logpoints (truST has them; a follow-up).
- Breakpoints in blocks.
- Writing while held.
- A user FB's frames in the call stack, until a project calls one.
