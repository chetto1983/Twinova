# Twinova brand verification — 2026-11-08

## Scope

This run verified the VectorCraft installation and Codex MCP registration, repository confinement, editable and exported logo assets, desktop icon formats, Svelte header integration, light/dark presentation, README presentation and links, frontend quality gates, and the production web build.

Implementation commits under review: `6ea7a38`, `e88f749`, and `2ddf2ad` plus the final verification fix.

## VectorCraft installation and MCP

- Release: VectorCraft `v0.22.0`, official Windows x64 MSI from `storytold/vectorcraft`.
- Published SHA-256 and locally calculated SHA-256 matched:
  `578A92BA4020D8935F64E0437F3FB3B5C5DCA7AF3D8FC70EFFB9C7FD9BF9D283`.
- `Get-AuthenticodeSignature` returned `Valid`; signer subject:
  `CN=Learning Machines Inc, O=Learning Machines Inc, L=Atlanta, S=Georgia, C=US`.
- `vectorcraft-cli.exe --help` exited 0 and listed the `mcp` subcommand.
- `codex mcp get vectorcraft` reported an enabled stdio server using the absolute command
  `C:\Users\chett\AppData\Local\Programs\ArtCraft\VectorCraft\vectorcraft-cli.exe`.
- Registered arguments were exactly:
  `mcp --headless --automation-read-root D:\Repo\Twinova --automation-write-root D:\Repo\Twinova`.
- Direct MCP initialization reported VectorCraft `0.22.0` and 33 tools.
- The confinement smoke test created and saved a document inside Twinova, then attempted the same write under the user temporary directory. The in-repository write succeeded; the outside write was rejected as outside `D:\Repo\Twinova` and no outside file was created.

## Logo assets

- VectorCraft MCP created `app/src-tauri/icons/twinova-logo.vectorcraft` and exported the canonical 1024 × 1024 transparent SVG.
- The 1024, 32, and 16 pixel previews retained the paired industrial-loop silhouette and alternating blue/green crossings.
- `npx tauri icon` regenerated 15 desktop PNGs plus ICO and ICNS.
- `node scripts/verify_brand_assets.mjs` checked:
  - required files and dimensions;
  - PNG signatures, square dimensions, and alpha-bearing colour types;
  - ICO and ICNS magic values;
  - transparent SVG canvas, required Aura colours, and no embedded text, fonts, or raster image;
  - equal packaged/UI geometry and paint;
  - README brand, navigation, and repository-relative Markdown links;
  - this dated verification record and its retained theme captures.

## Header presentation

Playwright loaded the local Vite build at a 1280 × 720 viewport. It selected each persisted theme, reloaded, measured `.brand img`, and saved a full-viewport capture.

| Theme | CSS size | Bounding box | Evidence |
|---|---:|---:|---|
| Dark | 26 × 26 px | `x=19, y=18, width=26, height=26` | [dark capture](img/2026-11-08-twinova-brand-dark.png) |
| Light | 26 × 26 px | `x=19, y=18, width=26, height=26` | [light capture](img/2026-11-08-twinova-brand-light.png) |

Both captures use identical geometry and keep the icon aligned with an Atkinson Hyperlegible Next wordmark. The mark and fixed light wordmark sit on the same graphite brand plate in both themes, so the lockup does not invert or lose contrast when the application theme changes.

## Commands and outcomes

| Command | Outcome |
|---|---|
| `node scripts/verify_brand_assets.mjs` | Passed: 15 PNGs, ICO, ICNS, SVG, editable master, UI asset, README, and verification evidence |
| `npm.cmd test -- --run src/lib/snapshot.test.ts` | 10/10 passed; mounted DOM contained the decorative SVG image and no placeholder circles |
| `npm.cmd run check` | Passed with 0 errors and 0 warnings |
| `npm.cmd run lint` | Passed with 0 warnings |
| `npm.cmd run format:check -- --end-of-line auto` | All matched files passed Prettier formatting |
| `npm.cmd run check:i18n` | 30 messages, English and Italian, 16 source files; no findings |
| `npm.cmd run deadcode` | Passed |
| `npm.cmd run dup` | Passed |
| `npm.cmd run coverage` | 4 files and 21 tests passed; statements, branches, functions, and lines all 100% |
| `npm.cmd run build` | Passed; Vite transformed 162 modules and produced the production bundle |
| `bash scripts/check-file-size.sh` | 42 files within the 600-LOC cap |
| `node .superpowers/sdd/2026-11-08-twinova-brand/mcp-confinement.mjs` | Allowed repository write and rejected outside write |

The unmodified `npm.cmd run format:check` reports the checkout's CRLF line endings because this Windows Git installation has `core.autocrlf=true` while `.editorconfig` requires LF. The full tree passed the same Prettier check with `--end-of-line auto`; committed blobs remain normalized by Git and CI checks them on Linux.

## What this does not prove

- The browser captures do not exercise Tauri IPC; direct Vite loading emits the expected missing-Tauri-bridge errors. The existing real-app E2E remains the integration proof for native commands.
- This run did not execute a fresh Codex process after MCP registration. It verified the saved configuration and initialized the installed server directly over stdio.
- The representative confinement probe does not exhaust every traversal, junction, or filesystem race case implemented by VectorCraft.
- The logo checks do not constitute a user study, trademark search, or legal clearance.
- This run does not test a physical PLC, PLCSIM Advanced, or the future lockstep commissioning mode.
