# Spec: runtime footprint, measured before deciding (ROADMAP 2.5)

Date: 2026-11-07. Status: **measured; decision pending** (the user chose "a measurement spike first").

ROADMAP 2.5 asked to replace trust-runtime's `full` feature with "only the encoder and the VM" (path A′ of research
note 10) and to measure the binary size before and after. This spec records what that would cost and gain, measured
on trust-platform `6405915973a7`, before any code is written.

## What `full` brings

`cargo tree -e normal` (Linux):
- the app has **547** crates;
- **406** of them sit under `trust-runtime`, and 249 under Tauri, with some shared. About 300 exist only because of
  `full`;
- among `full`'s direct dependencies: zenoh, rumqttc (MQTT), rustls, native-tls, tungstenite, mdns-sd, rapier3d,
  ratatui/crossterm, clap, notify, gpio-cdev, the ADS/TwinCAT crates. Twinova uses none of them.

## Binary size: `full` costs little

`cargo bloat --release --crates`, Linux, the release profile with `strip = false` (`.text` 6.2 MiB, file 12.6 MiB).
cargo-bloat calls its own numbers "guesswork", and under LTO code inlined into other crates is attributed to them.

| Crate | `.text` |
|---|---|
| twinova_lib (with code inlined from truST) | 1.1 MiB |
| trust_runtime | 1.0 MiB |
| trust_runtime_core (the VM) | 938 KiB |
| trust_hir | 834 KiB |
| tauri | 722 KiB |
| std | 566 KiB |
| trust_syntax | 132 KiB |
| tokio | 46 KiB |
| tiny_http | 7.6 KiB |

zenoh, rumqttc, rapier3d, rustls and ratatui are not among the first 40 crates: LTO already removes what is not
called. The size `full` adds is at most part of trust_runtime's 1.0 MiB plus tens of KiB, against the 8.8 MB stripped
binary (Linux, CI). Removing `full` would shrink the binary only a little; the exact amount is not measured.

## Build time: `full` costs about a third on Linux

`cargo build --release --timings`, 4 cores, Linux, no cache: **8 min 24 s** wall, 1 230 s of summed unit time.

| Unit | Time | Needed by Twinova |
|---|---|---|
| openssl-sys, build script | **174 s** | no: `native-tls` with the `vendored` feature, which trust-runtime turns on, compiles OpenSSL from source |
| trust-runtime | 141 s | partly |
| twinova (binary, LTO) | 84 s | yes |
| aws-lc-sys, build script | 41 s | no (rustls) |
| zenoh + zenoh-config + zenoh-transport | 69 s | no |
| trust-runtime-core | 28 s | yes |
| stabby-abi | 23 s | no (zenoh) |
| trust-hir | 23 s | yes |
| trust-plcopen | 11 s | no |
| rustls | 12 s | no |

The listed units Twinova does not need add up to about 340 s. Together with the smaller ones that is roughly a
third of the summed unit time. OpenSSL is Linux-only, because Windows `native-tls` uses SChannel, so the Windows
share is not measured. In CI `rust-cache` keeps compiled dependencies, so the gain applies to cold builds and to a new
cache key.

## What a fork would be

`TestHarness::from_source` runs:
1. `trust-syntax` and `trust-hir`, separate crates that stay;
2. lowering in `trust-runtime` (`host/harness/build.rs`, `lower_application`);
3. the bytecode encoder (`bytecode/`);
4. `materialize` into `trust_runtime::Runtime`, which wraps the VM of `trust-runtime-core` (a separate crate, stays).

Lines inside `trust-runtime` that a fork would own, counted without test files:

| Measure | Lines |
|---|---|
| The four directories on the path: `host/harness` 12 510, `bytecode` 10 816, `runtime` 14 316, `io` 8 104 | **≈ 45 700**, lower bound |
| Every module reachable from `harness::harness` and `harness::build` through `crate::`/`super::`/`self::` and `pub use`, kept whole with their submodules | **119 522 of 151 403** (305 of 417 modules), upper bound |

The closure script is `closure.py` in the session scratchpad. It counts whole files, so the upper bound includes code a
careful fork would cut. It gives the same 119 522 lines starting from `bytecode` alone: at file level the modules form
one connected block. The coupling that ties them:
- the encoder references the harness (`crate::harness`, 44 references in `bytecode/`);
- the runtime references the ADS server (`crate::ads`, 42) and the debugger (`crate::debug`, 40).

A fork therefore means cutting links inside truST's code, not copying four directories. Research note 10 estimated
path A′ at "about 10 800 lines (the encoder)": the measurement is at least four times that.

## Options

| Option | Cost | Gain |
|---|---|---|
| **Keep `full`** | none | none; a cold Linux build keeps ~1/3 of its time in unused crates |
| **A′: fork lowering, encoder and runtime** | ≥ 45 700 coupled lines to own and cut free | build time, about 300 fewer crates, a path to wasm (M9.4); little binary size |
| **B: an interpreter of our own on `trust-hir`** | an interpreter to write | the same, plus it follows truST's maintained frontend (ROADMAP 2.6) |

## What this does not prove

- The size `full` adds to the binary: cargo-bloat attributes `.text` approximately, and nothing was built without `full`.
- Windows build times: only Linux was timed.
- The size of B: no interpreter was written, so "smaller than A′" is a hypothesis.
