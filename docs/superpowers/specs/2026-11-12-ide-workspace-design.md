# Scene-first IDE workspace

The user requested a complete redesign of Twinova's desktop UI, grounded in Open Industry Project
and the seven screenshots supplied in `Documents/twinova-test/oip-screens` on 2026-11-12.

## Layout and interaction

- A compact top bar keeps build, play/pause, reset, single scan and speed together. The measured
  simulation/wall-clock ratio is sampled over at least 500 ms. Undo/redo and snapshot controls
  remain available as labelled icon buttons.
- Scene and History occupy the left sidebar. Rows show family icons, names, types, visibility and
  edit locks. Visibility and locks are editor-session state; they do not change simulation physics.
- The 3D viewport is central. The inspector occupies the full right edge. A resizable bottom dock
  spans the tree and viewport with Parts, PLC, Problems and Watch tabs. Collapsing the dock keeps
  the selected panel and edited sources alive.
- The inspector supports filtering, collapsible transform/parameter/size/guard/signal/connection
  sections, numeric fields with units and drag adjustment, and binding badges. `PLC` means a bound output
  signal; `TAG` means an input signal or an unbound tag. Clearing a numeric parameter restores its
  definition's default. A lock disables inspector mutations and detaches the transform gizmo.
- Catalogue thumbnails are rendered from the actual bundled component definitions and mesh
  generators, using one temporary WebGL context. Categories use existing placeable families;
  Structure explicitly has an empty state until structural components exist.
- A static warehouse supplies walls, beams, storage racks and floor markings. It is visual context,
  outside picking, snapping and physics. Geometry/material resources are disposed on rebuild.
- The PLC panel keeps an organizer, source tabs and the existing ST editor together. LD is visibly
  unavailable: this change does not introduce a Ladder editor or a Ladder compiler.
- Build errors open Problems. A failed build leaves Pause enabled for a previously running line.
- Build status is permanent in the status bar, alongside errors and scene edits. A failed compile
  with a working engine says "Previous build loaded". UI-authored open/save/export/source notices
  follow later locale changes; compiler diagnostics remain in their original language.
- Watch contains signals, operator controls, fault controls and measured PLC execution timing.

## Numeric controls revision

The user rejected the first thin-track sliders. The replacement uses compact filled number fields,
axis handles inside the X/Y/Z fields, separate unit suffixes and step buttons on hover/focus.
Drag the left handle to adjust, Shift for fine adjustment, click to type, arrow keys to step and
Escape to cancel. A drag commits once, preserving a single undo step. Bounds come only from the
component definition; unbounded coordinates do not invent a slider range. DINT remains integral.

Visual reference: [Figma Make properties panel, July 30, 2026](https://www.figma.com/blog/properties-panel-and-annotations-now-in-figma-make/).
Interaction rationale: [Fluent 2 slider guidance](https://fluent2.microsoft.design/components/web/react/core/slider/usage)
recommends numeric inputs when precision is needed. These references guide the implementation;
their code and visual assets are not copied into the app.

## Runtime timing

`scan_timing` returns the number of executed controller scans, the last/mean/maximum duration in
microseconds and the scheduler interval in milliseconds. The timer surrounds the existing
`harness.cycle()` call. It measures the entire controller cycle, including every task due in that
cycle, not a separately instrumented duration for each IEC task. It excludes scene rendering,
physics and field behaviour scans.

Timing is deliberately separate from deterministic `Snapshot` and replay fingerprints. Build,
Reset and successful online program replacement restart the statistics. Restore measures the
cycles executed while replaying the saved state. Watch polls only while visible, without
overlapping requests.

## Verification scope

Browser fixtures exercise layout, actual WebGL models and UI interactions without claiming to run
the PLC. Native tests exercise the real desktop bridge and the truST emulator. No external PLC or
new industrial connectivity is part of this UI change.
