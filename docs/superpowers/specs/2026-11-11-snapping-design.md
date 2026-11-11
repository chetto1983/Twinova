# Spec: the editor, second part — snapping by interfaces (ROADMAP 4.5b)

Date: 2026-11-11. Status: **approved** (user, 2026-11-11). Amended the same day after the user's remark that real
conveyors always have a gap between them: OIP's ends read again, gaps measured, decisions 3, 7 and 8.

ROADMAP 4.5 is "Editor: catalogue, drag and drop, snapping by interfaces, move/rotate, signal wiring, undo", accepted
by "build a two-conveyor line with a diverter by hand". 4.5a (PR #39) gave the catalogue, the gizmo, the editable
inspector and undo; ends and mounts are still set by hand. This spec is the rest: the parts snap together.

## What exists (inventory and measurements, 2026-11-11)

- **Ports carry no geometry.** A family's ports are a name and a kind: `flow_in`, `flow_out`, `mount_host`,
  `mount_device` (`project/src/model.rs`). A connection is a `Flow` between two ports or a `Mount` with the distance
  along the host in `position[0]`.
- **The geometry of the ends is the physics'**, computed from the generator and the parameters:
  - a conveyor's origin is its entry, on the floor, x along the flow (`physics/src/layout.rs`);
  - a straight's deck runs from 0 to `length`;
  - a curve ends at the tip of its exit stub (`Arc::end` in `path.rs`), 85 mm long, OIP's drum radius;
  - an incline tilts the conveyor about the centre of its surface;
  - a turntable's lane runs 1.5 widths across its centre at its start heading.
- **The view and the physics disagree at a belt's ends.**
  - The view draws a belt's end drums, frame and guards from −85 mm to `length` + 85 mm (`oip/parts/belt.ts`:
    `r = frameHeight / 2`).
  - The physics' deck stops at 0 and `length`.
  - A roller conveyor is drawn and simulated from 0 to `length`.
- **The fixtures butt the decks.** Measured with a throwaway probe over the five fixtures, each of the 12
  conveyor-to-conveyor flows joins deck ends that coincide: 0.0000 m and 0.00°. So the belts' drawn drums overlap
  their neighbours: by 85 mm against a roller conveyor, 170 mm against a belt (read in `belt.ts` and `roller.ts`;
  not yet checked on screen).
- **A flow between conveyors is only "the exit is open"** (`closed_exit`). Products cross from one deck to the next by
  contact.
- **A turntable feeds conveyors with no flow.**
  - `sort_line`'s `belt_c` starts at the turntable's rim at −90°, with no connection.
  - The turntable has no wall, and its ports exist only at the start heading.
- **Mount rules are checked at Build** (`layout_devices.rs`, `layout_sensors.rs`):
  - a stopper and a chain transfer need a roller conveyor;
  - a pusher needs a straight;
  - all three need a level, unskewed host;
  - a sensor needs any conveyor, with the angle into the arc on a curve;
  - the barcode scanner needs a straight.
- **A gap between two straights, measured** (throwaway probe, 2026-11-11): two 2 m conveyors at 0.5 m/s, five cubes
  each case, Twinova's flat decks.

  | Gap | Cube 0.10 m | 0.20 m | 0.30 m | 0.40 m |
  |---|---|---|---|---|
  | 0 to 50 mm | 5/5 | 5/5 | 5/5 | 5/5 |
  | 85 mm | 4/5, one tipped over | 5/5 | 5/5 | 5/5 |
  | 170 mm | 0/5, all fell | 4/5 | 5/5, tilt 3.5° | 5/5 |

  - A product crosses a gap up to about half its length, and falls into one of 0.5 to 0.85 of its length.
  - The physics needs no help to model a gap.
- **A side merge, measured** (throwaway probe, 2026-11-11): a 2 m spur meets a 4 m belt's right flank square, both at
  0.5 m/s, three cartons each case.

  | Case | Delivered | Worst tilt | Yaw near the exit |
  |---|---|---|---|
  | Host guard closed | 0/3 (held at the guard) | 0.41° | — |
  | Guard opened over the spur's width, 0.3 m carton | 3/3 | 0.41° | 25.4° |
  | OIP's box 0.6×0.4×0.4 m, host 0.8 m wide | 3/3 | 0.04° | 32.0° |
  | Same box, host 0.6 m wide | 3/3 | 0.04° | 25.4° |
  | Spur 20 or 50 mm off the flank | 3/3 | 0.42° | 28.3–28.6° |

  - The merge needs the host's guard opened, as a pusher opens it.
  - Products arrive upright but turned 25–32°, and nothing squares them again.
- **OIP** (MIT, `src/core/snap/` and `src/core/conveyor/conveyorBeds.ts` at `0572868`, read for ideas):
  - Every part declares plugs and sockets. The engine solves each plug against each compatible socket and keeps the
    cheapest join that engages.
  - A conveyor's ends are its real faces:
    - a belt's is the face of its end drum, half the 0.17 m belt height past the run;
    - a straight's is the end of its frame;
    - a curve's is the tip of its stub: 85 mm for a belt curve, 2 mm for a roller curve.
  - Joined ends touch face to face. The real interspace is the drums' curve, and the belt's collider box reaches the drum
    face, so in OIP's physics the gap is 0.
  - A conveyor join engages when the mating end moves at most 0.3 m and turns at most a quarter turn (45° onto a
    flank). A held join lets go past 0.6 m. The cost is the move plus 0.5 per radian of turn.
  - A side junction counts up to 0.15 m from the flank (`GUARD_CONTACT_TOLERANCE`). A spur parks 40 mm off it
    (`SPUR_SIDE_GAP`).
  - A device seats on a guard rail by projection.

## Decisions (user, 2026-11-11)

1. **Rust computes the geometry and makes each join; the view finds and shows candidates.**
   - Rust gives every instance its *interfaces*: its ends and, for a host, its rail. They come from the physics' own
     geometry and are sent in the view.
   - During a drag the view finds the best join and marks it.
   - On release, Rust computes the pose and makes the connection, in one undo step.
   - The agent gets the same joins.
2. **Heights.** Joined end to end, the moved conveyor's `height` takes the value that puts its mating end on the other's
   surface, an inclined one included. The notice says so, and undo restores it.
3. **A join between conveyors exists while their faces are within 0.15 m and on one axis** (OIP's contact tolerance).
   - A move that takes them further removes the flow or the merge, in the same undo step.
   - Build accepts any gap: the physics decides whether a product crosses it.
   - A flow set by hand in the inspector stays allowed.
4. **Four kinds of join:**
   - conveyor end to conveyor end;
   - a device onto a conveyor's rail;
   - a source or a sink onto an end;
   - the side merge.
5. **The side merge is a new connection kind,** `merge`: from a spur's `out` onto a host's rail, at a distance along it,
   on a side.
   - The host's guard opens there, as for a pusher.
   - The published families do not change.
   - A straight takes several spurs.
6. **The merge's 25–32° turn is accepted and documented.** Squaring products again (powered rollers, angled merges) is a
   later phase.
7. **Ends are the real faces, as in OIP.**
   - A belt's end is its drum face, 85 mm past each end of its run.
   - Its physics deck and guards reach that face, as OIP's collider does and as the view already draws them. This
     removes today's mismatch.
8. **A join leaves a gap, set by the user, 0 by default.**
   - The gap is the distance between the two faces.
   - It is real in the physics: the decks do not reach across it.

## The design

### Interfaces (Rust, `physics`)

`twinova_physics::interfaces` describes each instance from its definition and parameters, without needing the line to
be complete.

- **Ends,** each with a name, a direction (`in`, `out`, or either for a rim) and a frame in the instance's frame. The
  frame's origin is on the face, at the running surface, and its x points out of the part:
  - a belt: `in` and `out` on its drum faces, at −85 mm and `length` + 85 mm;
  - a roller conveyor: `in` and `out` at 0 and `length`;
  - a curve: `in` and `out` at the tips of its stubs;
  - a turntable: `in` and `out` at its start heading, and the rims `left` and `right` across it, which take no flow;
  - a source: `out`; a sink: `in`. Joined, each stands at the conveyor's face, on the floor, facing the flow;
  - a chain transfer and a pusher, once mounted: `out`, at their side exit.
- **A rail,** on a host:
  - a straight's centreline, from its entry to `length`;
  - a curve's arc, in angle;
  - a turntable's lane.

  The rail carries the mount rules of Build, so it offers only the devices Build accepts.
- **One geometry.** The conveyor geometry moves out of `layout()` into one function per instance, and both `layout()`
  and `interfaces` call it.

### The belt's ends in the physics (decision 7)

- **The deck and the guards** of a straight belt run from −85 mm to `length` + 85 mm, as OIP's collider and Twinova's
  view do.
- **A distance along it is unchanged.** A mount's `at`, a source's drop and a sensor's beam still count from the run's
  start, 0.
- **The fixtures are re-spaced so their joins stay face to face with no gap.**
  - A conveyor after a belt moves 85 mm downstream, and a belt after any conveyor 85 mm more.
  - The new positions come from that formula, written in the fixtures' own scripts, not from the snap code.
- **The pinned fingerprints and timings move.** Each is pinned again, with this reason.

### Joins (Rust, `physics::snap`, used by the app and the agent)

- **Precision (amended 2026-11-11, Task 5).** Each instance's ends come from the physics' own geometry, in f32, with
  the instance at the origin. Poses are composed in f64 (glamx's `DPose3`, the version rapier brings, with its `f64`
  feature).
  - A join is accurate to about 1e-7 m and 1e-7 rad, and the error adds up along a chain of joins.
  - Measured on the rebuilt fixtures: the worst is curve_line's last belt, 8.5e-7 m from the fixture after four
    joins.

- **`join(project, (id, end), (peer, end), gap)`.**
  - It poses `id` so that its end faces the peer's end, on one axis, with `gap` metres between the faces.
  - A conveyor also takes the height of decision 2.
  - When both ends are flow ports of opposite directions, it sets the flow, freeing what each end had. Onto a turntable
    rim it sets no flow (`sort_line`'s `belt_c`).
  - It refuses with a sentence:
    - two ins or two outs;
    - a gap below 0 or over 0.15 m;
    - a height outside the parameter's range;
    - a host refused by Build's mount rules.
- **`merge(project, spur, host, at, side, gap)`.**
  - It poses the spur square to the host's flank, its exit face `gap` off the flank at `at`. The default gap is 0;
    OIP's is 40 mm.
  - The spur takes the host's height, and the `merge` connection is set.
  - The host must be a level, unskewed straight, as for a pusher.
  - Spurs on one host must not overlap.
- **A device onto a rail** is 4.5a's `set_mount`, with `at` from the view's projection. A mounted device dragged away
  from every rail is unmounted.
- **`moved(project, id)`.** After a move or a turn, it removes the flows and merges of `id` that no longer hold
  (decision 3): faces more than 0.15 m apart, more than 1 mm off one axis, or more than 0.1° apart. It runs inside the
  same undo step as the move, and the agent's `move_instance` goes through it too.
- **Editing a gap.** The inspector shows the gap of each of the selection's flows. Writing it re-joins the selection,
  which moves, to its peer.

### The format and Build

- **New connection kind `merge`:**
  `{"kind": "merge", "from": ["spur", "out"], "host": ["belt_a", "rail"], "at": 2.0, "side": "right"}`.
  - It is checked like the others: `from` is a flow-out port, `host` a mount-host port.
  - A flow-out port has one peer, a flow or a merge.
- **Build:**
  - the host's guard opens over the spur's width at `at` (the pusher's `Opening`);
  - the spur's exit is open;
  - Build refuses a merge whose spur is not square to its flank (0.1°) or whose exit face is more than 0.15 m from it,
    naming both;
  - a flow's gap is not checked (decision 3).
- **The view** draws a merge's guard opening, as it draws a pusher's.

### The view

- **During a gizmo drag or a catalogue drop,** the view tries every end of the moved part against every free end within
  reach, and the rails and flanks:
  - an end joins within 0.3 m and a quarter turn;
  - a flank takes a spur within 0.3 m and 45°;
  - a rail takes a device within 0.3 m of it.

  The cheapest candidate wins: the move, plus 0.5 per radian of turn. These are OIP's values.
- **The gap of a new join** is the last one used, 0 at first. A field next to the snap toggle shows it.
- **Feedback.**
  - A marker sits on the target face, flank point or rail point.
  - The status line names the join ("joins belt_a.out, gap 20 mm").
  - A shown candidate is kept until the drag is 0.6 m from it (OIP's release).
- **Release** sends `join`, `merge` or `set_mount`. With no candidate it sends 4.5a's move, which `moved` follows.
  Holding Alt places freely, with no candidate.
- **A mounted device gets the gizmo** (4.5a detached it). On release it slides along its host, moves to another host, or
  is unmounted.

### The agent

- `describe` lists each instance's ends and rail.
- A `join` tool takes two ends and a gap, or a spur, a host, a distance, a side and a gap.
- `move_instance` drops the joins it breaks.

## Acceptance

1. **Joins, in Rust tests:**
   - each pair of conveyor families end to end, curves to the left and right;
   - each turntable end, with no flow at the rims;
   - source and sink, and a sink onto a chain transfer's and a pusher's side;
   - heights, an inclined mover included;
   - gaps of 0, 20 mm and 0.15 m, measured face to face;
   - each refusal, with its sentence;
   - a move within 0.15 m keeps the flow, one beyond removes it;
   - one undo step undoes the join, the height and the flow.
2. **The belt's faces in the physics:**
   - the deck reaches 85 mm past each end of the run;
   - a source's product, a mount's distance and a sensor's beam are where they were along the run;
   - the gap table above, measured again face to face through the new decks, is recorded in the verification.
3. **The re-spaced fixtures rebuilt by joins.** `curve_line`, `sort_line` and `two_belts_diverter` are rebuilt from the
   same instances placed anywhere, with joins (gap 0) and mounts only.
   - Every conveyor's pose equals the fixture's within 1e-6 m and 1e-6 rad.
   - Every connection is equal, a mount's distance included.
   - The poses of sources, sinks and mounted devices are not compared. The physics ignores them, and the fixtures place
     them by eye.
4. **The merge in the physics:**
   - the probe's line built through `merge`: 3 of 3 delivered, tilt under 1°;
   - the turn is recorded, not bounded;
   - Build refuses a merge onto a curve, an inclined host or a skewed one, and one that is not square.
5. **In the app, on Linux and windows-2022** (`e2e/snapping.mjs`): `two_belts_diverter` is built by hand from an empty
   project, with drag and snap only. The inspector is used for names and parameters.
   - **Conveyors:** belt, roller conveyor (`left_guard` off) and second belt, each dragged near the previous exit.
   - **Devices:** the chain transfer onto the roller conveyor, both sensors onto their rails.
   - **Source and sinks:** the source onto the first entry, one sink onto the last exit, the other onto the transfer's
     side.

   Then:
   - one join's gap is set to 20 mm in the inspector, and set back to 0;
   - it builds and runs until both sinks count;
   - the saved scene equals the fixture's: conveyors within 1 mm (the view rounds), and the same connections.
6. **A merge in the app:** a spur dragged onto a belt's flank, built and run until the host's sink counts.
7. **Gates** as 4.5a:
   - cargo mutants on `interfaces`, `snap`, the belt's deck and the merge's layout, with no survivor;
   - coverage ≥ 85 %;
   - Stryker ≥ 70 % on the view's candidate search;
   - the file cap and the rest of the Gate 2 set.

## What this will not prove

- **A drum's curve.**
  - The decks are boxes to the face, as OIP's. The V between two drums is not modelled.
  - A gap is a flat hole, harsher than a real drum: a 0.10 m product falls at 85 mm.
- **Products squared after a merge:**
  - they turn 25–32°, as measured;
  - angled merges (30°, 45°) and powered rollers are a later phase.
- **Moving a joined chain together:** each part moves alone, and its joins break beyond 0.15 m; there is no
  multi-select.
- **Reversing a conveyor on a join:** OIP flips a curve's flow to fit. Twinova refuses two ins or two outs.
- **OIP's 2 mm roller-curve stub:** Twinova's curves keep their 85 mm stubs.
- **Grid, face and vertex snapping:** 4.5b snaps by interfaces only.
- **Families from CAD:** their interfaces would be declared, not computed; that is ROADMAP 9.1.
- **A turntable's rims at headings other than ±90°.**
- **Joins exact to the micrometre down a long chain.**
  - Joins are accurate to about 1e-7 m each.
  - A chain of more than about ten joins may stray more than 1e-6 m from a hand-computed layout.
  - That is a thousandth of the millimetre the view rounds to.
