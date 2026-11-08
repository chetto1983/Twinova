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

## One hour on the fixed build

Four 15-minute phases. The result is pending and will be added here.

## What this does not prove

- **A machine with a GPU, and Windows (WebView2).** The rendering cost here is llvmpipe's. On a GPU, the gain in
  Run is the drop from 60 to about 30 frames per second, which is a hypothesis, not measured.
- **Complex lines.** There is one belt with at most 10 parts. A stress scene with many belts and parts belongs to M4,
  where the budget is to be measured.
- **A memory leak or its absence** over hours: 4 minutes cannot tell garbage collection from a leak, hence the one-hour
  run.
