# Verification: the PLC pane in a window of its own (ROADMAP 4.8b)

Date: 2026-11-13. Spec `2026-11-13-plc-window-design.md`, plan `2026-11-13-plc-window.md`.

## What was measured

- **E2E, `e2e/plc_window.mjs`, Linux under xvfb, debug build, the 4.4a line: 6/6.**
  - The dock opens the window and says where the pane went; no editor stays in the dock.
  - An edit in the window is built by the window's Build: the tab loses its edited mark and the main window reads
    "Built: ready to run".
  - The window's Run, Pause and Step drive the line: virtual time rises.
  - Monitoring in the window draws line 40's count while the line runs.
  - A force from the window's variables table is listed in the main window's Watch.
  - Back to the dock leaves one window, and the dock's editor holds the edit.
- **The suites the change touches, on Linux:** `monitor.mjs` 7/7, `workspace.mjs` 9/9, `editor_st.mjs` 6/6.
- **Closing the main window,** measured once (spec decision 6, test plan).
  - Setup: the window open, under xvfb; the main window closed with a window manager's `WM_DELETE_WINDOW` (python-xlib).
  - With the hook, no window is left.
  - The control build, with the hook's main-window arm off, leaves the PLC window open.
- **Rust:** `cargo test -p twinova plc_window`, 4/4: open, open again, close, close with none open, the event to main,
  the hook's decision.
- **Unit:** `plcWindow.test.ts`, 4/4. It covers the numbering (an echo of edit 1 is not taken while edit 2 is on its
  way; an old edit arriving late is dropped) and the events.

### The release soak: the 4.4a line, Linux under xvfb, 30 s per phase

`npm run soak`, with the new `SOAK_PLC_WINDOW=1`. Each WebKit web process is now reported apart (name and pid); before,
two of them would have been averaged into one row.

| Phase | Monitoring in the dock | PLC window, not monitoring | PLC window, monitoring |
|---|---|---|---|
| Run ×1: main web process | 278 % | 275 % | 266 % |
| Run ×1: PLC window's process | none | 0.1 % | 14.4 % |
| As fast as possible: virtual time in 30 s | 351.3 s | 363.1 s | 312.1 s (−11 % against the dock) |
| As fast as possible: PLC window's process | none | 0.2 % | 64.5 % |
| PLC window's RSS | none | 407-440 MB | 438-502 MB |

**Reading:**
- An open window that does not monitor costs its memory and no CPU. It gets no scene snapshots: Tauri delivers an
  event only to the webviews listening to it.
- Monitoring in the window costs what it costs in the dock, but in a process of its own.
- As fast as possible, that process competes with the engine. The container has 4 cores, and the main web process
  draws the scene in software (xvfb) on 2 to 3 of them. Virtual time falls by 11 %.

## Measurements that changed the design or the plan

- **WebDriver can't close a window as its user does.**
  - Neither `closeWindow` nor the page's `window.close()` fires `Destroyed`; `closeWindow` also leaves the label
    registered.
  - The spec's first figure for closing the main window came from `closeWindow`, and is amended.
  - The E2E closes the PLC window through the app, and closing the main window is measured with a window manager's
    close.
- **Tauri's mock runtime never emits `Destroyed`, and its `request_exit` is `unimplemented!()`.** So the hook's effects
  are covered by the E2E and the measurement above, and `plc_window.rs` is outside cargo mutants' scope (plan,
  decision 4).
- **A WebDriver click on Back to the dock answers "no such window",** since the click closes its window. The E2E
  clicks from the page.

## What this does not prove

- **Windows.** EdgeDriver's window switching is checked by CI's windows-2022 E2E, not here.
- **A real GPU.** With the scene drawn in hardware, the main web process frees cores, and the −11 % at full speed with
  monitoring in the window may shrink. It is not measured.
- **A remembered size and position,** a Problems row opening its line in the window, and a theme or language changed
  while the window is open (spec, "What this does not prove").
- **The remote branch `ccr-b0c12a7d-xlxkkd-plcwindow`.** It held these commits before 4.8a merged. The git proxy
  refused to delete it.
