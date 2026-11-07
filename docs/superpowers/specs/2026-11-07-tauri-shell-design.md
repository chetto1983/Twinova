# Spec: Tauri shell with one conveyor scene driven by Structured Text

Date: 2026-11-07. Status: **approved scope** (user, 2026-11-07: "Facciamo una Tauri app"; first slice = 3D conveyor + ST,
Aura design system, English + Italian).

## Why Tauri

The browser target needs ST without a toolchain, and truST's former bytecode VM does not build for wasm32 without a deep
fork (research note 10, prototype `prototypes/st-vm-probe`). A Tauri 2 app runs the engine natively in its Rust backend,
where the same VM measured 6 ms to compile a program and 137 421 scans/s. The web view only draws. This also follows the
user's existing Tauri app, chetto1983/ModbusTool (Tauri 2, Svelte 5, TypeScript, Vite, Paraglide, Aura tokens).

## Scope of this slice

One fixed scene: a straight belt conveyor. Parts arrive, ride the belt and leave at the end. A photocell sits near the
end. The motor is driven by a Structured Text program the user edits and builds in the app. Out of scope: editing the
scene, more than one component, a PLC link, discrete-event scheduling, Ladder, saving projects.

### The scene model (backend, `scene`)

- The belt is 3.0 m long. When the motor runs, it moves at 0.5 m/s. Part length is 0.30 m.
- A new part enters every 2.0 s of virtual time with its **rear edge at x = 0** (front at 0.30 m), as long as the entry
  is free, i.e. the last part's tail is past 0.30 m. A full belt therefore holds 3.0 / 0.30 = 10 parts. (Corrected
  2026-11-07: the first version placed the front at 0, so a part sat before the belt and a blocked belt held 11; the
  capacity test caught it.)
- Kinematics are closed-form for one scan step dt:
  - each part advances by v·dt;
  - a part never passes the part in front of it (accumulation);
  - a part never passes the end of the belt while the **outlet is blocked**.
- A part whose front reaches 3.0 m with the outlet free leaves the scene and is counted as delivered.
- The photocell is at x = 2.70 m. It reads TRUE while any part covers that point.
- The outlet-blocked switch is a fault the user (or a teacher) can toggle.

### The PLC binding (backend, `plc`)

- It wraps `trust_runtime::harness::TestHarness`, pinned to trust-platform `6405915973a77587748ffb061a61d37bb8a368b0`
  (the last commit read with the bytecode VM, MIT OR Apache-2.0).
- `build(source)` returns the program, or the compile errors as text.
- The scene ↔ program contract, as variables of `PROGRAM Main`:
  - `sensor : BOOL` (written by the scene);
  - `start : BOOL` and `stop : BOOL` (written by the UI buttons, held for one scan);
  - `motor : BOOL` (read by the scene).
- A program without those names is refused with a message that lists the missing ones.

### The engine (backend, `engine`)

Lockstep with a fixed scan of 10 ms of virtual time. Each step does, in order:
1. write the inputs;
2. advance the PLC time by 10 ms;
3. run one cycle;
4. read `motor`;
5. advance the scene by 10 ms.

The run modes are:
- **Run**, paced to the wall clock at speed ×1. It reports when it falls behind instead of hiding it.
- **Pause**.
- **Step**, one scan.
- **Reset**, which brings back the empty belt, time 0 and a fresh PLC.

A run publishes a snapshot (time, part positions, I/O, delivered count) to the UI at about 30 Hz. The pure step function
runs headless in tests.

### UI (frontend)

- **Header**: Build, Run/Pause, Step, Reset, Start/Stop buttons, theme and language switches.
- **Left**: the ST editor (monospace textarea in this slice) with build errors.
- **Centre**: three.js view of the belt, the parts, the photocell (lit when TRUE) and the motor state.
- **Right**: I/O table with the four variables, virtual time, delivered count, outlet-blocked toggle.
- Aura tokens and fonts as in ModbusTool. All texts come from Paraglide `messages/{en,it}.json`.

### Default program

A program that latches the motor on `start`, drops it on `stop`, and stops on a 2 s jam (`TON` on `sensor AND motor`,
latched). With the outlet blocked, the belt fills, the photocell stays covered, and the jam stops the motor.

## Acceptance (machine-checkable)

1. `cargo test` covers the following, headless:
   - accumulation;
   - the outlet block;
   - photocell edges;
   - entry spacing;
   - the default program running for 60 s virtual with the outlet free (deliveries > 0, motor on, no jam);
   - the same run with the outlet blocked (jam within 2 s of the sensor being continuously covered, motor off).
2. `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `npm run check`, `npm test`, `npm run build` are
   green.
3. E2E: the app is built and launched under a virtual X display, and a screenshot shows the scene running with the
   default program.

## What it will not prove

- Real-time pacing under load.
- Behaviour on Windows or macOS (Linux only).
- Determinism across machines.
- That the old VM matches IEC everywhere beyond the cases tested.
