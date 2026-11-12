# IDE workspace acceptance — 2026-11-12

## Scope

Windows, Svelte 5, Three.js, Tauri 2 and the pinned truST runtime. Visual references are the
user-supplied OIP screenshots in `Documents/twinova-test/oip-screens`. The implementation is in
the existing desktop application; no separate product or hosted mockup was substituted.

## Frontend and interaction evidence

- `npm run check`: zero errors and warnings.
- `npm run lint`, `npm run deadcode`, `npm run dup`: passed.
- `npm run check:i18n`: no findings. The static unused-message analysis cannot resolve every
  dynamic import; it does not establish that every translation is used.
- `npm run coverage`: 22 files, 195 tests passed; line coverage 91.27%, statements 90.78%,
  branches 88.45%, functions 92.56%.
- `npm run build`: passed. Vite reports a main chunk of 816.16 kB, above the 800 kB advisory.
- Prettier on all changed frontend files: passed. The initial repository-wide formatting command
  flagged 77 files with formatting/line-ending differences. Only files changed by this task
  were formatted.
- The 600-line limit check covers 289 tracked and new source files; no violations.
- `node e2e/workspace-preview.mjs --reuse`: passed in headless Edge. It exercises the real UI
  components and WebGL meshes with an explicit Tauri fixture: property filtering and numeric
  edits, locks, visibility, PLC/Watch navigation, collapse/expand, keyboard resizing, language
  and theme changes. At the compact width there is no horizontal document overflow and all
  eight equipment previews load. Pointer resizing now focuses the handle for keyboard use.
- After the user's numeric-control feedback, the same browser acceptance also verifies drag
  adjustment, Shift+ArrowUp precision and Escape cancellation on the replacement number fields.

Browser captures are in `%TEMP%/twinova-ui-review`. This fixture does not run the PLC.

## Engine evidence

- `cargo test -p twinova-engine`: 93 tests passed, including deterministic replay, online
  program replacement and the new measured scan statistics.
- The statistics test feeds durations of 40, 120 and 20 microseconds and verifies last=20,
  mean=60 and maximum=120. The controller test verifies scans accumulate and reset on reload.
- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets -- -D warnings`: passed with no warnings.

## Desktop acceptance

The native Windows app builds with `CARGO_PROFILE_DEV_DEBUG=0`, using the dedicated
`%TEMP%/twinova-ui-build` target directory. The original debug build hit LNK1318 while writing
its PDB, and a first Clippy attempt ran out of disk space. The shared target directory was then
removed outside this run. Isolating the target and omitting debug symbols completed the build
and the full Clippy check. This is an executable dev build, not an installer/release bundle.

`node e2e/workspace.mjs`: **8/8 passed** against the real Tauri app and truST runtime:
layout, actual 3D previews, parameter editing, one-step undo for a drag and fine keyboard
increments, locking/deletion protection, PLC organizer, measured runtime costs/speed, and a
failed Build opening Problems while the previous line remains controllable. The final run also
asserts that the error and "Previous build loaded" are both visible in the permanent status bar.

The desktop driver uses `E2E_PORT=4484` with its native driver on 4485, because port 4444 was
occupied by another run. Helpers now accept that port and `CARGO_TARGET_DIR`, and reject a
driver that exits before connecting instead of attaching to an unrelated test session.

- Existing runtime acceptance (`run.mjs`): **11/11 passed**, including carton delivery,
  jam detection/stopping, save/restore, a measured 1.000× pace (5.97 s virtual/5.97 s wall),
  2.002× pace (8.97 s/4.48 s), language changes and edited/broken ST behaviour.
- Existing scene editor acceptance (`editor.mjs`): **10/10 passed**, including picking,
  gizmo movement, a hand-built line connected/bound in the inspector, real delivery,
  full undo/redo and save/reopen.
- Open/save/export/source notices now react to locale changes. The Windows project test exposed
  stale Italian text after switching to English; UI messages are evaluated with the current locale.
- The user correctly identified the status bar hiding a valid build behind an opening error.
  Build status is now independent of diagnostics. Tests assert the visible status bar, not the
  hidden editor footer, and check that the opening error remains visible alongside it.

- Existing project acceptance (`project.mjs`): **46/46 passed**, covering edited project
  detection, save/reopen, bindings, invalid project diagnostics and a usable fallback demo.
  The two added checks verify saved notices across English/Italian language changes.

- Existing snapping acceptance (`snapping.mjs`): **10/10 passed**. The old sink staging point
  projected outside the new viewport, so it was moved to visible floor. Drops now assert that
  their point is inside the canvas and that an instance was actually added. The spur's return
  handle was below the canvas; the scenario collapses the dock and reframes before dragging.
  The physical assertions are unchanged: both sinks count, the saved flows/mounts match the
  fixture, and the spur returns to z=2.385 m and delivers. No snapping algorithm was changed.

The user separately reported successful line/curve/sort/code_sort and vscode (5/5) scenarios
on the earlier 16:05 build. Those are user-provided evidence, not reruns claimed here.

The executable and current captures are saved to `Documents/twinova-test/ui-redesign/`.
The current artifact is **`Twinova-UI-r3.exe`**, built at 16:34 (33,732,608 bytes), SHA-256
`109E54DCA01AA5CCFBC5F9EE67CB3CDA33738F5F5E15E0635DF3300FF83FA5D4`.
It includes the r2 fixes and restores the localized `editor_edited` tooltip on modified ST tabs.
Svelte diagnostics, lint, the Editor formatting check and both frontend/native builds pass again.
Fresh native runs on r3 pass **project 46/46** and **snapping 10/10**. Driver logs and snapping
captures are in `%TEMP%/twinova-ui-native-results/r3-project` and `r3-snapping` respectively.
An independent rerun must copy the current `app/e2e` scripts and launch r3 explicitly: the
existing isolated tree's junction still selects the earlier file named `Twinova.exe`.
The previous r2 artifact remains available, SHA-256
`83904505BFE29A4A702486C1C8752C935695A1BE65866CC2C110ED00B28AC94E`.
Earlier `Twinova.exe` and `Twinova-UI-final.exe` remain available because the user was trying
those builds; they predate the final build-status correction. `failed-build-r2.png` captures
the running old build and the compile error displayed together.

## Limits

- Timing covers the complete controller cycle, including all tasks due in that cycle. It is
  not an individual duration for every IEC task, a rendering benchmark or a hardware PLC measurement.
- Ladder editing/execution remains unavailable. The LD control communicates that limitation.
- Structure has an empty state because the current component library contains no placeable
  structural families. The warehouse is visual context, not physics geometry.
- Visibility and locks belong to the current editor session; they are not serialized into projects.
- No scheduler, codec, kinematics or protocol algorithm changed. No new mutation/soak run or
  combined Rust/frontend coverage measurement is claimed for this UI change.
- Mutation testing is explicitly excluded from this commit/push at the user's request.
