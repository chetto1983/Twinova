# Plan: the points v0.3.0 left open (ROADMAP 2.9)

## Goal

Close what v0.3.0 left open:
- the Ladder view's four defects;
- breakpoints in Ladder files, set on rungs and stepped rung by rung;
- the agent's PLC commands reaching the pane in its own window;
- `setup` checked against a real Claude Desktop, by the user.

## Architecture

- **The Ladder view.**
  - `ladder/layout.ts` lays a box's valued pins out on their rows, with a stub wire and room for a reading.
  - `LadderRung.svelte` draws them there.
  - `layoutRungs` takes the pane's width.
  - `Editor.svelte`'s table starts lower.
- **Breakpoints.**
  - `twinova-ladder` gives each rung's span (`parse_spans`).
  - `ProgramFiles` builds the debugger's places, a Ladder file's with a line map.
  - `Debugger` maps lines through it, and keeps stepping while a stop stays in the rung a step left.
  - The webview drops its refusal, and the drawing gets the rung's breakpoint dot, its click and the held mark.
- **Routing.**
  - The registry reports its commands (`control_commands`), and both windows listen.
  - `control.rs` keeps each window's commands, answers `list_commands`, and sends every other request to its
    command's window.
  - The dock gains `plc_window`.

## Tech Stack

Rust (twinova-ladder, twinova-engine, the app), TypeScript and Svelte 5, truST's debugger API as pinned (`DebugControl`,
`resolve_breakpoint_location`), Tauri 2's `emit_to`.

## Spec

`docs/superpowers/specs/2026-11-24-open-points-design.md`.

## Global Constraints

- Every file stays at 600 lines or fewer.
- Coverage stays at 85 % or more, and mutation leaves no survivor (CI).
- The determinism tests stay green.
- No test is weakened.
- E2E runs on the real app.

## Review Focus

- The line map: every generated line maps back to a rung, and every rung line maps forward.
- Stepping keeps the engine's thread free: `held()` only steps again, it never waits.
- Routing never sends a request to a closed window.

## Decisions taken while writing this plan

- **The rung's span is returned beside the parsed program, not stored in `Rung`.** `Rung` derives `PartialEq`, and
  the round trips (parse, print, PLCopen) compare programs; a position would make equal rungs unequal.
- **The continued step lives in `Debugger::held`.**
  - The editor and `hold` poll it, and nothing else learns of a stop: the engine's scan is parked while it is
    paused.
  - Stepping again from there is where the stop is seen.
  - Run and Step from the toolbar, which wait for the scan, are unchanged.
- **A tie between windows goes to the PLC window.** It is the only one that can hold the same command, the pane's,
  and it does only while it is open.

## File structure

- `app/src-tauri/ladder/src/parse.rs`: `parse_spans`.
- `app/src-tauri/engine/src/program_files.rs`: `places()`.
- `app/src-tauri/engine/src/debugger.rs`: `Place` with a line map, the continued step; the map may go to
  `debugger_lines.rs`.
- `app/src-tauri/src/debug_commands.rs`: the refusal removed.
- `app/src-tauri/src/control.rs`: windows' commands, routing, `control_commands`.
- `app/src/lib/control/registry.ts`, `plcCommands.ts` (`plc_window`), `app/src/PlcWindow.svelte`: listen and report.
- `app/src/lib/ladder/layout.ts`, `app/src/lib/LadderRung.svelte`, `app/src/lib/LadderEditor.svelte`,
  `app/src/lib/LadderRungHead.svelte`, `app/src/lib/Editor.svelte`, `app/src/lib/debug.ts`,
  `app/src/lib/breakpoints.svelte.ts`.
- `app/e2e/mcp.mjs`, `app/e2e/ladder.mjs`.

## Tasks

- [x] **Task 1: the Ladder view.**
  - Layout tests first: the rows do not overlap, a valued pin has a stub, the width fills the pane.
  - Then `layout.ts`, `LadderRung.svelte`, the ResizeObserver, and the table's height.
  - Check: `npx vitest run src/lib/ladder`, then `ladder.mjs` and pictures of rung 3 on the real app. Commit.
- [x] **Task 2: breakpoints on rungs.**
  - `parse_spans`, with tests on every fixture.
  - The places from `ProgramFiles`, and the line map in the debugger, tested on `ladder_line`: set, hold, the
    declarations, Run to cursor, a condition.
  - The continued step, tested: Over from rung 3 holds on rung 4.
  - The refusal removed. Commit.
- [x] **Task 3: Ladder breakpoints in the view.**
  - `takesBreakpoints` goes, and the text view's gutter is back for `.ld` files.
  - The drawing gets the rung's dot, its click and the held rung, with vitest.
  - `e2e/mcp.mjs` gets its rung checks. Commit.
- [x] **Task 4: the commands go to their window.**
  - `control.rs`'s routing, tested.
  - The registry reports its commands; the PLC window listens; `plc_window` in the dock.
  - `e2e/mcp.mjs` drives the detached pane. Commit.
- [ ] **Task 5: documents.**
  - The verification with the pictures, CHANGELOG, ROADMAP 2.9, and what the user's `setup` run showed.
  - The pull request, merged when green.
