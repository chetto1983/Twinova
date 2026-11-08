# Changelog

Notable changes to Twinova, newest first. Each section is also the text of its
[GitHub release](https://github.com/chetto1983/Twinova/releases): the Release workflow
publishes the section whose heading matches the pushed `vX.Y.Z` tag. The full history is in git.

## v0.1.0 — unreleased

The first runnable slice: one belt conveyor in 3D, driven by a Structured Text program you edit
and build inside the app, on Windows and Linux.

### Added

- A desktop app (Tauri 2, Svelte 5) with a 3D belt conveyor: parts arrive every 2 s, ride the
  belt at 0.5 m/s, accumulate nose to tail and leave at the end; a photocell near the end.
- The belt is computed in closed form: photocell edges and deliveries happen at their exact time,
  to the nanosecond, instead of at the next 10 ms step. The program reads the photocell at its next
  scan, and its outputs apply from that scan on, as on a real PLC.
- Structured Text (IEC 61131-3) control: `PROGRAM Main` reads `sensor`, `start` and `stop` and
  writes `motor`. Build reports compile errors and missing contract variables; a runtime fault
  (such as a division by zero) stops the run with its message.
- Run against the wall clock, Pause, Step one 10 ms scan, Reset; the lag behind real time is
  shown, never hidden.
- An outlet-blocked fault to toggle, and a default program with a 2 s jam detection.
- Run speed: ×0.25 to ×10 the wall clock, or as fast as possible; Save state and Restore bring a
  run back to a saved moment (replayed exactly).
- The scan time comes from the program's IEC tasks (`TASK … (INTERVAL := T#5ms)`); 10 ms without one.
- English and Italian, light and dark themes.
- Installers: MSI for Windows, deb and AppImage for Linux, not signed yet.
