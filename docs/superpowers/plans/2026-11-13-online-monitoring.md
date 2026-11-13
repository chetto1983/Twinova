# Plan: online monitoring in the ST editor (ROADMAP 4.8)

## Goal

- **Monitoring:** the PLC pane shows each visible variable's value after the scan, and TRUE or FALSE after each
  condition, while Monitoring is on (Ctrl+T).
- **Write and force from the code:** a program variable can be written once or forced, and a `%I`/`%Q` address can be
  forced. The Watch pane lists the forces and releases them.

## Architecture

- **Engine (`twinova-engine`).**
  - `Controller` attaches truST's `DebugControl` on demand. It forces and releases variables and addresses, writes
    variables, and reads watched paths and conditions after a scan.
  - The field's harness is read the same way, read-only.
  - `Engine` logs `Write`, `Force` and `Release` as inputs, so `restore` replays them. It holds the compiled watch list
    and reads it on request.
- **App (`twinova`).**
  - `src/monitor.rs` turns a file's visible lines into a watch list with `trust_ide::inline_values` and the syntax tree:
    the paths, FB members included, and the `Condition` nodes.
  - It owns the commands `monitoring`, `watch_range`, `force`, `release`, `write_value` and `forces`.
  - The Run loop and Step emit `monitor` beside `snapshot` while Monitoring is on.
- **Webview.**
  - `monitor.ts` turns the values into Monaco decorations.
  - `StEditor.svelte` holds the toggle, the visible range, the decorations, and the field a click on a value opens.
  - `WorkspaceDock.svelte` holds the Forces list.

## Tech Stack

- Rust 2024: truST's runtime at `6405915` (`DebugControl`, `parse_debug_expression`, `coerce_value_to_type`).
- truST's frontend at `5eb7f8c`: `trust-ide`, `trust-hir`, `trust-syntax`, `text-size`, as direct dependencies of the
  app. They are already built for `trust-wasm-analysis`.
- Svelte 5, Monaco 0.57.0 (inline decorations: `after` content).

## Spec

`docs/superpowers/specs/2026-11-13-online-monitoring-design.md`: approved by the user on 2026-11-13, decisions 1 to 8.

## Global Constraints

- **Determinism.** Reads change nothing. Writes and forces are engine inputs, replayed by `restore`. Monitoring on
  without forces leaves the fingerprint equal.
- **No cost when unused.** No debug control is attached until Monitoring is turned on or a value is forced. Build
  builds without it.
- **Twinova still never writes a program variable by name on its own.** Only the user's write and force do
  (`controller.rs`'s module comment is updated).
- **Files:** none over 600 lines. `App.svelte` is at 563 lines, so the monitoring's state lives in `monitor.ts` and
  `StEditor.svelte`.
- **Gates:**
  - fmt, clippy;
  - coverage ≥ 85 %;
  - cargo mutants on `monitor.rs` and the engine's new code;
  - Stryker ≥ 70 % on `monitor.ts`;
  - svelte-check, oxlint, knip, jscpd, i18n;
  - the E2E on Linux and windows-2022.

## Review Focus

- **Release of a forced input.** The spec says Twinova must write the plant's value again. `Line::step_ms` writes every
  input every millisecond, so this may hold already. Task 2 measures it, and the spec is amended if the measurement
  says so.
- **Qualified names.** Conditions are evaluated with each reference qualified by its program (`Main.`) or by the
  device's block instance (`F.<device>.`). A reference that does not resolve shows nothing.
- **Value types.** A written or forced text is parsed as an ST literal (`parse_debug_expression`), then coerced to the
  variable's current type (`coerce_value_to_type`).

## Decisions taken while writing this plan

1. **Write applies to variables only.** An input written once is overwritten by the plant at the next millisecond, and
   an output by the program at the next scan. Addresses are forced, not written.
2. **A force target is `Variable { program, path }` or `Address(text)`.** The program comes from `inline_value_data`'s
   owner. A dotted path is refused, because truST drops forces on members (measured in Task 2, spec amended).
3. **The watch list is compiled once** when the text, the visible range or the selection changes: paths kept,
   conditions parsed. The run loop only reads it.
4. **Values travel as text,** formatted in Rust: `TRUE`/`FALSE`, integers, reals, `T#…`. The webview shows them as
   they come.
5. **Ctrl+T toggles Monitoring** when the editor has focus, as in TIA Portal. The toggle button sits in the PLC
   pane's bar.

## File structure

| File | Change |
|---|---|
| `src-tauri/engine/src/controller.rs` (+ `controller_monitor.rs`, tests) | the debug control on demand; force, release, write; read paths and conditions |
| `src-tauri/engine/src/engine.rs` (+ tests) | `Input::Write/Force/Release`; the watch list; replay |
| `src-tauri/engine/src/line.rs`, `field.rs` | the field's harness reachable for reads |
| `src-tauri/Cargo.toml`, `Cargo.lock` | `trust-ide`, `trust-hir`, `trust-syntax`, `text-size` at `5eb7f8c` |
| `src-tauri/src/monitor.rs` (+ tests) | new: watch list from the text; the commands |
| `src-tauri/src/commands.rs`, `lib.rs` | the `monitor` event; the commands registered |
| `src/lib/monitor.ts` (+ tests) | new: decorations, targets |
| `src/lib/StEditor.svelte`, `Editor.svelte`, `WorkspaceDock.svelte`, `App.svelte`, `backend.ts` | toggle, decorations, write/force field, Forces list |
| `messages/{en,it}.json` | new strings |
| `e2e/monitor.mjs` | new |
| `.cargo/mutants.toml`, `stryker.config.json` | the new files |
| docs | verification, ROADMAP 4.8, CHANGELOG |

## Tasks

- [x] **Task 1: spec approval and plan** (docs commit).
- [x] **Task 2: the engine.**
  - Tests first, on the diverter:
    - a forced variable holds for 1,000 scans, and release gives it back;
    - a forced input holds, and after release it reads the plant's value (the spec's claim, measured);
    - a write lands once;
    - a dotted path (`carry.PT`) can be forced, or is refused with a message;
    - text parsed and coerced (`FALSE`, `5`, `T#2S`), wrong text refused;
    - paths and conditions read after a scan equal `variable()`;
    - a restored state with forces replays snapshot by snapshot;
    - debug attached without forces keeps the fingerprint.
  - `cargo mutants` on the new code.
  - Commit `feat(engine): write, force and watch the PLC's variables`.
- [x] **Task 3: the watch list in the app.**
  - The dependencies.
  - `monitor.rs`, tests first:
    - the diverter's visible lines give the paths, `entered.Q` and not `entered`, and each `Condition` qualified;
    - a block's file gives `F.<device>.` paths;
    - a range with no references gives nothing.
  - The commands and the `monitor` event.
  - `cargo mutants` on `monitor.rs`.
  - Commit `feat(app): watch the visible lines and force from commands`.
- [ ] **Task 4: the declarations in Rust** (spec decisions 9 and 10).
  - `src/declarations.rs`, tests first, on the diverter and a sample with every class:
    - rows read from the syntax tree: name, class, retain or constant, type, address, initial value, comment;
    - each edit's new text: add, delete, change of a field, change of class (the line moves), a new block when its
      class has none;
    - rename through `trust_ide::rename::rename_symbol`, every reference included;
    - an edit that does not parse is refused.
  - The commands `declarations` and `edit_declaration`.
  - Commit `feat(app): read and edit a POU's declarations`.
- [ ] **Task 5: the webview.**
  - `monitor.ts`, tests first: decorations from values, forced marks, rows' values.
  - The Monitoring toggle and Ctrl+T, the values in the code, the variables table above it (online values, write and
    force, edits), the Forces list; i18n.
  - Commit `feat(view): monitor, force and declare in the ST editor`.
- [ ] **Task 6: the E2E.**
  - `e2e/monitor.mjs`: the spec's list, and a variable added and retyped through the table, then built.
  - The whole suite on Linux under CI's screen.
  - Commit `test(e2e): online monitoring and the variables table`.
- [ ] **Task 7: acceptance and documents.**
  - The release soak with Monitoring on and off against `main`.
  - The verification file; ROADMAP 4.8; CHANGELOG.
  - Mutation and coverage in CI; the pull request ready, merged when green.
