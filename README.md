# Twinova

An open-source 3D factory simulator and virtual-commissioning tool: lay out a line from components,
run its material flow, and test a real or emulated PLC against it before the machine exists.

**Status: early prototype.** The research the design is built on, and a first desktop app (`app/`).

## Goals

- **One component model**: parts with joints, behaviors, properties and typed signals, in an open,
  documented format.
- **A deterministic hybrid engine**: a discrete-event core with integer time, analytic conveyors and
  sensors, and fixed-step physics only where it is needed.
- **PLC-in-the-loop as a core feature**, not a paid add-on. That means two run modes:
  - free-running against the wall clock;
  - lockstep with an emulated PLC (S7-PLCSIM Advanced `TimespanSynchronized`), so a test repeats exactly.
- **Open connectivity**: S7, OPC UA, Modbus, MQTT, ADS and more.
- **Headless first**: the engine runs as a library or service; the 3D editor is a client.

Planned stack, to be confirmed by measurements: the engine in **Rust**, the editor in **TypeScript**,
component logic in **IEC 61131-3** (Structured Text and Ladder), the languages PLC programmers already
use.

## The app

`app/` is the first runnable slice: a Tauri 2 desktop app with one belt conveyor in 3D, driven by an IEC 61131-3
Structured Text program you edit and build inside the app. It runs on truST's bytecode VM, with no toolchain needed.

- The engine is a discrete-event core (nexosim, on one thread). The belt is computed in closed form, so sensor edges
  are exact to the nanosecond. The PLC scan period comes from the program's IEC tasks.
- Runs at ×0.25 to ×10 the wall clock, or as fast as possible. You can step one scan at a time, and save and restore a
  state. The lag behind real time is shown, never hidden.
- It is available in English and Italian, with light and dark themes.

```sh
cd app && npm install && npm run tauri dev
```

Linux needs the Tauri prerequisites (WebKitGTK 4.1 and friends). Spec, plan and verification are in
[`docs/superpowers/`](docs/superpowers/).

CI builds the installers on every push to `main`: an MSI for Windows, and a deb and an AppImage for Linux. They are
not signed.
No release is published yet; the release text comes from [`CHANGELOG.md`](CHANGELOG.md).

Development checks run as git hooks (`cd app && npm run hooks`), and in CI on every pull request. The full list,
including the end-to-end run of the real app (`npm run e2e`), is in [`CLAUDE.md`](CLAUDE.md), "Post-edit validation".

## Roadmap

Milestones and their phases, with acceptance criteria and status: [`ROADMAP.md`](ROADMAP.md).

## Research (in Italian)

[`docs/research/00-indice.md`](docs/research/00-indice.md) is the entry point. It covers:

- a feature map of Visual Components 5.1, built from its public help;
- how simulation engines are organised (discrete-event, fixed-step and hybrid) and how they sync with a PLC;
- a benchmark of 13 other commercial simulators, plus open-source projects, and a "best of" matrix;
- an assessment of Open Industry Project, with test and benchmark measurements;
- an inventory of the Rust ecosystem for each building block;
- the open-source IEC 61131-3 runtimes that could run component logic;
- how to run Structured Text without a compiler, inside the app;
- the structure of the Visual Components eCatalog, and what it suggests for Twinova's first component library.

Every claim cites its source. Descriptions of commercial products are paraphrased from public
documentation (clean-room). Twinova is not affiliated with any of the vendors named there.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your
option.
