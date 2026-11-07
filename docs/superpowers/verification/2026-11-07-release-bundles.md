# Verification: release bundles (2026-11-07)

Plan: `docs/superpowers/plans/2026-11-07-release-bundles.md` (ROADMAP 2.4). PR #3, CI run 42387823991 on `02a14ae`:
all seven jobs green.

## Installers built by CI

| Platform | Package | Size (bytes) | Binary inside (bytes) |
|---|---|---|---|
| Windows (`windows-latest`) | `Twinova_0.1.0_x64_en-US.msi` | 4 550 656 | `twinova.exe` 11 078 144 |
| Linux (`ubuntu-24.04`) | `Twinova_0.1.0_amd64.deb` | 4 136 554 | `twinova` 8 852 208 |
| Linux (`ubuntu-24.04`) | `Twinova_0.1.0_amd64.AppImage` | 82 778 616 | the same |

- The AppImage is about 20 times the deb because it carries WebKitGTK and its libraries; the deb depends on the
  system's.
- The binaries have the same size as in the earlier `build` job (runs 42382743771 and 42385068785).
- The check "one Paraglide runtime in the frontend bundle" passed on both platforms.
- Times, with no Rust cache (the `bundle` cache key is new, see below): Windows job 15 min 44 s, of which
  `cargo build --release` 14 min 01 s; Linux job 11 min 58 s.

## release.yml

Not run: it needs a `vX.Y.Z` tag, and none is pushed in this phase (user, 2026-11-07). Checked instead:
- `actionlint` clean on `ci.yml`, `release.yml` and `mutation.yml`;
- the notes step's awk, run locally on `CHANGELOG.md` with `TAG=v0.1.0`, prints the v0.1.0 section;
- `jq -r .version app/src-tauri/tauri.conf.json` is `0.1.0`, so the version check passes for `v0.1.0`.

## Cache

The jobs found no Rust cache: only `main` saves it (the Aura scheme), and the job was renamed `build` → `bundle`, a new
key. The first run on `main` after the merge saves it.

## What this does not prove

- That `release.yml` publishes: the upload, the artifact layout after `merge-multiple` and `gh release create` run
  only on the first tag.
- That the packages install and start: no install test on a clean machine (ROADMAP 6.4). The E2E runs the debug
  binary, not the installed one.
- Signing: the installers are unsigned (user's choice), so SmartScreen asks on the first start.
