# Spec: online monitoring in the ST editor (ROADMAP 4.8)

Date: 2026-11-13. Status: **approved** (user, 2026-11-13), decisions 1 to 11.

ROADMAP 4.8: "Online monitoring in the editor (research [17]): each line's values beside the code, TRUE/FALSE on
conditions, an FB instance chosen to watch; modify and force from the code, as TIA Portal and CODESYS do. From the
pinned runtime's `DebugControl` (reached from the harness), not from VS Code". It is accepted by:
- values read at the end of a scan equal the Watch pane's;
- a forced input holds across scans;
- the monitoring's cost measured against plain Run.

## What exists (inventory and measurements, 2026-11-13)

Runtime at the pin `6405915`, frontend at `5eb7f8c`. The throwaway probes are gone; their figures are below.

### What the two IDEs do (research 17)

- **TIA Portal:**
  - "Monitoring on/off" (Ctrl+T) puts a column beside the SCL code, with one row per line;
  - IF, WHILE and REPEAT show TRUE or FALSE;
  - one instance is chosen to watch, the "call environment";
  - a value can be modified from the code;
  - its force table forces I/O only, and a force survives disconnecting.
- **CODESYS:**
  - a value field follows each variable in the code;
  - a value is first prepared, then written once (Ctrl+F7) or forced (F7) until it is released;
  - "Watch All Forces" lists every force.

### The runtime's debug control

- **How it attaches:** `TestHarness::runtime_mut().enable_debug()` returns a `DebugControl`. There is no call that
  detaches it: `set_debug_control` only takes `Some` (`runtime/installed.rs`).
- **What it costs:** while it is attached, the VM refreshes a storage snapshot before every statement
  (`runtime/vm/context.rs`, `on_statement`), breakpoints or not. Measured in release, 60 s of each line:

| Fixture | Scan, plain → debug (µs) | 60 s of the line, plain → debug | Fingerprint |
|---|---|---|---|
| `demo_m3` | 5.3 → 17.3 | 0.76 → 0.85 s | equal |
| `two_belts_diverter` | 16.5 → 84.6 | 3.02 → 3.27 s | equal |
| `code_sort_line` | 28.1 → 162.2 | 5.51 → 6.11 s | equal |
| `sort_line` | 24.0 → 141.5 | 5.41 → 6.20 s | equal |
| `curve_line` | 6.3 → 22.1 | 4.28 → 4.31 s | equal |
| `merge_line` | 3.4 → 10.4 | 1.05 → 1.18 s | equal |

### Reading values

- **Paths:** `TestHarness::get_output(path)` reads a program variable, an FB member (`carry.ET`, `arrived.Q`) or a
  qualified name (`Main.in_zone`). It takes 0.1 to 55 µs, the first read of a dotted path being the slowest. It needs
  no debug control.
- **Conditions:** `harness::parse_debug_expression` and `Runtime::evaluate_debug_expression` evaluate
  `Main.count MOD 3 = 0` to TRUE in 3 to 36 µs. Unqualified names fail with `UndefinedVariable`: the evaluation has no
  program scope, so each name must carry its program (`Main.`).
- **The blocks** run in the field's own harness (`field.rs`), whose program instance is `F`. Twinova already reads a
  device's block as `F.<device>.<variable>`.

### Forcing

All measured on the line's PLC, the controller's harness:
- **A variable:** `DebugControl::force_instance(Main's instance, "belt_b_run", FALSE)` holds it FALSE although the
  program writes TRUE each scan. `release_instance` gives it back to the program.
- **An address:** `force_io(%IX0.6, TRUE)` holds the input, and the program counted a carton that was not there, as on
  a PLC.
- **Releasing an input:** after `release_io` the input kept the forced value. Twinova writes an input only when the
  plant changes it, so on release it must write the plant's current value.
- **Getting Main's instance:** `get_output("Main")` returns it as `Instance(InstanceId(0))`.

### What to show on each line

- **What `trust-ide` gives:** `trust_ide::inline_values::inline_value_data(db, file, range)` gives each reference in a
  range, with its name, its scope (local, global, retain) and its owning POU.
- **How to reach it:** through a `trust_hir::project::Project` built from the same sources (`set_source_text`,
  `database()`). On the diverter's program it found 40 references, all owned by `Main`, in 26 ms in a debug build.
- **What it leaves out:**
  - an FB member yields only its instance: `entered` for `entered.Q`;
  - IF, ELSIF and WHILE conditions are not targets.

  Both come from the syntax tree (`trust_syntax`).
- **What it needs:** direct dependencies on `trust-ide`, `trust-hir` and `text-size` at `5eb7f8c`. They are already
  compiled, because `trust-wasm-analysis` uses them.

## Decisions

**Taken by the user on 2026-11-13:**

1. **End-of-scan values,** as CODESYS's inline monitoring.
   - Each shown variable is read after a scan, at the rate the Run loop emits snapshots.
   - Each condition is evaluated on those values.
   - Values as each line ran, as TIA's program status shows them, belong with breakpoints (4.9).
2. **The debug control attaches on demand:**
   - when the user turns Monitoring on, or forces a value;
   - it stays attached until the next Build, which builds the PLC without it;
   - with Monitoring off and no force, a run costs what it costs today.
3. **Forcing of variables and of I/O:**
   - any program variable, as CODESYS;
   - the `%I` and `%Q` addresses, as TIA's force table;
   - a list of the active forces, each releasable;
   - forces kept until released or until Build;
   - every force and release recorded in the replay.

**Proposed in the draft, approved with it:**

4. **What is monitored.**
   - The PLC tab shown, for its visible lines only, as TIA monitors only the visible window.
   - A program file's variables are qualified with its program, `Main.`.
   - A block's file shows the device selected in the scene, read as `F.<device>.`. It is read-only in 4.8: forcing
     inside a block was not measured.
   - Loops are monitored like any line, with end-of-scan values.
5. **What is shown.**
   - A grey value after each variable reference, as Monaco inline decorations.
   - TRUE or FALSE after each IF, ELSIF, WHILE or UNTIL condition.
   - An FB member shows the member (`carry.Q`), not the instance.
   - A variable that cannot be read shows nothing. It is not an error: the variable may be a constant, a type or a
     function.
6. **Modify and force from the code.**
   - Clicking a value opens a small field: Write sets the value once for the next scan, Force holds it.
   - A BOOL toggles.
   - A forced value is marked in the code.
   - The Watch pane gains a Forces list, whose Release lifts a force.
7. **Rust.**
   - The engine gains `Input::Write`, `Input::Force` and `Input::Release`, so a restored state replays the forces.
   - The controller attaches the debug control on the first of them, or when Monitoring is turned on.
   - Its I/O release writes the plant's current input.
   - A `monitor` module turns the visible lines into paths and conditions, with `inline_value_data` and the syntax tree,
     when the text or the visible range changes.
   - It reads them after each tick and sends them with the snapshot.
8. **Determinism.**
   - Reads change nothing. The debug control attached leaves the trajectory equal (measured above, six fixtures).
   - Writes and forces are inputs, recorded and replayed.

### Amendments of 2026-11-13, measured in plan Task 2

- **Decision 2: monitoring alone attaches nothing.**
  - Reads of paths and conditions need no debug control: `engine_monitor_tests.rs` reads after every scan with none
    attached.
  - So the debug control attaches at the first write or force, not when Monitoring is turned on.
  - Monitoring alone costs the reads, not the 3 to 6 times per scan.
- **Releasing a forced input needs nothing written.**
  - `Line::step_ms` writes every input every millisecond. After `release_io`, the next scan reads the plant's value
    (`a_forced_input_holds_against_the_plant_and_the_plant_comes_back`).
  - The forcing section's claim above came from a probe that read an input a carton was really covering.
- **A member of an instance is written or forced on that instance** (user: "devi modificare il valore della variabile
  di istanza").
  - truST drops a force named by a dotted path (`carry.PT`) at the next scan.
  - It holds a force given to the member's own instance (`carry`'s id, `PT`): measured at 100 ms for 1,000 ms
    although the call writes `PT := T#1500MS` every scan.
  - So Twinova resolves the path to the instance that holds its last name.
  - A write on a member the FB computes (a timer's `ET`) is computed again at the next scan, as on a PLC.
- **A forced variable at an address forces the address too** (measured in the E2E, then in `engine_monitor_tests.rs`).
  - truST forces the variable after the scan has written the image: `belt_b_run` forced FALSE read FALSE, while
    `%QX0.1` stayed TRUE and belt B ran.
  - On a PLC a forced output variable reaches its output, so the controller also forces the address the variable is
    declared at, when the image holds the same kind of value (a BOOL bit), and releases both.
- **A value is drawn on the reference's last character.** Monaco 0.57 draws no `after` text on an empty range, with or
  without `showIfCollapsed` (measured in the app). A condition that is one reference shares that reference's value,
  coloured as a condition.
- **A BOOL takes TRUE or FALSE.** truST does not parse `BOOL#1`; a click toggles a BOOL anyway.

### Amendment of 2026-11-13: a variables table, which also edits the declarations (user)

The user asked for the variables in a table, as in CODESYS and TIA Portal (a screenshot of OpenPLC Editor's table:
#, Name, Class, Type, Location, Initial Value, Option, Documentation). Two answers on 2026-11-13:
- the table **and** the values in the code;
- the table **also edits the declarations**, not only shows them online.

**9. The table.**
- **Where:** above the code of the open POU, as OpenPLC's and CODESYS's declaration part. Its height is resizable and
  it can be folded.
- **Columns:** #, Name, Class (VAR, VAR_INPUT, VAR_OUTPUT, VAR_IN_OUT, VAR_TEMP, VAR_GLOBAL, VAR_EXTERNAL, and
  RETAIN or CONSTANT), Type, Address (`AT`), Initial value, Comment; then, while Monitoring is on, Value and Forced.
- **Online:** each row's value is a path watch (`Main.<name>`, or `F.<device>.<name>` for a block).
  - An FB instance's row unfolds into its members, read-only.
  - A row's Value can be written or forced as decision 6 says, an FB member included.
- **Offline (and online, as an edit of the text):**
  - add a row;
  - delete a row;
  - change name, class, type, address, initial value or comment.
  - Each edit rewrites the ST text of the tab, which is then an edited tab, built or reverted as any other. The table
    and the code are two views of one text.

**10. What is reused and what is written** (inventory of 2026-11-13, frontend `5eb7f8c`).
- **Rename:** reused. `trust_ide::rename::rename_symbol` renames the declaration and every reference in the project's
  files.
- **Reading the declarations:** the public syntax tree of `trust_syntax`. A `VarBlock` gives the class and its
  qualifiers; a `VarDecl` gives `Name`, `DirectAddress`, `TypeRef`, the initializer after `Assign`, and the comment that
  follows it (measured on a sample).
- **Writing the declarations:** written by Twinova. `trust-ide` has no add, delete or retype refactor, and its
  `var_decl` helpers are crate-private. Every edit is a text edit on a node's range, with nothing reformatted around
  it:
  - a declaration's own line is regenerated as `name [AT address] : type [:= value]; (* comment *)`;
  - an added row goes before its block's `END_VAR`, and a new block is made when the class has none;
  - a deleted row loses its line;
  - a change of class moves the line to its block.
- **Each edit is checked** by truST's analysis before it is shown: an edit that makes the file fail to parse is
  refused.

**11. The plan grows** by two tasks, the declarations in Rust and the table in the webview, before the E2E.

## The design

### Rust

- `engine/src/controller.rs`:
  - `monitor(on)`;
  - `force(target, value)`, `release(target)`, `write(target, value)`, where a target is a variable or an address;
  - `forces()`.
- `engine/src/engine.rs`: the three inputs, replayed by `restore`.
- `src/monitor.rs`, new:
  - the lines in view becoming a `Watch { line, column, path | condition }` list;
  - the reads;
  - the commands `monitoring`, `watch_range`, `force`, `release` and `write_value`.
- The Run loop emits `monitor` beside `snapshot` while Monitoring is on.

### The webview

- **`StEditor.svelte`:**
  - a Monitoring toggle in the PLC pane's bar, with Ctrl+T;
  - the visible range sent on scroll;
  - the values drawn as inline decorations;
  - a click on a value opens the write and force field.
- **`WorkspaceDock.svelte`:** the Watch pane's Forces list.

### E2E

`e2e/monitor.mjs` on the diverter:
- Monitoring on: `count` beside its line equals the Watch pane's `count` after a Pause.
- The IF line of `arrived.Q` shows TRUE or FALSE.
- Forcing `belt_b_run` to FALSE holds it across 200 scans; belt B stops and no carton reaches `sink_ok`.
- Releasing it gives it back.
- Forcing `%IX0.6` counts a carton; releasing it restores the photocell's value.
- After a Build, Monitoring is off and the forces are gone.

## Acceptance

| # | Criterion | Evidence |
|---|---|---|
| 1 | End-of-scan values equal the Watch pane's | E2E, and a Rust test reading the paths after N scans against `variable()` |
| 2 | Conditions | TRUE/FALSE equal to the expression evaluated by hand on the read values (Rust test) |
| 3 | Forcing | a forced variable and a forced input hold for 1,000 scans; release restores the program's value and the plant's input (Rust tests, E2E) |
| 4 | Determinism | a run with forces, saved and restored, replays snapshot by snapshot; Monitoring on without forces leaves the fingerprint equal |
| 5 | Cost | the soak in release with Monitoring on, against Monitoring off and against `main` |
| 6 | Gates | fmt, clippy, coverage ≥ 85 %, `cargo mutants` on `monitor.rs` and the controller's force code, Stryker ≥ 70 % on the new view helpers, svelte-check, oxlint, knip, jscpd, i18n |

## What this will not prove

- **Values as each line ran** (TIA's program status), breakpoints, stepping: ROADMAP 4.9.
- **Forcing inside a block,** and choosing an instance for a block opened without a selection.
- **Monitoring on a real or emulated PLC** (PLCSIM Advanced): only Twinova's own PLC.
- **Windows,** until CI's windows-2022 E2E.
