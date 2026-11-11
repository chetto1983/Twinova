# Spec: Twinova for coding agents, an MCP server on stdio and HTTP (ROADMAP 2.7)

Date: 2026-11-11. Status: **approved** (user, 2026-11-11), with full scene editing in v1 (user's answer).

The user, 2026-11-11: build a native interface on stdio and HTTP so that coding agents, Claude Code included, can drive
Twinova and test everything. It comes before 4.4d. The server is headless, in a crate of its own (user's answers).

## Why

Today a line is checked in one of two ways:
- Rust tests, which cost a build and an edit for every question;
- the real app through WebdriverIO, which is slow and checks only what an E2E script foresaw.

An agent that can open a project, build it, run it, read and drive its I/O and compare states asks those questions in
seconds, on the same engine the app runs.

## Inventory (CLAUDE.md, "inventory before invention"; read 2026-11-11)

**MCP:** the official Rust SDK, `rmcp` 3.9.0, licence Apache-2.0, Rust ≥ 1.88 (Twinova: 1.99). Its installed source
under `~/.cargo/registry` has:
- the server and its tool system: `#[tool_router]`, `#[tool]`, `ServerHandler`, with argument schemas from `schemars`;
- the stdio transport, `transport-io` (`rmcp::transport::stdio`);
- the streamable HTTP server, `transport-streamable-http-server`. It is a tower service, `StreamableHttpService`, and
  `LocalSessionManager` gives each client session a service of its own.

**HTTP host:** rmcp does not ship one. Its own tests mount the service with `axum` (MIT):
`Router::new().nest_service("/mcp", service)`, then `axum::serve` on a tokio listener. Twinova does the same.

**The engine:** it already has the API the tools need:
- `Engine::new(project)` gives the line and Build's warnings;
- `step`, `catch_up`, `snapshot`;
- `press`, `set_sink_blocked`, `save` / `restore`;
- `controller().read(address)` and `variable(name)`.

`engine.rs`, `line.rs`, `field.rs`, `field_image.rs`, `controller.rs` and `drive.rs` do not use Tauri (grep: one
comment), and `twinova_project::storage::open_folder` opens a project.

**Editing, as the app does it:**
- the app's `build` command (`commands.rs`) checks an edited project the way the format checks it on open, by
  `parse_project(&encode_project(&project))`, then builds a fresh `Engine`;
- if either fails, the running line stays;
- `twinova_project::storage::save_folder` saves atomically;
- the app has no command yet that edits instances or parameters: the editor is 4.5.

**The gap Twinova writes:**
- the tool layer, each tool a few lines over the engine and the project model;
- the `twinova-agent` binary that serves it on stdio or HTTP;
- the engine moved into a crate without Tauri.

## Design

1. **The engine moves to its own crate, `twinova-engine`** (`app/src-tauri/engine`), with no Tauri, as
   `twinova-physics` and `twinova-project` are.
   - The app depends on it; its commands stay in the app.
   - The app's tests move with the code they test, unchanged but for the crate's name.
   - Windows CI can test the engine as it tests the project format, which the app's Tauri test binaries cannot.
2. **The binary `twinova-agent`** (`app/src-tauri/agent`) depends on `twinova-engine`, `rmcp`, `tokio` and `axum`.
   - `twinova-agent` serves MCP on stdio.
   - `twinova-agent --http 127.0.0.1:7650` serves it on streamable HTTP at `/mcp`.
   - Each HTTP session owns its engine, so two agents do not share a line.
   - It binds the loopback address unless told otherwise.
3. **The tools.** Each returns JSON; an error is a tool error with the engine's message.

   | Tool | Arguments | Returns |
   |---|---|---|
   | `open_project` | `path` | the project's name, instances and families, Build's warnings or its errors |
   | `step` | `ms` (default the scan period) | virtual time, sinks, actuators, signals |
   | `run_until` | a condition (`signal` = value, `sink` count ≥ n, or `time_ms`), `timeout_ms` | whether it held, at what virtual time, the state then |
   | `snapshot` | `cartons` (bool) | the engine's snapshot, cartons only when asked |
   | `read` | `names` or `addresses` | their values, as the I/O panel shows them |
   | `press` | `address` | — (the panel's button, for the next scan) |
   | `block_sink` | `sink`, `blocked` | — |
   | `save_state` / `restore_state` | — / `id` | an id / the state |
   | `reset` | — | the line rebuilt at time 0 |
   | `fingerprint` | — | the world's fingerprint, to compare runs |

   - Virtual time only advances when a tool asks: there is no wall clock. Run ×1 belongs to the app.
4. **Editing tools** (user, 2026-11-11: full scene editing).
   - They change the session's working project, not the running line. `build` checks the project as the format does
     on open and replaces the line at time zero. If it fails, the running line stays and the errors are returned. This
     is the app's own rule: an edit takes effect at the next Build.

   | Tool | Arguments | Effect on the working project |
   |---|---|---|
   | `describe` | — | its instances (definition, pose, parameters, signals), connections, definitions with their parameters, signals and ports, and source files |
   | `set_parameter` | `instance`, `name`, `value` | a parameter's value (`null` back to the definition's default) |
   | `bind_signal` | `instance`, `signal`, `address` or `null` | a signal bound to an address, or unbound |
   | `add_instance` | `id`, `definition`, `name`, `position`, `rotation`, `parameters`, `signals` | a new instance |
   | `move_instance` | `id`, `position`, `rotation` | its pose |
   | `remove_instance` | `id` | it and its connections |
   | `connect` / `disconnect` | a flow (`from` and `to` ports) or a mount (`device`, `host`, `at`) | a connection |
   | `read_source` / `write_source` | `file` (`plc/…st` or a definition's `components/…st`), `text` | the program or a behaviour |
   | `build` | — | as above: checked, then the line at time zero, with Build's warnings |
   | `save_project` | `path` (default the folder it was opened from) | the working project saved atomically |

   - An edit that names an unknown instance, definition, parameter, signal or file is refused with its message. The
     format's own checks run at `build`.
   - `build`'s check and engine are moved from the app's command into `twinova-engine`, and both the app and the
     agent call them: one way to build a project.
5. **Determinism.** The tools drive the same `Engine` as the app, so the same calls give the same fingerprint as the
   app and the tests.

## Acceptance

1. Through stdio and through HTTP, an MCP client opens `sort_line`, runs it until every sink has counted, and reads
   the counts. The fingerprint is equal to the one the same run gives on `Engine` directly.
   - The test client is rmcp's own (`client`, `transport-child-process`, `transport-streamable-http-client`), so
     both transports are exercised end to end in `cargo test`.
2. The tools list with their argument schemas. A bad path, a Build error or an unknown signal is a tool error with
   its message, and the server keeps serving.
3. Editing round trip: an agent adds a belt and a sink after `sort_line`'s belt C, retargets the sink connection,
   raises the turntable's rotation speed, rewrites the program to count at the new sink, builds, runs, and saves. The
   saved folder reopens, and in the app it builds and runs to the same counts. A broken edit (an ST syntax error, a
   connection to a missing port) fails `build` with the format's or the compiler's message, and the line runs on.
4. Two HTTP sessions run two lines independently.
5. The crate split changes no result: every existing test passes, the fingerprints unchanged, and the app's E2E
   passes on Linux and Windows.
6. CI builds and tests `twinova-agent` on Linux and Windows.
7. Gates: fmt, clippy, coverage ≥ 85 %, `cargo mutants` on the tool layer, the 600-line cap.
8. Measured in this container: Claude Code drives `sort_line` over HTTP with `curl`, and a project-scoped
   `.mcp.json` registers the stdio server for agents. The `.mcp.json` is the user's to approve.

## What this does not do

- **No 3D view and no screenshots.** Those stay with the app and its E2E. A server inside the app is a later step if
  wanted.
- **No authentication.** HTTP is for the loopback address; exposing it to a network is out of scope.
- **No undo.** An agent keeps its own copies with `save_project` to other folders, or reopens the project.
- **No snapping or geometric checks on edits.** A pose is taken as given, as in the file; the editor (4.5) brings
  snapping.
