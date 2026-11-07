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

## Direct runtime dependencies

Transitive dependencies carry their own licenses in their packages.

| Dependency | License | Source |
|---|---|---|
| trust-runtime (truST Platform) at `6405915973a77587748ffb061a61d37bb8a368b0` | MIT OR Apache-2.0 | <https://github.com/johannesPettersson80/trust-platform> |
| tauri | MIT OR Apache-2.0 | <https://github.com/tauri-apps/tauri> |
| serde, serde_json | MIT OR Apache-2.0 | <https://github.com/serde-rs> |
| webkit2gtk (Linux) | MIT | <https://github.com/tauri-apps/webkit2gtk-rs> |
| three | MIT | <https://github.com/mrdoob/three.js> |
| @tauri-apps/api | MIT OR Apache-2.0 | <https://github.com/tauri-apps/tauri> |
| svelte (build-time) | MIT | <https://github.com/sveltejs/svelte> |
