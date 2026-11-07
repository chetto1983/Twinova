# Plan: Tauri shell with one conveyor scene driven by ST

## Goal
The first runnable Twinova app: one conveyor scene in 3D, driven by a Structured Text program edited and built in the
app. Spec: `docs/superpowers/specs/2026-11-07-tauri-shell-design.md`.

## Architecture
The Tauri 2 backend in Rust owns the engine:
- `scene`: closed-form conveyor;
- `plc`: truST harness wrapper;
- `engine`: lockstep step and run loop;
- `commands`: the Tauri commands.

The Svelte 5 + TypeScript web view only draws snapshots (three.js) and sends commands.

## Tech Stack
- Tauri 2, Rust 1.99.0 (pinned for trust-runtime), trust-runtime at `6405915973a7` with the `full` feature;
- Svelte 5, Vite, TypeScript, three.js, Paraglide (en, it), Vitest;
- the Aura tokens and fonts copied from chetto1983/ModbusTool.

## Global Constraints
- CLAUDE.md:
  - files ≤ 600 lines;
  - no dead code;
  - texts only through Paraglide;
  - colours only from the tokens.
- The UI never computes physics. It renders the backend snapshot.

## Review Focus
- The scan order of the lockstep step (spec §engine).
- Accumulation and outlet-block kinematics.
- That the engine never hides falling behind real time.

## Decisions taken while writing this plan
- Plain textarea editor in this slice; CodeMirror and Ladder later.
- `full` feature of trust-runtime accepted for the desktop build, because only wasm needed the cut-down version.
  Binary size is a known cost.
- The updater and file plugins of ModbusTool are not included. No release channel exists yet.

## File structure
```
app/                     Tauri app (npm project root)
  package.json vite.config.ts svelte.config.js tsconfig.json index.html
  project.inlang/settings.json messages/en.json messages/it.json
  src/main.ts src/App.svelte src/app.css src/assets/fonts/*
  src/lib/snapshot.ts (+ .test.ts)   types and pure helpers for the view
  src/lib/SceneView.svelte           three.js view
  src/lib/IoPanel.svelte  src/lib/Editor.svelte
  src-tauri/Cargo.toml build.rs tauri.conf.json capabilities/default.json icons/
  src-tauri/src/{main,lib,scene,plc,engine,commands,webview_locale}.rs
  src-tauri/src/default_program.st
```

## Tasks
- [x] 1. Scaffold `app/` (configs, Aura tokens, fonts with attribution, Paraglide, Tauri shell); `npm run build`, `cargo check`.
- [x] 2. `scene.rs` and its tests: spacing, accumulation, outlet block, photocell, delivery.
- [x] 3. `plc.rs` and its tests: build errors, missing contract variables, one scan.
- [x] 4. `engine.rs` and its tests: step order, the 60 s scenarios with the outlet free and blocked, pacing report.
- [x] 5. `commands.rs`, plus the run loop on a thread emitting `snapshot` events.
- [x] 6. UI: editor, I/O panel, three.js view, header actions, en/it texts; `npm run check`, `npm test`.
- [x] 7. E2E: `tauri build --debug`, launch under Xvfb, screenshot of the running scene; verification file.
