# Plan: release bundles (ROADMAP 2.4)

## Goal

CI builds the installers on every change to `main` and every PR, and records their sizes. A `release.yml` is ready
to publish them on a `vX.Y.Z` tag, with the matching `CHANGELOG.md` section as the release text. No tag is pushed in
this phase.

## Architecture

ModbusTool's `bundle` job and `release.yml` (chetto1983/ModbusTool), with the paths under `app/` and without what the
user ruled out on 2026-11-07:

| ModbusTool | Twinova 2.4 |
|---|---|
| MSI on Windows, deb + AppImage on Linux, no macOS | the same |
| SignPath signing of the MSI | not ported (no signing yet) |
| Tauri updater: signatures and `latest.json` | not ported (ROADMAP 6.4) |
| winget submission | not ported |
| `scripts/check-single-runtime.mjs` after the build | ported: Twinova uses the same Paraglide setup |

## Tech Stack

Tauri CLI 2.12 bundler (WiX for MSI, linuxdeploy for AppImage), GitHub Actions, `gh release create`.

## Spec

None: packaging, no behaviour. The user's decisions of 2026-11-07 (session) stand in for one.

## Global Constraints

- Actions pinned by SHA, as in ModbusTool. Rust 1.99.0 pinned.
- The tag must equal `app/src-tauri/tauri.conf.json` version, and `CHANGELOG.md` must have its section, or the release
  stops before building.

## Review Focus

- The release job publishes only what the build jobs produced (`if-no-files-found: error`).
- Unsigned installers: say so in the release text.

## Decisions taken while writing this plan

- `ci.yml`'s `build` job becomes `bundle`: it builds the installers (`--bundles msi` / `deb,appimage`), which contain the
  release binary, so one build serves both 2.4 and 2.5's size baseline.
- `CHANGELOG.md` starts with `v0.1.0` (unreleased), describing M1 and the engineering foundation.

## File structure

```
CHANGELOG.md
.github/workflows/ci.yml        build -> bundle
.github/workflows/release.yml   notes, build-windows, build-linux, publish
scripts/check-single-runtime.mjs
```

## Tasks

- [ ] **1. CI bundles.** Verify: the `bundle` jobs list `.msi`, `.deb`, `.AppImage` with their sizes.
- [ ] **2. release.yml + CHANGELOG.md.** Verify: `actionlint`-clean YAML; the notes step extracts the v0.1.0 section
  locally (same awk).
- [ ] **3. Verification file, ROADMAP 2.4.**
