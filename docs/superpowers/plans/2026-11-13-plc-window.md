# Plan: the PLC pane in a window of its own (ROADMAP 4.8b)

## Goal

A button opens the PLC pane in a window of its own, for example on a second monitor. The window has Build, Run/Pause,
Step and the editor with Monitoring. Closing it puts the pane back in the dock, and closing the main window quits.

## Architecture

- **Rust, `src/plc_window.rs`:**
  - the async commands `open_plc_window`, which focuses the window when it is open, and `close_plc_window`, which
    closes it as its title bar does;
  - a window-event hook: when `plc` is destroyed it tells `main` (`plc-closed`); when `main` is destroyed it closes
    `plc`, so the app quits.
- **The capability** `default` grants `plc` too.
- **Webview, `src/lib/plcWindow.ts`:**
  - the events between the two windows, typed;
  - the numbering that keeps an echo from undoing a newer edit (spec decision 2).
- **Webview, the windows:**
  - `main.ts` mounts `PlcWindow.svelte` in the window `plc`, and `App.svelte` elsewhere;
  - `PlcWindow.svelte` holds the mirrored state, the bar (`Transport.svelte`) and the `Editor`;
  - `App.svelte` sends its state while the pane is out, and runs the window's requests;
  - `WorkspaceDock.svelte` shows a placeholder in the PLC tab while the pane is out.

## Tech Stack

- Tauri 2.12.5: `WebviewWindowBuilder`, `Manager::get_webview_window`, `Builder::on_window_event`, `Emitter::emit_to`.
- `@tauri-apps/api` 2.12.4: `emitTo`, `listen`, `getCurrentWebviewWindow`.
- Svelte 5, Paraglide, `@lucide/svelte`.

## Spec

`docs/superpowers/specs/2026-11-13-plc-window-design.md`: approved by the user on 2026-11-13, decisions 1 to 8, with
Run, Pause and Step added to decision 3.

## Global Constraints

- Files stay under 600 lines. `App.svelte` is at 565 lines, so the windows' wiring lives in `plcWindow.ts`, and
  `App.svelte` gains only the state it sends and the requests it runs.
- One editor at a time. Only one is mounted, so only one watches.
- No new JavaScript permission: the window is opened from Rust.
- Gates:
  - fmt and clippy;
  - the touched crates' tests;
  - svelte-check, oxlint, prettier, i18n, knip, jscpd;
  - vitest on the touched files, coverage ≥ 85 % in CI;
  - Stryker on `plcWindow.ts` in CI;
  - the E2E on Linux, the whole suite in CI with windows-2022.

## Review Focus

- **The echo.** The window takes main's texts only when main has taken its last edit (decision 2). A unit test drives
  the order "edit 1, edit 2, echo of 1": the echo is not taken.
- **A request with its texts.** Build, Run and Step carry the window's texts and their number, so main applies them
  first.
- **Unmounting a monitoring editor.** The dock's editor unwatches when it unmounts. Today it unwatches only when
  Monitoring is turned off: refactor on touch, tested in the E2E.
- **Closing main.** Tauri's mock runtime never emits `Destroyed`, and its `request_exit` is `unimplemented!()`
  (tauri 2.12.5, `test/mock_runtime.rs`). So closing main is covered by the E2E, if WebDriver's `closeWindow` on `main`
  fires `Destroyed` (measured in Task 3).
  - The Rust tests cover the decision (`follow`), the open and close commands, and the event to main.

## Decisions taken while writing this plan

1. **The toolbar's transport buttons become `Transport.svelte`:** Build, Run/Pause, Step, and Reset when it is given.
   The toolbar and the PLC window use it, so the buttons are not duplicated.
2. **The window is a WebviewWindow on `index.html`, told apart by its label.** No second entry point, no route.
3. **The state sent is what `Editor` takes, plus the run state:** sources, status, problems, selected, instances,
   `running`, `ready` and `ack` (the number of the last edit main took).
4. **`plc_window.rs` is not in cargo mutants' scope.**
   - It is wiring between windows, not one of CLAUDE.md's critical files.
   - Its hook's effects cannot be observed on the mock runtime (Review Focus).
   - The numbering that is critical lives in `plcWindow.ts`, under Stryker.

## File structure

| File | Change |
|---|---|
| `app/src-tauri/src/plc_window.rs` (+ tests) | new: the commands and the window-event hook |
| `app/src-tauri/src/lib.rs` | module, commands, hook |
| `app/src-tauri/capabilities/default.json` | `plc` |
| `app/src/lib/plcWindow.ts` (+ tests) | new: events, numbering |
| `app/src/lib/Transport.svelte` | new: the transport buttons |
| `app/src/lib/Toolbar.svelte` | uses `Transport` |
| `app/src/PlcWindow.svelte`, `app/src/main.ts` | new root for `plc`; root by label |
| `app/src/App.svelte`, `app/src/lib/WorkspaceDock.svelte` | state sent, requests run, placeholder and button |
| `app/src/lib/Editor.svelte` | unwatch on unmount |
| `app/messages/{en,it}.json` | new strings |
| `app/e2e/plc_window.mjs`, `app/package.json` | new E2E in the suite |
| `app/e2e/soak.mjs` | `SOAK_PLC_WINDOW=1` |
| docs | verification, ROADMAP 4.8b, CHANGELOG |

## Tasks

- [x] **Task 1: Rust, the window's commands and hook.**
  - Tests first, on the mock runtime:
    - open builds `plc`, and opening again keeps one;
    - close asks an open window to close, and does nothing when none is open;
    - `plc` destroyed emits `plc-closed`;
    - the hook's decision for each window and event.
  - The capability.
  - Commit `feat(app): open and close the PLC window`.
- [x] **Task 2: the webview.**
  - `plcWindow.ts`, tests first: event wrappers; the numbering (the echo, a request's texts).
  - Then `Transport.svelte`, `PlcWindow.svelte`, `main.ts`, the App and dock wiring, the unwatch on unmount, i18n.
  - The gates.
  - Commit `feat(view): the PLC pane in a window of its own`.
- [x] **Task 3: the E2E.**
  - `e2e/plc_window.mjs`, the spec's list.
  - The whole suite on Linux under CI's screen.
  - Commit `test(e2e): the PLC window`.
- [ ] **Task 4: acceptance and documents.**
  - The release soak at Run ×1: the window open and monitoring, against closed.
  - Verification, ROADMAP 4.8b, CHANGELOG.
  - The pull request; merged when green.
