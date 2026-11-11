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
- A roller conveyor and a chain transfer (ROADMAP 4.4a), copied and translated from Open Industry Project (MIT):
  - the transfer pops its chains up through the rollers and carries a carton out of the side;
  - its logic is an ST block with a `raised_fb` feedback;
  - the example line sends every third carton to a reject sink.
  The 3D view draws the rollers, the chain transfer and the photo eyes as OIP does.
- Curved conveyors and the conveyor options (ROADMAP 4.4b), from Open Industry Project (MIT):
  - curved belt and curved roller conveyors, left or right, 15° to 180°: a carton turns with the curve and leaves
    square;
  - side guards on belts, rollers and curves, an incline on belts and rollers, and roller skew that drifts the load
    against a guard;
  - every conveyor's drive ramps by `accel_time` and `decel_time`, and can report `velocity` (30 at its rated speed),
    an `encoder` of the surface's travel and a `pulse`, or take its speed from `speed_ref`. These signals are
    optional: a project binds only those it needs;
  - the belt, the curved belt and the curved roller conveyor are drawn as OIP draws them, the belt's fabric moving
    with its speed;
  - the friction model is Coulomb, which changes every run's numbers slightly; the M3 demo's photocell moves to
    2.75 m.
- The I/O panel shows a REAL signal as its number.
- Run as fast as possible uses its whole frame: about 30 snapshots a second instead of 17, and about twice the
  simulated time per second (ROADMAP 4.4b1). A conveyor's `velocity`, `encoder` and `pulse` are computed by the
  conveyor itself, not by its ST block, which makes every simulated millisecond about a fifth cheaper; the block keeps
  the ramp, `running` and `speed_ref`.
- The physics solves once per millisecond instead of four times, and the line parses its I/O addresses once at Build
  (ROADMAP 4.4b2): a simulated millisecond costs about 28 % less again, and as fast as possible runs the example
  lines at 16× and 11× real time. Every run's numbers change slightly.
- A stopper, a pusher and a turntable (ROADMAP 4.4c), copied and translated from Open Industry Project (MIT):
  - the stopper raises a blade between two rollers and holds a queue of cartons; its ST block reports `raised_fb`;
  - the pusher sweeps a passing carton across its conveyor onto a takeaway or into a sink at its side, and reports
    both ends of its stroke; its guard opens where it stands;
  - the turntable turns its deck to `target_angle` the short way, at OIP's acceleration and braking, stops while
    `enable` is off, reports `in_position`, and drives its lane as every conveyor (`run`, `speed_ref`, `running`,
    `velocity`, `encoder`, `pulse`); a photocell can stand beside it;
  - the three are drawn as OIP draws them, moving with the line;
  - a sorting line example (`sort_line`) uses all three.
- Twinova for coding agents (ROADMAP 2.7): `twinova-agent` serves the engine over the Model Context Protocol, on
  stdio or on HTTP (`--http`, `127.0.0.1:7650/mcp` by default). An agent opens a project, builds it, runs it scan by
  scan or until a condition holds, reads its signals, presses buttons, blocks sinks, saves and restores states and
  compares fingerprints; it also edits the scene (parameters, signals, instances, connections, ST sources), builds it
  as the app does and saves it. `.mcp.json` registers it for agents working in this repository.
- Products and four new sensors (ROADMAP 4.4d), after Open Industry Project (MIT):
  - a source makes products of a size, a mass, a colour and a code: colours and codes from lists, walked in turn or
    drawn at random, sizes set or drawn from OIP's range, all from a seed, the same on every platform;
  - a through-beam sees what passes between its emitter and its receiver;
  - a colour sensor tells the PLC which listed colour it sees, as a number (0 for none);
  - a laser reports the distance to what it sees, in metres;
  - a barcode scanner over a conveyor reads each product's code once, with a quarter-second pulse and a count;
  - products are drawn at their size and colour, and the sensors as OIP draws them;
  - a parameter can be a list of numbers or of colours;
  - a sorting line by code and colour (`code_sort_line`) uses them all.
- A carton now weighs the 2.7 kg it was meant to: the physics gave it 1.76 kg. Every run's numbers change slightly.
- The editor, first part (ROADMAP 4.5a):
  - "New project" starts an empty scene;
  - a catalogue offers every component family, by group: click to place one, or drag it into the 3D view;
  - a click in the 3D view selects a component; a gizmo moves it on the floor (W) or turns it in 15° steps (E);
  - the inspector edits its name, position, heading, parameters, signal addresses, what each flow port is connected to,
    and where a sensor or device is mounted;
  - Undo and Redo (Ctrl+Z, Ctrl+Y), up to 100 steps; Delete removes the selection;
  - the scene runs the edits at the next Build, which the status line recalls; the ST sources being edited are kept;
  - "Frame all" frames the scene as it is now.
- Coding agents can place any family of the library, which is pinned into the project as the app does.
- Snapping (ROADMAP 4.5b):
  - dragged with the gizmo, or dropped from the catalogue, a conveyor snaps end to end onto another, a source or a sink
    onto an end, a device onto a conveyor's rail, and a spur square onto a straight's flank;
  - the join is marked and named while dragging; hold Alt to place freely; a toggle and a gap (mm) beside the view;
  - a joined conveyor takes the other's height; a move that separates two conveyors by more than 0.15 m drops their
    flow; the inspector shows and edits each flow's gap;
  - a spur merges through an opening cut in the host's guard (new `merge` connection);
  - coding agents get the same joins (`join`) and each part's ends and rail (`describe`).
- A belt's deck and guards now reach its drum faces, 85 mm past its run, as it is drawn and as Open Industry Project's
  physics has them. Saved scenes from before join belts at their runs: their drums overlap their neighbours by 85 mm
  until the parts are snapped again.
- Installers: MSI for Windows, deb and AppImage for Linux, not signed yet.
