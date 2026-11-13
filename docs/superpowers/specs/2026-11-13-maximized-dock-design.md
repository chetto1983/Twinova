# Spec: the dock maximized over the workspace (ROADMAP 4.8a)

Date: 2026-11-13. Status: **approved** (user, 2026-11-13), decisions 1 to 5.

The user, 2026-11-13: "Bisognerebbe espandere la parte di PLC magari staccandola dalla pagina principale". Asked
between a maximize button and a window of its own, they chose "Pulsante massimizza prima, poi finestra separata".
This spec is the first part. The PLC pane in its own window is ROADMAP 4.8b, with a spec of its own after its
measurements (open questions at the end).

Accepted when:
- a button in the dock's bar spreads the dock over the scene and both side panels, and the same button gives the
  previous layout back;
- the toolbar (Run, Pause, Step, Build) and the status bar stay visible and working while the dock is maximized;
- the ST editor and the variables table take the new size, and monitoring goes on.

## What exists (inventory, 2026-11-13)

- **Layout** (`App.svelte`):
  - `.workspace` is a grid with columns of 224 px, `minmax(280px, 1fr)` and 292 px;
  - the scene tree sits in column 1 and the scene in column 2, both in row 1;
  - the inspector spans both rows of column 3, and the dock (`WorkspaceDock.svelte`) fills row 2 of columns 1–2;
  - the toolbar sits above `.workspace`, the status bar below.
- **The dock** (`WorkspaceDock.svelte`, 298 lines):
  - tabs Parts, PLC, Problems and Watch;
  - a height from 150 px to 60 % of the window, set by dragging its top edge or with the arrow keys;
  - a collapse button;
  - nothing is persisted between sessions.
- **The ST editor** (`StEditor.svelte`): Monaco with `automaticLayout: true`, so it follows its container's size
  (Monaco's `IEditorOptions.automaticLayout`).
- **What the two IDEs do** (public pages, partial; nothing official found on a float or maximize command):
  - TIA Portal V21's manual shows a group of editors opened as separate, overlapping windows from the editor bar's
    context menu ([docs.tia.siemens.cloud](https://docs.tia.siemens.cloud/r/it-it/v21/introduzione-al-tia-portal/interfaccia-utente-e-comandi/struttura-dell-interfaccia-utente/area-di-lavoro/utilizzo-degli-elementi-raggruppati-dell-area-di-lavoro));
  - CODESYS users report an editor tab floated out with "Float" or a double-click
    ([forge.codesys.com](https://forge.codesys.com/forge/talk/Engineering/thread/add38cdbc7)); a CODESYS-based IDE
    has "Undock/Dock" on the tab's menu ([WAGO eCOCKPIT](https://techdocs.wago.com/Software/eCOCKPIT/en-US/312662795.html)).

## Decisions

1. **The whole dock is maximized, whichever tab is open.** One button, one behaviour. The user asked for the PLC
   pane, but Watch (forces, I/O) is used with it while debugging.
2. **The dock covers the workspace instead of the grid changing.**
   - While maximized, the dock is laid over `.workspace` (`position: absolute; inset: 0`).
   - The scene keeps its size and keeps drawing behind it, so nothing in the scene, its camera or its resize changes.
   - Its CPU cost is today's with the dock open. Stopping the drawing while covered would be an optimisation, and it
     waits for a measured baseline (CLAUDE.md "BASELINE OR DROP").
3. **The button sits in the dock's bar, before collapse.**
   - The icons are `Maximize2` and `Minimize2`, from `@lucide/svelte`, already a dependency.
   - The labels are "Maximize panel" and "Restore panel", in English and Italian.
   - It is a button, so the keyboard reaches it. No shortcut is added: no IDE's shortcut was verified.
4. **While maximized, the resize edge and the collapse button are hidden.**
   - A collapsed dock that is maximized opens first.
   - Restoring gives back the height it had.
5. **Nothing is persisted:** the dock's height isn't today either.

## Test plan

- **E2E, in `workspace.mjs`:**
  - maximize from the PLC tab, then check:
    - the dock's rectangle equals `.workspace`'s;
    - the toolbar and the status bar are visible;
    - the Monaco editor is wider than before;
  - restore, and the dock's height equals the one before;
  - a screenshot of each state.
- **E2E, in `monitor.mjs`:** with Monitoring on and the dock maximized, Step still draws the values. This file has
  the built line (plan decision 1).
- **Unit:** none. The behaviour is layout, and its only state is one boolean in the component.

## What this does not prove

- The PLC pane in its own window (4.8b).
- Any CPU saving: the scene draws as it does today.

## Open questions for 4.8b (the PLC pane in its own window), to measure before its spec

- **Where the program's text lives.**
  - The editor's texts are webview state (`App.svelte`'s `sources`, edited by `Editor.svelte`), not Rust's.
  - A second window needs one owner of the text, and the two windows kept in step.
  - The candidates are moving the open texts into the app's Rust state, or syncing through Tauri events. This is
    measured on Tauri 2.12.5's installed `WebviewWindow` and event API.
- **Capabilities.** `capabilities/default.json` grants only the window `main`; a second label needs its own grant.
- **E2E.** Whether tauri-driver (WebKitWebDriver, and EdgeDriver on Windows) switches between the two windows.
- **Closing.** What closing the PLC window, or the main one, does to the other.
