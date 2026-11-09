# Spec: the project format (ROADMAP 4.2)

Date: 2026-11-08. Status: **decided; Linux measured; schema frozen (annex)** (user, 2026-11-08: a folder plus an archive; the manifest is
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
   - **A save** proceeds in this order:
     1. create the marker `.twinova/saving`;
     2. copy the files it replaces into `.twinova/previous/`;
     3. write each changed file through a temporary file and a rename;
     4. write the manifest the same way;
     5. delete the marker.
   - **On open, the marker decides, not the hashes.** *Amended 2026-11-08, user's decision after a measurement.*
     - **The marker is present:** a save was interrupted. The project opens from `.twinova/previous/`, and the user is
       told.
     - **The marker is absent but a hash differs:** the file was changed outside Twinova, by git, an editor or a
       copy. It is validated like any other file and opened, the change is reported, and nothing is restored.
   - **Why the amendment.** The first rule was "a hash that differs means an interrupted save". The Windows CI job
     (run 42510999331) refuted it: git checked the fixtures out with `autocrlf`, every JSON and `.st` file changed
     its line endings, and five intact files read as an interrupted save. In 4.6, VS Code editing a `.st` file would
     have been undone in the same way.
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
8. **A folder named on the command line opens at start**, as in OIP (user, 2026-11-09).
   - `twinova <folder>` opens the folder as the menu's Open does: the same validation, the same notices, and a folder
     that fails to open shows its diagnostics and leaves the M3 demo current.
   - **The rule:** the value of `--project=<folder>` is the folder; without that switch, the first argument that
     does not start with `-` is. Other switches are ignored, so a launcher that adds its own does not hide the folder.
   - It opens once per process: a reload of the webview does not open it again.
   - **Why now.** The native dialogs cannot be driven by tauri-driver, so this is how the E2E opens and saves a real
     project (plan Task 5). It is also a feature on its own: a file manager or a script can open a project.
   - **Measured on Linux (2026-11-09):** tauri-driver 2.1.0 passes `tauri:options.args` to the application through
     `webkitgtk:browserOptions.args`; a probe binary received them unchanged and in order, with no switch added.
   - **Measured on Windows (2026-11-09, CI run 42572872545, windows-2022):** a positional folder never reached the
     app. Every check that needed the project failed (12 of 40 passed: the demo-still-built ones), while the app
     started normally. Microsoft's WebDriver page says that for a WebView2 app the `args` "are passed to your app
     instead of the underlying Microsoft Edge browser process". ChromeDriver, which msedgedriver is built from,
     declares in `chrome/test/chromedriver/capabilities.h` that each one is set as a switch "of the form
     [--]name[=value]", and `ParseSwitches` in `capabilities.cc` hands every entry to that parser (both read
     verbatim, 2026-11-09). So a WebDriver can pass only switches, and the switch form was added for it. The
     switch parser's body (`switches.cc`) was not read; the run with `--project=` decides.

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

## Measured on Windows (2026-11-08, CI run 42514839589, windows-2022, NTFS)

The storage tests of `twinova-project` ran on windows-2022:

| Case | Expected | Measured |
|---|---|---|
| Save over a read-only `scene.json` | the rename is refused, the previous save survives | as expected: `save_folder` fails, and once the flag is cleared the folder opens as the previous save |
| Save over `plc/Main.st` while another handle holds it open (`std::fs::File::open`) | the rename succeeds, because Rust opens with `FILE_SHARE_DELETE` | **refuted**: `MoveFileExW` fails with error 5, "Access is denied" |
| All other storage, archive and codec tests | pass | pass |

Consequence, the user's decision of 2026-11-08: a rename refused with `PermissionDenied` is retried for about 2 s with
growing waits. A file held for a moment by an antivirus, an indexer or OneDrive is the common case, and Open Industry
Project's desktop shell retries for the same reason (research 14, OF-10). A file still held after that fails the save,
which names the file and rolls back. The retry is Windows-only: on Linux a refused rename is a permission, which
waiting does not change.

Still not measured: case-insensitive collisions on NTFS (the codec refuses them before any write), and network shares.

Paths (traversal, absolute, drive letters, backslashes) are validation rules, and are tested with the codec in
Task 2.

## Annex: schema, format version 1

Frozen for 4.2 on 2026-11-08. Names are snake_case. Lengths are metres and angles are radians, as `f64` written
with `serde_json`'s `float_roundtrip`. Rotations are unit quaternions `[x, y, z, w]`, the glTF convention. Every
identifier matches `^[a-z][a-z0-9_]*$`, and a definition id is a dotted name such as `twinova.belt`.

### `twinova.json`, the manifest

```json
{
  "format": "twinova-project",
  "version": 1,
  "name": "Two belts and a diverter",
  "app": "0.1.0",
  "scene": "scene.json",
  "components": ["components/twinova.belt@1.json", "components/twinova.diverter@1.json"],
  "program": ["plc/Main.st"],
  "files": {
    "scene.json": "<sha256 hex>",
    "components/twinova.belt@1.json": "<sha256 hex>",
    "components/twinova.diverter@1.json": "<sha256 hex>",
    "plc/Main.st": "<sha256 hex>"
  }
}
```

- `files` lists every file of the project, and only those. `scene`, `components`, `program` and assets name files
  listed there.
- Paths are relative, use `/`, and contain no `..`, no drive letter and no leading `/`.
- Two paths that differ only in case are an error, because they collide on Windows.
- `version` greater than 1 is refused, with both versions named in the diagnostic.

### `components/<id>@<version>.json`, a pinned definition

```json
{
  "id": "twinova.belt",
  "version": 1,
  "origin": { "app": "0.1.0" },
  "generator": "twinova.belt/1",
  "parameters": [
    { "name": "length", "type": "REAL", "unit": "m", "default": 3.0, "min": 0.3, "max": 50.0 },
    { "name": "speed", "type": "REAL", "unit": "m/s", "default": 0.5, "min": 0.0, "max": 5.0 }
  ],
  "signals": [
    { "name": "run", "type": "BOOL", "direction": "plc_output" },
    { "name": "running", "type": "BOOL", "direction": "plc_input" }
  ],
  "ports": [
    { "name": "in", "kind": "flow_in" },
    { "name": "out", "kind": "flow_out" },
    { "name": "rail", "kind": "mount_host" }
  ],
  "nodes": [ { "id": "root", "parent": null, "position": [0, 0, 0], "rotation": [0, 0, 0, 1], "asset": null } ],
  "joints": [],
  "behaviour": null
}
```

- **`generator`** names a native, versioned family generator. It builds the geometry, the frames of the ports and the
  transport path from the parameters. An unknown generator keeps the document and shows a bounding placeholder, but
  blocks Run.
- **Types.** `parameters[].type` is an IEC type. 4.2 accepts REAL, DINT, BOOL and TIME; TIME is written as integer
  nanoseconds. `signals[].type` is BOOL, DINT or REAL, as decided in M4.1.
- **`direction`** is seen from the PLC: `plc_input` is a component output, read by the program.
- **`nodes`** form a tree rooted in one node with `parent: null`. A cycle is an error. `asset` names a listed
  `assets/<sha256>.glb`, or is `null`.
- **`joints`** are `{ "id", "kind": "fixed" | "revolute" | "prismatic", "parent", "child", "axis", "limits" }`, by
  node id.
- **`behaviour`** is `null` for a built-in whose logic is native. Otherwise it is `{ "function_block": "<name>" }`,
  and `components/<id>@<version>.st` holds that block, listed in the manifest. The block runs in the field runtime,
  not in the line program (amended 2026-11-09, spec `2026-11-09-component-behaviour-design.md`).

### `scene.json`

```json
{
  "instances": [
    {
      "id": "belt_a", "name": "Belt A", "definition": "twinova.belt@1",
      "position": [0, 0, 0], "rotation": [0, 0, 0, 1],
      "parameters": { "length": 3.0 },
      "signals": { "run": "%QX0.0", "running": "%IX0.0" }
    }
  ],
  "groups": [ { "id": "infeed", "name": "Infeed", "position": [0, 0, 0], "rotation": [0, 0, 0, 1], "children": ["belt_a"] } ],
  "connections": [
    { "kind": "flow", "from": ["source", "out"], "to": ["belt_a", "in"] },
    { "kind": "mount", "host": ["belt_a", "rail"], "device": ["sensor_a", "mount"], "position": [2.7, 0, 0], "rotation": [0, 0, 0, 1] }
  ],
  "controller": { "panel": [ { "name": "Start", "address": "%IX1.0" } ] }
}
```

- **`parameters`** holds overrides only. An override of a parameter the definition lacks is a diagnostic, never
  silently dropped. A value out of range, or not finite, is an error.
- **`signals`** maps each of the definition's signals to an IEC direct address. Every required signal must be bound
  before Run.
- **Address rules.** The size must match the type: `X` for BOOL, `D` for DINT and REAL. Addresses must not overlap,
  and the direction must match the area (`%I` for `plc_input`, `%Q` for `plc_output`).
- **Connections** reference `[instance, port]`, by id. A flow `from` is a `flow_out` and its `to` a `flow_in`. A flow
  endpoint and a mounted device each have one peer; a `mount_host` may carry several devices, for example two sensors
  on one belt.
- **`groups`** hold instance ids. A group id is unique among instances and groups; an instance belongs to at most one
  group.
- **`controller.panel`** lists the operator's buttons: a name, unique, and an input bit (`%IX`) that overlaps no
  signal. A press holds the bit for one controller scan (amended 2026-11-09; it replaces `program_instance`, which the
  field runtime made unnecessary).

### Order and identity

- Arrays keep authored order.
- JSON object keys are written sorted, so the same document always gives the same bytes and the same hashes.
- The construction order of a scene follows instance ids, sorted, not file order, as M4.1 requires.

### The two first fixtures

- **`demo_m3`**: one source, one belt, one diffuse sensor and a sink (the sink added 2026-11-09).
  - Belt: 3.0 m at 0.5 m/s. Sensor: 2.7 m.
  - Signals: `run` `%QX0.0`, `running` `%IX0.1`, `detected` `%IX0.0`, the sink's `count` `%ID4`; the panel's Start
    and Stop at `%IX1.0` and `%IX1.1`. Its program is M3's at those addresses.
- **`two_belts_diverter`**: the M4.1 acceptance line.
  - Source → belt A → diverter → belt B → normal sink, and the second diverter output → reject sink.
  - Two diffuse sensors.
  - Addresses: `%IX0.0`–`%IX0.4`, `%QX0.0`–`%QX0.2`, and the two sinks' counters at `%ID4` and `%ID8`.

Negative fixtures, one per rule above:
- a version that is too new;
- a hash mismatch;
- a path with `..`;
- a case collision;
- a parent cycle;
- a duplicate id;
- an endpoint with two peers;
- an unknown parameter override;
- an out-of-range value;
- overlapping addresses;
- a BOOL bound to `%ID`;
- a `plc_input` bound to `%Q`.

## What this will not prove

- **Two-way sync with an external editor** while the app runs: that is 4.6.
- **Running an opened multi-component project**: 4.2 opens, validates, shows and saves. Running is 4.3–4.5, and an
  opened project that cannot run says so instead of running the M3 demo.
