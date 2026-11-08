# Verification: CPU and memory of the release app (2026-11-08)

The user asked how much CPU and RAM the production build uses, because complex lines must not crash it.
`app/e2e/soak.mjs` drives the release binary through tauri-driver and samples `/proc` every 2 s: CPU in % of one core
(utime + stime) and the resident set (VmRSS), for the app and its WebKit children. Phases: idle after start, Run ×1
with the line running, as fast as possible, paused.

Machine: the cloud container, 4 cores, 16 GB, Xvfb with **software OpenGL** (Mesa llvmpipe). Release build of
`0fad8c7` and of its parent.

## Before: the view redrawn every frame (`718e6b7`), 60 s per phase

| Phase | twinova | WebKitWebProcess | RSS twinova | RSS WebKitWebProcess |
|---|---|---|---|---|
| Idle | 6.7 % | 279 % | 168 → 167 MB | 403 → 406 MB |
| Run ×1 | 7.0 % | 285 % | 168 → 168 MB | 416 → 421 MB |
| As fast as possible | 42.2 % | 254 % | 168 → 168 MB | 428 → 428 MB |
| Paused | 6.6 % | 283 % | 168 → 167 MB | 427 → 429 MB (peak 438) |

WebKitNetworkProcess: 0 %, 46 MB, in every phase.

- **Where the CPU went.** Per-thread CPU at idle, over 10 s: `llvmpipe` threads 269 %, compositor 8 %, web process
  main thread 6 %, `twinova` 5 %.
- **The cause.** `SceneView.svelte` rendered on `setAnimationLoop`, i.e. every animation frame even with nothing
  changing.

## After: render on demand (`0fad8c7`), 60 s per phase

The fix follows the three.js manual, "Rendering on demand", and Bevy's `WinitSettings::desktop_app()` (reactive
updates). A new snapshot, an `OrbitControls` `change` or a resize queues at most one `requestAnimationFrame`.

| Phase | twinova | WebKitWebProcess | Frames/s drawn | RSS twinova | RSS WebKitWebProcess |
|---|---|---|---|---|---|
| Idle | 0.0 % | 0.0 % | 0 | 173 MB | 396 → 394 MB |
| Run ×1 | 7.2 % | 284 % | 25.8 | 173 MB | 406 → 422 MB |
| As fast as possible | 44.4 % | 179 % | 16.8 | 173 MB | 425 → 428 MB |
| Paused | 0.0 % | 0.2 % | 0 | 173 MB | 428 → 425 MB |

Frames/s counts the `requestAnimationFrame` callbacks over 10 s, in a separate run.

- **Idle and paused.** The UI drops from about 280 % to 0 %. The app process drops too, from 6.7 % to 0 %, because it
  no longer serves the compositor's frames.
- **Run ×1 is unchanged here.** The backend emits a snapshot every 33 ms plus the work of the tick. llvmpipe draws
  25.8 of them per second, so the software rasterizer is the limit, at about 3 cores. Before the fix, the 60 Hz loop
  was bound by the same limit.
- **As fast as possible.** The engine ran 38 585 s of virtual time in 60 s, about 640 × real time, on 44 % of one
  core. Fewer snapshots reach the UI (16.8/s), so the UI uses less.
- **The engine's memory is flat** at 168–173 MB in every phase. This matches the model: the belt holds at most 10
  parts, and the input log grows only with user inputs.

## One hour on the fixed build (`0fad8c7`), 15 min per phase

| Phase | twinova | WebKitWebProcess | RSS twinova | RSS WebKitWebProcess |
|---|---|---|---|---|
| Idle | 0.0 % | 0.0 % | 173 → 173 MB | 396 → 394 MB |
| Run ×1 | 7.2 % | 276 % | 174 → 174 MB | 405 → 429 MB (peak 445) |
| As fast as possible | 41.3 % | 182 % | 174 → 174 MB | 431 → 451 MB |
| Paused | 0.0 % | 0.0 % | 174 → 173 MB | 453 → 448 MB |

- **The engine's memory is flat over the hour**: 173–174 MB.
- **The UI grows while snapshots arrive**: +24 MB in 15 min of Run ×1, +20 MB in 15 min as fast as possible. It does not
  grow while idle or paused, and it gives back 5 MB once paused.
  - Whether that is a leak is not settled. If the growth were linear, it would be about +90 MB per hour of running, and
    a heap profile of the web process would tell garbage from retained objects.
  - Hypothesis, not measured: retained snapshot objects or the JIT. The check belongs to the M4 stress scene.
- **As fast as possible**: 493 185 s of virtual time in 900 s, i.e. 548 × real time with the UI attached.
- **Disturbance**: the phases ran 09:13–10:13 UTC. A local E2E with its own app process ran for one minute at 09:22
  (idle phase), and a low-priority (`nice 19`) release build for 5 min at 09:47 (as-fast-as-possible phase).

## After the industrial view (`f65572f`, PR #22), 45 s per phase

The 3D view gained the lighting rig, soft shadows (a 2048 × 2048 map) and the floor after Open Industry Project's.
Same machine and software GL as above. Frames/s counts the `requestAnimationFrame` callbacks over 10 s, as before.

| Phase | twinova | WebKitWebProcess | Frames/s, before → now | RSS WebKitWebProcess |
|---|---|---|---|---|
| Idle | 0.0 % | 0.0 % | 0 → 0 | 442 → 440 MB (396 before) |
| Run ×1 | 5.4 % | 282 % | 25.8 → 23.1 | 447 → 477 MB |
| As fast as possible | 39.0 % | 217 % | 16.8 → 17.1 | 478 → 466 MB |
| Paused | 0.0 % | 0.5 % | 0 → 0 | 466 → 465 MB |

- **Shadows cost about 10 % of the frames in Run ×1 here.** The UI's CPU did not change, because the software
  rasteriser was already the limit. On a GPU the cost is expected to be smaller; that is a hypothesis.
- **The UI starts 46 MB higher.** The shadow map is part of it, and the rest is not broken down. A 1024 × 1024 map is
  the first lever if it matters.
- **Idle and paused stay at zero**: rendering on demand holds with shadows.
- **The as-fast-as-possible phase overlapped** with a `git push` hook compiling Rust, so its CPU figure is disturbed.
- **The engine is unchanged**: 175 MB, flat.



- **A machine with a GPU, and Windows (WebView2).** The rendering cost here is llvmpipe's. On a GPU, the gain in
  Run is the drop from 60 to about 30 frames per second, which is a hypothesis, not measured.
- **Complex lines.** There is one belt with at most 10 parts. A stress scene with many belts and parts belongs to M4,
  where the budget is to be measured.
- **A memory leak or its absence** over a working day: one hour shows the UI growing only while it receives snapshots,
  not whether that growth stops.
