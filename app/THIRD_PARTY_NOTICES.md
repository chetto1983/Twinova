# Third-party notices (app)

## Bundled assets

### Fonts

- Fraunces: Copyright 2018 The Fraunces Project Authors, <https://github.com/undercasetype/Fraunces>.
- Atkinson Hyperlegible Next: Copyright 2020-2024 The Atkinson Hyperlegible Next Project Authors,
  <https://github.com/googlefonts/atkinson-hyperlegible-next>.
- Commit Mono: Copyright (c) 2023 Eigil Nikolajsen, <https://github.com/eigilnikolajsen/commit-mono>.
- License: SIL Open Font License 1.1. The full text is in [`src/assets/fonts/OFL.txt`](src/assets/fonts/OFL.txt).
- Use: copied unmodified from chetto1983/ModbusTool, which took them from chetto1983/Aura (`web/public/fonts`).

### Design tokens

- Source: <https://github.com/chetto1983/Aura> `web/tokens/tokens.json`, same owner, by way of chetto1983/ModbusTool
  `src/app.css`.
- Use: colour, radius, font and motion values in `src/app.css`.

## Code taken from other projects

| What | From | License |
|---|---|---|
| `src-tauri/src/webview_locale.rs` | chetto1983/ModbusTool `src-tauri/src/webview_locale.rs` | MIT |
| `src/lib/i18n.svelte.ts`, `src/lib/preferences.ts` | chetto1983/ModbusTool `src/lib/` | MIT |
| `src/app.css` (component classes) | chetto1983/ModbusTool `src/app.css` | MIT |
| `e2e/run.mjs` (structure and helpers) | chetto1983/ModbusTool `e2e/run.mjs`, `e2e/helpers.mjs` | MIT |
| `src/lib/oip/geometry/`, `src/lib/oip/parts/`, `src/lib/materials.ts` (values), `src/assets/oip/` | Open Industry Project at `1e7488c` (`src/core/geometry/`, `examples/parts/`, `public/textures/`); notice in the repository's `THIRD_PARTY_NOTICES.md` | MIT |
| `src-tauri/physics/src/world.rs`, `world_devices.rs`, `layout_devices.rs`, `path.rs`, `surfaces.rs` (surface velocity, a curve's deck and rotating field, the guards, a skewed roller's drive, the chain transfer's deck and lift, the blade stop's blade, the diverter's paddle, the turntable's lane, translated) | Open Industry Project at `1e7488c` (`src/sim/conveyorContacts.ts`, `src/sim/deckDrive.ts`, `src/sim/rigDriver.ts`, `src/core/geometry/collisionShapes.ts`, `examples/parts/roller-conveyor/behavior.ts`, `examples/parts/{blade-stop,diverter,turntable-conveyor}/`) | MIT |
| `src-tauri/project/tests/fixtures/**/twinova.*.st` (the conveyor drive, the chain transfer, the stopper, the pusher and the turntable, re-expressed in ST) | Open Industry Project at `1e7488c` (`examples/parts/_shared/conveyorDrive.js`, `examples/parts/{chain-transfer,blade-stop,diverter,turntable-conveyor}/`) | MIT |

## Direct runtime dependencies

Transitive dependencies carry their own licenses in their packages.

| Dependency | License | Source |
|---|---|---|
| trust-runtime (truST Platform) at `6405915973a77587748ffb061a61d37bb8a368b0` | MIT OR Apache-2.0 | <https://github.com/johannesPettersson80/trust-platform> |
| tiverse-mmap, truST's patched copy (`third_party/tiverse-mmap`) at the same commit, through `[patch.crates-io]` | MIT OR Apache-2.0 | <https://github.com/johannesPettersson80/trust-platform> |
| rapier3d 0.36.1, with `enhanced-determinism` | Apache-2.0 | <https://github.com/dimforge/rapier> |
| tauri | MIT OR Apache-2.0 | <https://github.com/tauri-apps/tauri> |
| serde, serde_json | MIT OR Apache-2.0 | <https://github.com/serde-rs> |
| webkit2gtk (Linux) | MIT | <https://github.com/tauri-apps/webkit2gtk-rs> |
| three | MIT | <https://github.com/mrdoob/three.js> |
| @tauri-apps/api | MIT OR Apache-2.0 | <https://github.com/tauri-apps/tauri> |
| svelte (build-time) | MIT | <https://github.com/sveltejs/svelte> |
