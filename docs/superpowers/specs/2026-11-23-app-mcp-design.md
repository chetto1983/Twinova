# Spec: the open app driven over MCP, with screenshots, and a release one can use (ROADMAP 2.8)

## The user's choices (2026-11-23)

- **"Nella release manca completamente Mcp … adesso è completamente inusabile".** v0.2.0's installers hold the app alone:
  the `.deb` holds `usr/bin/twinova`, its icons and its menu entry, and the MSI has no `twinova-agent` in it (both
  read 2026-11-23). The only MCP configuration, `.mcp.json`, runs `cargo run` in the repository.
- **"L'MCP deve pilotare l'app aperta, vai con tutto e fare gli screenshot".** An agent drives the window the person has
  open, sees what it shows, and every feature of the app is reachable over MCP, so that everything can be tested over
  MCP (the user, 2026-11-14).
- **Everything the audit of 2026-11-23 found is in scope:** the agent in the release, the app's MCP, the tools the agent
  lacks, the example lines, and a guide.

## What exists (inventory 2026-11-23, installed sources)

- **`twinova-agent`** (spec `2026-11-11-agent-server-design`): rmcp 3.9.0 on stdio and streamable HTTP (axum 0.8.9,
  `127.0.0.1:7650/mcp`). A session owns a project and an engine of its own: it never sees the app's window. Its tools,
  and the app's commands they lack:

  | The app has | Over MCP |
  |---|---|
  | program variables and instance members read, written, forced, released (`watch_range`, `write_value`, `force`, `release`) | no: `read` takes scene signals and I/O addresses |
  | breakpoints, HOLD, the steps, run to cursor, logpoints | no |
  | diagnostics as one types (`analyze`) | no: only `build`'s errors |
  | a new project, undo, redo, archive export and import, rename | no |
  | which panel, file and view the window shows | no |

- **Tauri 2.12.5** has no API to capture a window. `WebviewWindow::with_webview` hands the platform's webview:
  - Linux: `webkit2gtk::WebView` (2.0.2), whose `snapshot(SnapshotRegion, SnapshotOptions, …)` gives a cairo surface;
    cairo-rs 0.18.5 (MIT), already in the lockfile through webkit2gtk, writes PNG with its `png` feature;
  - Windows: `ICoreWebView2Controller` (webview2-com 0.39.1), whose `CoreWebView2().CapturePreview(PNG, stream, …)`
    writes a PNG.
- **The app's events to the webview:** `snapshot`, `monitor`, `engine-error`, `stopped` from the Run loop, and
  `disk-sources` from the folder watch. A Build, a step or an edit answers the webview that asked, and nobody else.
- **The library and the demo** are compiled into the binaries (`include_str!`, `include_bytes!`); the example lines are
  only the repository's test fixtures.

### Measured (a probe in the working tree, not committed, 2026-11-23)

| What | Result |
|---|---|
| `webkit2gtk::WebView::snapshot(Visible)` of the debug app under Xvfb, written with cairo's PNG writer | 1440 × 900 in 117 ms; the 3D view (WebGL) and every panel in it, as the window draws them |

What this does not show: Windows' `CapturePreview` (measured in Task 3 on windows-2022); a window hidden or minimized.

## Decisions

1. **One tool set, two hosts** (new crate `twinova-mcp`, rmcp): the tools' names, arguments and answers are written
   once, over a `Host` trait.
   - **The headless host** is today's session: a project and an engine of its own. `twinova-agent` serves it, and the
     tests and CI drive it without a display.
   - **The app host** is the open app: each tool calls the function the app's own command calls, on the app's state,
     then tells the window what changed (decision 3). What the person sees and what the agent drives are one line.
   - A tool that needs the window (decision 4) answers "this needs the app" on the headless host.
2. **The app serves MCP while it runs** on streamable HTTP at `127.0.0.1:7650/mcp`, the agent's address. When the port is
   taken, the next free one up to 7659; none free, no server, and the status bar says so. The status bar shows the
   address. Loopback only: any program of the computer's user can drive the app, as it can type into it. A setting turns
   the server off (on by default: the user asked for everything testable over MCP).
3. **The window follows the agent.** After a tool changes something, the app host emits what the window listens to: a
   `snapshot` after a step, a Build or a reset; `project-changed` after an edit, an open or a new project, on which the
   window reads the project again; `sources-changed` after a source is written or imported, on which the tabs take the
   new texts as the folder watch's do (a tab with edits of its own is held, with its notice); `hold-changed` after a
   breakpoint or a step.
4. **Tools of the window** (app host only):
   - `screenshot` (`window`: `main` or `plc`): the window as it is drawn, as an MCP image (PNG): Linux by WebKit's
     snapshot, Windows by WebView2's `CapturePreview`;
   - `show` (`panel`, `file`, `view` `text` or `ladder`, `maximized`, `monitoring`, `theme`): what the window shows, so a
     screenshot shows what the agent wants to see.
5. **The tools the agent lacks**, on both hosts, each the app's own function:
   - `watch` (variables and members by path, as the variables table reads them), `write`, `force`, `release`, `forces`;
   - `set_breakpoints`, `hold`, `resume` (`continue`, `over`, `into`, `out`), `run_to_cursor`, `logs`;
   - `diagnostics` (truST's analysis of the sources as they are, without a Build);
   - `new_project`, `undo`, `redo`, `export_archive`, `import_archive`, `rename`;
   - `open_example` (decision 6).
6. **The example lines ship in the binaries,** compiled in as the demo is (`demo.rs`): an example opens as the demo does,
   a project with no folder, which Save asks a folder for. The app's project menu lists them ("Open example"), and
   `open_example` opens one by name.
7. **The release carries the agent.** `twinova-agent` is built for each platform and goes:
   - into the MSI and the deb, beside the app, as Tauri's `bundle.externalBin` (a sidecar);
   - onto the release as a file of its own per platform, for the AppImage and for a computer without the app.

   `twinova-agent --app [address]` bridges stdio to the open app's HTTP server, for MCP clients that only start a
   process (rmcp's client and server, both already dependencies); `twinova-agent` alone stays the headless server.
8. **A guide** (`docs/guide.md`): install, the example lines, the scene, the PLC pane, ST and Ladder, online tools, and
   connecting an agent (Claude Code over HTTP, Claude Desktop through `twinova-agent --app`). The release text and the
   README link it.

## Test plan

- **`twinova-mcp`:** the agent's end-to-end tests (`agent/tests/*.rs`) unchanged, on the headless host; a test per new
  tool through a real MCP client.
- **The app host, end to end** (`e2e/mcp.mjs`, Linux and Windows): the debug app is started; an MCP client on its HTTP
  address opens an example, builds, runs, writes and forces a variable, sets a breakpoint and steps, edits the scene
  and a source; after each, the window is read through WebDriver and shows the change; `screenshot` answers a PNG
  of the window's size, and `show` changes what it shows.
- **The release:** the deb's and the MSI's contents listed in CI; the agent started from the installed path answers
  `tools/list`.

## What this would not prove

- macOS: neither the app nor the agent ships there.
- An agent and a person acting at the same moment on the same thing: the last one wins, as two people at one keyboard.
- The screenshot of a window covered by another or minimized (WebKit and WebView2 draw off-screen; not measured).
