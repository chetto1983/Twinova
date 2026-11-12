# IDE workspace implementation

## Goal

Replace the permanent ST column with a scene-first industrial editor based on the user's OIP reference.

## Architecture

Retain App's project/engine state and commands. Extract the toolbar and bottom dock; reuse the existing
tree, inspector, ST sources, I/O bridge, mesh generators and Three.js viewport.

## Tech Stack

Svelte 5, TypeScript, Three.js, Tauri 2, Rust, truST and Lucide icons (ISC/MIT).

## Spec

[Workspace specification](../specs/2026-11-12-ide-workspace-design.md).

## Global Constraints

Preserve project editing, undo/redo, source changes, both languages, both themes and deterministic
simulation snapshots. Keep changes separate from the pre-existing source-watcher work.

## Review Focus

Panel sizing, disabled/hidden state, GPU disposal, keyboard operation, build-error handling,
truthful timing and clear limits for unsupported Ladder/structural components.

## Decisions taken while writing this plan

Use the user's concrete screenshots directly. Implement in the existing desktop app. Do not create
a separate hosted prototype or a substitute simulation. Keep timing outside the replay state.

## File structure

`Toolbar.svelte`, `WorkspaceDock.svelte` and `NumberField.svelte` split shared UI concerns.
`warehouse.ts` and `partPreviews.ts` own the additional rendering work. `timing.rs` owns wall-clock
statistics; the existing controller and Tauri bridge expose them. Browser fixtures live in `e2e/`.

## Tasks

- [x] Read current app and reference screenshots.
- [x] Implement three-column workspace and bottom dock.
- [x] Implement inspector, tree controls, catalogue previews and warehouse.
- [x] Implement PLC organizer and measured cycle statistics.
- [x] Verify frontend build, static checks and unit coverage.
- [x] Exercise browser layout, filtering, editing, locking, visibility, tab navigation and resizing.
- [x] Run engine tests including statistics, online replacement and deterministic replay.
- [x] Complete native desktop acceptance and record its limits.

Acceptance evidence: [verification report](../verification/2026-11-12-ide-workspace.md).
The final build also preserves visible build status beside errors and translates existing UI
notices when the locale changes. Sink staging and spur dragging tests use points inside the
new viewport; their physics, delivery and saved-project assertions remain intact.
