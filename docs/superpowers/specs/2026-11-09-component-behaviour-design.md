# Spec: component behaviours in ST and the field runtime (ROADMAP 4.3)

Date: 2026-11-09. Status: **approved** (user, 2026-11-09); transport amended to physics the same day (user, research
15). Three decisions were the user's choices and are marked
(user); the user approved the rest as proposed, with the precisions of "Amended while planning".

ROADMAP 4.3: behaviours written as ST function blocks bound to component signals. Acceptance: a component's logic
edited and rebuilt in the app. The model is the approved M4.1 spec (`2026-11-08-component-model-design.md`), which
leaves two probes to this phase: the same-instant order with several components, and a product spanning two
conveyors. The format is 4.2's (`2026-11-08-project-format-design.md`), which this spec amends.

## Decisions

1. **Behaviours run in a field runtime, separate from the controller** (user). It is a second truST runtime that
   simulates the devices, as SIMIT's I/O simulation does.
   - It sees the controller's outputs as its inputs and writes the controller's inputs.
   - In M5 the controller becomes a real PLC (S7, PLCSIM Advanced) and the devices stay in Twinova unchanged.
   - This replaces the 4.2 schema's reading, in which a component's block sat inside the line program.
2. **The belt and the diverter have ST behaviours in 4.3** (user). Source, diffuse sensor and sink stay native.
   - Belt: `run` → motor → `running` after a start delay.
   - Diverter: `divert` → the flap moves for `switch_time` → `in_position`.
3. **The field runtime scans every 1 ms, fixed** (user), whatever the controller's period.

## Measured (truST `6405915`, 2026-11-09)

Two throwaway tests, in a worktree outside the repository, built against the app's dependencies.

| Question | Result |
|---|---|
| A field runtime beside the controller, in one process | works: two `TestHarness`, the field one with a `CONFIGURATION`, a 1 ms `TASK` and `VAR_CONFIG` |
| Two instances of one behaviour block | independent: `d1` took `switch_time := T#200ms` from `VAR_CONFIG`, `d2` kept its 300 ms default |
| Signals across the two | copied per address, `%QXn.m` of the controller to `%IXn.m` of the field and back: the controller's command at 50 ms gave its feedback at 250 ms (200 ms), the second at 200 → 500 ms (300 ms) |
| A TON in the field | its `Q` comes one cycle before PT when `IN` rises on its own: `IN` first seen at the 2 ms cycle, `Q` at the 301 ms cycle with PT 300 ms. A TON called with `IN := FALSE` and then `TRUE` in the cycle of the command gives exactly PT (the diverter's block, Task 2). A device delay is accurate to 1 ms |
| A block compiled alone | refused when it uses `AT %I*`: "direct address '%I*' requires VAR_CONFIG mapping" (E202), so the field compiles as one source, and Build rewrites truST's byte ranges (`at 385..386`) as the behaviour file and line |
| The native side reading and writing a block's variables | by path, with no address: `get_output("F.b0.motor")` reads a `VAR_OUTPUT`; `try_set_input("F.b0.fault", true)` writes a `VAR_INPUT`, and the value stays |
| Cost of a 1 ms field cycle (release build) | 2 devices 7.7 µs; 20 devices 0.10 ms; 100 devices 0.74 ms. About 7 µs per device |

The cost row bounds the decision. At 100 devices the field alone uses 74 % of one core at ×1, and "as fast as
possible" cannot exceed about 1.35 × real time. The M4 scenes have under 20 devices. Skipping idle cycles is
possible later, under the rule "baseline or drop".

## The field runtime

- **What Twinova generates at Build:** from the scene and the pinned definitions, a `PROGRAM Field` with one
  instance per component that has a behaviour, named by its instance id. Also a `CONFIGURATION` with a 1 ms
  `TASK`, and a `VAR_CONFIG` that has:
  - each signal at its mirrored address: `F.belt_a.run AT %IX0.0 : BOOL` for the scene's `run` at `%QX0.0`;
  - each parameter override as an initial value: `F.belt_a.start_delay : TIME := T#200ms`.

  None of this is stored in the folder.
- **The mirror:** a field address is the controller's address with `I` and `Q` swapped, same byte and bit. The
  scene keeps the controller's view, as 4.2 decided.
- **A behaviour block's contract**, checked at Build with a diagnostic that names the definition and the variable:
  - each signal of the definition is declared under its name: `AT %I*` for a `plc_output` (a command the device
    receives), `AT %Q*` for a `plc_input`;
  - a parameter the block uses is a `VAR_INPUT` of the same name and IEC type, and gets the instance's value. A
    parameter the block does not declare is not passed;
  - its family's native variables, with the types below. The Rust model of the family reads and writes them by path.

| Family | Native variables (the native model reads ← / writes →) |
|---|---|
| Belt | ← `motor : BOOL` (`VAR_OUTPUT`): the belt moves while it is true |
| Diverter | ← `position : DINT` (`VAR_OUTPUT`): 0 at output A, 1 at output B, −1 while the flap moves. The block owns `switch_time`; the native model of 4.4 routes by `position` |

- **A native component** (no behaviour) writes its `plc_input` signals straight into the controller's image, and
  reads its `plc_output` signals from it.
- **Instance ids are ST identifiers.** 4.2's ids match `^[a-z][a-z0-9_]*$`; Build also refuses an id that is an ST
  keyword (`end_if`, `on`, …), with a diagnostic.

## The controller by address

- **The line program declares its I/O at addresses**, as on a real PLC: `belt_run AT %QX0.0 : BOOL;`. Twinova
  writes and reads the controller's image by address, never by name; the M3 contract `sensor/start/stop/motor`
  goes.
- **Build checks the scene against the program:** a scene signal whose address the program does not declare is a
  warning, not an error, because a real line can leave a signal unused. The variable the program declares at a
  scene signal or a panel button must have that signal's type (BOOL for a button), or Build fails.
  - *Amended in Task 3:* truST sizes a binding from the variable's type, not from the address in the source:
    `a AT %ID8 : BOOL` becomes a bit at byte 8 (measured 2026-11-09). So the check cannot be "the type fits the
    written size". It is done where Twinova and the program meet: the scene's address, matched by area, byte and
    (for a bit) bit, against the declared variable's type.
  - An address outside the image reads as zero; writing one is an error.
- **Operator buttons (proposal):** `scene.controller.panel` lists `{ "name": "Start", "address": "%IX1.0" }`. The UI
  shows one button per entry, and a press holds the bit for one controller scan, as M3's Start and Stop do today.
  `controller.program_instance` is no longer used and is removed.

## The M3 demo becomes a project

- The demo becomes the `demo_m3` project, bundled with the app and run by the generic bench. The hard-coded demo
  bench and `CONTRACT` go.
- Its program is M3's, moved to addresses: `sensor AT %IX0.0`, `motor AT %QX0.0`, Start and Stop on the panel at
  `%IX1.0` and `%IX1.1`. The jam logic is unchanged.
- **The timing changes by design.** The PLC's `motor` now reaches the belt through the belt's behaviour, one field
  cycle later, plus the start delay (default 0). The replay fingerprints are recorded again in that commit, with
  the reason. The M3 scan behaviour, deliveries and jam detection must hold, and the E2E checks them.

## Transport with physics (amended 2026-11-09, user; research 15)

*This section replaces the analytic transfer rule and the event order proposed in the approved draft. While the plan
was being written, the user asked to do as Open Industry Project does, with physics. Rapier and Jolt (via jolt-rust)
were then measured on the same probe, and the user chose Rapier.*

- **Products are rigid bodies in Rapier 0.36.1**, the version truST already brings, now a direct dependency with
  `enhanced-determinism`. Belts are fixed boxes whose contacts get a tangent velocity in
  `PhysicsHooks::modify_solver_contacts`, Rapier's documented mechanism for conveyor belts.
- **No transfer rule.** A product passes from one belt to the next through its contacts and friction, as in OIP. A
  product spanning two belts moves at a speed between theirs; a stopped or full belt holds it; boxes accumulate by
  collision.
- **Seam gate, as OIP's** (`conveyorContacts.ts`): on a belt contact, a contact whose normal is within 45° of the
  horizontal and whose depth is at most 2 mm is dropped. Without it a box lifts 4–5 mm at a seam, slows down or
  bounces back, and can stay stuck on the next belt's edge (measured).
- **Friction as OIP declares it:** a belt's friction is the coefficient against a standard carton (0.5). Rapier has
  no geometric mean, so contacts use `Multiply`, and a belt's collider friction is μ / 0.5.
- **A fixed step of 1 ms** (user), the field runtime's cycle. At each step:
  1. the physics step;
  2. the native models read it: the diffuse sensors cast a ray across the belt, a sink removes the boxes it holds,
     a source adds a box when its interval has passed and its exit is free;
  3. the field cycle;
  4. the controller's scan, when it is due.

  A sensor edge and a delivery therefore fall on a millisecond, not on a nanosecond as in M3.
- **Why a fixed-step loop and not discrete events** (research 16): the most industrial virtual-commissioning setup, NX
  MCD with SIMIT's "bus synchronous" mode and PLCSIM Advanced, is physics with transport surfaces in lockstep on a
  fixed time slice (≥ 1 ms) of virtual time. A fixed-step loop is that lockstep; in M5 the same loop drives PLCSIM
  Advanced through its sync points (TimespanSynchronized, `RunToNextSyncPoint`).
- **M3's analytic belt is replaced** (user). `belt.rs`, `scene.rs` and spec `2026-11-08-analytic-belt-design.md` stay
  in git history only. nexosim no longer drives the line: with one fixed step the line is a loop, not a discrete-event
  model. Replay is unchanged, because the inputs are logged with their time.
- **Determinism:** the same inputs give the same bits; a test pins a state fingerprint that must hold on Linux and on
  Windows (CI). Same process measured; across platforms promised by `enhanced-determinism`, to be measured.
- **Cost (measured, release, `enhanced-determinism`):** 3.7 / 22 / 98 µs per 1 ms step for 1 / 20 / 100 boxes.

## The app

- **Editing a behaviour:** the editor has one tab for the controller program and one per definition that has a
  behaviour. A definition's block serves every instance of it, as its pinned copy does. Build checks and builds
  both runtimes. A failure keeps the running line and shows the diagnostic for the tab and the line.
- **Running a project:** Run is enabled for a project whose components all have a model in this build: sources,
  belts, diffuse sensors and sinks in 4.3, in any arrangement physics allows. Otherwise Run stays disabled and the UI
  names what is missing, as 4.2 requires. The diverter runs from 4.4, with its physical flap.

## Format amendments (format 1, not yet released, amended in place)

- `components/<id>@<version>.st` holds the behaviour block of a definition whose `behaviour` is not `null`. It is
  listed in the manifest with its hash, and its block must be the one `behaviour.function_block` names.
- `scene.controller` becomes `{ "panel": [ … ] }`.
- The fixtures change accordingly, and `demo_m3` gains the M3 program and its panel.

## Acceptance of 4.3

1. In the real app, on Linux and Windows:
   - open `demo_m3` and run it: parts are delivered, and Start, Stop and the jam behave as in M3;
   - edit the belt's behaviour (a start delay of 500 ms) and Build: `running` rises 500 ms after `run`, to the
     millisecond and its measured bound;
   - break the block: Build fails with its diagnostic and the line keeps running.
2. The diverter's behaviour, with a controller program, in field-runtime tests: both directions, `switch_time` per
   instance.
3. Transport with physics: two belts at unequal speeds, a stopped belt, accumulation and release, no box lost; the
   step order; a pinned state fingerprint, equal on Linux and Windows.
4. Coverage, mutation (the field runtime's generator and the transfer rule join `mutants.toml`) and the soak on the
   project demo against M3's numbers.

## What this will not prove

- **A line with a diverter running in the app**, which needs the line engine of 4.4.
- **A real PLC on the controller side**, which is M5.
- **Field-runtime cost beyond the measured 100 devices**, and any optimisation of idle cycles.
- **User-defined families.** Only the built-in definitions' behaviours are edited.

## Amended while planning (2026-11-09)

- **`demo_m3` gains a sink.** M4.1 makes an unconnected exit block, so without a sink the demo would deliver
  nothing. The sink is native and runs in 4.3. M3's "outlet blocked" toggle becomes the sink's runtime fault, an
  input recorded for replay as before.
- **The belt definition gains `start_delay`** (TIME, default 0), the parameter its behaviour uses. A behaviour block
  declares only the parameters it uses.
- **The 4.3 bench runs one line shape:** source → belt → sink, with sensors on the belt. *Superseded by "Transport
  with physics": any arrangement of sources, belts, sensors and sinks runs.*
- **The diverter's native variable is `position`,** not `drive` and `at_b`. With `at_b` the flap's timing would sit
  both in the block and in the native model, and no native diverter writes it before 4.4. The block owns
  `switch_time`, and its `default_output` is the flap's position at start.
