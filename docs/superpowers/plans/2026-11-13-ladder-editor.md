# Plan: the Ladder editor (ROADMAP 4.11)

## Goal

Ladder (LD) as the PLC pane's second language. An `.ld` file of the PLC is drawn and edited as CODESYS LD2 does, built
to ST for truST, and run on the line. Online it shows the runtime's own power flow and its values, and takes writes
and forces. PLCopen `<LD>` is imported and exported.

## Architecture

- **`twinova-ladder`, a new crate in the app's workspace** (Rust, no Tauri):
  - **the model:** a rung is a series of elements, a branch a set of paths, nested without limit;
  - **`parse` and `print`:** OIP's text grammar, round trip exact;
  - **`lower`:** to ST, with OIP's semantics:
    - each element once, left to right;
    - a condition captured before a later write;
    - branch order;
    - hidden P/N temporaries;
    - jumps as flags;
    - one hidden BOOL per element for its power (spec decision 5);
  - **`plcopen`:** the TC6 `<LD>` reader and writer.
  - Each module is ported from OIP (MIT), its tests with it. A NOTICE gives the attribution.
- **Engine and project:**
  - an `.ld` file is a program file of the manifest;
  - Build lowers it before truST compiles (Task 1 measures how a second PROGRAM joins `Main.st`).
- **App commands:** parse, print, lower; the catalogue of boxes; PLCopen import and export.
- **Webview:**
  - `LadderEditor.svelte` and its parts, each under 600 lines: rungs drawn from the tree with an automatic layout;
  - edits as pure functions on the tree, ported from OIP's `ladderEdit.ts`;
  - the LD button opens an `.ld` file;
  - online, the power and the values come from the existing watches.

## Tech Stack

- Rust: the new crate, serde for the tree as JSON.
- truST, which runs the lowered ST.
- Svelte 5 and SVG.
- Monaco is not used for LD.
- Sources read, not copied:
  - OIP at `0572868` (MIT);
  - nautilus (Apache-2.0), for the renderer and the power evaluator.

## Spec

`docs/superpowers/specs/2026-11-13-ladder-editor-design.md`, approved by the user on 2026-11-13 ("Approvata, procedi
con il piano"), decisions 1 to 8. Research 18, 19 and 20.

## Global Constraints

- **Clean-room and licences:**
  - OIP's and nautilus's code ported with attribution in `NOTICE`;
  - nothing from GPL projects;
  - CODESYS's help only as described behaviour.
- **One compiler:** only truST compiles. The crate never runs a program of its own.
- **Determinism:** an LD program and its hand-written ST twin give the same fingerprint on the 4.4a line.
- **Files** stay under 600 lines.
- **Gates** as for 4.10:
  - coverage at least 85 %;
  - mutation on the crate's `lower` and `parse`, which are critical as a codec is.

## Review Focus

- **The lowering's order:**
  - writes seen by later elements;
  - conditions captured;
  - branch order.
  
  Each is tested by running the lowered ST.
- **The power temporaries:** they must not change the program's behaviour (fingerprint) and must cost little
  (measured).
- **Positions:** a diagnostic of the lowered ST maps back to the element, as OIP's `mapBack` does.

## Decisions taken while writing this plan

1. **The tree is edited in TypeScript, parsed, printed and lowered in Rust.**
   - The editor holds the tree as JSON and applies OIP's pure gestures to it.
   - Rust prints it back to text, one printer.
   - Build and the analysis read the text.
2. **An `.ld` file's variables are an ST `VAR … END_VAR` block before its rungs,** so the variables table (4.8) reads
   and edits them as an ST file's.

## File structure

| File | Change |
|---|---|
| `src-tauri/ladder/` (new crate: `model.rs`, `parse.rs`, `print.rs`, `lower*.rs`, `plcopen*.rs`, tests, `NOTICE`) | model, grammar, lowering, PLCopen |
| `src-tauri/Cargo.toml`, `engine/src/build.rs`, `controller.rs`, `project/src/*` | `.ld` files in the manifest and the Build |
| `src-tauri/src/ladder_commands.rs` (+ tests), `lib.rs` | parse, print, lower, catalogue, PLCopen |
| `src/lib/ladder/*.ts` (+ tests), `src/lib/Ladder*.svelte` | tree, edits, layout, editor, online |
| `src/lib/Editor.svelte` (the LD button), `messages/*` | the pane |
| `e2e/ladder.mjs`, `package.json` | the E2E |
| docs | verification, CHANGELOG, ROADMAP |

## Tasks

- [x] **Task 1: measurements, before any code.** Throwaway probes in a worktree outside the repository.
  - How a second PROGRAM joins `Main.st` in `TestHarness::from_sources`, with no CONFIGURATION:
    - does it run;
    - in which order;
    - with which instances.
  - OIP's lowered ST for a start/stop rung, a timer and a P edge, hand-written: does truST compile and run it?
  - The power temporaries' cost: 20 s of virtual time on the 4.4a line, with the program as ST, then with a hidden
    BOOL per element.
  - The `plcopen` crate on Beremiz's examples, against porting OIP's reader.
  - Record each answer in the spec. Commit.
- [ ] **Task 2: the crate's model, parser and printer** (OIP's `ladder.ts` types and `parserLadder.ts`, their tests
  ported). Commit.
- [ ] **Task 3: the lowering, with the power temporaries** (OIP's `Lowerer`). Tests run the lowered ST in truST, scan by
  scan, against expected values and against an ST twin's fingerprint. Commit.
- [ ] **Task 4: `.ld` in the project and the Build; the app's commands.** Tests first. Commit.
- [ ] **Task 5: PLCopen `<LD>` reader and writer,** OIP's tests ported, with the semantic round trip. Commit.
- [ ] **Task 6: the editor.**
  - The tree, edits and layout in TypeScript (vitest first).
  - The SVG rungs, markers, modifiers, picker, keyboard, undo.
  - The LD button.
  - Commit.
- [ ] **Task 7: online:** power and values drawn, write and force from the view. Commit.
- [ ] **Task 8: the E2E,** `ladder.mjs` (spec test plan). Commit.
- [ ] **Task 9: documents.** Verification, CHANGELOG, ROADMAP. The pull request, merged when green.
