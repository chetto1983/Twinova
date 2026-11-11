# Verification: the editor, first part (ROADMAP 4.5a, 2026-11-11)

Spec: `2026-11-11-editor-basics-design.md` (approved). Plan: `2026-11-11-editor-basics.md`.
Machine: the cloud container with 4 cores. CI: the 4.5a pull request.

## What runs

- **The library:** `app/src-tauri/library/components/`, one definition and block per family, embedded by
  `twinova_project::library` in catalogue order and group. The fixtures copy it.
- **Shared edits:**
  - `twinova_project::edit`: the agent's operations, moved, plus `pin`, `place`, `set_name`, `set_mount` and
    `set_flow`;
  - `twinova_project::history`: whole-project undo and redo, 100 deep, keeping the current ST sources;
  - the app's `edit`, `undo`, `redo`, `new_project` and `catalogue` commands;
  - the agent's `add_instance` places library families, and its `describe` lists them.
- **The view:**
  - a catalogue panel: click, or drag into the view;
  - an editable inspector;
  - picking;
  - three's `TransformControls`: translate on the floor, turn about +Y in 15° steps;
  - Delete, Ctrl+Z, Ctrl+Y and Ctrl+Shift+Z, W and E;
  - Undo and Redo buttons;
  - "New project";
  - "Frame all" frames the scene as it is now.

## The acceptance

| # | Criterion | Evidence | Result |
|---|---|---|---|
| 1 | The library: 15 families validate; every fixture's copy equals the library's | `project/src/library_tests.rs`: validation, the list against the folder, every fixture's copy byte-equal, every family used by a fixture the engine builds | ✅ |
| 2 | Shared edits | the agent's tests unchanged on the moved operations; `edit_tests.rs` for `place` (numbering, a pinned copy kept, a gap refilled), `set_name`, `set_mount`, `set_flow` (one peer, from either end, the replaced ends freed); the agent places an unpinned library family, and a refused add pins nothing | ✅ |
| 3 | History | `history.rs`: back to each earlier project and forward again; a new edit clears redo; a failed edit records nothing; depth 100; the ST sources survive undo and redo, a block comes back with its definition. `edit_commands_tests.rs`: 12 edits, 12 undo steps to the empty scene, redo to the same view; opening clears it, Build and Save keep it | ✅ |
| 4 | In the app, Linux and Windows: a line by hand from an empty project, built and run to the sink; undo to empty and redo; save and reopen | `e2e/editor.mjs`, 10 checks: 10/10 on Linux twice (WebKitGTK, debug build, Xvfb); windows-2022 (WebView2) in CI | ✅ Linux; Windows on the PR |
| 5 | Picking and the gizmo on WebKitGTK and WebView2 | the same E2E: a drag of the gizmo's x arrow moves the belt along x, and one Ctrl+Z puts it back; a click on empty floor clears the selection and one on the belt selects it, at points projected with the viewport's camera | ✅ Linux; Windows on the PR |
| 6 | Gates | fmt, clippy, the 600-line cap; cargo mutants on `edit`, `history`, `library`: no survivor (88 mutants, then the lines rechecked); Stryker 89.7 % (`inspector.ts` 85.5 %, `picking.ts` 97.1 %, `shortcuts.ts` 100 %); vitest 175 tests, 95.4 % of lines | ✅ |

## Found by measuring

- **Undo would have reverted the ST code.**
  - The text enters the project at Build, so a whole-project snapshot from before a Build holds the old text.
  - Undo and redo now keep the current program and blocks; a block whose definition an undo brings back comes back
    too.
- **The camera reframed at every edit,** because its target followed the scene's extent.
  - Now it frames when a project opens and on "Frame all".
  - The first fix left "Frame all" framing the scene as it was opened; the E2E's screenshot showed the built line out
    of view. It now frames the scene as it is.
- **The app's test mocked the backend without the catalogue.**
  - Vitest reported an unhandled error, with every test green.
  - Stryker's dry run crashed on it.
  - The mock now has it.
- **A survivor in `set_flow`:** replacing a port's peer could have dropped every other flow and no test noticed. One
  checks that the others stay.

## What this does not prove

- **Snapping by interfaces:** conveyor ends and mounts are set by hand; that is 4.5b, with the ROADMAP's acceptance
  (a two-conveyor line with a diverter by hand).
- **Upgrading a pinned family to the library's version:** M4.1's explicit upgrade, later.
- **Multiple selection, groups, copy and paste.**
- **The view's editing code (`sceneEditing.ts`) by unit tests:** it needs WebGL, so the E2E covers it.
- **A placed device's own position:** it is cosmetic, since it is drawn from its host. The inspector shows it, and
  4.5b's snapping will set it.
