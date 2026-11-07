# Twinova

An open-source 3D factory simulator and virtual-commissioning tool: lay out a line from components,
run its material flow, and test a real or emulated PLC against it before the machine exists.

**Status: research.** There is no code yet. This repository currently holds the study the design will
be built on.

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
component scripting in **Python**.

## Research (in Italian)

[`docs/research/00-indice.md`](docs/research/00-indice.md) is the entry point. It covers:

- a feature map of Visual Components 5.1, built from its public help;
- how simulation engines are organised (discrete-event, fixed-step and hybrid) and how they sync with a PLC;
- a benchmark of 13 other commercial simulators, plus open-source projects, and a "best of" matrix;
- an assessment of Open Industry Project, with test and benchmark measurements;
- an inventory of the Rust ecosystem for each building block.

Every claim cites its source. Descriptions of commercial products are paraphrased from public
documentation (clean-room). Twinova is not affiliated with any of the vendors named there.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your
option.
