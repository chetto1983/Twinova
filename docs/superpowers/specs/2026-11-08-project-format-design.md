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

## Measured on Linux (2026-11-08)

A throwaway crate outside the repository, with `sha2` 0.10.9, `tempfile` 3.27.0, `serde_json` 1.0.154 and `zip`
9.4.2 (MIT, `default-features = false`, deflate only). It used the save protocol of decision 2 on a three-file project:
- the replaced files are copied to `.twinova/previous/`;
- each file is written to a temporary file, `sync_all`, then `persist` (a rename);
- the manifest is written last, the same way.

| Case | Result |
|---|---|
| Process killed (`abort`) after 1, 2 or 3 of the 3 files, before the manifest | each open reports exactly the rewritten files as hash mismatches, and `.twinova/previous/` holds the last save intact |
| Complete save over a previous one | opens with the new content |
| Disk full (a 64 KB tmpfs), second save | `save` returns `No space left on device`; the files written before it mismatch, `previous` is intact, no temporary file is left |
| Target file read-only | on Linux the rename replaces it: the read-only flag protects nothing there (Windows refuses it, to be measured) |
| A file the manifest does not list | ignored; the project opens |
| metres → nm → JSON → nm, 0 to 10⁷ m and negative values | exact (worst error 0 nm) |
| `f64` through `serde_json` with default features | **8 575 of 200 000 random values not bit-exact** |
| The same, with `serde_json`'s `float_roundtrip` feature | 0 of 200 000 |
| The same project zipped twice, fixed timestamps and permissions | identical bytes (417 B) |

Consequences for the implementation:
- **`float_roundtrip` becomes explicit** in Twinova's `serde_json` dependency. The app's build has it today only
  through feature unification from another crate (`cargo tree -e features -i serde_json`).
- **A save that fails restores `previous` at once**, rather than leaving the folder for the next open to recover. The
  disk-full case shows that a failed save can leave rewritten files behind.
- **Recovery must itself survive an interruption.** That is not measured yet: a second kill during the restore is a
  Task 3 test.

Not measured, because this environment is Linux:
- Windows rename over a read-only file and over a file open in another program (VS Code);
- case-insensitive path collisions;
- the whole protocol on NTFS.

Paths (traversal, absolute, drive letters, backslashes) are validation rules, and are tested with the codec in
Task 2.

## What this will not prove

- **Two-way sync with an external editor** while the app runs: that is 4.6.
- **Running an opened multi-component project**: 4.2 opens, validates, shows and saves. Running is 4.3–4.5, and an
  opened project that cannot run says so instead of running the M3 demo.
