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

Run 42380805842 on `d586dbc`:

| Job | Result |
|---|---|
| `rust` (Linux): fmt, clippy, `cargo llvm-cov --fail-under-lines 85` | green |
| `frontend`: svelte-check, oxlint, prettier, inlang, knip, jscpd, Vitest coverage, build | green |
| `file-size` | green |
| `e2e (ubuntu-24.04)`: the real app through tauri-driver, under Xvfb | green, 6/6 |
| `e2e (windows-2022)`: the real app through tauri-driver and msedgedriver on WebView2 | **green, 6/6** |
| `build (ubuntu-24.04)`: release binary | green, **8 852 208 bytes** (`twinova`) |
| `build (windows-latest)`: release binary | built in 12 min 01 s; the size step failed (below) |

Run 42382743771 on `543a878`, after the shell fix: **all seven jobs green**. The Windows release binary is
**11 078 144 bytes** (`twinova.exe`), the Linux one 8 852 208 bytes as above. The 557 MB of M1 was a debug
binary with debug info, so it is not comparable; these two release sizes are the baseline for ROADMAP 2.5.

The E2E replays the M1 acceptance: the default program builds, Start latches the motor, parts are delivered,
virtual time follows the wall clock within 10 %, a part held at a blocked outlet trips the jam and stops the motor,
and Italian applies without a reload. Locally under Xvfb: 6/6, 5.09 s virtual in 5.05 s wall.

**Windows** is tested this way, as in ModbusTool, and not with `cargo test`. An earlier run (42378063592) ran the
unit tests on Windows: they compiled, but the test binary did not start (`0xc0000139 STATUS_ENTRYPOINT_NOT_FOUND`).
tauri-build 2.7.4 embeds the Common Controls v6 manifest in binaries only (`WindowsResource::compile()`,
`tauri-build/src/lib.rs:871`; tauri-winres documents `compile_for_everything()` for test binaries), and the test
binary links tauri, whose menus import `TaskDialogIndirect` from ComCtl32 v6 (muda README). The unit tests run on
Linux; the app runs on both.

The Windows size step ran in PowerShell, where `2>/dev/null` does not exist: a job's `defaults.run` (here the working
directory) replaces the workflow's `shell: bash`. Every job now repeats the shell, and the step fails if the binary is
missing.

### On `main`

After PR #1 was merged (`fc0d725`): CI run 42385068785 **all green**; the first `mutation.yml` run, 42385068809, gives
cargo mutants **62 caught of 63 (1 unviable)** in 12 min and Stryker **27/27**, the same as the local runs.

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

- **Windows beyond the E2E.** The E2E reads the DOM; nobody looked at the 3D view on Windows, and pacing there was
  checked only by the 10 % ratio. The Rust unit tests do not run on Windows (see CI).
- macOS: not built.
- The Svelte components are outside the Vitest coverage, as in ModbusTool; their behaviour was checked by the M1 E2E
  only.
- Mutation testing covers `scene.rs`, `engine.rs` and `snapshot.ts`, not `plc.rs` or `commands.rs`.
- No installers were built (ROADMAP 2.4).
- The CI mutation workflow (`mutation.yml`, with ModbusTool's three caches) runs on `main` only: the figures above are
  local runs.
