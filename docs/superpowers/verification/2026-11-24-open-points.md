# Verification: the points v0.3.0 left open (ROADMAP 2.9)

Date: 2026-11-24. Spec `2026-11-24-open-points-design.md`, plan `2026-11-24-open-points.md`, pull request #56.

## What was measured

- **The Ladder view** on the real app, `ladder_line`'s `plc/Main.ld`, the dock maximized, over MCP:
  - Before: [picture](img/2026-11-24-ladder-before.png). Rung 3's pins are crowded, MOVE.IN's expression is
    detached, the right rail is at x ≈ 963 in a pane about 1210 wide, and the table takes 8 rows.
  - After, online: [picture](img/2026-11-24-ladder-after.png).
    - Each valued pin's place is on its row with a wire to the pin, and its reading is beside it: `[0] count ─ IN1`,
      `1 ─ IN2`, `OUT ─ count [0]`, `count MOD 3 = 0 ─ IN`, `OUT ─ reject_next [FALSE]`.
    - The rail is at the pane's edge (x ≈ 1410).
    - The table shows about 4 rows.
  - `layout.test.ts` checks:
    - on rung 3, no place or reading reaches another on its row or an element beside its box;
    - every valued pin has a wire from its place to the box's edge;
    - the rungs fill the width given, or keep the widest rung's.
  - `layout.ts` coverage: 98.6 % of lines. Vitest: 301/301.
- **Breakpoints in Ladder files.**
  - `twinova-ladder`:
    - `parse_spans` gives `ladder_line`'s 13 rungs, each holding its comment and its text;
    - a rung written on several lines, with a comment of several, is one span;
    - `rung_lines` gives each rung's generated lines in order.
  - The debugger's places (`debugger_places_tests.rs`, on `ladder_line`):
    - rung 1, its comment on line 26 and itself on 27, maps to generated row 51, and back to 27;
    - rung 3's four generated rows show on its line 30;
    - the declarations take none;
    - every line of every rung reaches its rung's first row.
  - The debugger running `ladder_line` (`debugger_tests.rs`):
    - a breakpoint on rung 2's comment (line 28) is set on line 29, and one on a declaration is dropped;
    - Run holds on 29, Step over holds on 30, then on 31;
    - with the continued step disabled by hand, Over held on 29 again, so the test sees it;
    - Into from rung 4 goes to rung 5, since a standard block has no source;
    - Run to cursor holds on line 38.
  - The app (`debug_commands_tests.rs`): `set_breakpoints` on line 28 answers 29, Run holds there, and Over goes to 30.
  - E2E on the real app (`e2e/mcp.mjs`):
    - a breakpoint set over MCP on line 28 answers 29, and rung 2's number shows the dot;
    - Run holds there: the status bar reads "HOLD at plc/Main.ld:29" and rung 2 is marked;
    - Step over marks rung 3;
    - [picture](img/2026-11-24-ladder-hold.png), held on rung 3.
  - Regressions: `breakpoints.mjs` 15/15, `ladder.mjs` 6/6.
- **The commands reach the PLC pane in its own window.**
  - `control_tests.rs`:
    - a request goes to the window that registered its command, and the PLC window wins a tie;
    - a command nobody holds goes to the main window;
    - `list_commands` merges the windows' lists;
    - a closed window's commands go;
    - a request waits for a PLC window that has not told its commands yet, and then goes to it.
  - E2E (`e2e/mcp.mjs`, 23/23 on Linux):
    - the pane is detached with `plc_window`;
    - `open_file`, `monitoring`, `force`, `step` and `values` reach the PLC window, and WebDriver sees `motor` forced
      TRUE there;
    - `screenshot` of window `plc` answers;
    - attached again, `values` answers from the dock.
  - Regression: `plc_window.mjs` 6/6.
- **CI**: the pull request is merged only with every job green on its last commit, the E2E on windows-2022 included.

## Open

- **`twinova-agent setup` with a real Claude Desktop.** The container has no Claude Desktop and no computer of the
  user's is attached. The user runs `twinova-agent setup` on a computer with Claude Desktop, and checks that the
  `twinova` server appears and answers. This file is then completed with what they saw.

## What this does not prove

- Stepping into a standard library block (R_TRIG, TON) in Ladder, or in ST: it has no source here.
- A breakpoint on one element of a rung: a rung is the smallest place to stop.
- The drawing at a device pixel ratio other than 1.
