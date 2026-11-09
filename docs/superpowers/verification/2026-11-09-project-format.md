# Verification: the project format through the real app (ROADMAP 4.2, 2026-11-09)

Spec: `2026-11-08-project-format-design.md`. Plan: `2026-11-08-component-format.md`, Task 5. The acceptance of 4.2 is
"round-trip tests; a corpus of saved scenes opens". Tasks 1–4 met it in the crate and through the commands; this run
meets it through the app as a user drives it.

## How the app is driven

tauri-driver cannot drive the native file dialogs. The user chose a folder named on the command line (spec, decision
8), as OIP does. The E2E saves through the menu's Save, which needs no dialog.

| Platform | How the folder reaches the app | Evidence |
|---|---|---|
| Linux (WebKitWebDriver) | `tauri:options.args` unchanged and in order | a probe binary under tauri-driver 2.1.0, 2026-11-09 |
| Windows (msedgedriver, WebView2) | only as a switch: a positional path never arrived | CI run 42572872545: 12 of 40 checks passed, all of them "the demo stays built" |

ChromeDriver, which msedgedriver is built from, sets each argument as a switch "of the form [--]name[=value]"
(`chrome/test/chromedriver/capabilities.h`, read verbatim). So the app also takes `--project=<folder>`, with
precedence over a positional folder. The E2E uses the switch on both platforms.

## What was measured

`e2e/project.mjs`, which `npm run e2e` runs after `e2e/run.mjs`. Debug build. Linux under Xvfb with software GL;
Windows on the windows-2022 runner. Final revision `e7e8088`, CI run 42574552486.

| Scenario | Checks | Linux, local | Windows, CI |
|---|---|---|---|
| Each valid fixture (`demo_m3`, `two_belts_diverter`), with `scene.json` re-indented as an editor would | the tree lists every instance; the inspector shows an instance's IEC addresses; the edit is reported as changed outside Twinova | ✅ | ✅ |
| Save through the menu, then compare with the fixture | the manifest and every file it lists equal the fixture's bytes; on unix, each file keeps its mode | ✅ | ✅ (no modes on Windows) |
| Open the saved folder again | the same tree, nothing reported as changed outside | ✅ | ✅ |
| Start on each of the 12 invalid fixtures | its diagnostic is shown, the same text as `tests/corpus.rs` expects (`tests/fixtures/invalid.json`); the M3 demo stays current and built | ✅ | ✅ |
| Total | | 42/42 | 40/40 |

The M3 E2E (`e2e/run.mjs`) stays 8/8 on both platforms.

## What the run found

- **Saved files were readable by their owner only.** Every file a save wrote was `0600`: tempfile creates its
  temporary files with that mode, and the rename keeps it. Now a replaced file keeps its mode, and a new file gets
  0666 less the umask, as `fs::write` gives. Tested in the crate; the test fails without the fix (0o600 against
  0o644).
- **The first draft of the E2E changed a value.** `JSON.stringify` rewrote `2.0` as `2`, and Twinova saved that `2`
  faithfully. The canonical form does not normalise numbers, which this run shows; the E2E now re-indents the text
  without touching values.
- **The Mutation workflow had been red on main since #18.** It runs only on main, after the merge, so the PR checks
  did not show it. Six mutants in `storage.rs` survived:
  - four in the Windows-only retry loop, which the Linux runner does not compile. It is now one function on every
    platform, with no waits outside Windows, and a fake rename tests it;
  - `StorageError`'s `Display`, which only the app's tests read. It is now tested in the crate;
  - a branch of `restore_previous` that no caller could reach. It was removed.

  Locally, `cargo mutants -f project/src/storage.rs`: 203 caught, 15 unviable, 0 missed. The push filter of
  `mutation.yml` now includes `project/src/**`.

## Gates

- **Rust:** fmt; clippy `-D warnings`, on the workspace and with `--target x86_64-pc-windows-gnu` for the project
  crate; tests; `cargo llvm-cov` 97.25 % of lines.
- **Frontend:** svelte-check, oxlint, prettier, i18n, knip, jscpd; vitest coverage 100 %.
- **CI run 42574552486:** the Windows E2E (40/40), `project-windows`, `rust`, `frontend` and `file-size` are green; the
  Ubuntu E2E was still running when this was written. The Linux 42/42 above is local. The same suite was 42/42 on
  the CI's Ubuntu runner in run 42572872545, with the positional folder.

## What this does not prove

- **Save as, Export and Import through the dialogs.** They are covered by the command tests
  (`project_commands_tests.rs`) and the crate's archive tests, not by the app.
- **"The previous project stays usable" with a project open.** The E2E starts on an invalid fixture, so the previous
  project is the demo. The command test `a_project_that_fails_to_open_keeps_the_current_one` covers a project being
  replaced.
- **A positional folder on Windows**, from a file manager or a shortcut. It is not reachable through msedgedriver.
  On Linux it is covered only by the unit test of `launch_folder`.
- **Migrations.** Only format version 1 exists, so there is nothing to migrate. A newer version is refused, as the
  corpus shows.
- **Geometry.** `assets/<sha256>.glb` files are stored and checked by hash, but none is drawn: the 3D view sizes
  simple shapes from parameters until 4.4.
- **Running a project.** That is 4.3–4.5.
