# Plan: engineering foundation (ROADMAP 1.4, 2.1–2.3)

## Goal

Close the Definition-of-Done gaps of M1 (coverage ≥85 %, mutation ≥70 % on `scene`/`engine`, a Windows run) and put
in place the gates every later phase relies on: CI on Linux and Windows, coverage and mutation gates, the 600-LOC
check, lint, format, dead-code and duplication checks.

## Architecture

No new design: the set is taken from `chetto1983/ModbusTool` (user, 2026-11-07: "ModbusTool ha già le stesse
funzioni"), where it runs green on the same stack (Tauri 2, Svelte 5, Vite, Paraglide). Only the paths change, because
Twinova keeps the app under `app/` and the research at the root.

| ModbusTool | Twinova |
|---|---|
| `.github/workflows/ci.yml` (rust, frontend, file-size, bundle, e2e) | `ci.yml`: rust, **rust-windows**, frontend, file-size, build |
| `.github/workflows/mutation.yml` | the same, scope `scene.rs`, `engine.rs` and `src/lib/snapshot.ts` |
| `.github/actions/linux-deps` | the same, without `libudev-dev` (no serial port here) |
| `scripts/check-file-size.sh`, `memory-capped.sh` | the same; the cap also covers `.st` files (CLAUDE.md) |
| oxlint, prettier, knip, jscpd, Stryker, lefthook configs | the same, paths under `app/` |
| `commands_tests.rs` with `tauri::test::mock_app` | the same pattern for Twinova's commands |

## Tech Stack

cargo-llvm-cov 0.9.1, cargo-mutants 27.1.0, Vitest 5 with `@vitest/coverage-v8`, Stryker 10, oxlint 1, prettier 3, knip 6,
jscpd 4, lefthook 2. Rust 1.99.0 (pinned by `rust-toolchain.toml` for trust-runtime).

## Spec

`docs/superpowers/specs/2026-11-07-tauri-shell-design.md` (the code under test). This plan adds no behaviour.

## Global Constraints

- CLAUDE.md: coverage floor 85 %, mutation ≥70 % on critical files, no skip-as-green, 600-LOC cap, tests not edited to
  pass (a test may be strengthened).
- Actions pinned by commit SHA, as in ModbusTool.

## Review Focus

- The coverage figures are the whole crate and the whole `src/lib`, not a selection.
- What stays uncovered, and why.

## Decisions taken while writing this plan

- **Windows**: `cargo test` runs on `windows-latest`, and the release binary is built there. The E2E of the app on
  Windows (tauri-driver, as in ModbusTool) is not ported: Twinova has no WebDriver E2E yet (its M1 E2E was driven
  with xdotool).
- **Installers** are ROADMAP 2.4: CI builds the release binary with `--no-bundle` and prints its size for 2.5.
- **fast-check** is not added: knip flags it as unused, there is no TS codec to property-test yet.
- **Svelte components** stay outside the Vitest coverage, as in ModbusTool: the figure covers `src/lib/**/*.ts`.

## File structure

```
.editorconfig, .node-version, lefthook.yml
.github/actions/linux-deps/action.yml
.github/workflows/ci.yml, mutation.yml
scripts/check-file-size.sh, memory-capped.sh
app/.oxlintrc.json, .prettierrc.json, .prettierignore, .jscpd.json, knip.json, stryker.config.json
app/vite.config.ts                      coverage thresholds
app/src/lib/{backend,i18n,preferences}.test.ts
app/src-tauri/.cargo/mutants.toml
app/src-tauri/src/commands_tests.rs     commands on a mock app
```

## Tasks

- [x] **1. Frontend gates.** Add the dev dependencies and configs; `npm run lint`, `format:check`, `deadcode`, `dup`,
  `check:i18n` green; tests for `backend.ts`, `i18n.svelte.ts`, `preferences.ts`.
  Verify: `npm run coverage` ≥85 % on all four metrics; `npm run mutation` ≥70 %.
- [x] **2. Rust coverage.** `run` generic over `tauri::Runtime`; `commands_tests.rs` on `mock_app`; a PLC fault test
  (truST reports `DivisionByZero` from the cycle). Verify: `cargo llvm-cov --fail-under-lines 85`.
- [x] **3. Rust mutation.** `.cargo/mutants.toml` on `scene.rs` and `engine.rs`. Verify: `cargo mutants` ≥70 %.
- [ ] **4. CI and hooks.** `ci.yml`, `mutation.yml`, `linux-deps`, `lefthook.yml`, file-size script.
  Verify: every job green on the pushed SHA. (Open: the Windows test binary needs the app manifest.)
- [x] **5. Docs.** CLAUDE.md Gate 2 with the real commands; verification file; ROADMAP status.
