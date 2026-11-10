# Plan: one solver iteration per millisecond, and I/O addresses resolved at Build (ROADMAP 4.4b2)

## Goal

A simulated millisecond costs about a fifth less again: the physics solves once per step, and the line copies I/O
through addresses parsed at Build.

## Architecture

- **`twinova-physics` `world.rs`:** `num_solver_iterations = 1`.
- **`controller.rs`:** an address the program declares, parsed, and reads and writes through it.
- **`field.rs`:** the mirror of each command and feedback, parsed at Build; `set_command` and `feedback_value` look it
  up.
- **`line.rs`:** native inputs, commands and feedbacks kept as parsed addresses; no clone per millisecond.

## Tech Stack

As 4.4b.

## Spec

`docs/superpowers/specs/2026-11-10-solver-substeps-io-links-design.md` (approved 2026-11-10).

## Global Constraints

- The 4.3–4.4b1 tests pass unchanged, but for the three fingerprints (Task 1). A test that fails otherwise is
  reported before it is edited.
- Task 2 changes no result: the fingerprints pinned in Task 1 stay.
- Gates: 600 LOC, coverage ≥ 85 %, no surviving mutant on the changed code.

## Review Focus

- The fingerprints equal in dev and release before they are pinned.
- An input the program does not declare stays skipped, as Build warned.
- A command or feedback address the field does not have is an error, not a silent zero.

## Decisions taken while writing this plan

- **The field's API keeps the controller's address strings;** it looks their mirror up in a map built at Build, so
  its tests do not change.

## File structure

| File | Change |
|---|---|
| `app/src-tauri/physics/src/world.rs` (+ the three fingerprint tests) | one iteration |
| `app/src-tauri/src/controller.rs` (+ test) | `declared_at`, `read_at`, `write_at` |
| `app/src-tauri/src/field.rs` (+ test) | mirrors resolved at Build |
| `app/src-tauri/src/line.rs` | parsed addresses |

## Tasks

### Task 1 — One solver iteration

- [ ] Set it; run every test; the fingerprints in dev and release; pin them.
- [ ] Verify: Rust gates; windows-2022 in CI.

### Task 2 — Addresses resolved at Build

- [ ] Tests first: a declared address resolves and an undeclared one does not; a write and a read through it equal the
  string's; an address the field does not have is refused.
- [ ] Implement; measure before and after; verify: mutants on the changed code.

### Task 3 — Acceptance

- [ ] E2E on Linux; the release measurement and soak; verification, ROADMAP, CHANGELOG.
