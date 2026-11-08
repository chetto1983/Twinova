# Spec: the project format (ROADMAP 4.2)

Date: 2026-11-08. Status: **decided, to be measured** (user, 2026-11-08: a folder plus an archive; the manifest is
written last and checks the other files by hash). The concrete schema is an annex that Task 1 of the plan writes
after the probes below.

ROADMAP 4.2: an open scene and component format, JSON plus glTF geometry, versioned, with migrations. Acceptance:
round-trip tests, and a corpus of saved scenes opens. The model it stores is the approved M4.1 spec
(`2026-11-08-component-model-design.md`). Research 14 reads how Open Industry Project stores a project.

## Decisions

1. **A folder, and the same folder as an archive** (user).
   - **Working form: a folder**, readable in git, and the one VS Code opens in 4.6. Its layout:

     | Path | Content |
     |---|---|
     | `twinova.json` | the manifest |
     | `scene.json` | the scene |
     | `components/<id>@<version>.json` | the pinned definitions |
     | `assets/<sha256>.glb` | the assets |
     | `plc/<POU>.st` | the ST sources, in the order the manifest lists |
     | `.twinova/previous/` | the copy kept by the save protocol (decision 2) |
   - **Sharing form: one zip of the same files**, by Export and Import. It is byte-reproducible: fixed entry order
     and timestamps.
   - **One validator** reads both forms, and any folder edited by hand.
2. **The manifest is written last and checks every file by hash** (user).
   - **The manifest:** `twinova.json` lists every file of the project with its SHA-256.
   - **A save** writes each changed file through a temporary file and a rename, then the manifest the same way.
     Before that, it copies the files it replaces into `.twinova/previous/`.
   - **On open**, a file whose hash does not match the manifest shows that a save was interrupted. The project
     then opens from `.twinova/previous/`, and the user is told.
   - **A file the manifest does not list is ignored, not an error.** The manifest decides what the project is
     (research 14, OF-09).
3. **One format version per project, migrated on read.**
   - Older versions are read and migrated, and only the current one is written.
   - A newer version is refused, with the version it carries and the newest this build reads, and the file is not
     touched.
   - A migration returns the list of what it changed, which the user sees (research 14, OF-02 and OF-03).
4. **Definitions are pinned copies**, as M4.1 says.
   - Each copy of a built-in carries the app version it was copied from, so the app can say that a newer one
     exists.
   - It never updates the copy by itself (research 14, OF-04).
5. **Signals store IEC direct addresses.** Each instance's addresses and parameters are written into the
   controller program as a generated `VAR_CONFIG` at Build. They are not stored twice in the folder (M4.1,
   "Binding to the controller").
6. **A corpus of saved projects.**
   - Every format version leaves fixture projects, positive and negative, in `app/src-tauri/tests/fixtures/projects/`.
   - Every build must keep opening them, or keep rejecting them with the same diagnostic.
   - This is 4.2's acceptance corpus (research 14, OF-15).
7. **The validator also runs outside the app**, as a command that CI calls on the corpus (research 14, OF-16).

## To be measured in Task 1, before the schema is frozen

- **The save protocol under failure, on Linux and Windows.** A process killed between files and before the
  manifest, a read-only file, a full disk. Each case is measured by reopening and comparing with the previous save.
- **Rename semantics on Windows**: replacing an existing file with a rename, and with the file open in another
  program (VS Code).
- **Numeric round-trip**: metres and radians in JSON against the runtime's integer nm and ns, at the limits a belt and
  a scene can reach.
- **Paths**: traversal (`..`), absolute paths, and case on Windows, in the manifest and in asset references.
- **The zip's reproducibility**: same project, same bytes, on both platforms.

## What this will not prove

- **Two-way sync with an external editor** while the app runs: that is 4.6.
- **Running an opened multi-component project**: 4.2 opens, validates, shows and saves. Running is 4.3–4.5, and an
  opened project that cannot run says so instead of running the M3 demo.
