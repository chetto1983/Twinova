# Spec: the points v0.3.0 left open (ROADMAP 2.9)

The user, 2026-11-24: "Chiudiamo i punti aperti". After v0.3.0 these were left open:
1. the Ladder view's four defects (the question asked on 2026-11-14, answered "No preference");
2. breakpoints in Ladder files, blocked in v0.3.0;
3. the agent's commands for the PLC pane, which do not reach it once it is in its own window;
4. `twinova-agent setup` against a real Claude Desktop.

## Measured first (2026-11-24, the debug app of v0.3.0 under Xvfb, driven over MCP)

**The Ladder view.** `ladder_line`, `plc/Main.ld`, dock maximized, window 1440 × 900, online.

| Defect | What the picture shows | Cause in the code |
|---|---|---|
| Box pins crowded | Rung 3, ADD: `count` beside IN1, its value `0` under it, and IN2's constant `1` touch, and no wire joins a name to its pin. | `LadderRung.svelte` writes a pin's place 3 px above its row and its reading 1 to 13 px below. The next row's place starts 12 px below, and the rows are `PIN_STEP` = 24 px apart. |
| Expression detached | MOVE.IN's `count MOD 3 = 0` stands left of the box with no wire and no value. | Same drawing, no stub. An expression has no reading: `monitor_ladder.rs` watches references only. |
| Rungs narrow | The right rail is at x ≈ 963 in a pane about 1210 wide. | `layoutRungs` takes the widest rung or `MIN_WIDTH` (560), never the pane's width. |
| Table too tall | With the dock maximized, the variables table takes 8 rows (≈ 270 px) and leaves 2½ rungs in view. | `.table-area` is `height: 35%` of the editor. |

**Breakpoints in Ladder.**
- `ladder_line`'s `plc/Main.ld` has 42 lines and 13 rungs. Lowered, it is 94 lines, and its rungs are lines 52 to 93.
- A rung generates 1 to 6 statements, and no statement stands outside a rung.
- `Lowered` already knows each generated line's element, and so its rung (`lines`).
- The parser does not keep where a rung's text is.
- The debugger maps a user's file onto its runtime file by an offset (`Place.first_line`), which is right for ST and wrong for a lowered file. In v0.3.0, a breakpoint on line 30 held at "line 52" of the 42-line file.

**The PLC window.**
- `Control::ask` emits `control-request` to `main` only.
- The PLC window's `Editor` registers the pane's commands in its own webview's registry, which listens to nothing.
- While the pane is detached, the main window has no editor, and so no PLC command.
- No command detaches or attaches the pane.

**Setup.** The container has no Claude Desktop, and no computer of the user's is attached (`list_computer_folders`:
none).

## Decisions

1. **The Ladder view.**
   - A pin with a value is drawn on its row: the place (a reference, a constant or an expression), then a short
     wire to the pin. Online, a reference's reading sits on the same row, beyond the place, so no row reaches the
     next. The room beside a box counts the place, the reading and the wire.
   - An expression keeps no reading; its wire joins it to its pin.
   - The rungs are laid out as wide as the pane (`width` measured with a ResizeObserver): the widest rung or
     `MIN_WIDTH` when the pane is narrower, as before.
   - The variables table starts at `min(35%, 13rem)` (about five rows), and is still resized by its handle.
2. **Breakpoints in Ladder are set on rungs, as CODESYS sets them on networks.**
   - `twinova-ladder` gives each rung's span with the program (`parse_spans`): from its first comment line to its
     last line. The rung's own first line is where it is shown.
   - A `Place` of a Ladder file maps its lines through its rungs, not by an offset:
     - a line of a rung, comment included, goes to the rung's first generated statement;
     - a generated line goes back to its rung's line;
     - a line outside every rung (the declarations) takes no breakpoint, as an ST line without a statement does.
   - Step over and Step into, from a stop in a Ladder file, go on stepping while the stop stays in the rung they left.
     So a step goes to the next rung, or into a block of the project's that the rung calls. Run to cursor and
     breakpoint conditions, hit counts and log messages work as in ST.
   - The drawing:
     - a click on a rung's number sets or clears its breakpoint, and Shift+click or a right click opens the
       breakpoint bar;
     - a rung with a breakpoint shows the dot, and the held rung is marked;
     - the text view keeps its gutter.
   - v0.3.0's refusal goes, in `debug_commands.rs` and in the webview.
3. **The commands go to the window that holds them.**
   - Each webview's registry tells the app its commands (`control_commands`) whenever they change, and both windows
     listen to the channel.
   - The app sends a request to the window that registered its command. When two windows registered it, the PLC
     window gets it, since it holds the pane while it is open.
   - `list_commands` is answered from what the windows told, merged by name; `describe_command` goes to the
     command's window.
   - A window that closes takes its commands away.
   - The dock gains `plc_window` (`{ open }`), which detaches the pane as its button does, or attaches it again.
4. **Setup and Claude Desktop** cannot be run here. The user runs `twinova-agent setup` on their computer with Claude
   Desktop installed, and the verification records what they report. Nothing changes in the code unless that run
   shows a defect.

## Test plan

- **Layout** (vitest, `layout.test.ts`):
  - a box's rows never overlap: each row's place, reading and wire stay within its row;
  - a valued pin has a wire to the box;
  - `layoutRungs` fills the width it is given and keeps the widest rung's width when the pane is narrower.
- **Spans** (`twinova-ladder`): each fixture rung's span holds its comment and its text, and the spans do not overlap.
- **Debugger** (`twinova-engine`, on `ladder_line`):
  - a breakpoint on a rung's line, or on its comment, holds at that rung's line and is answered on it;
  - a line of the declarations takes none;
  - a step goes to the next rung;
  - Run to cursor stops on the rung asked;
  - a condition and a hit count hold as in ST.
- **Routing** (`control_tests.rs`):
  - a request goes to the window that registered its command, and the PLC window wins a tie;
  - `list_commands` merges the windows' lists;
  - an unknown command is refused;
  - a window closed takes its commands away.
- **E2E on the real app** (`e2e/mcp.mjs`, Linux and Windows):
  - a breakpoint set over MCP on a rung of `ladder_line` holds the line, the status bar says the `.ld` line, and the
    drawing marks the rung;
  - Step over goes to the next rung;
  - with the pane detached through `plc_window`, `open_file`, `values`, `force` and `breakpoints` reach the PLC
    window, and the PLC window's screenshot shows it;
  - `ladder.mjs` keeps its checks.
- **The pictures**: rung 3 before and after, offline and online, and the pane maximized, kept in the verification.

## What this would not prove

- Stepping into a standard library block (R_TRIG, TON): it has no source here, in ST as in Ladder.
- Breakpoints on one element of a rung: a rung is the smallest place to stop.
- Claude Desktop on Linux, which has no official build.
