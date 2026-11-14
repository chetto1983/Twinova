# Verification: the Ladder editor (ROADMAP 4.11)

Date: 2026-11-14. Spec `2026-11-13-ladder-editor-design.md`, plan `2026-11-13-ladder-editor.md`, pull request #51.

## What was measured

- **Before any code** (plan, Task 1; the spec's "Measured in the plan's Task 1"):
  - a second PROGRAM runs beside `Main` with no CONFIGURATION;
  - OIP's lowered ST compiles and runs in truST;
  - the `plcopen` crate keeps an `<LD>` body's elements grouped by kind, so its order is lost. OIP's reader was ported
    on roxmltree instead (Task 5).
- **The crate `twinova-ladder`,** 73 tests:
  - OIP's cases for the parser, printer and lowering;
  - each fixture's lowered ST run scan by scan in truST against expected values;
  - a property test that runs random rungs against an interpreter of the tree written in the test;
  - 2000 random programs written to PLCopen, read back and run alike;
  - the writer's drawing checked by `misdrawn` (no overlap, every element between its rung's rails, pins on edges) and
    against a golden file.
  - Coverage of its files: lines 98.3 to 100 % (CI, `cargo llvm-cov`, on 225dbd0).
- **Engine and app:**
  - the 4.4a line's program in Ladder gives the ST program's fingerprints over 15 s (`program_files_tests.rs`);
  - exported to PLCopen, imported and run, it gives them again, and exports again to the same file;
  - the box catalogue comes from truST: the compiled runtime's function blocks and functions, and the standard library;
  - online, each element's power and each operand's value come from the existing watches, in drawing order
    (`monitor_ladder.rs`).
  - Rust workspace coverage: 98.04 % of lines (CI on 225dbd0).
- **Unit (TypeScript):**
  - edits, history, operand completion, catalogue, layout and online state, in `src/lib/ladder/*.test.ts`;
  - vitest 277/277, coverage 91.51 % of statements and 91.8 % of lines (`npm run coverage`).
- **E2E on Linux,** `e2e/ladder.mjs`, 6/6, in five runs in a row. It uses the fixture `ladder_line`, the 4.4a line
  with its program as `plc/Main.ld`:
  - the 13 rungs are drawn;
  - a new rung is drawn with the toolbox and the keys: a coil, a negated contact, a parallel branch with the seal-in.
    Auto Declare declares its three BOOLs, and the file reads `[|start_pb|, |motor|] |/stop_pb| (motor);`;
  - built and run with Monitoring on:
    - start_pb written TRUE from the drawing energises the coil, and the readings are TRUE, TRUE, FALSE, TRUE;
    - start_pb let go, the rung holds itself through the seal-in;
    - stop_pb drops the coil;
  - a force on start_pb from the drawing energises the coil and marks the value as forced.
  - Regressions: `monitor` 7/7, `plc_window` 6/6, `editor` 10/10, `editor_st` 6/6.
- **Seen in the app** (Linux, xvfb), dark and light themes:
  - the toolbar, the operand completion and Auto Declare;
  - the box picker, which inserts `TON_1` and declares it;
  - a rung's label, exit and comment; copy and paste;
  - the power flow online.
- **The power temporaries' cost** (BASELINE OR DROP, decision 5; the spec's "Measured in the plan's Task 9"):
  - on the 4.4a line, nothing measurable;
  - the program's scan alone takes about 10 µs more than the same lowering without them, +45 to +68 %.
  - They stay, since the drawing's power flow is read from them.
- **Mutation:** `cargo mutants` in CI on the lines the pull request changes. Each survivor was killed with a test, or
  excluded as equivalent with its reason in `.cargo/mutants.toml`. Two survivors were real drawing faults, now fixed:
  - a value drawn over its box, a comment over the rail, a label at x = 0, a jump across the right rail (0d07347);
  - a box alone on its rung drawn below the rails (225dbd0).

## Measurements that changed the design

- **truST's completion names no members after a dot** (spec, decision 4, amended in Task 6), so `T1.Q` is typed
  whole.
- **An empty path, a branch that ends a path of another, and a rung that is one branch cannot be drawn in TC6 as they
  read.** The 2000 random programs found each case. The writer puts a contact on `TRUE` there, and the reader takes it
  back as a wire (spec, Task 5).
- **A minified `#fff`** made Monaco refuse the light theme's token colours, so the ST editor never started in the light
  theme. Found while taking the light-theme screenshots; `hexColor.ts` expands it (728b0fe).

## What this does not prove

- **Insertion markers and drag and drop**, as LD2's: elements go in from the toolbox and the keys at the selection.
  The test plan's "draw by dragging" is not done. The user chose to close 4.11 without them (2026-11-14).
- **Start pressed on the I/O panel:** the E2E writes and forces the rung's variables from the drawing. The rung's
  variables have no I/O address.
- **PLCopen against CODESYS's and TIA's own exports:** only Beremiz's files and Twinova's own round trip are measured.
- **A program of a thousand rungs,** for the editor's speed and the temporaries' cost (the spec's estimate is a
  hypothesis).
- **Breakpoints, backward jumps, ST inside a rung** (4.11b), and FBD: left out by the spec, decision 8.
- **Windows,** until CI's windows-2022 E2E on the pull request's last SHA.
