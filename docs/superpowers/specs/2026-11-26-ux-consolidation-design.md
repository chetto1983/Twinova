# Spec: a usable app before M5 (ROADMAP M4b, phases U0–U8)

The user, 2026-11-26, after testing v0.4.0:
- "La UI/UX è impresentabile";
- "non si capisce niente, manca completamente la parte di UX di un simulatore industriale";
- "prima si consolida quello che c'è, io voglio che sia simile a Visual Components e SIMIT";
- "M5 non ha senso se la app non è usabile".

They also judged Open Industry Project's demo "fatta bene e chiara". This spec is U0: it fixes the usage model, the layout,
the visual rules and the acceptance. Phases U1–U8 implement it; no new feature enters before them (4.11c and M5 wait).

## Measured first

Sources:
- the live audit of the v0.4.0 AppImage, `docs/superpowers/verification/2026-11-26-ux-audit.md`;
- Twinova's UI code at `886a78b`;
- research [01](../../research/01-ui-layout-simulazione.md) (Visual Components),
  [g1 §3](../../research/benchmark/g1-virtual-commissioning.md) (SIMIT) and
  [21](../../research/21-oip-funzionalita.md) (Open Industry Project).

**SIMIT's interface is not verified.** Its public manual is on `cache.industry.siemens.com`, which this environment
cannot reach (the host was not added, 2026-11-26). Research g1 covers SIMIT's couplings, charts, trends, messages,
snapshots and 3D viewer from that manual, not its windows. Where this spec says "as SIMIT", the user, who uses it,
confirms or corrects.

| What | Measured | Where |
|---|---|---|
| Shell | One grid: scene tree 224 px, view, Inspector 292 px; a dock below with Parts, PLC, Problems, Watch; a status bar | `App.svelte:492`, `WorkspaceDock.svelte`, `StatusBar.svelte` |
| Type sizes | Body 12 px. Of 102 `font-size` declarations, 86 are below 12 px: 10 px ×38, 11 px ×33, 9 px ×14, 8 px ×1 | `app.css`, `*.svelte` (grep) |
| Colour | Contrast is not the fault: dark text `#eef2f6` on `#101214`, muted `#c8d0d8` | `app.css:32-51` |
| Live data | The snapshot already carries every signal's name, address and value, and each actuator's value | `lib/snapshot.ts:21-36` |
| Where live data shows | Only in Watch's I/O list and the PLC editor. The Inspector shows a signal's address as editable text and no value | `IoPanel.svelte`, `Inspector.svelte:168-204` |
| 3D state | Belt fabric scrolls and the pusher's lamps light. A sensor draws no beam and no state; no flow arrows; no value in the scene; counters in a corner | `ProjectScene.svelte:137-275`, `App.svelte:436` |
| Plant and controller | The PLC organizer lists the components' behaviours (`kind: "behaviour"`) as "Function blocks" beside the line program | `PlcOrganizer.svelte:41-47`, `lib/sources.ts:9` |
| Problems | The analyzer's W001 (unused variable), W008 (complexity) and W009 (unused program) show as problems: 23 on `sort_line`, 25 on `code_sort_line`, with the build at 0 warnings. W009 names `Main`, the program the task runs | audit; `src-tauri/src/analysis_tests.rs:49` |
| Editing while running | A scene edit is accepted at any time; the gizmo stays on the selection while the line runs | `App.svelte:222`, audit |
| Operator panel | `project.panel` buttons, drawn in Watch; no example has one, so the section is a heading over nothing | `lib/project.ts:68`, `WorkspaceDock.svelte:226` |
| Commands | A registry of app and PLC commands exists (used by MCP) | `lib/control/registry.ts`, `appCommands.ts`, `plcCommands.ts` |
| PLC dock at 1366 × 768 | 1 line of code, 1½ rows of the variables table | audit |

## The usage model (decisions)

The references, in our own words:
- **Visual Components:** work contexts as tabs that change the commands and panels in view (UI-01, UI-02); a catalogue
  with pictures (CAT); a component list and a properties panel (LAY-08, UI-09); signals connected in a table or in 3D,
  and toggled by hand while the simulation runs (LAY-12, SIM-19); the run controls above the 3D view (SIM-01); an
  output panel for messages (SIM-17).
- **SIMIT** (g1, to confirm): the I/O of each coupling as a table of signals; live values on the model; trends and
  messages; snapshots.
- **Open Industry Project** (21): the state drawn in the scene; a signal field that says its direction, type, whether it
  is connected, and its live value with Force; "Wired to" between a variable and the parts; a tutorial driven by real
  actions; every disabled control says why; written empty states.

### 1. Three work contexts

Tabs at the top left: **Layout**, **Signals**, **PLC**. Each one decides the panels in view.

| Context | Left | Centre | Right | Below |
|---|---|---|---|---|
| Layout | component tree | 3D view | Inspector | catalogue |
| Signals | component tree | 3D view | Inspector | the signal table (decision 3) |
| PLC | organizer | code or ladder, full height | variables of the file | Problems |

Low-fidelity wireframes (`wireframes/2026-11-26-ux.html`, rendered at 1440 × 900):
[Layout, stopped](img/2026-11-26-ux-wireframe-layout.png), [Signals, running](img/2026-11-26-ux-wireframe-signals.png),
[PLC](img/2026-11-26-ux-wireframe-plc.png). They fix where things go, not how they look.

- **The run controls stay in the top bar in every context.** Running does not change the context.
- **Watch disappears as a tab.** Its I/O list becomes the signal table, its timings go to the PLC context, its faults go
  to the Inspector of the sink (decision 4).

### 2. The plant is not the controller

- The PLC organizer lists the controller only: task, programs, the user's own blocks.
- A component's behaviour is reached from the component: the Inspector gets a **Behaviour** section with "Edit
  behaviour", which opens its ST in the PLC context's editor under a heading that names the component.
- The organizer gains a **Devices** list (as OIP): each component with signals, opening its rows in the signal table.

### 3. Signals at the centre

- **One table** for the whole line: Component | Signal | Direction (PLC input / PLC output) | Type | Address |
  PLC variable | Value | Force.
- **Linked both ways:** a row selects its component in the tree and the view; a selected component filters the table.
- **Editable while stopped:** the address and the variable. **Live while running:** the value, a BOOL toggles with a
  click, Force holds it.
- **The Inspector's Signals section** says the same per component: direction and type as words ("PLC output · BOOL"),
  the address, a verdict (connected / not in the program / wrong type / unbound), and while running the value with
  Force.
- **Assign addresses** (deterministic, as OIP's Assign I/O) and **export/import CSV** (Component, Signal, Direction,
  Type, Address, Variable), so a TIA Portal tag table can be prepared from it.

### 4. The state is drawn in the scene

- A sensor draws its beam: green when clear, red when interrupted.
- A running conveyor keeps its scrolling fabric; flow arrows on demand.
- Actuators' lamps as today (pusher), and for the new operator devices.
- Counters stand at their source and sink, not in a corner.
- A label with a live value can be pinned to a component (as OIP's annotations), off by default.
- A blocked sink is drawn as blocked, and blocked or freed from its Inspector (the fault leaves Watch).

### 5. Operator devices are components

Push button (momentary or latching, NO or NC), emergency stop, selector switch, pilot lamp, stack light. They are placed
in the scene and listed in an **Operator** strip that appears while running. `project.panel` buttons stay readable and
are shown there too. The examples gain Start, Stop and E-stop.

### 6. Layout is locked while the line runs

- While running or paused, the scene cannot be edited: no gizmo, no catalogue, the Inspector's fields read-only except
  faults and forces. Stop returns to editing.
- **Why:** Twinova's value is a replay that gives the same states for the same inputs. A layout edited mid-run has no
  replay, and Visual Components resets to the state saved at Play anyway.
- The PLC's online change stays allowed: it is the controller's edit, not the plant's.

### 7. Problems tell the truth

- **Errors first.** The tab's badge counts errors only.
- **Hints hidden by default:** the analyzer's style warnings (W001, W008) are shown on demand.
- **W009 removed** for a program a task runs.
- **The shipped examples show zero problems at the default level.**

### 8. Visual rules

- **Type:** body 13 px; labels at least 11 px; code 13 px. Nothing smaller.
- **Targets:** at least 28 px high.
- **Hierarchy:**
  - the scene's name appears once, in the project button;
  - the object count appears once, in the tree;
  - the status bar keeps run state, time and build state only;
  - language and theme move to Settings.
- **Nothing empty on screen:**
  - a section with no content is not drawn;
  - the Inspector with nothing selected explains the scene's step and scan (as OIP).
- **Names:** families by their name ("Belt", "Pusher"), without `@1` or "Twinova · v1".
- **The scene:**
  - the opening camera frames the line;
  - the warehouse decoration is lighter and can be turned off.
- **Disabled controls say why,** from the command registry.

### 9. First use

- **A tutorial driven by real actions** (the structure of OIP's, our own text and steps), in this order:
  1. open an example and run it;
  2. watch a sensor's beam and its signal;
  3. stop;
  4. place a belt and a photo-eye;
  5. bind their signals;
  6. write the interlock;
  7. run it.
- **Every example has a description:** what it does, its I/O, and what to watch.

## Acceptance

**Machine-checked** (unit, E2E on the real app, or a check script in CI):

| # | Check | How |
|---|---|---|
| A1 | No `font-size` below 11 px | script over `app/src`, beside `check-file-size.sh` |
| A2 | At 1366 × 768: the PLC context shows at least 20 lines of code; the signal table at least 12 rows | E2E, window resized |
| A3 | Every shipped example builds with 0 problems at the default level | E2E over `list_examples` |
| A4 | While running, each sensor's beam colour follows its signal in the snapshot | E2E over MCP `state` and the scene graph |
| A5 | The Inspector's value of each signal equals the snapshot's | E2E |
| A6 | A signal row selects its component; a selected component filters the table | E2E |
| A7 | No component behaviour in the PLC organizer | E2E |
| A8 | While running, no scene edit is accepted (gizmo absent, `edit` refused with a reason) | E2E |

**Task tests with a person** (the real acceptance of the milestone):

| # | Task | Done when |
|---|---|---|
| T1 | Open the sorting example, run it, and say why a carton goes to the reject sink | right answer, no help |
| T2 | Find which PLC input the stopper's photo-eye drives, and force it | done in the app, no help |
| T3 | Build a line from an empty project: two belts, a photo-eye, a pusher, Start/Stop buttons; bind the signals; write the program; run it | the line sorts as asked |
| T4 | A broken program is given (a wrong address): find and fix the fault | fixed, no help |

- **Baseline:** v0.4.0, run by the user ("non si capisce niente").
- **Who runs them:** ideally someone who has never seen Twinova, as well as the user.
- **What is recorded:** time per task, where they got stuck, and success, in a dated verification file per phase.

## Phases

| Phase | Content | Decisions | Acceptance |
|---|---|---|---|
| U1 | Remove the noise: problems, duplicates, empty sections, names, language and theme out of the bar, AppImage built on Ubuntu 22.04 | 7, 8 (part) | A3; the audit's list cleared |
| U2 | Plant apart from controller: Behaviour section, Devices list | 2 | A7 |
| U3 | State in the scene | 4 | A4 |
| U4 | Signals: the table, the Inspector section, assign, CSV | 3 | A5, A6 |
| U5 | Operator devices | 5 | examples with Start/Stop/E-stop |
| U6 | The three contexts; layout locked while running | 1, 6 | A2, A8 |
| U7 | Visual rules; the light theme by default | 8, answer 3 | A1 |
| U8 | Tutorial and example descriptions | 9 | T1–T4 by the user |

**Proposed order:** U1, U7, U6, U2, U4, U3, U5, U8. The type sizes and the contexts come first, because every later
phase draws inside them.

## The user's answers (2026-11-26: "sì a tutto, tema chiaro, parti con U1")

1. **Contexts:** Layout / Signals / PLC as tabs (decision 1).
2. **Layout locked while running** (decision 6).
3. **Default theme: light**, as Visual Components and TIA Portal. Dark stays a choice in Settings.
4. **Task tests:** run by the user; anyone else is a bonus, not a condition.
5. **SIMIT:** the signal table and live values on the model, as assumed; trends wait (below).

## What this spec does not decide

- Trends and a logic analyzer, and faults injected while running: after U8, if the task tests ask for them.
- M5 (external PLCs): after this milestone.
- 4.11c (Ladder drag and drop): after this milestone.
