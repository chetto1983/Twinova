# Spec: ST in VS Code (ROADMAP 4.6)

Date: 2026-11-12. Status: **approved** (user, 2026-11-12), with decisions 6 to 9. Amended the same day: VS Code itself
is tested by the user on a Windows virtual machine, from an MSI that CI builds for the pull request (acceptance 8).

ROADMAP 4.6 is "the scene's programs are `.st` files in a project folder, rebuilt on save; VS Code with truST's
`trust-lsp` extension is the external editor", accepted by "edit in VS Code, save, the running scene picks up the new
program". Today Twinova reads a project's `.st` files when it opens the folder, and writes them when it saves. Nothing
watches the folder.

## What exists (inventory and measurements, 2026-11-12)

### The project folder and the app

- **The sources are files already.**
  - The line program is `plc/*.st`, listed in the manifest's `program`.
  - Each family's behaviour block is `components/<id>@<version>.st`.
  - Open validates them (`project/src/codec.rs`). Save writes each file to a temporary and renames it over the old one
    (`storage.rs`, `write_atomic`), after copying what it replaces into `.twinova/previous/`.
  - Save removes only the files the previous manifest listed: a file the manifest never listed stays.
  - A file changed outside Twinova is reported at the next open (`changed_outside`).
- **The in-app editor holds the texts until Build.** The `build` command (`src/commands.rs`):
  - receives the program and the behaviours from the webview;
  - checks the whole project as Open does;
  - replaces the engine with a fresh one at time zero;
  - only then makes the texts the project's (`set_sources`), so Save writes what was built.
- **Two runtimes.**
  - The PLC (`engine/src/controller.rs`) compiles the program files alone in a truST `TestHarness`.
  - The field (`engine/src/field.rs`) compiles the behaviour blocks together with a generated `PROGRAM Field` and
    `CONFIGURATION Twinova`. Its `VAR_CONFIG` places every block's `AT %I*` and `AT %Q*` variables.
- **The line binds the PLC's addresses at Build** (`engine/src/line.rs`, `Line::build`): which declared input each
  sensor, sink, scanner and drive writes, and which output the field reads.
- **A snapshot is the inputs that led to it**, replayed on a fresh engine (`engine/src/engine.rs`, `Saved`): truST
  cannot save its VM's state.
- **The Run loop locks the engine for each tick** (`commands.rs`, `run`). Another thread that takes the same lock acts
  between two ticks.

### truST's reload (measured on the pinned harness, `6405915`)

truST swaps a program without stopping it.
- `TestHarness::reload_sources` and `Runtime::apply_online_change_bytes` work the same way.
- Its specification calls it "one composed transaction: load the retain snapshot, prepare a warm restart and
  retained-value migration, validate and materialize the new bytecode, then commit"
  (`docs/specs/10-runtime-semantics.md:1492`).
- A failed compile leaves the old program running.

A throwaway test ran 50 scans of 10 ms, then `reload_source` with one line changed:

| | Before | After the reload |
|---|---|---|
| `count : DINT` (VAR) | 50 | **0**, reinitialised |
| `kept : DINT` (VAR RETAIN) | 50 | 50, then 51 at the next scan |
| a running TON's `ET` | 490 ms | **0**, timing again |
| virtual time, scans | 500 ms, 50 | 500 ms, 50: they go on |
| a variable added, `added : INT := 7` | — | 7 |

So truST's online change is a **warm restart of the program's data**: RETAIN kept, the rest reinitialised, as an S7
OB100 restart. It is not a download that keeps every value. That would migrate variables by name, and truST does not do
it at the pin.

### trust-lsp on a Twinova folder (measured)

**Method.** An LSP client over stdio (scratchpad `lsp_probe.py`):
- it opens every `.st` file as VS Code would, with pull diagnostics and refresh;
- it waits 25 s and counts the diagnostics published;
- it runs on copies of `two_belts_diverter` and `sort_line`.

**Two servers:**
- `trust-lsp` built at the pin (0.24.75);
- the one bundled in the Marketplace extension, **0.38.0** (published 2026-11-11), the one a user installs.

| `two_belts_diverter`, pin | Errors | Warnings | Total |
|---|---|---|---|
| As it is | 8 × E202 | 26 × W010/W011 nondeterminism, 4 × W009 unused block or program, 3 × W001, 2 × W005, 2 × W013 | 45 |
| + `trust-lsp.toml` with `warn_nondeterminism = false` | 8 × E202 | 4 × W009, 3 × W001, 2 × W005, 2 × W013 | 19 |
| + a generated `PROGRAM Field` and its `VAR_CONFIG` in `.twinova/` | 8 × E202 | `Main`'s W009; 3 × W001, the sink counts and a sensor the program declares but does not read; 2 × W005; 2 × W013 | 16 |

The E202 fall on the blocks: 3 in the belt's, 3 in the roller conveyor's, 2 in the chain transfer's.

- **E202 is a false error, on each block's `AT %I*` and `AT %Q*`:** "direct address requires VAR_CONFIG mapping".
  - truST checks a block's wildcard addresses against the `VAR_CONFIG` of the same file
    (`trust-hir/.../collector/validation.rs`, `check_at_bindings`).
  - Twinova's field compiles blocks and configuration as one source, so they build; the generated file does not remove
    the error.
  - E202 is truST's generic `InvalidOperation` code, so lowering it would lower every other check under that code too.
- **The editor still catches real errors.** In `plc/Main.st`, a `BOOL := 5`, an undeclared identifier and a `TON` with
  `PT := 15` give E203, E101 and E205, on both servers.
- **0.38 and the pin agree.** With the same two generated files, their diagnostics are identical on both fixtures (the
  same hash over every diagnostic). Bare, 0.38 is a little quieter: no W010, and W011 only on the blocks.
- **`.twinova/previous/` breaks it.** After any Save it holds copies of the replaced `.st` files. The LSP indexes them,
  and `plc/Main.st` gets `E104 duplicate imported declaration of 'Main'`.
  - truST only skips `.git`, `.hg`, `.svn`, `node_modules` and `target`.
  - `[project] include_paths = ["plc", "components", ".twinova/lsp"]` limits indexing to those folders. Measured on
    both servers, the diagnostics are then the same as without the copy.
- **The LSP writes a cache into the folder:** `.trust-lsp/index-cache/index.json`.
- **The extension** (`trust-platform.trust-lsp`, MIT OR Apache-2.0) bundles its own `trust-lsp`;
  `trust.languageServer.executablePath` replaces it. It works on any folder with `.st` files.

### Watching files

- `notify` 8.2.1 (CC0-1.0, its types MIT OR Apache-2.0) is already built: trust-runtime's `full` feature uses it.
- truST's own watcher reloads its HMI descriptors after 250 ms without events (`trust-runtime/src/control.rs`). It is
  private to the HMI and is not reused.

## Decisions (user, 2026-11-12)

1. **A change to `plc/*.st` is an online change of the PLC; the plant keeps running.**
   - The PLC is swapped between two engine steps, with truST's warm restart.
   - Products, positions, conveyors and components keep their state.
   - The PLC keeps its RETAIN variables and the time.
2. **A change to `components/*.st` rebuilds the whole scene at time zero**, as Build does. A behaviour is the plant
   itself; swapping the field alone would make an actuator jump.
3. **VS Code uses the Marketplace extension as it is.** Measured equal to the pin on these projects. Build remains the
   referee.
4. **Twinova writes two files for the LSP into a project folder:**
   - `trust-lsp.toml`;
   - `.twinova/lsp/field.st`.

   Neither is in the manifest.
5. **The false E202 errors in the blocks are accepted and documented.** They appear only in `components/*.st`.

Proposed by this spec, for approval:

6. **Unbuilt edits win.** If the in-app editor has unbuilt edits in a file that changes on disk:
   - nothing is applied;
   - the notice names the file;
   - the user builds from Twinova, or discards the tab's edits, and the next save in VS Code applies.

   Nothing typed is lost silently.
7. **Only a project with a folder is watched.** The demo, an imported archive or a new project has nothing on disk until
   it is saved; once saved, its folder is watched.
8. **A change applies 250 ms after the last event,** truST's own debounce. Of the files the manifest lists, a file
   whose text equals the project's is no change: that is Twinova's own Save.
9. **Reset after an online change starts from zero with the current program.**
   - A snapshot keeps the program its engine was built with, and replays each change at its time.

## The design

### Watching the folder (app, `src/watch.rs`)

- A `notify` watcher on the current project's folder, recursive. It is replaced when another folder is opened or the
  project is saved elsewhere, and dropped when the project has none.
- **Events count only for the files the manifest lists** as program or behaviour:
  - each burst is read again after 250 ms without events;
  - a file that cannot be read yet, half written, is read again at the next event.
- **The texts read are compared with the project's.**
  - Equal: nothing happens.
  - A behaviour changed: decision 2.
  - Otherwise a program changed: decision 1.
- **An outcome is sent to the webview as a `sources` event,** with the project view. The editor shows it in its status
  line:
  - "applied online at 12.340 s";
  - "rebuilt";
  - "not applied", with the error, as Build shows one.

### The PLC's online change (engine)

- **`Engine::change_program(program)`:**
  1. compiles through `Controller::reload`, which is truST's `reload_sources`;
  2. runs Build's checks on the new declarations: a scene signal's address declared with another type is an error, and
     an undeclared one is a warning;
  3. the scan period is taken again from the program's tasks;
  4. `Line` binds the addresses again, with the code `Line::build` uses, split out so both share it.
- **The process image is carried over** (amended 2026-11-12, Task 2):
  - truST's reload starts from an empty image, and a new runtime's outputs read zero until its first scan;
  - so the inputs and outputs are copied into it, and the outputs hold until the new program's first scan, as a PLC's
    do through an online change;
  - `write_io` accepts `%Q` inside the image the new program admits (measured), and the scan then overwrites every
    bound output;
  - an input the line wrote for the old program and does not write for the new one is cleared, so it reads zero, as
    after Build.
- **A latched output that is not RETAIN drops at the first scan.** The demo's `motor` does, and its belt stops; declared
  `RETAIN`, it keeps running. This is the warm restart's semantics, tested.
- **On any error nothing changes:** the old program keeps running, and the error carries the file and line as Build's
  does.
- **On success the change is recorded as an input** `(time, Program(sources))`, in order with the presses and the sink
  blocks. A replay applies it at the same time, between the same two steps.
- **Faults.** A faulted line keeps its fault until Reset or Build, as today.
- **The app.**
  - It applies the change under the engine's lock, whether the line runs or not.
  - It then makes the texts the project's (`set_sources`), so Save writes them and the editor shows them.
  - It stores them as the built project's, for Reset (decision 9).

### The components' rebuild

- It is the `build` command's path, with the scene as it is and the sources from disk.
- On success the Run loop restarts if it was running. On failure the running line stays, as for Build.

### Files for the LSP

- **`trust-lsp.toml`**, at the folder's root, written when a folder is opened or saved, only if it does not exist (the
  user may tune it):

  ```toml
  # Written by Twinova (spec 2026-11-12-st-vscode-design.md). Index only the project's sources: .twinova/previous keeps
  # copies from the last save. The nondeterminism warnings flag every AT variable and TIME of a simulated line.
  [project]
  include_paths = ["plc", "components", ".twinova/lsp"]

  [diagnostics]
  warn_nondeterminism = false
  ```

- **`.twinova/lsp/field.st`**:
  - `PROGRAM Field` and `CONFIGURATION Twinova` as the field generates them, without the blocks themselves
    (`field.rs`, `generate`, split so both use it);
  - rewritten when a folder is opened or saved and at each Build or rebuild, if its text differs;
  - outside the manifest, and `read_folder` skips `.twinova/`;
  - the LSP indexes it, so the blocks are not "unused" and go-to-definition reaches each instance.
- **The LSP's cache.** Where truST's `[indexing] cache_dir` would move `.trust-lsp/index-cache/` (inside `.twinova/`) is
  measured in the plan's first task, before it is set.

### The in-app editor

- A `sources` event replaces the texts of tabs without unbuilt edits (decision 6).
- An "edited" mark on a tab compares its text with the project's.
- The 4.7 editor (Monaco) replaces this one later.

## Acceptance

| # | Criterion | Evidence |
|---|---|---|
| 1 | An online change keeps the plant | Engine test on `two_belts_diverter`, running: Main changed to reject every 2nd carton. The products on the belts, the time and the field's state are unchanged across the change; a non-RETAIN counter restarts and a RETAIN one does not; the next cartons follow the new rule |
| 2 | Determinism | A snapshot taken before a change and restored after it replays to the same fingerprint as the line that ran through it; two changes and a press at the same time apply in their recorded order |
| 3 | A refused change leaves the running program | Compile error, wrong type at a scene signal: the error names file and line, the scan and the outputs go on as before |
| 4 | Addresses bound again | A change that declares a sensor's address the old program did not: the sensor now writes it; one that drops it: Build's warning |
| 5 | The watcher | Writing in place and writing a temporary then renaming (the two ways editors save) give one change each, 250 ms after the last event; Twinova's own Save gives none; a behaviour gives a rebuild at time zero, still running; unbuilt edits in a tab hold the change with a notice |
| 6 | Files for the LSP | On every valid fixture, opened then saved: `trust-lsp.toml` and `.twinova/lsp/field.st` written; the probe on the pin and on the Marketplace server finds no error in `plc/*.st`, and in `components/*.st` only E202 |
| 7 | In the app | `e2e/vscode.mjs`: open a fixture folder, Run, write `plc/Main.st` as an editor does; the scene goes on without a reset, the I/O panel shows the new rule, the editor's tab shows the text; then a behaviour file: a rebuild at time zero. Linux here, windows-2022 in CI |
| 8 | With VS Code itself, on Windows | The user, on a Windows virtual machine: the pull request's MSI installed, VS Code with the Marketplace extension, a fixture's folder opened in both. An edit to `plc/Main.st` saved in VS Code is picked up online, a behaviour's rebuilds; the diagnostics are those of acceptance 6. The outcome is recorded in the verification file |
| 9 | Gates | fmt, clippy, coverage ≥ 85 %, cargo mutants without survivors on `change_program`, the line's binding and the watcher's filter; svelte-check, oxlint, knip, jscpd, i18n, Stryker ≥ 70 % on the editor's helpers |

## What this will not prove

- **A change that keeps every value.** truST's online change is a warm restart. A state that must survive a change is
  declared `RETAIN`, as on an S7.
- **A clean editor for the blocks:** the E202 errors stay, with the pin and with 0.38.
- **Other LSP versions.** Only 0.24.75 and 0.38.0 are measured; a later Marketplace version may differ, and Build stays
  the referee.
- **New `.st` files.** A file the manifest does not list is ignored by the watcher, and Build does not compile it,
  though the LSP indexes it.
- **A call from `Main` into a block.** The LSP sees the program and the blocks as one project, while Twinova compiles
  them apart. A `Main` that calls a behaviour block would pass in VS Code and fail at Build.
- **Folders on network drives, and macOS.** `notify` is measured here on Linux (inotify) and in CI on Windows.
- **Live values and the debugger in VS Code:** ROADMAP 4.8.
