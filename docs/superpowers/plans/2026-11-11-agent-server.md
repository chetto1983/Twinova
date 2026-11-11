# Plan: Twinova for coding agents (ROADMAP 2.7)

## Goal

Agents (Claude Code among them) drive and edit Twinova through MCP, over stdio and over HTTP, on the same engine as the
app.

## Architecture

- **`twinova-engine`** (`app/src-tauri/engine`): the engine, the line, the field, the controller and the drive moved
  out of the app, unchanged, with Build (format check + engine) as one function the app and the agent share.
- **`twinova-agent`** (`app/src-tauri/agent`): a binary on `rmcp` (stdio; streamable HTTP mounted on `axum`). Its
  session owns a working project and an engine.

## Tech Stack

Rust 2024, rmcp 3.9.0 (Apache-2.0), axum (MIT), tokio (MIT), schemars (MIT), truST `6405915`, Rapier 0.36.1.

## Spec

`docs/superpowers/specs/2026-11-11-agent-server-design.md` (approved 2026-11-11, with full scene editing).

## Global Constraints

- **Determinism:** the split moves code, so every existing test and fingerprint is unchanged.
- **Files:** no file over 600 lines.
- **Tests:** the agent's tests use rmcp's own client on both transports; no sleep as sync.
- **Gates:** fmt, clippy, coverage ≥ 85 %, `cargo mutants` on the tool layer.
- **CI:** Windows tests `twinova-engine` and `twinova-agent` as it tests `twinova-project`.

## Review Focus

- The split changes no behaviour, and the app's Build keeps its rule: on failure, the running line stays.
- The tools' errors are tool errors, and the server keeps serving after one.
- HTTP binds the loopback address by default.

## Decisions taken while writing this plan

1. **Port 7650 for HTTP,** overridable by `--http <addr>`.
2. **`run_until` checks its condition after every millisecond,** bounded by `timeout_ms` of virtual time, so it cannot
   run forever.
3. **Values cross as JSON.** BOOL is a boolean, DINT and REAL are numbers, and TIME is nanoseconds, as the format
   stores it.

## File structure

| File | Change |
|---|---|
| `app/src-tauri/engine/` | new crate: `engine.rs`, `line.rs`, `field.rs`, `field_image.rs`, `controller.rs`, `drive.rs` and their tests, moved; `build.rs` (Build) new |
| `app/src-tauri/src/` | the app keeps its commands, project commands, demo and webview locale; `commands.rs` calls Build |
| `app/src-tauri/agent/` | new crate: `main.rs` (transports), `session.rs` (working project and engine), `tools_run.rs`, `tools_edit.rs`, tests |
| `app/src-tauri/Cargo.toml` | workspace members |
| `.github/workflows/ci.yml` | Windows tests the two crates |
| `.mcp.json` | the stdio server for agents in this repository |
| docs | verification, ROADMAP 2.7, CHANGELOG |

## Tasks

- [x] **Task 1: the engine crate.**
  - Move the files with `git mv`; fix the paths; Build as `twinova_engine::build`, which `commands.rs` calls.
  - `cargo test --workspace` gives the same counts and fingerprints; clippy and fmt clean; the app's E2E passes.
  - Commit `refactor(engine): the engine in a crate without Tauri`.
- [x] **Task 2: the agent and its run tools.**
  - Tests first, through rmcp's client on stdio and HTTP: open `sort_line`, run until every sink counts, read the
    counts, compare the fingerprint with `Engine` run directly; tool errors; two HTTP sessions.
  - Then the tools.
  - Commit `feat(agent): Twinova over MCP, stdio and HTTP, with the run tools`.
- [x] **Task 3: the editing tools.**
  - Tests first: the round trip of the spec's acceptance 3, refusals with their messages, Build failing with the line
    running on.
  - Commit `feat(agent): scene editing over MCP`.
- [x] **Task 4: CI, measurement and documents.**
  - Windows CI for the two crates.
  - Drive `sort_line` with `curl` over HTTP from this container.
  - `.mcp.json`, the verification file, ROADMAP 2.7 and the CHANGELOG.
  - PR, merge when green.
