# Spec: following truST upstream (ROADMAP 2.6)

Date: 2026-11-07. Status: **approved policy** (user, 2026-11-07); first review recorded below.

Twinova uses two parts of truST (`johannesPettersson80/trust-platform`, MIT OR Apache-2.0), and upstream treats them
differently. This spec says how each is kept current.

## Facts, measured on 2026-11-07

Partial clone of trust-platform (`git clone --filter=blob:none`), HEAD `7cfa8844`, release 0.34.0. Our pin is
`6405915973a7`, which is 0.24.75.

| | Pin `6405915` | HEAD `7cfa8844` |
|---|---|---|
| Commits reachable from HEAD and not from the pin | — | **1 678**: 680 committed after the pin's date (2026-10-23), 142 on HEAD's first-parent line; the rest were merged from older branches |
| VM and bytecode files in `trust-runtime-core/src` | 241 | **0** (deleted on 2026-10-24, research note 10) |
| `trust-wasm-analysis` depends on | trust-syntax, trust-hir, trust-ide | **the same** |
| Toolchain | Rust 1.99.0 | Rust 1.99.0 |

Commits since the pin, per crate (`git rev-list --count 6405915973a7..origin/HEAD -- crates/<crate>`):

| Crate | Commits |
|---|---|
| trust-syntax | 17 |
| trust-hir | 104 |
| trust-ide | 7 |
| trust-wasm-analysis | 4 |
| trust-runtime-core | 215 |
| trust-runtime | 469 |

These counts use the same range, so they include commits merged from older branches.

What the frontend gained since the pin: vendor dialect rules selected by `[project] vendor_profile`
(`trust-hir/src/dialect.rs`). `codesys`, `beckhoff` and `twincat` select CODESYS rules; `iec`, `siemens`,
`mitsubishi` and `gxworks3` select IEC rules. The parser contracts were also hardened (`trust-syntax` log).

None of these crates is on crates.io. That covers `trust-hir`, `trust-ide`, `trust-wasm-analysis`, `trust-runtime`
and `trust-runtime-core`. A `trust-syntax` 0.2.2 exists there, but it is not established to be the same project. So
no tool updates them: a git `rev` moves only by hand.

## Policy

### 1. Execution: frozen at the pin, ours to maintain

`trust-runtime` (lowering, encoder, `Runtime`) and `trust-runtime-core` (VM) stay at `6405915`, the last commit with
the bytecode engine. Upstream has no successor to move to.
- Upstream fixes are ported by hand, as a reviewed change with a test, citing the upstream commit.
- M3 closed with the existing `full` pin retained (spec `2026-11-08-m3-closure-design.md`). A′ and B are not selected.
  If measured compatibility, correctness or resource constraints later require a fork or replacement, the copied
  code keeps truST's licence notice and names the commit it came from. The browser milestone M9.4 was removed.

### 2. Language frontend: followed upstream, monthly

`trust-syntax`, `trust-hir`, `trust-ide` and `trust-wasm-analysis` are maintained upstream and build without the
runtime. Once Twinova depends on them directly (ROADMAP 4.7), their `rev` is followed:
1. Once a month, list what changed since our `rev`, for those crates only:
   ```sh
   git clone --filter=blob:none --no-checkout https://github.com/johannesPettersson80/trust-platform
   git -C trust-platform log --format='%h %cs %s' <our-rev>..origin/HEAD -- \
     crates/trust-syntax crates/trust-hir crates/trust-ide crates/trust-wasm-analysis
   ```
2. Read those commits, licence included. Move the `rev` in one PR whose body lists them. CI decides: tests, E2E and
   mutation.
3. Check for drift: the editor's diagnostics must agree with what the pinned VM accepts. A construct the new frontend
   accepts and the old engine cannot run shows up as a build error in the app, and it is the PR's job to say how it is
   handled.

Cargo builds two `rev`s of the same git repository as separate sources, so the frontend at a newer `rev` and the
runtime at the pin can coexist. The cost is that `trust-hir` is compiled twice.

### 3. The VS Code extension

`trust-lsp` (ROADMAP 4.6) comes from the marketplace and reads only `.st` files, so it needs no pin. Its Live Values
and debugger talk to truST's runtime protocol, which may change after the pin; ROADMAP 4.8 checks that before relying
on it.

## First review (2026-11-07)

Done with the commands above against HEAD `7cfa8844`. There is nothing to move yet: Twinova has no direct frontend
dependency. Its `trust-syntax` and `trust-hir` come through `trust-runtime` at the pin, and adding
`trust-wasm-analysis` before 4.7 uses it would be dead code (CLAUDE.md). The first `rev` move therefore comes with
ROADMAP 4.7.

## What this does not prove

- That a newer frontend builds next to the pinned runtime: it was not built. The `rev` coexistence is Cargo's
  documented behaviour for distinct git sources, not a run.
- The rate of upstream change ahead: 680 commits in the 15 days after the pin is one sample.
