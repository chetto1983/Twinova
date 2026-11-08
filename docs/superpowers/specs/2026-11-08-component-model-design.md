# M4.1 — Component model and editor foundation

Status: **draft for technical review**, 2026-11-08. The user asked to start the recommended catalogue-first
direction, then requested a live exploration of Open Industry Project before finalising M4.
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

- Right-handed coordinates, Y up; metres and radians in document geometry, mm and degrees available in the UI.
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
input; an actuator's input is a PLC output. The editor must label that perspective. Initial binding types are
BOOL, DINT, REAL and STRING; implementation must verify their exact truST representations before enabling them.
No implicit numeric coercion; no competing writers. Invalid binding prevents Run with a path to the offending endpoint.

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

Not yet measured: general parameter binding to the pinned VM, product spanning semantics, multi-file atomic
saves, catalogue scale, multi-component performance and editing ergonomics. These are named implementation
probes, not completed capabilities. This draft will absorb the live OIP findings before approval of 4.1.
