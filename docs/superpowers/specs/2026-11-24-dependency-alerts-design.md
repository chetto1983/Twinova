# Spec: the dependency alerts on main

The user, 2026-11-24: "guarda le vulnerabilità c'è ne sono 22". These are the Dependabot alerts on `main` (11
moderate, 11 low).

## Measured first

Dependabot's API answers 403 to this session, so the set was rebuilt from the same sources: `npm audit`, and
`cargo audit` (cargo-audit 0.22, RustSec's database).

| Ecosystem | Package | Through | Advisories | Fix in reach |
|---|---|---|---|---|
| npm | dompurify 3.4.15 | monaco-editor 0.57.0 | 5 GHSA, low; Monaco's own entry follows them | 3.4.17, which Monaco 0.58.0 takes |
| npm | minimatch 10.2.6 | @stryker-mutator/core, readdir-glob (webdriverio's archiver) | GHSA-g3hv-75wg-m249, moderate | from 11.0.1 only, no 10.x fix |
| npm | qs 6.15.1 | typed-rest-client 2.3.1 (Stryker), which pins it | 4 GHSA, moderate | 6.17.1, as express already has |
| Cargo | cookie 0.18.2 | Tauri | RUSTSEC-2026-0594 | 0.18.3, a patch update |
| Cargo | url 2.5.8 | Tauri's build | RUSTSEC-2026-0592 | 2.5.9, a patch update |
| Cargo | rustls 0.20.9, ring 0.16.20 | tiny_http 0.12's `ssl-rustls`, which truST's fork turns on in trust-runtime | RUSTSEC-2024-0336, -2026-0349, -2025-0009 | none for tiny_http 0.12; the TLS is optional |
| Cargo | heapless 0.8.0 | rstar 0.13.1, which parry3d 0.31.1 (rapier3d 0.36.1) pins | 9 "unsound" | none: parry3d 0.31.1 is the latest |
| Cargo | glib 0.18.5 | gtk 0.18, Tauri's Linux stack | RUSTSEC-2024-0429, "unsound" | none in Tauri 2.12 |

What reaches them:
- **rustls and ring.** trust-runtime uses tiny_http's TLS only in the web server's `Server::https`
  (`web/server.rs`). Twinova never starts that server: it runs the PLC through `TestHarness`.
- **heapless.** rstar uses `heapless::binary_heap` only (`nearest_neighbor.rs`).
  - No advisory names `BinaryHeap`.
  - The one on its storage, RUSTSEC-2026-0577 (`vec::IntoIter::drop`), needs an element whose `Drop` panics.
  - rstar's elements are wrappers of references.
- **glib.** RUSTSEC-2024-0429 is about `glib::VariantStrIter`'s iterator impls. Twinova calls no glib itself.

## Decisions

1. **Updates within range:** cookie and url by `cargo update`.
2. **npm overrides, scoped to the consumer:**
   - DOMPurify 3.4.17 under monaco-editor. Monaco stays at 0.57.0, which the editor's specs measured; 0.58.0 was
     published the day before, with a new dependency.
   - minimatch ^11.0.5 under @stryker-mutator/core and readdir-glob. Both import `minimatch` and `Minimatch`, which 11
     keeps.
   - qs ^6.17.1 under typed-rest-client.
3. **rustls and ring: a patch to truST's fork** (the user's choice). Commit `0d09efe` on
   `twinova/6405915-breakpoint-body`:
   - tiny_http no longer turns `ssl-rustls` on; a new `web-tls` feature does;
   - trust-runtime's default features keep `web-tls`, so a default build serves HTTPS as before;
   - `full`, which Twinova builds with no default features, leaves it out;
   - without it, a web configuration asking for TLS is refused, never served in clear.

   The `[patch]` rev and engine's direct trust-hir and trust-syntax move to `0d09efe` together. Upstream stays at
   `6405915`, as the truST policy holds; the fork carries the change. The fork's twinova workflow checks both sides:
   - `full` without `web-tls` has no rustls 0.20;
   - the security contract tests pass with it.
4. **heapless and glib stay**, recorded here as not reachable, until rapier or Tauri moves.

## Verified

- `npm audit`: 0 vulnerabilities.
  - Stryker's dry run finds its 12 files and runs its 198 tests with minimatch 11.
  - readdir-glob matches with it.
  - vitest 301/301, svelte-check, lint and the build are clean.
- `cargo audit`: 0 vulnerabilities. Left are 5 "unmaintained" notices (chrono, humantime, paste, proc-macro-error,
  unsafe-libyaml) and the 10 "unsound" ones above.
- `cargo tree`: no rustls 0.20, ring 0.16, webpki 0.22 or untrusted 0.7.
- Clippy on the workspace is clean; twinova-engine and twinova-agent tests pass.

## What this does not prove

- That Dependabot closes each alert. It reads the lockfiles on `main` after the merge, and this session cannot read
  its list. The 10 unsound advisories may stay listed.
- That truST's web server serves HTTPS with `web-tls` beyond its contract tests. Twinova does not start it.
