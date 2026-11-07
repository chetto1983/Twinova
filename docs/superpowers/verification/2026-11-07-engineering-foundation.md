# Verification: engineering foundation (2026-11-07)

Plan: `docs/superpowers/plans/2026-11-07-engineering-foundation.md`. Closes the measurable DoD gaps of M1 (ROADMAP 1.4)
and sets up ROADMAP 2.1–2.3 with the gate set of chetto1983/ModbusTool.

Machine: cloud container, Ubuntu 24.04, 4 cores, Rust 1.99.0, Node 22.23.4. Tools: cargo-llvm-cov 0.9.1, cargo-mutants
27.1.0, Vitest 5.1.0 with @vitest/coverage-v8, Stryker 10.1.1.

## Coverage

| Side | Command | Before | After |
|---|---|---|---|
| Rust, whole crate | `cargo llvm-cov --summary-only` | 73.45 % lines (`commands.rs` 0 %) | **93.61 % lines**, 92.43 % regions, 27 tests |
| TypeScript, `src/lib/**/*.ts` | `npm run coverage` | 26.53 % lines, 0 % branches | **100 %** lines, statements, functions, branches |

Still uncovered on the Rust side, and why:
- `lib.rs` (13 lines) and `main.rs` (3): they open the real window, which needs a display and a webview;
- `webview_locale::apply` (12 lines): it needs a WebKitGTK webview; its pure part, `fallback_language`, is covered;
- `plc.rs` (2 lines): errors that the contract check at build time makes unreachable (a contract variable that is not
  a BOOL, an input write that fails).

## Mutation

| Scope | Command | First run | After the added tests |
|---|---|---|---|
| `scene.rs`, `engine.rs` | `cargo mutants -j 2 --timeout 60` | 58 / 62 caught (93.5 %) | **62 / 62** (1 unviable) |
| `src/lib/snapshot.ts` | `npm run mutation` | 26 / 27 killed (96.3 %) | **27 / 27** |

The five survivors were gaps in the tests, not in the code:
- the scan's seconds in `Engine::step` (`/ 1000` mutated to `%` or `*`): no test checked how far the scene moves per
  scan, only deliveries and the jam;
- delivery at the end of the belt (`<` to `<=`): the exact case of a front reaching 3.0 m had no test;
- part ids (`+=` to `*=`): nothing checked that they differ;
- `framingDistance` (`/` to `*`): its only case used a 90° field of view, where tan = 1.

Each has a new test. No test was changed.

## Gates run locally

Green: `cargo fmt --check`, `cargo clippy --all-targets -D warnings`, `npm run check`, `lint`, `format:check`,
`check:i18n`, `deadcode`, `dup`, `build`, `bash scripts/check-file-size.sh` (22 files within 600 LOC), and the
lefthook pre-commit and pre-push hooks.

## CI

Run 42378063592 on `567ecea` (this branch): `rust` (Linux: fmt, clippy, coverage gate), `frontend` and `file-size` are
**green**. `rust-windows` is **red**:

- the crate now **compiles on Windows** (5 min 32 s, after the `tiverse-mmap` fix below);
- the lib's test binary does not start: `0xc0000139 STATUS_ENTRYPOINT_NOT_FOUND`.

Cause, from the installed sources. tauri-build 2.7.4 embeds the application manifest (Common Controls v6,
`windows-app-manifest.xml`) with `WindowsResource::compile()`, which links it into binaries only
(`tauri-build/src/lib.rs:871`). tauri-winres 0.3.7 documents `compile_for_everything()` for the case where "the
application manifest has to be present in test binaries too, for example because the code under test depends on
Common Controls v6". The test binary links muda, which imports `TaskDialogIndirect` from ComCtl32 v6 (muda README,
`common-controls-v6`); without the manifest Windows loads ComCtl32 v5, which lacks it. Not yet confirmed by a run.

`build` (release binary on both platforms) was skipped because it needs `rust-windows`.

## Found and fixed

- **Windows build.** The first CI run on Windows failed in `tiverse-mmap` 1.0.0, a trust-runtime dependency: the
  crates.io copy does not compile against `windows` 0.52. truST patches it in its own workspace (`[patch.crates-io]`
  with `third_party/tiverse-mmap`), and Cargo does not pass a dependency's patches on. The same patch is now in our
  manifest. Reproduced with `cargo check -p tiverse-mmap --target x86_64-pc-windows-msvc`: fails before, passes after.
- **knip on CI.** It flagged `lefthook` as unused on CI only: its plugin reads `lefthook.yml` from its working
  directory (`app/`) or the installed git hooks, which CI does not have. Reproduced in a clone without hooks under
  `CI=true`; fixed with an `npm run hooks` script.
- **Disk.** Mutation testing with `--copy-target` copies the 12 GB target directory; with a third build from the
  pre-push hook, the container's disk ran out. Locally `cargo mutants` ran without `--copy-target`; CI keeps it, as in
  ModbusTool.

## What this does not prove

- **The app running on Windows.** CI runs the Rust tests on Windows and builds the release binary there; nobody has
  launched the window, and there is no WebDriver E2E yet (ModbusTool has one; Twinova's M1 E2E was driven by hand with
  xdotool on Linux).
- macOS: not built.
- The Svelte components are outside the Vitest coverage, as in ModbusTool; their behaviour was checked by the M1 E2E
  only.
- Mutation testing covers `scene.rs`, `engine.rs` and `snapshot.ts`, not `plc.rs` or `commands.rs`.
- No installers were built (ROADMAP 2.4).
