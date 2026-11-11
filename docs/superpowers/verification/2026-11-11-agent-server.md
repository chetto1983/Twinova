# Verification: Twinova for coding agents (ROADMAP 2.7, 2026-11-11)

Spec: `2026-11-11-agent-server-design.md` (approved, with full scene editing). Plan: `2026-11-11-agent-server.md`.
Machine: the cloud container with 4 cores. CI: the 2.7 pull request.

## What runs

- **`twinova-engine`:** the engine, the line, the field, the controller and the drive, moved byte for byte out of the
  app. Build (the format's check, then a fresh engine) is `twinova_engine::build`, and the app's Build calls it.
- **`twinova-agent`:** a binary on rmcp 3.9.0.
  - `twinova-agent` serves MCP on stdio.
  - `twinova-agent --http [address]` serves streamable HTTP at `/mcp` (axum), on `127.0.0.1:7650` by default, with
    one engine per HTTP session.
- **The tools (23):**
  - run: `open_project`, `step`, `run_until`, `snapshot`, `read`, `press`, `block_sink`, `save_state`,
    `restore_state`, `reset`, `fingerprint`;
  - edit: `describe`, `set_parameter`, `bind_signal`, `add_instance`, `move_instance`, `remove_instance`, `connect`,
    `disconnect`, `read_source`, `write_source`, `build`, `save_project`.
- **`.mcp.json`** registers the stdio server for agents working in this repository (`cargo run -p twinova-agent`).

## The acceptance

| # | Criterion | Evidence | Result |
|---|---|---|---|
| 1 | On stdio and HTTP an MCP client runs `sort_line` until every sink counts, with the fingerprint the engine gives on its own | `agent/tests/transports.rs`: rmcp's client spawns the binary (stdio) and connects to the router on a loopback port (HTTP); the fingerprints equal the engine run directly to the same time | ✅ |
| 2 | Tools with their schemas; errors are tool errors and the server keeps serving | the tool list and the server's instructions; a missing project, a bad path, an unknown signal, a malformed condition, then the same session runs on | ✅ |
| 3 | Editing round trip | `agent/tests/editing.rs`: belt D added after belt C, the sink retargeted and moved, the turntable at 180°/s, the program rewritten, built (no warning), run to the sink, saved; the saved folder reopens and builds with `twinova_engine::build` and runs to the same fingerprint. A broken program fails Build with truST's message (`expected type at 21..22`) and the line runs on; a connection to a missing port fails Build and save with the format's message | ✅ |
| 4 | Two HTTP sessions keep two lines | `two_http_sessions_keep_two_lines_and_the_run_tools_drive_one` | ✅ |
| 5 | The split changes no result | app 24 + engine 76 = the app's 99 tests and Build's; physics 72, project 48; every fingerprint unchanged; the five E2E scripts pass on Linux | ✅ |
| 6 | CI on Linux and Windows | `ci.yml`: Windows now tests `twinova-engine` and `twinova-agent`, which no longer link Tauri | on the PR |
| 7 | Gates | fmt, clippy, the 600-line cap; cargo mutants: the agent's 120 mutants, every survivor closed (`get_info`, and `touches` for a mount's host); Build's 5 | ✅ |
| 8 | From this container | Claude Code drove the HTTP server with `curl`: `sort_line` to its first reject, 25.6 s of virtual time in 2.8 s with a debug build; an unknown signal came back as a tool error | ✅ |

## Mutation testing, made faster (user, 2026-11-11, after reading the cargo-mutants book)

Measured in place, per mutant, against the test profile:

| Crate (set) | Test profile: build + test | `mutants` profile (no debug info) | Gain |
|---|---|---|---|
| physics (`path.rs` spans, 17) | 0.73 + 2.07 s | 0.58 + 2.00 s | −8 % |
| engine (`drive.rs` pulse, 20 viable) | 1.74 + 10.63 s | 1.15 + 10.20 s | −8 % |

- **The `mutants` profile:** debug info costs the build a fifth to a third of its time, and nothing a mutant's failing
  test uses (https://mutants.rs/performance.html). The profile is now `.cargo/mutants.toml`'s, local and in CI. CI had
  the same effect through `CARGO_PROFILE_DEV_DEBUG=0`, which it replaces.
- **Pull requests mutate their own lines** (`--in-diff`, https://mutants.rs/ci.html). PR #36's diff has 300 of the
  workspace's 1 204 mutants. Before, mutation ran only after a merge, and a survivor showed on main.
- **`--baseline=skip`** in each shard: `ci.yml` already tests the workspace and the timeout is set, so each shard saves
  a full build and test (https://mutants.rs/baseline.html).
- **A bug fixed in passing:** the workflow's paths did not cover `engine/src` and `agent/src`, so a change to the engine
  would not have started a run.
- **Left as they are:**
  - `-j 2` (the book: start at 2 or 3);
  - copying the target dir, because shards start from the dependency cache;
  - nextest (the book: often slower for cargo-mutants itself).

  Most of a mutant's time is its tests (the engine's line scenarios, 10 s): making those faster is the next lever, not
  measured here.

## Found by measuring

- **truST gives the controller's compile error as a byte span without the file** (`expected type at 21..22`). The
  field locates its errors per behaviour file; the controller does not. The program is compiled from several sources,
  so locating it needs truST's offsets across files read first. This is a separate improvement, not done here.
- **The disk:** the workspace's debug target reached 17 GB with this crate and the mutation runs, and filled the
  container twice. Stale test binaries and the app's static library (2.2 GB) were the bulk; all are regenerable.

## What this does not prove

- **No 3D view, no screenshots:** those stay with the app's E2E.
- **No authentication on HTTP:** it is meant for the loopback address.
- **No undo, snapping or geometric checks on edits.** The format's checks run at Build.
- **The workspace coverage figure is CI's** (llvm-cov over the workspace, ≥ 85 % gate): an instrumented build did not
  fit the container's disk.
