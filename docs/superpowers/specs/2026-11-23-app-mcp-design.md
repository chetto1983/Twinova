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

## SolidCraft, read for its design (2026-11-23, the user: "Guarda come fa tangency-labs/solidcraft")

A Rust CAD (egui, wgpu; MIT OR Apache-2.0), read in its documents (`docs/mcp.md`, `docs/control-protocol.md`,
`docs/architecture.md`), not its code; ideas only, described here in Twinova's words:
- the open app listens on a local control channel (loopback TCP, one JSON message per line), on the first free port of
  a small range, with a token made at each launch and kept in a file only its user reads; the port is written to a file
  a client finds it by;
- one MCP server, a command-line binary on stdio: with no app it runs a session of its own, with one it attaches to it
  (found through that file, or named);
- everything the user does is a command of one registry, which the toolbar, the scripts and the channel run alike; the
  MCP tools list the commands, describe their parameters as JSON Schemas, run one, or run a batch that rolls back on a
  failure; a few tools are written for agents (inspect the design, undo, redo, screenshot);
- the screenshot is the app's window as drawn, as a PNG;
- the binary ships beside the app on the PATH, and a `setup` command writes the MCP clients' configurations.

The user chose this design for Twinova (2026-11-23).

## Decisions

1. **The control channel.** While it runs, the app listens on `127.0.0.1`, on the first free port from 27650 to 27659,
   one JSON message per line. At each launch it makes a 256-bit token and writes the port and the token to a file in
   the app's data folder that only its user can read (`control/current.json`); the first message of a connection
   carries the token, compared in constant time, or the connection is closed. A setting turns the channel off; it is
   on by default (the user: everything testable over MCP). The status bar shows the port.
2. **Everything the window does is a command of one registry, in the webview** (`src/lib/control/`): each has a name,
   a sentence, its parameters as a JSON Schema, and a function that does what the window's own button does, through the
   same code. A request from the channel reaches the webview as an event, the command runs there, and its answer goes
   back through a Tauri command; the window shows the result as if the person had clicked, so it never needs telling.
   The registry covers every action of the app: projects and examples, the scene's edits, Build and the run controls,
   the sources, online values and forces, breakpoints and the steps, PLCopen, what the window shows (panel, file, view,
   monitoring, theme, language), undo and redo. Queries answer the state as the window holds it: the project, the
   snapshot, the problems, the values, the hold.
3. **The screenshot** is taken in Rust, not in the webview: `screenshot` (`window`: `main` or `plc`) answers the window
   as drawn, as PNG; Linux by WebKit's snapshot (measured above), Windows by WebView2's `CapturePreview` (measured first
   on windows-2022).
4. **`twinova-agent` is the one MCP server** (stdio, rmcp):
   - with the app open, it attaches to it, found through `control/current.json` or named with `--connect PORT`, and
     serves `list_commands`, `describe_command`, `execute` (a command and its arguments), `batch` (commands in order;
     the first failure stops it, and the project's undo steps the batch made are undone), `screenshot`, and `state`
     (the project, the snapshot, the problems and the hold, in one answer);
   - with no app, it serves today's headless session and its tools, for CI and for a computer without a display;
   - `--http` stays, for clients that speak streamable HTTP.
5. **The example lines ship in the binaries,** compiled in as the demo is (`demo.rs`): an example opens as the demo
   does, a project with no folder, which Save asks a folder for. The project menu lists them, and the registry has
   `open_example`.
6. **The release carries the agent.** `twinova-agent` is built for each platform and installed with the app: in the
   deb in `/usr/bin` beside it, in the MSI beside the app's executable (Tauri's `bundle.externalBin`), and as release
   files of their own. `twinova-agent setup` writes the MCP configurations of Claude Code and Claude Desktop with the
   agent's own path.
7. **A guide** (`docs/guide.md`): install, the example lines, the scene, the PLC pane, ST and Ladder, the online tools,
   and connecting an agent. The release text and the README link it.

## Test plan

- **The channel:** a wrong or missing token closed; a request answered; a busy port skipped; the file written only for
  its user (Rust tests).
- **The registry:** every command's schema valid, and each command run against a stubbed backend (vitest).
- **End to end, the app driven over MCP** (`e2e/mcp.mjs`, Linux and Windows): the debug app is started, `twinova-agent`
  attaches to it, and an MCP client opens an example, edits the scene, builds, runs, writes and forces a variable, sets
  a breakpoint and steps, imports PLCopen, undoes; after each the window, read through WebDriver, shows the change;
  `screenshot` answers a PNG of the window's size.
- **Headless:** the agent's end-to-end tests (`agent/tests/*.rs`) unchanged.
- **The release:** the deb's and the MSI's contents listed in CI; the installed agent answers `tools/list`.

## What this would not prove

- macOS: neither the app nor the agent ships there.
- An agent and a person acting at the same moment on the same thing: the last one wins, as two people at one keyboard.
- The safety of the token against a program running as the same user: such a program can read the file, as it can
  read the person's projects.
- The screenshot of a window covered by another or minimized (WebKit and WebView2 draw off-screen; not measured).
