# Spec: online monitoring in the ST editor (ROADMAP 4.8)

Date: 2026-11-13. Status: **draft for approval**. Decisions 1 to 3 were taken by the user on 2026-11-13; 4 to 8 are
proposed here.

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

**Proposed here:**

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
