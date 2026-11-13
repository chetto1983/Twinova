# Plan: the dock maximized over the workspace (ROADMAP 4.8a)

## Goal

A button in the dock's bar spreads the dock, PLC pane included, over the scene and both side panels, and gives the
previous layout back. The toolbar and the status bar stay.

## Architecture

- `WorkspaceDock.svelte` holds a `maximized` flag beside `collapsed` and `height`.
- While the flag is set, the dock is laid over `.workspace` with `position: absolute; inset: 0`, and `.workspace`
  becomes its containing block (`position: relative` in `App.svelte`).
- Nothing else moves: the scene keeps its size and draws behind it.

## Tech Stack

Svelte 5, `@lucide/svelte` (`Maximize2` and `Minimize2`, in the installed version), Paraglide messages.

## Spec

`docs/superpowers/specs/2026-11-13-maximized-dock-design.md`: approved by the user on 2026-11-13, decisions 1 to 5.

## Global Constraints

- Files stay under 600 lines. `App.svelte` is at 564 lines and gains one line.
- Gates:
  - svelte-check and oxlint, 0 warnings;
  - prettier, i18n, knip, jscpd;
  - `npm run build`;
  - the E2E of the touched files on Linux, under CI's screen.

## Review Focus

- **Stacking.** The maximized dock covers the scene's own overlays. The project menu's `z-index: 10` sits in the
  toolbar, outside `.workspace`, so it stays above the dock.
- **Monaco.** `automaticLayout` follows the new size: the E2E checks the editor's width, not only the dock's.

## Decisions taken while writing this plan

1. **The E2E is split by what it checks.**
   - The layout goes into `workspace.mjs`, beside the dock's other layout checks.
   - Monitoring while maximized goes into `monitor.mjs`, which has a built line with Monitoring on.
   - The spec's test plan is amended to match.

## File structure

| File | Change |
|---|---|
| `app/src/lib/WorkspaceDock.svelte` | the flag, the button, the maximized style |
| `app/src/App.svelte` | `.workspace` positioned |
| `app/messages/{en,it}.json` | `ui_maximize`, `ui_restore` |
| `app/e2e/workspace.mjs`, `app/e2e/monitor.mjs` | the checks |
| docs | spec amendment, verification, ROADMAP 4.8a and 4.8b, CHANGELOG |

## Tasks

- [x] **Task 1: the maximized dock.**
  - The E2E checks first:
    - the rectangle, the bars and Monaco's width;
    - the height restored;
    - values drawn while maximized.
  - Then the component, the style and the messages.
  - The gates, and the two E2E files.
  - Commit `feat(view): maximize the dock over the workspace`.
- [x] **Task 2: acceptance and documents.**
  - The verification file; ROADMAP 4.8a ✅, 4.8b listed; CHANGELOG.
  - Push; the pull request merged when CI is green.
