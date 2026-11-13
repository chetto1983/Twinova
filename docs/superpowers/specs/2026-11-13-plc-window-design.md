# Spec: the PLC pane in a window of its own (ROADMAP 4.8b)

Date: 2026-11-13. Status: **approved** (user, 2026-11-13), decisions 1 to 8, decision 3 with Run, Pause and Step added by the user.

The user, 2026-11-13: "Bisognerebbe espandere la parte di PLC magari staccandola dalla pagina principale", then
"Pulsante massimizza prima, poi finestra separata". 4.8a, the maximized dock, is pull request #46. This spec is the
second part: the PLC pane in a window of its own, for example on a second monitor.

Accepted when:
- a button opens the PLC pane in its own window and the dock's PLC tab says where it went;
- an edit, a Build, Monitoring and a force work from that window as they do in the dock;
- closing it puts the pane back in the dock;
- closing the main window closes it too.

## What exists (inventory and measurements, 2026-11-13)

### The code

- **Who owns the program's text.** The webview, not Rust.
  - `App.svelte` holds `sources`: each tab's text and the text it was built from.
  - The analysis, Build, the folder's changes (`fromDisk`) and the scene's edits (`mergeSources`) all go through it.
  - `Editor.svelte` takes `sources` (bindable), `status`, `problems`, `reveal`, `selected` and `instances`.
- **Monitoring.** `Editor.svelte` calls the backend itself: `watch_range`, `force`, and the `monitor` event.
  - The engine holds **one** watch list (`Engine::watch`).
  - Two mounted editors would replace each other's list.
- **Windows.** One window, `main` (`tauri.conf.json`). The capability `default` grants `core:default` and the dialogs to
  `main` only.

### Tauri 2.12.5, read in the installed crate and package

- **Building a window.**
  - `WebviewWindowBuilder::new(app, label, WebviewUrl::App("index.html"))` builds a window.
  - Its documentation: "On Windows, this function deadlocks when used in a synchronous command", so the command that
    opens the window is `async`.
- **Creating a window from JavaScript** needs `core:webview:allow-create-webview-window`, which `core:default` does
  not include. A Rust command needs no new permission.
- **Event delivery.** `event/listener.rs` (`emit_js_filter`) delivers an event only to the webviews with a listener
  for its name. A window that does not listen to `snapshot` is not sent the scene's snapshots.
- **Targeted events.** `@tauri-apps/api/event` `emitTo(label, event, payload)` sends to one window.
  `getCurrentWebviewWindow().label` tells a webview which window it is in.

### Measured (throwaway probe in a worktree outside the repository, debug build, Linux under xvfb, the 4.4a line)

The probe opened a window `plc` from an async command. It mounted the real `Editor.svelte` there, and mirrored the
state through `emitTo`: main to `plc` with the whole state, `plc` to main with the sources on each edit.

| What | Figure |
|---|---|
| WebKitWebDriver with two windows | `getWindowHandles` gives 2; `switchToWindow` reaches each |
| The editor in the second window | Monaco, the variables table and Monitoring work; 14 values drawn while the line ran |
| An edit in `plc` | reaches the main window's `sources` and its editor |
| A Build from main | reaches `plc`'s status |
| The sources of the 4.4a line | 16,266 bytes of JSON |
| Round trip main → `plc` → main, 200 messages | 100 B: median 3 ms, p95 8 ms; 10 kB: 4 ms, 6 ms; 100 kB: 11 ms, 15 ms |
| Second webview at Run ×1, not monitoring | 0.1 % CPU, 419 MB RSS |
| Second webview at Run ×1, monitoring | 23.9 % CPU, 440 MB RSS |
| Main webview, the same 10 s | 260-277 % CPU with or without the second (three.js on xvfb's software GL) |
| Closing `plc` with `Window::close` (as its title bar does) | the label is freed; it opens again |
| Closing `plc` with WebDriver's `closeWindow` | the label stays registered: reopening fails, "a webview with label `plc` already exists" |
| Opening `plc` while it is open | the same error |
| Closing the main window as a window manager does (`WM_DELETE_WINDOW`), with `plc` open and no hook | `plc` stays open (amended in Task 3: the first figure came from WebDriver's `closeWindow`, which is no user's close) |
| The page's `window.close()` | closes nothing |

**What these do not show:**
- **Windows.** Whether EdgeDriver switches between the two windows is not measured: no Windows machine here. CI's
  windows-2022 E2E is the first measurement.
- **A release build's memory and CPU.** The figures above are a debug build under xvfb. The release soak measures them
  in the verification.

## Decisions

1. **The pane lives in one window at a time.**
   - A button in the PLC tab's bar opens it in the window `plc`.
   - While it is there, the dock's PLC tab unmounts its editor and shows "The PLC is in its own window", with a button
     to bring it back.
   - Reason: one editor, since the engine watches one list.
2. **The main window keeps the text.**
   - `App.svelte` stays the owner of `sources`, the analysis, Build and the folder's changes.
   - Main sends the window, with `emitTo("plc", …)`, the state the editor takes: sources, status, problems, the
     selected instance and the instances.
   - The window sends main the sources after each edit, numbered.
   - Main's state names the last number it took. The window takes main's texts only when main has taken all of its
     edits, so an echo never undoes what was typed since.
   - Main's own change of a text (Build marks it built, or the folder changed) made while an edit is still on its way
     is overwritten by that edit. The window is a few milliseconds behind.
   - Measured at 4 ms round trip for 10 kB, against 16 kB of sources for the 4.4a line.
   - Moving the texts into Rust would rewrite `fromDisk`, `mergeSources` and Build for no measured need.
3. **The window's bar has Build, Run/Pause and Step (user, 2026-11-13), and the editor's own Monitoring toggle.**
   - The window asks main for each (an event), so one Build and one run state exist.
   - The request carries the window's texts, so main builds what the window shows, whatever order the events arrive
     in.
   - Reset, the speed, Problems and Watch (forces) stay in the main window.
   - The buttons are the toolbar's, moved into a component both windows use.
4. **The window is opened by an async Rust command** (`open_plc_window`), as Tauri's documentation requires on
   Windows.
   - The capability `default` grants `plc` too.
   - No JavaScript window-creation permission is added.
   - Opening it while it is open brings it to the front.
5. **Closing the window puts the pane back.** Main listens for the window's destruction and remounts the editor in the
   dock. The editor's open tab and Monitoring state are lost; the text is not, since main holds it.
6. **Closing the main window closes the PLC window and quits.** Without it, a window manager's close of the main window
   leaves the PLC window open, and the process with it (measured, the control run of Task 3).
7. **The window's size and position are not remembered** (first version). It opens at 1000 × 700 px.
8. **The E2E closes the window through the app.**
   - It uses the window's own "Bring back" button, and the title bar's close through `Window::close`.
   - It never uses WebDriver's `closeWindow`, which leaves the label registered (measured).

## Test plan

- **Rust:** a test of `open_plc_window` on Tauri's mock runtime, if it builds windows; otherwise the E2E covers it
  (checked in Task 1).
- **Unit (vitest):** the state mirrored between the windows, kept in a helper:
  - an echo of the window's own edit does not loop;
  - main's newer text wins after a Build.
- **E2E, new file `plc_window.mjs`, on the 4.4a line:**
  - open; the dock's PLC tab says where the pane went;
  - edit in the window and Build from its bar: the main window built it;
  - Monitoring on in the window with Run from main: values drawn;
  - a force from the window: listed in main's Watch;
  - bring back: the dock's editor holds the edited text.
- **Closing the main window, measured once.**
  - WebDriver can't close a window as its user does: `closeWindow` and `window.close()` fire no `Destroyed`.
  - So the main window is closed with a window manager's `WM_DELETE_WINDOW` (python-xlib, under xvfb).
  - This is measured once, with a control run without the hook, and recorded in the verification.
- **Soak:** the release build at Run ×1 with the window open and monitoring, against the window closed.

## What this does not prove

- A Problems row opening its line in the PLC window: the row opens the dock's PLC tab, which says where the pane is.
- A theme or a language changed while the window is open: the window reads both when it opens.

- Two PLC panes at once, or the scene in a window of its own.
- Windows' window switching, until CI's windows-2022 E2E runs.
- A remembered size and position.
