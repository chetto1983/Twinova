# Verification: native reads through the resolved block (ROADMAP 4.3a, 2026-11-09)

Spec: `2026-11-09-component-behaviour-design.md`, "Amended after the acceptance". Plan:
`2026-11-09-field-native-reads.md`. Baseline: `275074b`, the 4.3 merge. Same machine as the 4.3 measurements:
the cloud container with 4 cores, and Xvfb with Mesa llvmpipe.

## Before and after

| Measure | Before (4.3) | After (4.3a) |
|---|---|---|
| `replay_cost 600`, release, two runs each | run 14.5–15.2 s | run 10.3–10.4 s, restore 10.2 s |
| Soak, as fast as possible | 734.7 s virtual in 45 s, **16.3×** real time | 984.2 s virtual in 45 s, **21.9×** |
| Soak, Run ×1: twinova CPU | 9.1 % | 7.0 % |
| Soak, twinova RSS, all phases | 175–177 MB | 168–169 MB |
| callgrind, `replay_cost 2`, total instructions | 1.020 G | 0.698 G |
| Native reads (4 000 in 2 s of virtual time) | 31 % of the total, as `TestHarness::get_output` | 1.27 M instructions, 0.18 % |

Everything else in the soak is unchanged: idle and paused at 0 % CPU; WebKitWebProcess at 268 % in Run ×1, bound by
the software rasterizer as in M3.

Where the time goes now (callgrind, after):

| Part | Instructions | Share |
|---|---|---|
| Rapier `PhysicsPipeline::step` | 335 M, the same as before | 48 % |
| truST cycles, field and controller | 177 M | 25 % |
| Build of the two runtimes, once | 112 M | 16 % |

On one core, the engine now runs 58× real time (600 s in 10.3 s). At 1 000 steps per virtual second, the physics
alone would cap it at about 120×.

## The results did not change

- **Tests:**
  - every engine, line and field test passes unchanged, including the generated program's text, the exact-restore
    test and the replay fingerprint test;
  - `twinova-physics`: 20/20.
- **E2E**, debug build, Linux (local): `run.mjs` 11/11, `project.mjs` 42/42. Linux and Windows on CI: see PR #27.
- **Mutants:**
  - `cargo mutants` over the `.cargo/mutants.toml` scope, in place: 0 survivors in `field.rs`;
  - it found 2 survivors in `line.rs` (`Line::output`). The Mutation run on `main` at `275074b` (42709307511) also
    found 2 more: the deleted behaviour arm of `Line::step_ms`, and `World::sinks` returning 1;
  - all four came from #26, not from this change. Two tests now kill them, and a filtered `cargo mutants --re` run
    catches all 5 matching mutants. That run proves nothing about the rest of the scope; the full run on `main` will.

## What this does not prove

- **Other lines.** Only the demo is measured. The gain grows with the number of behaving belts, because the read was
  per belt and per millisecond, but no larger line was measured.
- **Fast mode at 640× again.** The rest of the cost is physics stepped every millisecond, and the ST cycles.
  Lowering it would mean a coarser physics step, a kinematic mode, or fewer field cycles. Each of these is a
  decision about fidelity, which is not taken here.
- **Restore time.** It is still a replay from zero, linear in the saved time: about 62 s for an hour, by
  extrapolation from 600 s.
