# Plan: component behaviours and the field runtime (ROADMAP 4.3)

## Goal

A project runs in the app with its devices simulated by ST behaviours in a field runtime, and its line program
reads and writes the process image by address. The M3 demo becomes the `demo_m3` project. A user edits the belt's
behaviour, rebuilds, and sees the change. The transfer rule for a product spanning two belts is implemented and
tested, ready for 4.4.

## Architecture

- **`twinova-project`:** the format amendments, and the built-in definitions with their behaviour sources.
- **`field.rs`:** generates the field program and configuration from a project, builds it, checks each behaviour's
  contract, and exchanges by path with the native models.
- **`controller.rs`** replaces `plc.rs`: the line program, scanned with its image read and written by address, and
  its panel.
- **`line.rs`** replaces the fixed bench: it builds the nexosim models of a supported line from a project, in the
  order the spec fixes:
  - a source, a belt and a sink, with the belt parametrised;
  - the field runtime, every 1 ms;
  - the controller, at its scan period.
- **`transfer.rs`:** the spanning rule, as a pure library over integer nm and ns.
- **The engine, commands and UI** run a project instead of the demo. The editor has tabs for the program and for
  each behaviour; the panel buttons come from the project.

## Tech Stack

Rust (nexosim, truST `6405915` through `TestHarness`), Tauri 2, Svelte 5, three.js; tests with cargo test,
proptest, vitest; E2E with tauri-driver on Linux and Windows.

## Spec

`docs/superpowers/specs/2026-11-09-component-behaviour-design.md` (approved 2026-11-09), amending
`2026-11-08-project-format-design.md` and `2026-11-08-component-model-design.md`.

## Global Constraints

- **Determinism:**
  - a replay fingerprint test stays green at every task;
  - the fingerprints are recorded again only in Task 5, where the timing changes by design, with the reason in the
    commit.
- **Gates:**
  - no file over 600 LOC;
  - coverage ≥ 85 %;
  - zero surviving mutants on the files in `mutants.toml`: `field.rs`, `controller.rs`, `line.rs` and
    `transfer.rs` join it.
- **The app works at every commit.** The demo keeps running until Task 5 swaps it for the project.
- **Fixtures** are written by a script independent of the Rust encoder, as `tests/corpus.rs` requires.

## Review Focus

- The same-instant order: native events, then the field, then the controller.
- The mirror of addresses, and the build-time contract checks with diagnostics that name the definition and the
  variable.
- The transfer rule's invariants: one owner, no loss or duplicate, exact times.
- Replays after the switch to the project.

## Decisions taken while writing this plan

- **The belt's native model stays `belt.rs`'s closed form, parametrised.** It already holds the arrivals, the
  accumulation and the outlet. In 4.3 the source feeds it and the sink is its outlet. `transfer.rs` is the piece
  4.4 uses to chain belts.
- **Field cycles are scheduled by the line model at whole milliseconds** on the nexosim clock, so a run without a
  behaviour has no field events at all.
- **The UI's I/O panel lists the scene's signals and the panel buttons by name and address,** read from the
  controller's image.

## File structure

| File | Change |
|---|---|
| `app/src-tauri/project/src/model.rs`, `codec.rs`, `validate*.rs` | behaviour sources, `controller.panel` |
| `app/src-tauri/project/tests/fixtures/**` | regenerated; `demo_m3` gains a sink, the panel and M3's program |
| `app/src-tauri/src/field.rs` (+ `_tests`) | new |
| `app/src-tauri/src/controller.rs` (+ `_tests`) | new; `plc.rs` removed in Task 5 |
| `app/src-tauri/src/transfer.rs` (+ `_tests`) | new |
| `app/src-tauri/src/line.rs` (+ `_tests`) | new; `bench.rs` and `scene.rs` removed in Task 5 |
| `app/src-tauri/src/belt.rs` | parametrised |
| `app/src-tauri/src/engine.rs`, `commands.rs`, `project_commands.rs` | run a project |
| `app/src/App.svelte`, `lib/Editor.svelte`, `lib/IoPanel.svelte`, `lib/ProjectScene.svelte` | tabs, panel, parts on the belt |
| `app/e2e/run.mjs`, `project.mjs` | the demo as a project; the behaviour edit |

## Tasks

### Task 1 — Format amendments

- [x] Tests first:
  - a definition with a behaviour lists `components/<key>.st` in the manifest, and the codec reads and writes it;
  - a missing behaviour file is refused, and so is a block whose name differs from `behaviour.function_block`;
  - a panel button needs a Bit address in `%I` that overlaps no signal.
- [x] Implement in the crate. `controller.program_instance` goes.
- [x] Regenerate the fixtures with the script:
  - `demo_m3` gains a sink and the panel, and its program becomes M3's at addresses;
  - the belt definition gains `start_delay` and its behaviour;
  - the diverter gains its behaviour.
- [x] Verify: `cargo test -p twinova-project`, corpus, mutants on the crate's changed files.
  - *Delivered:* `migrate_fixtures_4_3.py` (scratchpad, independent of the encoder) migrated every fixture; the
    corpus test confirms the encoder writes the same bytes. The diverter's native variable became `position`
    (spec, amended while planning). 0 surviving mutants in the crate.

### Task 2 — The field runtime

- [ ] Tests first:
  - the generated source for `demo_m3` and for `two_belts_diverter`;
  - the mirror of addresses;
  - each contract failure with its diagnostic;
  - the diverter's behaviour with a controller program: both directions, `switch_time` per instance;
  - the belt's start delay, to the measured bound;
  - an ST keyword as an instance id.
- [ ] Implement `field.rs`.
- [ ] Verify: tests, clippy, mutants on `field.rs`.

### Task 3 — The controller by address

- [ ] Tests first:
  - a scan reads its inputs from and writes its outputs to the image by address;
  - a panel press holds for one scan;
  - a scene signal the program does not declare is a warning; a type that does not match its address is an error;
  - the scan period from `TASK`s is kept.
- [ ] Implement `controller.rs`, beside `plc.rs` until Task 5.
- [ ] Verify: tests, mutants.

### Task 4 — The transfer rule

- [ ] Property tests first, with unequal speeds and lengths, and a blocked downstream released later. Each case
  checks one owner, conservation of products, exact times, and accumulation against a spanning rear.
- [ ] Implement `transfer.rs`.
- [ ] Verify: tests, mutants.

### Task 5 — The line bench runs a project; the demo becomes `demo_m3`

- [ ] Tests first:
  - the same-instant order, with a belt, the field and the controller on equal instants;
  - a replay fingerprint for `demo_m3`;
  - M3's behaviours on the project: deliveries, Start/Stop, jam on a blocked sink;
  - a project whose line is not supported is refused with what is missing.
- [ ] Parametrise `belt.rs`. Implement `line.rs`, and move the engine and the commands to a project; Build builds
  both runtimes.
- [ ] Remove `bench.rs`, `scene.rs`, `plc.rs`, `default_program.st` and the demo-only commands. Record the
  fingerprints again with the reason.
- [ ] Verify: all Rust gates and mutants.

### Task 6 — The app

- [ ] Editor tabs: the program, and each definition with a behaviour.
- [ ] Panel buttons from the project; the I/O panel by signal.
- [ ] Parts drawn on the belt of the project view.
- [ ] The app opens the bundled `demo_m3` at start.
- [ ] Verify: frontend gates, Stryker on changed helpers.

### Task 7 — Acceptance

- [ ] E2E on Linux and Windows:
  - `demo_m3` runs as M3 did;
  - the belt's start delay set to 500 ms, then Build: `running` follows `run` by 500 ms within the bound;
  - a broken behaviour fails Build with its diagnostic, and the line keeps running.
- [ ] Soak on the release build against M3's numbers.
- [ ] Write the verification file; ROADMAP 4.3 ✅; CHANGELOG.
