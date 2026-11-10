# Plan: native reads through the resolved block (ROADMAP 4.3a)

## Goal

Remove the cost of reading a behaviour block's native variables by path. In the 4.3 profile it was 31 % of a run.
The line's results stay the same.

## Architecture

`Field::build` resolves each behaving instance's block once: from the global `F` to the member `<instance id>`, in
truST's `VariableStorage`. `Field::native` reads the variable from that block. The generated program, Build's checks
and `line.rs` do not change.

## Tech Stack

Rust, truST `6405915` (`trust_runtime::memory::{InstanceId, VariableStorage}`); cargo test, cargo mutants, tauri-driver.

## Spec

`docs/superpowers/specs/2026-11-09-component-behaviour-design.md`, "Amended after the acceptance".

## Global Constraints

- The results of a run do not change:
  - every engine, line and field test passes unchanged;
  - the physics fingerprint is unchanged;
  - an exact restore is unchanged.
- No surviving mutant in `field.rs`.

## Review Focus

- Whether resolving at Build can fail on a program Build accepted. It cannot: the generated program always declares
  `PROGRAM F` and one member per behaving instance.
- Name matching stays case-sensitive, as the read by path is (probe, 2026-11-09).

## Decisions taken while writing this plan

- The resolved block, not a `VAR_ACCESS` alias. Both were measured in a throwaway worktree, and both bring 600 s of
  replay from 14.5–15.2 s down to 9.9–10.3 s. The alias would replace Build's message for a missing native variable
  with truST's "unknown instance field".
- The direct-I/O reads stay as they are: their address parsing is about 1 % of the profile.

## File structure

- `app/src-tauri/src/field.rs`: the resolved blocks and `native`.
- `docs/superpowers/verification/2026-11-09-field-native-reads.md`: before and after.

## Tasks

### Task 1 — Read through the resolved block

- [x] The existing native-read tests are the contract (`field_tests.rs`, `line_tests.rs`, `engine_tests.rs`). They
  pass before the change and must pass unchanged after it.
- [x] Implement in `field.rs`.
- [x] Verify:
  - `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`;
  - `cargo llvm-cov --workspace --fail-under-lines 85`;
  - `cargo mutants -f src/field.rs`.

### Task 2 — Measure and record

- [x] `cargo run --release --example replay_cost`, before and after, twice each.
- [x] Soak of the release build (`xvfb-run node e2e/soak.mjs 45`), the as-fast-as-possible phase against 4.3's.
- [x] A callgrind profile after the change.
- [x] E2E (`npm run e2e`) on the debug build.
- [x] Write the verification file; add ROADMAP 4.3a.
