# Verification: breakpoints, the rest of ROADMAP 4.9

Date: 2026-11-13. Spec `2026-11-13-breakpoints-complete-design.md`, plan `2026-11-13-breakpoints-complete.md`.

## What was measured

- **Before the spec**, with a throwaway probe on the 4.4a line, release (spec, "What exists"):
  - a condition held at the second carton;
  - a hit count held at the third;
  - a logpoint logged without holding, its numbers as `DInt(1)` unless sent through `TO_STRING`;
  - a block's breakpoint held the engine;
  - a write while held landed at the next cycle, and replayed one millisecond later to the same fingerprint.
- **Engine:** `cargo test -p twinova-engine debugger`, 16/16; the engine suite, 125/125.
  - A condition `count = 2`: names resolve in the stopped frame, so no qualification is needed.
  - A hit count of 3, and each operator mapped to truST's.
  - A logpoint `count={count}, PT={carry.PT}` logs `count=1, PT=Time(Duration { nanos: 1500000000 })`: the number
    through `TO_STRING`, the TIME bare.
  - A message's parsing, and a condition or a message that does not parse is refused.
  - A breakpoint on line 20 of the chain transfer's block, through the field's spans: a HOLD that names
    `chain_transfer`, and a write there refused.
  - A write of count while held: shown at once; the held scan still reads 1; 101 at 9 s; `save` then `restore` gives
    the same fingerprint.
- **App:** `cargo test -p twinova`, 79/79.
  - A block's file goes to the field's debugger, whose HOLD holds the engine.
  - The logpoints' lines are drained.
  - While held, `write_value` is taken and a force is refused.
- **Unit:** `debug.test.ts`, `breakpoints.test.ts`, `backend.test.ts` and `snapshot.test.ts`, 30/30.
- **E2E on Linux,** `e2e/breakpoints.mjs`, 15/15. The five checks added:
  - a condition `count = 2` on line 41 holds with count 2 drawn;
  - a hit count of 3 holds with count 3;
  - a logpoint never holds, and the Log lists `count=1` and `count=2`;
  - a block's breakpoint holds with "HOLD at components/twinova.chain_transfer@1.st:20 (chain_transfer)";
  - count written to 100 while held at line 40: the bar says it lands at the next cycle, and the next stop at line 40
    shows 100.

  Regressions: `monitor` 7/7, `plc_window` 6/6, `editor_st` 6/6, `editor` 10/10, `workspace` 9/9.

## Measurements that changed the design

- **Whether a log expression reads with `TO_STRING` is decided at log time** (plan, decision 1).
  - Trying it when the breakpoint is set failed: `count` resolves only in the stopped frame, and a block has no
    context there.
  - Each expression is logged through `TO_STRING` and bare, between control characters. The line keeps the first unless
    it is an error.
- **The first Log list covered the glyph margin.** The E2E's click on a block's line 20 hit the list instead of the
  margin. The list now sits under the code (`LogList.svelte`), with Clear.
- **vitest reports a rejected promise a `vi.fn` returned as unhandled,** though the code catches it. The test of the
  breakpoint state stubs the backend by hand.

- **On Windows a breakpoint just set or removed could reach the app after Run or Continue** (CI's windows-2022 E2E,
  on `main` after PR #48 and on this PR's `ca12379`: HOLD at the old line 40, and a HOLD after Continue with the
  breakpoint off).
  - Each `invoke` is a fetch of its own (Tauri 2.12.5, `scripts/ipc-protocol.js`), so two not awaited have no order.
  - Run, Step, Continue and Run to cursor now wait for the breakpoints set before them (`backend.ts`). The vitest that
    checks it fails without the change.
  - Linux never showed the race locally, so its absence on Windows rests on the next CI run.

- **truST's debug, once on, costs even with no breakpoint, and truST cannot turn it off** (`enable_debug`, no disable
  at `efceb12`).
  - The windows-2022 E2E then failed: Max ran at 0.14× to 0.30×, too slow to hold the line within 10 s.
  - Measured in a throwaway worktree, release, 20 s of virtual time on the 4.4a line:

    | Debug | Wall time |
    |---|---|
    | none | 0.84 s |
    | the PLC's, no breakpoint | 1.10 s |
    | the field's, no breakpoint | 6.53 s (7.7×) |

  - Taking a block's breakpoint off after Reset attached the field's debugger only to clear an empty list. It no longer
    does: an empty list attaches no debugger (`set_breakpoints`).
  - Linux E2E, the never-hit check at Max for 4 s: 8.3 s of virtual time before, 49.8 s after.
  - Then the field's debug goes off with its last breakpoint (spec decision 8): the fork's `disable_debug`
    (`1e249be`), called when no block has a breakpoint and the line is not held. App tests: taken off, the field's
    debugger is dropped; taken off while held, at Continue; the PLC's stays. `breakpoints.mjs` now takes the block's
    breakpoint off while held and continues; the checks after it at Max: 50 s of virtual time in 4 s.

## What this does not prove

- **A breakpoint bound to one device** (decision 4), and the hit count "a multiple of N" (decision 2).
- **Logpoint values in IEC form for every type:** a TIME shows in truST's form.
- **Writing in a block, and forcing while held.**
- **The PLC's debug off.** Once attached, by a breakpoint, a write or a force, it stays on to the next Build or Reset:
  30 % at no breakpoint.
- **The fork's `disable_debug` test,** until the fork's CI runs it.
- **Windows,** until CI's windows-2022 E2E.
