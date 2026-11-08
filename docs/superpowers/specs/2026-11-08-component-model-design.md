# M4.1 — Component model and editor foundation

Status: **draft for approval**, 2026-11-08. The user asked to start the recommended catalogue-first
direction, then requested a live exploration of Open Industry Project before finalising M4. Three decisions of
2026-11-08 are recorded below: decisions are commanded by the PLC, signals bind to IEC direct addresses
(measured on the pinned truST), and the document is Y up.
This document records that direction and implementation proposals; it does not claim that M4 functionality exists.

## Intent and boundaries

An automation engineer assembles a line from working, parameterised components, connects sensors and actuators
to Structured Text, saves the project and repeats the same experiment. The first complete acceptance scene is
a source, two conveyors, a two-output diverter, two sensors and explicit normal/reject destinations.

Retain Tauri/Svelte/Three.js, the Rust/nexosim deterministic core and the frozen truST runtime. The initial user
workflow is catalogue placement and parameter editing. A complete CAD modeller, user-authored component editor,
robot solver, collaboration service and external PLC drivers are outside this first slice.

The component format must describe nodes, joints, behaviour, parameters and typed interfaces so that later
authoring does not require a second incompatible model. Supporting a field in the format is not permission to
silently run an unimplemented behaviour: a required unsupported capability prevents Run and is reported.

## Evidence and build/reuse inventory

- [Research 12](../../research/12-m4-componenti-editor-opensource.md) records exact source revisions, licences and files read.
- [Research 11](../../research/11-ecatalog-componenti.md) surveys the public Visual Components eCatalog: families,
  variants and the parameters and signals named publicly (clean-room).
- [Live OIP study](../../research/13-oip-app-comportamenti-live.md) records catalogue/inspector capture, actual editing,
  project round-trip, ST diagnostics, binding/force/release and Ladder authoring, with explicit evidence limits.
- Existing `belt.rs` owns integer-nanometre analytic motion; `bench.rs` currently builds one belt and one PLC.
- `plc.rs` currently enforces `sensor/start/stop/motor`; generic component binding is new work for 4.3.
- `SceneView.svelte` currently builds one hard-coded belt. `scene.rs` is the older fixed-step reference, not the new document.
- Three.js already provides `TransformControls`, transform snapping, raycasting and GLB loading. Reuse these.
- Serde/serde_json already provide Rust serialization; they do not provide domain validation or migrations.
- nexosim supplies models, connections and virtual-time scheduling. Twinova must define the scene-to-model mapping.
- OIP provides useful MIT reference designs for definitions/instances, local project packages and inspectors.
  Its JavaScript/Jolt tick model is not Twinova's runtime. No external code or asset is copied by this spec.

## Approaches

1. **Catalogue-first (selected direction):** parameterised working families and reusable groups, with an inspector.
   Fastest route to assembling a real line. Full family authoring is a later increment.
2. Full component authoring first: geometry/node/joint/behaviour editing immediately. Greater authoring freedom,
   but materially more work before the first multi-component simulation.
3. CAD-first: imported geometry subsequently acquires simulation semantics. Useful later; pivot, units, collision
   and semantic authoring would dominate this first slice.

## Document model

| Object | Responsibility and invariants |
|---|---|
| Project | Versioned document, scene, exact component definitions, referenced local assets and ordered ST source files |
| Component definition | Stable ID and version; parameter schema/defaults, node tree, exposed interfaces and behaviour capability |
| Instance | Stable instance ID, definition reference, display name, pose and explicit parameter overrides |
| Node | Definition-local ID, one parent, local pose and optional geometry; hierarchy must be acyclic |
| Joint | Parent/child node references, fixed/revolute/prismatic kind, local axis and limits; driven state is runtime data |
| Group | Scene assembly with child instance IDs, group pose and explicitly exposed child interfaces |
| Connection | Typed endpoint references using IDs, never mutable labels or Three.js object UUIDs |
| Product | Global runtime identity, product type/dimensions and exactly one current owner; visual meshes are projections |

Definitions used by a saved project are pinned copies. Catalogue updates are explicit user actions with a diff;
opening a project never silently upgrades a family. Overrides remain attached to their instance. Changing a
parameter that no longer exists must produce a diagnostic, not silently discard it.

Copying a group allocates new IDs for its instances and remaps internal connections. External connections are
not copied automatically. Renaming an instance changes presentation only. Deleting restores all dependent
connections on Undo. Grouping for selection and defining a reusable assembly are separate commands.

Builder-review resolutions (implementation proposals): the first persisted format includes reusable assembly
templates made of child instances, internal links and exposed endpoint aliases. Placing a template expands it
into a normal scene group with fresh IDs and a provenance reference; placed groups are independent copies,
not live-inheriting instances. Definition recursion/cycles are rejected. Updating a template never changes
already placed groups automatically.

## Coordinates and parameters

- Right-handed coordinates, Y up (user, 2026-11-08): the convention of glTF and three.js, so assets and the view
  need no conversion. The UI may show Z-up coordinates to CAD users; importing CAD (STEP) converts once, later. Metres and radians in document geometry, mm and degrees available in the UI.
- Poses are local to the parent. Matrices are derived; do not store competing local and world transforms.
- A transport path is defined in component-local coordinates, independently of the mesh origin.
- Parameters carry type, unit, default, valid range/choices and display grouping. Reject non-finite values,
  invalid ranges and inconsistent dimensions. Do not resize collision or transport geometry by visual scale alone.
- Preserve the existing integer nm/ns arithmetic in the runtime. Geometry-to-runtime conversion, precision and
  overflow limits must be measured in the 4.2/4.3 implementation before publishing a numeric format contract.
- A physical dimension change rebuilds its geometry, local interface frames and transport path together.

First-family parameterisation uses explicitly versioned native family generators selected by a capability ID,
plus validated parameter values. Static glTF assets are optional visuals; they do not encode the transport logic.
No new expression language is introduced in 4.2. A missing generator preserves the document and shows its bounds
placeholder, but blocks simulation of that component. A general declarative geometry authoring language is later work.

## Three distinct connection graphs

**Mounting:** a compatible mechanical attachment and relative frame, for example a sensor attached to a rail.
Moving the host moves the attached device. Attachments cannot create hierarchy cycles.

**Material flow:** directed product transfer between compatible interfaces. Each endpoint has an explicit
direction, frame and capacity policy. V1 supports a single peer per endpoint; branching belongs to a diverter's
multiple output ports, not an ambiguous multi-peer output. Unconnected exits block unless the component is an
explicit sink; products must never disappear merely because a connection is missing.

**Signals:** IEC type, component-relative direction, initial value and endpoint ID. A sensor's output is a PLC
input; an actuator's input is a PLC output. The editor must label that perspective.
No implicit numeric coercion; no competing writers. Invalid binding prevents Run with a path to the offending endpoint.

**Binding to the controller by IEC direct address** (user, 2026-11-08). Each bound signal has an address in the
controller's process image, and the ST program declares the variable at it, as on a real PLC:
`sensor_a AT %IX0.0 : BOOL;`, `divert AT %QX0.1 : BOOL;`.

- **V1 types:** BOOL on a bit address (`%IX`, `%QX`); DINT and REAL on a double word (`%ID`, `%QD`). The other
  sizes work in truST (table below) and are enabled when a family needs them; `STRING[n]`, for example, suits a
  barcode reader.
- **The editor proposes addresses** and the user may change them. Twinova validates what truST does not (table
  below): the address size must match the type, two signals must not overlap (for example `%IX4.0` and `%ID4`),
  and an output has one writer.
- **Per-instance addresses through `VAR_CONFIG`.** A component's ST function block declares its I/O with an
  incomplete address (`x AT %I* : BOOL`), and the project's configuration assigns each instance its own address
  (`P1.belt_a.run AT %QX0.0 : BOOL`). An address fixed inside a function block is shared by every instance, so
  component definitions never contain one. An instance left unmapped fails the build.
- **Bound variables are written only through their addresses.** The input image is copied into the variables at each
  scan, so a write by name to an `AT` variable is lost (measured below).
- **M5** maps the same image to S7, Modbus or PLCSIM Advanced. Siemens images are big-endian, and truST's is
  little-endian (measured below), so the M5 driver converts the byte order. The document stores addresses, not bytes.
- **The M3 demo** binds by name (`sensor`, `start`, `stop`, `motor`). It is migrated to addresses when 4.3
  generalises the bench; until then it keeps running unchanged.

Measured on truST `6405915` (2026-11-08), through the `TestHarness` that Twinova uses: a throwaway integration test
declared `AT` variables, wrote the input image with `Runtime::write_io`, ran a cycle, and read the output image with
`runtime().io().read`.

| Question | Result |
|---|---|
| `AT %IX0.0 : BOOL` and `AT %QX0.0 : BOOL` in a `PROGRAM` | compile; `motor := sensor` follows `%IX0.0` to `%QX0.0` scan by scan |
| BYTE/SINT on `%IB`, WORD/INT/UINT on `%IW` | work; signed values travel as two's-complement bits (INT -5 is `Word(65531)`) |
| DINT and REAL on `%ID`/`%QD` | the image is bit-string typed: `Value::DInt` on `%ID4` is `TypeMismatch`, `Value::DWord(41)` works and the program reads DINT 41; REAL travels as its IEEE-754 bits (3.0 in, 1.5 out) |
| LREAL and LINT on `%IL`/`%QL` | work through `LWord` (2.5 in, 5.0 out; 41 in, 42 out) |
| ARRAY and STRUCT at an address | work, laid out contiguously: `ARRAY[0..3] OF INT` at `%IW10` covers `%IW10`–`%IW16` |
| STRING | plain `STRING` is refused at build ("requires STRING[n]"); `STRING[8]` at `%IB0` maps 8 raw bytes |
| Memory area `%M` | `%MX`/`%MW` work |
| `AT %IX1.0` fixed inside a function block, two instances | both instances read the same `%IX1.0` |
| `AT %I*` in the function block, `VAR_CONFIG P1.d1.x AT %IX2.0` and `P1.d2.x AT %IX2.1` | each instance follows its own address, with or without `RESOURCE`/`TASK`; the resource-prefixed path `R.P1.d1.x` is not resolved; an unmapped `%I*` fails the build ("unresolved wildcard I/O bindings") |
| Overlap: `%IX4.0` and `%IB4` | allowed; they alias the same byte, as on S7 |
| Two variables at the same `%QX0.0` | compiles silently; the variable published last wins |
| INT declared at a bit address, BOOL at a word address | accepted silently: truST does not check that size and type match |
| An address beyond the declared image | `write_io` returns `InvalidIoAddress` |
| Image size and byte order | allocated from the declarations; little-endian: DINT 41 is `[41, 0, 0, 0]`, REAL 3.0 is `[0, 0, 64, 64]` |
| Write by name to an `AT` input | `try_set_input` returns `Ok`, but the next input sampling overwrites it from the image |
| Harness access by address | `set_direct_input("%IX2.0", …)` and `get_direct_output("%QX0.0")` exist alongside `Runtime::write_io` and `io().read` |
| Drivers already in truST | `io/` holds Modbus, MQTT, EtherCAT, GPIO and loopback drivers behind `IoDriver` (`read_inputs`/`write_outputs` on byte images): an inventory item for M5, not yet exercised |

The probes were throwaway integration tests in a worktree outside the repository, three runs on 2026-11-08. A
`TASK`'s program runs only once its interval has elapsed (`advance_time`, as Twinova's scan does); a first run
without it read as a failed mapping and was repeated.

**Parameters reach the component's function block the same way.** The generated `VAR_CONFIG` gives each instance
the initial values of its `VAR_INPUT` parameters. This was measured with two instances of one block:
- REAL speeds of 0.5 and 2.0, and DINT limits of 5 (the default) and 9, were read back per instance.
- A `TIME` delay of 200 ms drove one instance's `TON` on after 400 ms, while the other kept the 1 s default.
- A parameter of the wrong type fails the build (`E201`, "VAR_CONFIG type 'DINT' does not match target type 'REAL'"),
  and so does a parameter that does not exist (`E202`).

Initial values only: a parameter edited in the inspector takes effect at the next Build, which the run lifecycle
below already requires.

What this does not prove: `VAR_CONFIG` paths into nested function blocks or arrays of instances (Twinova can name
each instance at the top of the controller program instead), the build time of a large generated configuration,
and any external driver.

**Decision components** (diverter, turntable, later shuttle) are commanded by the PLC only in v1 (user,
2026-11-08): the component executes the commanded output and reports its position. A component-internal routing
rule, as some commercial catalogues offer, is later work and is added without changing existing documents.

A geometric snap may propose a transport connection, but only after compatibility validation and with a visible
result. Two touching meshes alone do not establish a signal or material-flow connection.

## Behaviour and runtime contract

Native analytic transport implements movement and occupancy. Component ST function blocks implement device
logic and bind to typed signals; the line controller remains distinct from component behaviour. Each component
instance has independent behaviour state. Built-ins and future user definitions use the same published contract.

The runtime owns clock and state; the 3D view consumes snapshots. External inputs are recorded with virtual time
and a deterministic sequence. Scene IDs establish stable construction order independent of JSON map ordering.
Loading the same authored project and input log must produce the same event trace.

Before generalising the bench, probe and record the ordering of transport events, sensor updates, PLC input
sampling and output application at the same timestamp. Preserve M3's tested scan behaviour. Do not promise a
new simultaneous-event priority scheme without measuring it against the existing fingerprints.

A transfer requires acceptance by the destination and an atomic ownership change. A blocked destination keeps
the product upstream; notification when space becomes available schedules a deterministic retry. The precise
front/rear crossing and reservation rule for products spanning two conveyors is a required 4.3 design probe.
It must cover unequal speeds and product lengths before the first multi-belt simulation is accepted.

The live OIP probe showed that a scene can expose active Pause/Stop controls while Problems reports that its ST
program does not compile. Twinova's proposed contract is deliberately explicit: required controller build failure
prevents entering Run for that line, and the UI must separately identify scene state and controller state.

## Editor and run lifecycle

Proposed layout: catalogue and scene tree at left, 3D viewport in the centre, contextual inspector at right,
ST and signal table in an expandable lower dock. Final density and shortcuts are informed by the live OIP study.

- Edit: place, move/rotate, configure, group, connect and undo. A drag commits one undoable command.
- Run: lock structural changes; manual commands/forces remain explicit recorded simulation inputs.
- Pause: freezes virtual time; Step advances a documented controller boundary. Structural editing requires
  returning to Edit and rebuilding, with a visible reset of runtime state.
- Build: validate document and all bindings before replacing the running bench. Failure preserves authored content.
- Reset: restores runtime initial state, not the document's undo history.
- Keep authored values, effective runtime values and forced values distinguishable in the inspector.

Do not inherit OIP's bound-but-undriven tag rules merely because they exist. Twinova's proposed first rule is:
required bindings must be complete before Run; optional unbound inputs use explicit definition defaults.
Runtime faults stop the run and identify component, signal/program location and error.

## Persistence and file integrity

Proposed project folder: manifest, scene JSON, component definitions, local glTF/GLB assets and `.st` sources.
Prefer a readable folder for Git and VS Code; a portable archive can wrap the same representation later.
Distinguish project saves from M3's in-memory replay snapshots.

Version schema and migrations; reject a newer unsupported version without overwriting it. Resolve referenced
assets relative to the project and validate paths; missing files must produce actionable diagnostics.
Open into a candidate document, validate completely, then replace the active document. Save through a temporary
file/validated commit procedure appropriate to the target platform. A failed save cannot corrupt the previous one.
The exact manifest and atomic multi-file save protocol belong to 4.2 and require a round-trip/failure probe.

## First acceptance families

The minimal set for the first acceptance scene, derived from research 11 §7 and from M3's belt. Units are those of
the document; addresses are examples the editor would propose.

| Family | Parameters | Signals (direction seen from the PLC) | Material-flow interfaces |
|---|---|---|---|
| Source | product type, interval (s), first arrival (s), maximum count (0 = unlimited) | none in v1 | one output |
| Belt | length, width, height of the top (m), speed (m/s) | input `running` (BOOL); output `run` (BOOL) | one input, one output |
| Diverter | length, width (m), switch time (s), default output | output `divert` (BOOL); input `in_position` (BOOL) | one input, two outputs |
| Diffuse sensor | position along the host belt (m), range (m) | input `detected` (BOOL) | none; mounted on a belt |
| Sink | none | input `count` (DINT) | one input |

A belt's `run` is the actuator command (`%QX`), and its `running` the feedback (`%IX`). The sink's counter is a
DINT on a double word (`%ID`), which exercises the measured DWORD path. Through-beam, colour and barcode sensors,
the stopper, pusher and turntable belong to 4.4's later families.

## First acceptance scene and proof obligations

The line is source → belt A → diverter → belt B → normal sink, with a second diverter output to a reject sink.
Two sensors drive the ST controller. Acceptance is not a successful render alone:

1. Assemble it through the UI, edit parameters, wire signals, save, close and reopen.
2. Block belt B; upstream accumulation remains finite and no product is lost or duplicated.
3. Select the reject path; only products admitted after the route commitment use the new route.
4. Repeat identical inputs under Step, ×1 and unpaced execution; compare IDs, event order and results.
5. Rename, duplicate and undo a group; internal references remain valid and external links do not silently change.
6. Corrupt a reference, remove an asset, use an unsupported type and interrupt a save; preserve the original project.
7. Measure multi-component throughput and webview memory growth against M3. Report scale, duration and hardware.

## Phase handoff and remaining probes

- 4.1: this model, connection distinctions and first-user workflow. Technical review precedes format implementation.
- 4.2: specify and implement the concrete format/loader/migrations and project round-trip in the actual app.
  Its acceptance is open, validate, display and save fixture projects, not execution of arbitrary loaded lines.
  The existing M3 demo remains runnable. Imported documents without an executable model show why Run is disabled;
  they must never silently execute the unrelated M3 demo. Multi-component execution belongs to 4.3–4.5.
- 4.3: validate native/ST binding and multi-component timing/transfer semantics before engine integration.
- 4.4–4.5: ship the first complete line vertically, then expand to the roadmap's remaining component families.
- 4.6–4.8: external ST files, diagnostics and debugger work remain their own integration gates.

Measured for 4.1 (2026-11-08): signal binding by direct address, per-instance addresses and parameters through
`VAR_CONFIG`, and the validation truST leaves to Twinova (see "Binding to the controller").

Not yet measured, and owned by the phase that implements them, because each needs the implementation to exist:
- product spanning semantics across two conveyors, and same-timestamp ordering with several components (4.3);
- the multi-file atomic save (4.2);
- catalogue scale, multi-component performance and editing ergonomics (4.4–4.5).

These are named probes with their phase, not completed capabilities.
