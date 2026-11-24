# Twinova guide

Twinova simulates a conveyor line in 3D and runs its PLC program, written in Structured Text or Ladder (IEC 61131-3),
inside the app. This guide takes you from the installer to a line you have edited, programmed and debugged, and to a
coding agent that drives the same window over MCP.

## Install

Download from the [latest release](https://github.com/chetto1983/Twinova/releases/latest):

| System | File | What it installs |
|---|---|---|
| Windows | `Twinova_<version>_x64_en-US.msi` | the app and `twinova-agent.exe` beside it |
| Debian, Ubuntu | `Twinova_<version>_amd64.deb` | `/usr/bin/twinova` and `/usr/bin/twinova-agent` |
| Other Linux | `Twinova_<version>_amd64.AppImage` | the app in one file; take the agent from the next row |
| Agent alone | `twinova-agent-x86_64-pc-windows-msvc.exe`, `twinova-agent-x86_64-unknown-linux-gnu` | the MCP server only |

The installers are not signed yet: Windows SmartScreen asks once (More info, then Run anyway).

## The example lines

The app opens on the M3 demo, one belt with a photocell, a Start and a Stop button. The project menu (the project's
name, top left) lists every example under **Examples**:

| Example | What it shows |
|---|---|
| M3 demo | one belt, Start and Stop, a jam alarm after 2 s under the photocell |
| Merge line | a spur merging into a main line |
| Two belts and a chain transfer | every third carton raised by the chain transfer and sent to reject |
| Two belts and a chain transfer, in Ladder | the same line, its program in Ladder |
| A line with two curves | belt and roller curves |
| A sorting line with a stopper, a turntable and a pusher | sorting at a stopper, a turntable and a pusher |
| Code and colour sort line | sorting by barcode and colour, with laser and colour sensors |

An example opens as a project with no folder: **Save** asks for one, and from then on it is your project. The same
menu creates, opens and saves projects, and exports or imports a project as one `.zip`.

## The scene

- **Catalogue** (Parts tab, below the scene): drag a conveyor, a device, a sensor, a source or a sink into the 3D view.
  Conveyors snap end to end; a sensor or a device mounts on the conveyor it is dropped on.
- **Move and turn** with the gizmo (W move, E rotate); drag elsewhere to orbit, scroll to zoom.
- **Inspector** (right): the selected instance's name, position, parameters (length, speed, …), its signals' I/O
  addresses, and its connections.
- **Undo and redo** in the toolbar; the History tab lists every edit.

## Build and run

The toolbar: **Build** compiles the program and the components' logic and puts the line at time zero; **Run** at ×0.25
to ×10 the wall clock or Max; **Pause**; **Step**, one PLC scan; **Reset**; **Save state** and **Restore**. The status
bar shows the virtual time and the build. The Watch tab lists every signal, the panel's buttons and the sinks' counts;
press a button there, or block a sink as its fault does.

## The PLC pane

The PLC tab holds the organizer (the task, the programs, the function blocks) and the editor.

- **Structured Text** files (`plc/*.st`) are edited as text, with completion, call templates, go to definition and
  rename. Each component's logic is a function block you can open and edit too.
- **Ladder** files (`plc/*.ld`) open drawn: the toolbox adds rungs, contacts, coils, boxes and branches; Enter edits an
  operand, and Auto Declare offers a declaration for an undeclared name. The button beside the editor's `LD` label
  switches between the drawing and the file's text.
- **The variables table** above the code lists the file's declarations with their addresses and initial values.
- **PLCopen** (two buttons in the pane's header): export the Ladder file shown as PLCopen TC6 XML, or import a file's
  Ladder programs from another tool.
- The pane can go in a window of its own (its header's button), and back.

## The online tools

- **Monitoring** (Ctrl+T): values beside the code, on the Ladder drawing and in the table's Value column.
- **Write and force**: click a value to open the bar under it; Write sets it once, Force holds it until Release. A
  forced value is marked.
- **Breakpoints**: click the gutter, or in a Ladder drawing a rung's number. A breakpoint holds the line as an
  S7-1500's HOLD: virtual time stands, the call stack and the values are shown, and Continue (F5), Step over (F10), Step
  into (F11), Step out (Shift+F11) and Run to cursor (Ctrl+F10) go on. A breakpoint takes a condition, a hit count or a
  log message (Shift+click, or a right click). In a Ladder file a breakpoint goes on a rung, as on a CODESYS network:
  the held rung is marked, and a step goes to the next rung.
- **VS Code**: a saved project's `.st` files can be edited in VS Code while the line runs ([ST in VS Code](vscode.md)).

## Connecting an agent (MCP)

`twinova-agent` is an MCP server on stdio. Register it once with the clients you have:

```sh
twinova-agent setup
```

It adds the agent to Claude Code for every project (`claude mcp add --scope user`) and to Claude Desktop's
configuration, and says what it did; restart Claude Desktop afterwards. For another client, run `twinova-agent` as a
stdio server.

**With the app open**, the agent attaches to it, and what it does happens in the window you are looking at:

| Tool | What it does |
|---|---|
| `list_commands`, `describe_command` | every action of the window by name, and a command's parameters as a JSON Schema |
| `execute` | runs one command, as its button does |
| `batch` | runs commands in order; when one fails, the scene's edits the batch made are undone |
| `state` | the project, the build, the line's snapshot, the sources, notices and errors |
| `screenshot` | the window as drawn, as a PNG (`main`, or `plc` for the PLC pane's own window) |

The commands cover the projects and the examples, the scene's edits, Build and the run controls, the sources, the PLC
pane (file, view, monitoring, the table's values, write, force, release, breakpoints and the steps, PLCopen), the
panel's buttons and the sinks, the dock's tabs, the theme and the language. `plc_window` moves the PLC pane to a window
of its own and back; while it is there, the pane's commands go to it, and `screenshot` takes it with window `plc`.

**With no app open**, or with `twinova-agent --headless`, the agent runs a session of its own, without a window:
`open_project`, `open_example`, `list_examples`, the scene's edits (`add_instance`, `move_instance`, `join`,
`set_parameter`, `bind_signal`, `connect`, …), `build`, `step` and `run_until` on virtual time, `snapshot`, `read`,
`press`, `block_sink`, `save_state` and `restore_state`, `read_source` and `write_source`, `change_program` and
`reload_sources` (online changes), `export_ladder` and `import_ladder`, `fingerprint`, `save_project`.

`twinova-agent --connect <port>` attaches to the app on that port, and `twinova-agent --http [address]` serves the
headless session on streamable HTTP at `/mcp`.

The app opens its control channel on `127.0.0.1` only, with a token made at each start, written in a file only your user
reads (`control/current.json` in the app's data folder). To keep it closed, start the app with `TWINOVA_CONTROL=off`,
or create the file `control/off` in that folder.
