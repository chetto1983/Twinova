# Verification: the dock maximized over the workspace (ROADMAP 4.8a)

Date: 2026-11-13. Spec `2026-11-13-maximized-dock-design.md`, plan `2026-11-13-maximized-dock.md`.

## What was measured

- **Layout, `e2e/workspace.mjs`, Linux under xvfb, 1440 × 900 window.**
  - Maximized from the PLC tab, the dock's rectangle equals `.workspace`'s: [0, 52, 1440, 873].
  - The Monaco editor is wider than before, and the toolbar and the status bar are displayed.
  - Restore gives back the dock's previous rectangle.
  - 9/9 checks.
- **Monitoring, `e2e/monitor.mjs`, on the 4.4a line.**
  - With the dock maximized and Monitoring on, a run as fast as possible raises the count drawn after line 40 of
    `Main.st`.
  - 7/7 checks.
- **Gates:**
  - svelte-check and oxlint: 0 warnings;
  - prettier, i18n, knip and jscpd: clean;
  - `npm run build`;
  - the 600-line cap: `WorkspaceDock.svelte` has 334 lines, `App.svelte` 565.

## The measurement that changed the code

- **The first run stopped at the inspector.** The dock reached [0, 52, **1148**, 873].
- **The cause.** An absolutely positioned grid item takes as containing block the grid area its `grid-column` and
  `grid-row` name (CSS Grid Level 1, §9.1). The dock's `grid-column: 1 / 3` left out the inspector's 292 px column.
- **The fix.** While maximized, the dock takes `grid-column: 1 / -1` and `grid-row: 1 / -1`.

## What this does not prove

- **The PLC pane in a window of its own:** ROADMAP 4.8b.
- **Any CPU change.** The scene draws behind the dock as it does with the dock open; no soak was run, since nothing in
  the scene or the run loop changed.
- **Windows,** until CI's windows-2022 E2E.
