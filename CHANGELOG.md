# Changelog

Notable changes to Twinova, newest first. Each section is also the text of its
[GitHub release](https://github.com/chetto1983/Twinova/releases): the Release workflow
publishes the section whose heading matches the pushed `vX.Y.Z` tag. The full history is in git.

## v0.1.0 — unreleased

The first runnable slice: one belt conveyor in 3D, driven by a Structured Text program you edit
and build inside the app, on Windows and Linux.

### Added

- A desktop app (Tauri 2, Svelte 5) with a 3D belt conveyor: cartons arrive every 2 s, ride the
  belt at 0.5 m/s, accumulate and leave into a sink at the end; a photocell near the end.
- Transport is rigid-body physics (Rapier, deterministic across Linux and Windows): the belt moves
  cartons by friction, as in Open Industry Project, stepped every millisecond.
- Structured Text (IEC 61131-3) control: the line program reads and writes the scene's signals and
  the panel's buttons by IEC address (`sensor AT %IX0.0`). Build reports compile errors and warns
  about addresses no signal has; a runtime fault (such as a division by zero) stops the run with
  its message.
- Component behaviours (ROADMAP 4.3): each component's logic is an ST function block, edited in its
  own tab and rebuilt with the line. The belt's block drives its motor and reports `running`. A
  broken block fails Build with its file and line, and the line keeps running on the last good build.
- Run against the wall clock, Pause, Step one 10 ms scan, Reset; the lag behind real time is
  shown, never hidden.
- A blocked-sink fault to toggle, and a starter program with a 2 s jam detection.
- Run speed: ×0.25 to ×10 the wall clock, or as fast as possible; Save state and Restore bring a
  run back to a saved moment (replayed exactly).
- The scan time comes from the program's IEC tasks (`TASK … (INTERVAL := T#5ms)`); 10 ms without one.
- English and Italian, light and dark themes.
- The Twinova mark: application icons on every platform, and the logo and wordmark in the app's
  header, legible in both themes.
- The 3D view draws only when something changes, so an idle or paused app uses no CPU to render it.
- Projects (ROADMAP 4.2): open a project folder, save it, save it to another folder, export it as one archive and
  import one, through the system dialogs.
  - `twinova <folder>` opens a project folder at start, from a file manager or a script.
  - A save keeps each file's permissions, and gives a new file the usual ones.
  - An opened project shows its scene tree, a 3D view of its components sized from their parameters, and a read-only
    inspector with each signal's IEC address.
  - A save interrupted by a crash is rolled back when the project is next opened. Files changed outside Twinova are
    opened and listed.
  - An opened project builds and runs; the app starts on the bundled demo project.
- Installers: MSI for Windows, deb and AppImage for Linux, not signed yet.
