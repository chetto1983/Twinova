# Spec: the editor, first part — catalogue, placing, moving, editing, undo (ROADMAP 4.5a)

Date: 2026-11-11. Status: **approved** (user, 2026-11-11).

ROADMAP 4.5 is "Editor: catalogue, drag and drop, snapping by interfaces, move/rotate, signal wiring, undo", accepted
by "build a two-conveyor line with a diverter by hand". The user split it, as 4.4 was (2026-11-11):
- **4.5a, this spec:** catalogue, placing, moving and rotating, editing in the inspector, undo, a new project.
- **4.5b:** snapping by interfaces (conveyor end to end, devices onto a conveyor's rail), with the ROADMAP's acceptance.

## What exists (inventory, 2026-11-11)

- **The app shows a project and cannot edit its scene.**
  - The inspector is read-only ("read-only in 4.2").
  - The 3D view has orbit controls and no picking: selection is in the scene tree only.
  - The only project edit is the ST text, which Build puts back (`project_commands.rs` `set_sources`).
- **The edit operations exist, in the agent** (`agent/src/edit.rs`, 2.7):
  - set or reset a parameter;
  - bind or unbind a signal;
  - add, move and remove an instance (removing also drops its connections and group membership);
  - connect, disconnect, read and write a source.

  They are free functions over `twinova_project::Project`, with no rmcp type.
- **There is no catalogue.** Definitions exist only as JSON and ST in the test fixtures. The 15 families are byte-equal
  in every fixture that has them (measured: one SHA-256 each). The component model spec asks for a catalogue first,
  pinned copies and explicit upgrades (`2026-11-08-component-model-design.md`).
- **The format has what the editor needs:**
  - poses (position, quaternion);
  - `Flow` and `Mount` connections between named ports, checked at Build (kinds, one peer per flow end);
  - a mount's distance along its host, `position[0]`.
- **three 0.187 ships `TransformControls`** (MIT), unused.
- **OIP** (read for ideas only, clean-room):
  - its undo keeps the whole document before and after each commit;
  - its catalogue lists part folders;
  - its gizmo is its own (1 100 lines).

## Decisions (user, 2026-11-11)

1. **4.5a, then 4.5b.**
2. **A library in the repository.**
   - `app/src-tauri/library/components/` holds one definition (`.json`) and its behaviour block (`.st`) per family, in
     the project format's own file names.
   - The app and the agent embed it at compile time.
   - Placing a family pins a copy of its definition in the project. A project that already pins the same key keeps its
     copy: upgrading is explicit (M4.1), not 4.5a.
   - The fixtures copy the library through their scripts, and a test holds every fixture's copy equal to it.
3. **Edits in Rust, shared with the agent.**
   - `edit.rs` moves to `twinova-project`.
   - The app gets Tauri commands over its open project; the agent calls the same functions.
   - Undo and redo keep whole-project snapshots (OIP's idea), in Rust.
4. **three's `TransformControls`** for moving and rotating, measured on WebKitGTK and WebView2.
   - It moves on the floor plane.
   - It turns about +Y in steps of 15° (OIP's `DEFAULT_ROTATE_SNAP_DEG`).

## The design

### The library and placing

- **`twinova_project::library`:** the families, as `(key, definition, behaviour)`, parsed and validated once by a test.
  Each family has a display group: conveyors, devices, sensors, flow (source and sink).
- **`place(project, family, id, pose)`:**
  - pins the definition (and its block) if the project does not have that key;
  - adds an instance with no overrides and no signals bound;
  - picks the first free id from the family's name (`belt_1`, `belt_2`, …).
- **The agent:** `describe` lists the library too. `add_instance` places a library family that is not pinned yet,
  instead of refusing it.

### Editing and history

- **`twinova_project::edit`:** the agent's operations, moved unchanged, plus:
  - `place`;
  - `set_name`;
  - `set_mount_at`: a mounted device's distance along its host;
  - `set_flow`: a port's peer, set or cleared.

  Ports, kinds and addresses stay checked at Build, as for the agent.
- **`twinova_project::history::History`:**
  - an undo and a redo stack of `Project`s, 100 deep;
  - each command records the project before it, and a new edit clears redo.
- **The app's `ProjectState`:**
  - holds the working project and its history;
  - `edit(op)` applies one operation and returns the new view;
  - `undo` and `redo` swap snapshots;
  - opening, importing or a new project clears the history.

  The running line changes only at Build, as now; the toolbar shows that the scene changed since the last Build.
- **A gesture is one undo step:** a drag of the gizmo commits once, on release; a field commits on Enter or blur.

### The view

- **Picking:** each instance's group carries its id. A click in the 3D view selects the instance under the cursor (a
  `Raycaster`); a click on empty floor clears the selection.
- **The gizmo:** on the selected instance, translate (floor plane, x and z) or rotate (+Y, 15° steps), switched with W
  and E. A mounted device has none: it moves along its host in the inspector.
- **The catalogue panel:** the library's families by group, each a card with its name.
  - A click places one at the floor point at the view's centre.
  - Press-drag from a card to the view places it where it is released. These are pointer events, not HTML5 drag and
    drop, as OIP does, so the view keeps its own pointer handling.
- **The inspector becomes editable:**
  - name;
  - position (x, y, z) and heading (degrees about +Y);
  - each parameter by its type: a number, a duration, a checkbox, a list as comma-separated items;
  - each signal's address, empty to unbind;
  - for a device, its host and its distance along it;
  - for a flow port, the port it feeds or is fed by.
- **Keys:** Delete removes the selection; Ctrl+Z undoes, Ctrl+Y and Ctrl+Shift+Z redo, unless the focus is in a text
  field or the ST editor.
- **A new project:** a scene with no instances and an empty `PROGRAM Main`.
- **The view carries its connections and ports:** the `ProjectView` gains the flow connections and each definition's
  ports, so the inspector can offer them.

## Acceptance

1. **The library:** the 15 families parse and validate; every fixture's copy equals the library's.
2. **Shared edits:**
   - the agent's tests pass unchanged on the moved operations;
   - `place`, `set_name`, `set_mount_at` and `set_flow` each have tests of what they do and what they refuse;
   - placing a pinned key keeps the project's copy.
3. **History:**
   - undo after N edits gives back each earlier project, equal;
   - redo gives the later ones;
   - a new edit clears redo;
   - the depth is bounded.
4. **In the app (E2E on Linux and Windows):**
   - from a new project, by hand: a source, two belts, a sink and a diffuse sensor are placed from the catalogue;
   - the belts are moved end to end with the gizmo;
   - the flows are set in the inspector, the sensor is mounted on belt B, and the motors and the sensor are bound;
   - the program is written, Build passes and Run delivers to the sink;
   - undo back to the empty scene, then redo forward, gives the same scene;
   - Save and reopen gives the same project.
5. **Picking and the gizmo measured** on WebKitGTK and WebView2: a click selects what is under the cursor; a drag of
   the gizmo is one undo step.
6. **Gates:**
   - fmt, clippy, coverage ≥ 85 %;
   - cargo mutants on `edit`, `history` and `library`;
   - Stryker ≥ 70 % on the TS helpers;
   - the 600-line cap.

## What this will not prove

- **Snapping:** ends and rails are set by hand; snapping is 4.5b.
- **Upgrading a pinned family to the library's newer version:** M4.1's explicit upgrade, later.
- **Groups, multiple selection, copy and paste.**
- **Undo across Build or a run:** history covers the scene and its sources, not the running line.
