# Plan: breakpoints, the rest of ROADMAP 4.9

## Goal

Complete breakpoints: conditions, hit counts and logpoints; breakpoints in blocks; writing a program variable while held,
replayed exactly.

## Architecture

- **Engine, `debugger.rs`.**
  - A `Breakpoint` carries its line and, optionally, a condition, a hit count (`Hit { op, count }`) and a log message.
  - The `Debugger` builds truST's `DebugBreakpoint` from it:
    - the condition and each `{expression}` are parsed with the runtime's registry;
    - each log expression goes as `TO_STRING(…)` when it reads, and bare when not (spec decision 3).
  - Its files become **places in runtime files**: a path, its runtime file id, and its first line in that file.
    - The PLC's files are one runtime file each, from line 1.
    - The blocks sit in the field's one generated source, at their spans' lines.
  - A field `Hold` names its device, from the stopped frame's instance and the field's device instances.
  - `write` parses a value as `watch::value_for` does, from the paused snapshot. It queues truST's write and shows it
    in the snapshot, and hands the engine an `Input::Write` through a shared queue.
- **Engine, `engine.rs`.** `step` first takes that queue into its input log, without applying it (spec decision 6),
  so `restore` replays it one step after the held one.
- **Engine, `field.rs`.** The field keeps its generated text, its spans and its devices' instances, for its
  `Debugger`.
- **App.** `AppState` keeps a PLC debugger and a field debugger.
  - `set_breakpoints` routes a block's file (`components/`) to the field.
  - `hold`, `resume` and `run_to_cursor` serve whichever is held or named.
  - `write_value` writes while held.
  - `debug_logs` drains both debuggers' logs.
- **Webview.**
  - Breakpoints become `{ line, condition, hit, log }`.
  - A right click on a dot, or Shift+F9, opens `BreakpointBar.svelte`.
  - The dot's class shows its kind.
  - The Log list sits in the PLC pane.
  - A block's tab takes breakpoints.
  - `Editor.svelte` (569 lines) hands its breakpoint state to `breakpoints.svelte.ts`.

## Tech Stack

truST's runtime at the fork's `efceb12` (`DebugBreakpoint`, `HitCondition`, `LogFragment`, `drain_logs`,
`enqueue_instance_write`, `VariableStorage::set_instance_var`), Svelte 5, Monaco 0.57.

## Spec

`docs/superpowers/specs/2026-11-13-breakpoints-complete-design.md`: approved by the user on 2026-11-13 ("Perfetto"),
decisions 1 to 7.

## Global Constraints

- **Determinism.** A write while held replays to the same fingerprint, measured by a test through `save` and `restore`.
- **The app never waits for a held engine** (spec 2026-11-13-breakpoints-hold-design.md, decision 4).
- **Files** stay under 600 lines.
- **Gates** as for 4.9. The E2E runs locally once, and in CI.

## Review Focus

- **The field's line mapping**, through the spans, not by hand as in the probe.
- **The write queue.** The engine logs a held write without applying it again.
- **Unqualified names in a condition** (`count = 2`). Whether they resolve in the stopped frame is measured in Task 1.
  If they do, no qualification is needed, for blocks either.

## Decisions taken while writing this plan

1. **Whether a log expression reads with `TO_STRING` is decided when the line is logged, not when it is set**
   (changed in Task 1).
   - Trying it at set time failed for names in the stopped frame (`count`), which do not resolve outside it, and in
     blocks.
   - Each expression is logged twice, through `TO_STRING` and bare, between control characters. The drained line
     keeps the first unless it is an error: the outcome of spec decision 3, in every frame.
2. **Names in a condition or a message need no qualification:** `count = 2` resolves in the stopped frame (measured
   in Task 1).

## File structure

| File | Change |
|---|---|
| `engine/src/debugger.rs` (+ `debugger_spec.rs` if it passes 600 lines, tests) | breakpoint kinds, places, write, device |
| `engine/src/engine.rs`, `field.rs` | the write queue; the field's debugger |
| `src/debug_commands.rs` (+ tests), `commands.rs`, `monitor.rs` | two debuggers, logs, writes while held |
| `src/lib/debug.ts` (+ tests), `breakpoints.svelte.ts`, `BreakpointBar.svelte`, `DebugBar.svelte`, `StEditor.svelte`, `Editor.svelte` | the webview |
| `messages/{en,it}.json` | strings |
| `e2e/breakpoints.mjs` | five checks |
| docs | verification, CHANGELOG |

## Tasks

- [x] **Task 1: the engine.** Tests first, then a commit.
  - Condition, hit count, logpoint (TO_STRING and bare), unqualified names.
  - A block breakpoint through the spans, naming `chain_transfer`.
  - A write while held, and the restored state's fingerprint.
- [x] **Task 2: the app.** Tests first, then a commit.
  - Block files to the field.
  - Logs drained.
  - A write while held accepted, a force refused.
- [x] **Task 3: the webview.** `debug.ts` tests first, then the bar, the dots, the Log, block tabs. Commit.
- [x] **Task 4: the E2E.** Five checks, local once. Commit.
- [x] **Task 5: documents.** Verification, CHANGELOG, ROADMAP note. The pull request, merged when green.
