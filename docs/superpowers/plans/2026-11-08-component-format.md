# M4.2 Component Format Implementation Plan

> **For agentic workers:** Use superpowers:executing-plans for the user's native execution preference. Steps use checkbox tracking.

**Goal:** Open, validate, display and save a versioned Twinova project with pinned component definitions.

**Architecture:** Rust owns a validated project document; the webview receives a read-only projection for the scene
tree, inspector and Three.js view. Project persistence is separate from M3 runtime snapshots. Native family generators
materialise parameterised geometry; unsupported runtime capabilities block Run rather than falling back to the demo.

**Tech Stack:** existing Rust/serde/serde_json, Tauri, Svelte/TypeScript and Three.js. No runtime pin changes.

**Spec:** [project format](../specs/2026-11-08-project-format-design.md), on the approved
[M4.1 model](../specs/2026-11-08-component-model-design.md). Research: [14](../../research/14-oip-formato-progetto.md).

**Readiness:** the container and the save protocol are decided (user, 2026-11-08). Task 1 measures them and writes
the concrete schema as the spec's annex; Tasks 2–5 wait for it.

## Global Constraints

- Work on the session's branch with a pull request, so CI checks each change before it reaches `main`. Probes run
  in throwaway worktrees or directories outside the repository.
- Stable IDs for definitions, instances and endpoints; labels are presentation.
- Definitions used in a project are pinned copies; no automatic catalogue upgrade.
- Mount, transport and signal links are distinct; no implicit type conversion or multiple signal writers.
- Signals store IEC direct addresses; Build generates the controller's `VAR_CONFIG` (M4.1). Overlapping addresses,
  two writers and a size that does not match the type are diagnostics, because truST accepts them silently.
- Project documents contain SI geometry; runtime remains integer nm/ns. Do not change M3 timing or replay.
- No structural editing during Run/Pause; no imported project may silently run the unrelated hard-coded demo.
- Keep code files under 600 lines. No AGPL source/assets copied into Twinova.

## Review Focus

1. Invalid references and cycles must not replace the current project.
2. A failed save must preserve the previous project, including shared asset files.
3. Definition upgrades and duplicate groups must preserve/remap only the intended references.
4. Integer precision and unit conversion must survive a Rust → JSON → webview → JSON round-trip.
5. The legacy runnable demo and an imported, not-yet-executable scene must remain visibly distinct.

## Decisions taken while writing this plan

By the user, 2026-11-08:
- a folder, with an Export/Import archive of the same files;
- the manifest is written last and lists every file's SHA-256, and the replaced files are kept in
  `.twinova/previous/` for recovery;
- dependencies: `sha2` and `tempfile` at the versions already in the lockfile, and `zip` 9.4.2 (MIT, deflate only);
- the Windows cases are measured by tests in the repository, run by CI on windows-2022.

Taken while writing the plan, from those decisions: the project format is its own crate, `twinova-project`, with no
Tauri dependency, in a Cargo workspace with the app.
- A test binary that links Tauri does not start on Windows (0xc0000139, run 42378063592), and every test of the
  current crate links it through `twinova_lib`.
- A crate without Tauri can be tested on windows-2022. It also keeps the format usable headless, as the README's
  goals ask.

4.2 closes on native open/display/save, not multi-component runtime execution. Reusable assemblies are templates
expanded into independent scene groups with new IDs. Parametric built-ins use versioned native family generators;
a new geometry expression language is not required. These are assistant proposals from the builder review.

## File structure

- `app/src-tauri/Cargo.toml`: becomes a workspace with the app and `project/`.
- `app/src-tauri/project/` (crate `twinova-project`, no Tauri): model, validation, codec, storage and tests, split by
  responsibility.
- `.github/workflows/ci.yml`: a job running `cargo test -p twinova-project` on ubuntu-24.04 and windows-2022.
- `app/src-tauri/src/project_commands.rs`: project command boundary and transaction checks.
- `app/src/lib/project.ts`: UI projection types and document state; no simulation authority.
- `app/src/lib/ProjectPanel.svelte`: open/save/status and actionable validation errors.
- `app/src/lib/SceneView.svelte`: render document projections alongside the explicit legacy demo path.
- `app/src-tauri/src/lib.rs`, `app/src/lib/backend.ts`, `app/src/App.svelte`: register and consume commands.
- `app/src-tauri/tests/fixtures/projects/`: portable positive/negative corpus, one set per format version.
- `app/src-tauri/examples/validate_project.rs`: the validator outside the app, which CI runs on the corpus.
- `app/e2e/run.mjs`: actual-app round-trip acceptance.

## Task 1 — Freeze the format after bounded probes

- [ ] Read installed serde/Tauri file-I/O and glTF APIs, plus existing repository patterns, before choosing a helper.
- [ ] Write the concrete manifest, definition, assembly, connection and scene schemas as a spec annex; include one
  legacy-demo fixture and one two-belt/diverter fixture. List exact supported capability IDs and parameter fields.
- [x] Linux probes measured 2026-11-08, recorded in the spec ("Measured on Linux"); the Windows cases move to Task 3's
  tests on windows-2022.
- [ ] Measure, in a throwaway directory, the items of the spec's "To be measured" list: the manifest-last save under
  a killed process, a read-only file and a full disk; rename over an existing and an open file on Windows; numeric
  round-trip at the limits; path traversal, absolute paths and case; zip reproducibility.
- [ ] Record the findings in the spec, and amend decision 2 if a measurement contradicts it. No claim of recovery
  without a demonstrated interrupted save and reopen on Linux and Windows.
- [ ] Update the remaining task interfaces with the final DTO field names and fixture paths. Commit the spec annex.

## Task 2 — Validated document and round-trip codec

Produces `ProjectDocument`, `ValidatedProject` and structured `ProjectDiagnostic` values with document paths.
Proposed boundary: `parse_project(bytes: &[u8]) -> Result<ValidatedProject, Vec<ProjectDiagnostic>>` and
`encode_project(project: &ValidatedProject) -> Result<Vec<u8>, ProjectDiagnostic>`.

- [ ] Add failing corpus tests: round-trip retains identity, definitions, overrides and connections; duplicate IDs,
  parent cycles, missing endpoints, non-finite/out-of-range values and unsupported versions are rejected.
- [ ] Implement with existing serde facilities and domain validation, then prove deterministic output ordering.
- [ ] Add group-copy reference tests, including internal remapping and exclusion of external links.
- [ ] Run Rust format/clippy/tests/coverage and the existing replay suite. Commit only after all pass.

## Task 3 — Project storage and Tauri boundary

Consumes the validated document/codec. Produces candidate-open and safe-save operations whose failure leaves
the active document and last saved project unchanged; exact file transaction follows Task 1's measured protocol.

- [ ] Add failing tests for missing files, path escape, failed writes, unsupported assets, a hash mismatch (an
  interrupted save) and a file the manifest does not list.
- [ ] Implement candidate validation before activation and save recovery. Expose typed errors, not panics.
- [ ] Test interrupted-save recovery on both supported platforms, in `twinova-project`'s tests that CI runs on
  windows-2022:
  - rename over a read-only file;
  - rename over a file held open;
  - names that differ only in case;
  - a second interruption during recovery.

  Keep project save separate from `save_state`.
- [ ] Export and Import the archive through the same validator; prove the same project gives the same bytes.
- [ ] Register project commands and prove a failed open does not replace the current project. Commit.

## Task 4 — Visible project round-trip

- [ ] Mount the project panel and exercise open/cancel/error/status interactions with the actual command contract.
- [ ] Render the validated scene with built-in generators and glTF loader. Use explicit placeholders for unsupported
  visuals and explain why unsupported runtime capabilities disable Run.
- [ ] Keep the M3 demo path runnable and covered by the existing E2E. Do not enable arbitrary scene simulation yet.
- [ ] Run TypeScript/Svelte gates, translations, coverage and production build. Commit.

## Task 5 — Native acceptance and handoff

- [ ] Through the actual Tauri app, open each positive fixture, inspect the scene, save and reopen; compare authored
  content and references. Attempt each negative fixture and verify the previous project remains usable.
- [ ] Re-run existing Windows/Linux E2E and appropriate mutation gates on new validation/storage logic.
- [ ] Record results and limits in dated verification; update ROADMAP 4.2 only after the acceptance is met.
- [ ] Commit, push and check applicable CI for that revision. The next plan addresses ST bindings and transport probes.
