# Plan: the editor, first part — catalogue, placing, moving, editing, undo (ROADMAP 4.5a)

## Goal

In the app, a user builds a line by hand from an empty project:
- place families from a catalogue;
- move and turn them with a gizmo;
- edit names, parameters, signals, flows and mounts in the inspector;
- undo and redo every step;
- build, run and save.

## Architecture

- **`twinova-project` gains three modules, shared by the app and the agent:**
  - `library`: the families embedded from `app/src-tauri/library/components/`;
  - `edit`: the agent's operations, moved, plus `place`, `set_name`, `set_mount_at` and `set_flow`;
  - `history`: whole-project undo and redo.
- **The app:**
  - `ProjectState` holds the working project and its history;
  - new Tauri commands apply one edit each and return the view;
  - the view gains the flows and the ports.
- **The webview:**
  - picking in the 3D view, `TransformControls` on the selection;
  - a catalogue panel;
  - an editable inspector;
  - keys for delete, undo and redo;
  - "New project".

## Tech Stack

Rust 2024 (`twinova-project`, the app, `twinova-agent`), Tauri 2, Svelte 5, three 0.187 (`TransformControls`,
`Raycaster`, MIT).

## Spec

`docs/superpowers/specs/2026-11-11-editor-basics-design.md` (approved 2026-11-11).

## Global Constraints

- **The running line changes only at Build,** as now. An edit never touches the engine.
- **One gesture is one undo step:** a gizmo drag commits on release, a field on Enter or blur.
- **Clean-room:** OIP's editor read for ideas only; no code copied from it. `TransformControls` is three's.
- **Files:** none over 600 lines. `App.svelte` (314 lines) gives the catalogue and the edit state to their own components.
- **Library:** the fixtures' copies stay equal to the library; their scripts copy it.
- **Gates:**
  - fmt, clippy;
  - coverage ≥ 85 %;
  - cargo mutants on `edit`, `history` and `library`, with no survivor;
  - Stryker ≥ 70 %;
  - svelte-check, oxlint, knip, jscpd, i18n (English and Italian);
  - the E2E on Linux and Windows.

## Review Focus

- The agent's behaviour and tests are unchanged by the move. `add_instance` of a library family that is not pinned now
  places it.
- History:
  - opening, importing and creating a project clear it;
  - Build keeps it;
  - a failed edit records nothing.
- Picking selects the instance under the cursor, a mounted device included (it is drawn from its host's pose).
- Keys are ignored while focus is in a text field or the ST editor.

## Decisions taken while writing this plan

1. **The library's file names are the format's:** `components/<key>.json` and `<key>.st`. `library.rs` embeds them
   with `include_str!`, one line per family, listed explicitly. A test compares the list with the folder, so a family
   added without its line fails it.
2. **Display groups:**
   - conveyors: belt, roller, belt curve, roller curve, turntable;
   - devices: chain transfer, stopper, pusher;
   - sensors: diffuse, through-beam, colour, laser, barcode scanner;
   - flow: source, sink.
3. **A placed mounted device is added unmounted;** the inspector mounts it. A new mount defaults to the host's middle
   (4.5b snaps).
4. **A new project's id and name:**
   - name "Untitled";
   - an empty scene;
   - `plc/Main.st` holding `PROGRAM Main\nEND_PROGRAM\n`;
   - no folder: Save asks for one, as Save As does today.
5. **`edit` returns the operation's own error,** and the app shows it in the status line, as Build errors are shown.
6. **The heading field is degrees about +Y.** Writing it sets the rotation `[0, sin(θ/2), 0, cos(θ/2)]`; a rotation
   with tilt (none in the library today) shows its heading and keeps the tilt only if the field is untouched.

## File structure

| File | Change |
|---|---|
| `app/src-tauri/library/components/*` | new: 15 families, definition and block |
| `project/src/library.rs` (+ tests) | new: the families, by group |
| `project/src/edit.rs`, `edit_tests.rs` | moved from the agent, plus `place`, `set_name`, `set_mount_at`, `set_flow` |
| `project/src/history.rs` (+ tests) | new |
| `agent/src/{lib,tools_edit}.rs` | use `twinova_project::edit`; `describe` lists the library |
| `src/project_commands.rs`, `src/edit_commands.rs` (+ tests) | the working project's history; edit, undo, redo, new |
| `src/lib/project.ts`, `backend.ts` | flows, ports; edit commands |
| `src/lib/Catalogue.svelte` | new |
| `src/lib/Inspector.svelte`, `src/lib/inspector*.ts` | editable fields |
| `src/lib/ProjectScene.svelte`, `Viewport.svelte`, `src/lib/picking.ts` | picking and the gizmo |
| `src/App.svelte`, `src/lib/shortcuts.ts` | the catalogue panel, keys, the "changed since Build" mark |
| `messages/{en,it}.json` | the new strings |
| fixture scripts | copy the library |
| `e2e/editor.mjs` | new |
| docs | verification, ROADMAP 4.5a, CHANGELOG |

## Tasks

- [x] **Task 1: spec and plan** (docs commit).
- [x] **Task 2: the library.**
  - Extract the 15 families from the fixtures into `library/components/` (a script; they are byte-equal).
  - `library.rs`; tests:
    - each family parses and validates as a definition;
    - every family is used by a fixture the engine's tests build and run, so its block compiles in the field (the
      format crate has no ST compiler);
    - every fixture's copy equals the library's.
  - Commit `feat(project): the component library, embedded`.
- [x] **Task 3: shared edits and history.**
  - Move `edit.rs` and its tests to `twinova-project`; the agent's tests pass unchanged.
  - Tests first for `place`, `set_name`, `set_mount_at`, `set_flow` and `History`.
  - The agent's `describe` lists the library; its `add_instance` places library families.
  - Commit `feat(project): edits and undo shared by the app and the agent`.
- [ ] **Task 4: the app's edit commands.**
  - Commands: `edit` (one operation, tagged), `undo`, `redo`, `new_project`.
  - The view carries flows and ports.
  - Tests in `edit_commands_tests.rs`:
    - each operation through the command;
    - history cleared on open;
    - Build keeps it.
  - Commit `feat(app): edit the open project, with undo`.
- [ ] **Task 5: the catalogue and the inspector.**
  - The catalogue panel: click and press-drag to place.
  - The inspector's fields by type; signals; flows and mounts.
  - Vitest on the field helpers; i18n.
  - Commit `feat(view): the catalogue and the editable inspector`.
- [ ] **Task 6: picking and the gizmo.**
  - Ids on the instance groups; the `Raycaster`; `TransformControls`, translate (x, z) and rotate (+Y, 15°), W and E.
  - Delete, Ctrl+Z, Ctrl+Y.
  - Measure on WebKitGTK (here) and WebView2 (CI E2E).
  - Commit `feat(view): pick, move and turn in the 3D view`.
- [ ] **Task 7: the acceptance and documents.**
  - `e2e/editor.mjs`, the spec's acceptance 4, on Linux and Windows.
  - Mutation, coverage, Stryker; the verification file; ROADMAP 4.5a; CHANGELOG.
  - PR; merge when green.
