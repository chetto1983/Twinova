# Spec: the editor, second part — snapping by interfaces (ROADMAP 4.5b)

Date: 2026-11-11. Status: **draft**, for the user's approval.

ROADMAP 4.5 is "Editor: catalogue, drag and drop, snapping by interfaces, move/rotate, signal wiring, undo", accepted
by "build a two-conveyor line with a diverter by hand". 4.5a (PR #39) gave the catalogue, the gizmo, the editable
inspector and undo; ends and mounts are still set by hand. This spec is the rest: the parts snap together.

## What exists (inventory and measurements, 2026-11-11)

- **Ports carry no geometry.** A family's ports are a name and a kind: `flow_in`, `flow_out`, `mount_host`,
  `mount_device` (`project/src/model.rs`). A connection is a `Flow` between two ports or a `Mount` with the distance
  along the host in `position[0]`.
- **The geometry of the ends is the physics'**, computed from the generator and the parameters:
  - a conveyor's origin is its entry end, on the floor, x along the flow (`physics/src/layout.rs`);
  - a straight's exit is `length` along x;
  - a curve's exit is `Arc::end` (`path.rs`), after the entry stub, the arc and the exit stub;
  - an incline tilts the conveyor about the centre of its surface;
  - a turntable's lane runs 1.5 widths across its centre at its start heading.

  The view draws the same shapes in TypeScript (`families.ts`), with no snapping.
- **A flow between conveyors is only "the exit is open"** (`closed_exit`). Products cross from one deck to the next by
  contact, so the ends must really meet.
- **Measured (throwaway probe over the five fixtures, 2026-11-11):** each of the 12 conveyor-to-conveyor flows joins
  ends that coincide.
  - Gap 0.0000 m, heading difference 0.00°, and equal heights.
  - Straight, roller, both curves and the turntable's start heading are all covered.

  So "a flow between conveyors means their ends touch" already holds.
- **A turntable feeds conveyors with no flow.**
  - `sort_line`'s `belt_c` starts at the turntable's rim at −90°, with no connection.
  - The turntable has no wall, and its ports exist only at the start heading.
- **Mount rules are checked at Build** (`layout_devices.rs`, `layout_sensors.rs`):
  - a stopper and a chain transfer need a roller conveyor;
  - a pusher needs a straight;
  - all three need a level, unskewed host;
  - a sensor needs any conveyor, with the angle into the arc on a curve;
  - the barcode scanner needs a straight.
- **A side merge, measured** in a throwaway probe (2026-11-11): a 2 m spur meets a 4 m belt's right flank square, both
  at 0.5 m/s, with 3 cartons each case.

  | Case | Delivered | Worst tilt | Yaw near the exit |
  |---|---|---|---|
  | Host guard closed | 0/3 (held at the guard) | 0.41° | — |
  | Guard opened over the spur's width, 0.3 m carton | 3/3 | 0.41° | 25.4° |
  | OIP's box 0.6×0.4×0.4 m, host 0.8 m wide | 3/3 | 0.04° | 32.0° |
  | Same box, host 0.6 m wide | 3/3 | 0.04° | 25.4° |
  | Spur 20 or 50 mm off the flank | 3/3 | 0.42° | 28.3–28.6° |

  - The merge needs the host's guard opened, as a pusher opens it.
  - Products arrive upright but turned 25–32°, and nothing squares them again.
- **OIP** (MIT, `src/core/snap/` at `0572868`, read for ideas):
  - every part declares plugs and sockets;
  - the engine solves each plug against each compatible socket and keeps the cheapest join that engages;
  - for conveyors, a join engages when the mating end moves at most 0.3 m and turns at most a quarter turn (45° onto
    a flank);
  - a held join lets go past 0.6 m;
  - the cost is the move plus 0.5 per radian of turn;
  - a device seats on a guard rail by projection;
  - a side merge places the spur square to the flank, straight across from where it is.

## Decisions (user, 2026-11-11)

1. **Rust computes the geometry and makes each join; the view finds and shows candidates.**
   - Rust gives every instance its *interfaces*: its ends (local frames) and, for a host, its rail. They come from the
     physics' own geometry and are sent in the view.
   - During a drag the view finds the best join and marks it.
   - On release, Rust computes the exact pose and makes the connection, in one undo step.
   - The agent gets the same joins.
2. **Heights.** Joined end to end, the moved conveyor's `height` takes the value that puts its mating end on the other's
   surface (an inclined one included). The notice says so, and undo restores it.
3. **A join between conveyors exists while the ends touch.** A move that separates them removes the flow, or the merge,
   in the same undo step. A flow set by hand in the inspector stays allowed.
4. **Four kinds of join:**
   - conveyor end to conveyor end;
   - a device onto a conveyor's rail;
   - a source or a sink onto an end;
   - the side merge.
5. **The side merge is a new connection kind,** `merge`: from a spur's `out` onto a host's rail, at a distance along it,
   on a side. The host's guard opens there, as for a pusher. The published families do not change, and a straight takes
   several spurs.
6. **The merge's 25–32° turn is accepted and documented.** Squaring products again (powered rollers, angled merges) is a
   later phase.

## The design

### Interfaces (Rust, `physics`)

`twinova_physics::interfaces` describes each instance from its definition and parameters, without needing the line to
be complete.

- **Ends,** each with a name and a frame in the instance's frame (origin on the running surface, x out of the part
  along the flow), and a direction (`in`, `out`, or either for a rim):
  - a straight, a roller conveyor or a curve: `in` and `out`;
  - a turntable: `in` and `out` at its start heading, and the rims `left` and `right` across it, which take no flow;
  - a source: `out`; a sink: `in`. Joined, each stands at the conveyor's end, on the floor, facing the flow;
  - a chain transfer and a pusher, once mounted: `out`, at their side exit.
- **A rail,** on a host:
  - for a straight, its centreline from 0 to `length`;
  - for a curve, its arc, in angle;
  - for a turntable, its lane.

  The rail carries the mount rules of Build, so it offers only the devices Build accepts.
- **One geometry.** The conveyor geometry moves out of `layout()` into one function per instance, and `layout()` and
  `interfaces` both call it.

### Joins (Rust, `physics::snap`, used by the app and the agent)

- **`join(project, (id, end), (peer, end))`.** It poses `id` so that its end meets the peer's end, facing it. A conveyor
  also takes the height of decision 2.
  - When both ends are flow ports of opposite directions, it sets the flow, freeing what each end had.
  - Onto a turntable rim it sets no flow (`sort_line`'s `belt_c`).
  - It refuses with a sentence:
    - two ins or two outs;
    - a height outside the parameter's range;
    - a host refused by Build's mount rules.
- **`merge(project, spur, host, at, side)`.**
  - It poses the spur square to the host's flank, with its exit on the flank at `at`.
  - The spur takes the host's height.
  - It sets the `merge` connection.
  - The host must be a level, unskewed straight, as for a pusher.
  - Several spurs on one host must not overlap.
- **A device onto a rail** is 4.5a's `set_mount`, with `at` from the view's projection. A mounted device dragged away
  from every rail is unmounted.
- **`moved(project, id)`.** After a move or a turn, it removes the flows and merges of `id` whose ends no longer meet
  (decision 3). It runs inside the same undo step as the move. The agent's `move_instance` goes through it as well.
- **"Meet"** means within 1 mm and 0.1°. A join is exact in the format's f64; the view rounds poses to the millimetre
  (4.5a's `pose`).

### The format and Build

- **New connection kind `merge`:**
  `{"kind": "merge", "from": ["spur", "out"], "host": ["belt_a", "rail"], "at": 2.0, "side": "right"}`.
  - It is checked like the others: `from` is a flow-out port, `host` a mount-host port.
  - A flow-out port has one peer, a flow or a merge.
- **Build:**
  - the host's guard opens over the spur's width at `at` (the pusher's `Opening`);
  - the spur's exit is open.
- **Build refuses a join that does not meet** (decision 3), naming both ends:
  - a conveyor-to-conveyor flow whose ends are more than 1 mm or 0.1° apart;
  - a merge whose spur is not square on its flank.

  The five fixtures pass, as measured.
- **The view** draws a merge's guard opening, as it draws a pusher's.

### The view

- **During a gizmo drag or a catalogue drop,** the view tries every end of the moved part against every free end within
  reach, and the rails and flanks:
  - an end joins within 0.3 m and a quarter turn;
  - a flank takes a spur within 0.3 m and 45°;
  - a rail takes a device within 0.3 m of it.

  The cheapest candidate wins: the move, plus 0.5 per radian of turn. These are OIP's values.
- **Feedback:** a marker on the target end, flank point or rail point, and the status line names the join
  ("joins belt_a.out"). A shown candidate is kept until the drag is 0.6 m from it (OIP's release).
- **Release** sends `join`, `merge` or `set_mount`. With no candidate it sends 4.5a's move, which `moved` follows.
  Holding Alt places freely, with no candidate.
- **A mounted device gets the gizmo** (4.5a detached it). On release it slides along its host, moves to another host,
  or is unmounted.

### The agent

- `describe` lists each instance's ends and rail.
- A `join` tool takes two ends, or a spur, a host, a distance and a side.
- `move_instance` drops the joins it separates.

## Acceptance

1. **Joins, in Rust tests:**
   - straight to straight;
   - straight to curve and curve to straight, left and right;
   - each turntable end, rims without a flow;
   - source and sink;
   - a sink onto a chain transfer's and a pusher's side;
   - heights, an inclined mover included;
   - each refusal, with its sentence;
   - a move that separates removes the flow, one that does not keeps it;
   - one undo step undoes the join, the height and the flow.
2. **The fixtures rebuilt by joins:** `curve_line`, `sort_line` and `two_belts_diverter` are rebuilt from the same
   instances placed anywhere, with joins and mounts only. The fixtures were authored by hand before this spec, so they
   are the independent reference.
   - Every conveyor's pose equals the fixture's within 1e-6 m and 1e-6 rad.
   - Every connection equals the fixture's, a mount's distance included.
   - The poses of sources, sinks and mounted devices are not compared. The physics ignores them, and the fixtures
     place them by eye: the source of `two_belts_diverter` stands 0.5 m before its belt.
3. **The merge in the physics:**
   - the probe's line built through `merge`: 3 of 3 delivered;
   - tilt under 1°;
   - the turn is recorded, not bounded;
   - Build refuses a merge onto a curve, an inclined host or a skewed one, and one that is not square.
4. **In the app, on Linux and windows-2022** (`e2e/snapping.mjs`): `two_belts_diverter` is built by hand from an empty
   project, with drag and snap only. The inspector is used for names and parameters only.
   - **Placing the conveyors:** belt, roller conveyor (`left_guard` off) and second belt, each dragged near the
     previous exit.
   - **Mounting the devices:** the chain transfer onto the roller conveyor, both sensors onto their rails.
   - **Source and sinks:** the source onto the first entry, one sink onto the last exit, the other onto the transfer's
     side.

   Then it builds and runs until both sinks count. The saved scene equals the fixture's:
   - the conveyors' poses within 1 mm (the view rounds);
   - the same connections, matched by family and order, as in acceptance 2.
5. **A merge in the app:** a spur dragged onto a belt's flank, built and run until the host's sink counts.
6. **Gates** as 4.5a:
   - cargo mutants on `interfaces`, `snap` and the merge's layout, with no survivor;
   - coverage ≥ 85 %;
   - Stryker ≥ 70 % on the view's candidate search;
   - the file cap and the rest of the Gate 2 set.

## What this will not prove

- **Products squared after a merge:**
  - they turn 25–32°, as measured;
  - angled merges (30°, 45°) and powered rollers are a later phase.
- **Moving a joined chain together:** each part moves alone, and its joins break; there is no multi-select.
- **Reversing a conveyor on a join:** OIP flips a curve's flow to fit. Twinova refuses two ins or two outs.
- **Grid, face and vertex snapping:** 4.5b snaps by interfaces only.
- **Families from CAD:** their interfaces would be declared, not computed; that is ROADMAP 9.1.
- **A turntable's rims at other headings than ±90°:** the rims are at its quarter turns only.
