# Plan: breakpoints as HOLD (ROADMAP 4.9)

## Goal

Breakpoints in the PLC program's gutter stop the whole simulation (HOLD), with:
- Continue, Step over, Step into, Step out, Run to cursor;
- the stopped line and a call stack;
- the values as the stopped statement sees them.

The app never waits for a held engine.

## Architecture

- **Engine, `engine/src/debugger.rs` (new).**
  - `Engine::debugger()` attaches truST's debug control, as a force does. It returns a `Debugger` that works without
    the engine.
  - The `Debugger` holds:
    - a clone of the `DebugControl`;
    - the program's texts;
    - each file's statement locations, so a line is resolved with
      `trust_runtime::debug::resolve_breakpoint_location`;
    - clones of the type registry, the date-time profile and the standard library, so a value is read from the paused
      snapshot.
  - It sets a file's breakpoints, says where it is held, resumes, reads values, and lets go (no breakpoints, running).
- **App:**
  - `AppState` keeps the `Debugger` beside the engine's mutex.
  - `with_engine` never waits while held: it tries the lock and answers "The PLC is in HOLD: continue first".
  - The commands that step the engine leave the main thread. A stop inside them would otherwise freeze the app, with no
    one left to resume it (spec, "What exists").
  - A thread forwards truST's stops as the event `hold`.
- **Webview:**
  - `debug.ts`: breakpoint decorations, keys, the HOLD text.
  - `StEditor.svelte`: the glyph margin, F9 and the stopped line.
  - `Editor.svelte`: the debug bar and the call stack.
  - The status bar moves out of `App.svelte` (598 lines) into `StatusBar.svelte`, where HOLD shows.

## Tech Stack

- Rust 2024, truST's runtime at `6405915`: `DebugControl`, `DebugBreakpoint`, `resolve_breakpoint_location`,
  `DebugSnapshot::evaluate_expression`.
- Tauri 2.12.5, Svelte 5, Monaco 0.57 (`glyphMargin`, decorations with `glyphMarginClassName`).

## Spec

`docs/superpowers/specs/2026-11-13-breakpoints-hold-design.md`: approved by the user on 2026-11-13, decisions 1 to 10,
with VS Code's keys.

## Global Constraints

- **Determinism.** Breakpoints are not inputs; a stop changes no state. A test keeps the fingerprint equal across
  stops.
- **No wait on the main thread.** No synchronous command may block on the engine while it is held.
- **Files** stay under 600 lines.
- **Gates:**
  - fmt, clippy;
  - the touched crates' tests;
  - svelte-check, oxlint, prettier, i18n, knip, jscpd;
  - vitest on the touched files;
  - coverage ≥ 85 %, cargo mutants on `debugger.rs` and Stryker on `debug.ts`, all in CI;
  - the E2E on Linux, and the whole suite with windows-2022 in CI.

## Review Focus

- **The matching rule** waits for the fork (decision 1). At the pin, a breakpoint on a top-level statement already
  stops on its own line. A breakpoint in an IF's body stops at the IF. Until Task 6, the tests use top-level lines.
- **A stop during a Step command.** The command runs off the main thread and returns when the scan ends, after the
  resume.
- **Field watches while held** read nothing: they are the field's runtime, not the PLC's snapshot.

## Decisions taken while writing this plan

1. **The `Debugger` resolves lines itself,** from statement locations copied at attach, so Run to cursor and breakpoint
   edits work while the engine is held.
2. **`with_engine` tries the lock in a loop** and checks the hold between tries. This covers a hold that starts after
   the check.
3. **The status bar is extracted from `App.svelte`** before HOLD is added to it (refactor on touch, 600-line cap).
4. **The session's breakpoints live in `AppState`, armed again at each Run and Step** (decided in Task 3).
   - Build, Reset and Restore replace the engine, and Pause lets a held scan go: each leaves no breakpoint in the
     runtime.
   - The editor sends its lines again after a Build, since lines move with edits.
6. **Breakpoints keep their line numbers while the text is edited** (a departure from spec decision 8, reported to
   the user).
   - Each Build sets them again on those numbers, and each resolves to its nearest statement.
   - Following edits needs the marks read back from Monaco's decorations on every change, which is left for later.
5. **The webview asks where the program is held** (`hold`, every 200 ms while breakpoints are set).
   - A stop sends no event, since its scan holds the Run loop.
   - A forwarding thread would keep the debug control alive, and never end.

## File structure

| File | Change |
|---|---|
| `app/src-tauri/engine/src/debugger.rs` (+ tests) | new |
| `app/src-tauri/engine/src/engine.rs`, `lib.rs` | `debugger()`, module |
| `app/src-tauri/src/debug_commands.rs` (+ tests) | new: commands, the hold event |
| `app/src-tauri/src/commands.rs`, `monitor.rs`, `lib.rs` | `with_engine`, async step, watched while held |
| `app/src-tauri/.cargo/mutants.toml` | `engine/src/debugger.rs` |
| `app/src/lib/debug.ts` (+ tests), `backend.ts` | new; the commands |
| `app/src/lib/StEditor.svelte`, `Editor.svelte`, `StatusBar.svelte`, `App.svelte` | gutter, bar, stack, HOLD |
| `app/messages/{en,it}.json` | strings |
| `app/e2e/breakpoints.mjs`, `package.json` | new E2E |
| `app/src-tauri/Cargo.toml`, `Cargo.lock` | the fork's `[patch]` (Task 6) |
| docs | verification, ROADMAP 4.9, CHANGELOG |

## Tasks

- [x] **Task 1: the engine's debugger.**
  - Tests first, on the 4.4a line, at top-level lines:
    - a stop at the first scan;
    - steps move the line;
    - Continue reaches the next scan;
    - a value read at the stop;
    - the fingerprint unchanged;
    - letting go runs free;
    - a line with no statement takes the nearest one.
  - Commit `feat(engine): breakpoints that hold the line`.
- [x] **Task 2: the app's commands.**
  - Tests first:
    - while held, `with_engine` answers HOLD within 100 ms;
    - Step stopped by a breakpoint returns after a resume;
    - Reset ends a HOLD;
    - watched values come from the snapshot.
  - Commit `feat(app): debug commands and the hold event`.
- [x] **Task 3: the webview.**
  - `debug.ts` tests first.
  - Then the gutter, F9, the stopped line, the bar and keys, the call stack, `StatusBar.svelte`, i18n.
  - Commit `feat(view): breakpoints, stepping and HOLD in the ST editor`.
- [x] **Task 4: the E2E.** `breakpoints.mjs`, the spec's list on top-level lines. Commit `test(e2e): breakpoints as HOLD`.
- [x] **Task 5: the soak.** A breakpoint never hit, against none, in release.
- [x] **Task 6: the matching rule (needs the user's fork).**
  - The fork's branch from `6405915` with `c9635b1c`'s rule and its tests, and the `[patch]`.
  - A test and an E2E check: line 40 in the IF's body.
  - Commit `fix(deps): stop a body breakpoint in the body (truST c9635b1c)`.
- [x] **Task 7: documents.** Verification, ROADMAP 4.9, CHANGELOG. The pull request, merged when green.
