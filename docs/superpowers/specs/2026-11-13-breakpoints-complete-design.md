# Spec: breakpoints, the rest of ROADMAP 4.9

Date: 2026-11-13. Status: **proposed**, decisions 1 to 7 awaiting the user's approval.

The user, 2026-11-13: "Finiamo la 4.9". The first part (spec `2026-11-13-breakpoints-hold-design.md`, PR #48)
left out what its verification lists:
- conditions;
- hit counts;
- logpoints;
- breakpoints in blocks;
- writing while held.

This spec adds them.

## What exists (inventory and measurements, 2026-11-13)

### What the two IDEs do (research 17, §3)

- **CODESYS:**
  - per breakpoint: a condition, a hit count (equal to, a multiple of, at least N), one FB instance;
  - "execution points" that write a log without stopping;
  - writing a value while stopped.
- **TIA Portal:**
  - on S7-300/400, a breakpoint can be bound to a call environment (an instance);
  - values can be modified once while stopped.

### truST at the fork's commit `efceb12` (read in the installed source)

- **`DebugBreakpoint`** has:
  - `condition`, an `Expr`;
  - `hit_condition`, a `HitCondition` of `Equal`, `AtLeast` or `GreaterThan`. There is no "multiple of";
  - `log_message`, a list of `LogFragment`s, each `Text` or `Expr`. The messages come from `drain_logs`.
- **A logpoint never stops** (`breakpoints.rs`, the contract tests). Its values are formatted by `format_log_value`:
  - BOOL as TRUE or FALSE, and a STRING as itself;
  - anything else in Rust's debug form, such as `DInt(1)`.
- **A paused write** (`try_write_lvalue`, `enqueue_*_write`) shows in the paused snapshot at once, and reaches the
  program "at the next cycle boundary": the rest of the held scan does not see it.
- **The field (the blocks)** is a runtime of its own (`field.rs`). It compiles one generated source, where each block
  is copied unchanged and its span recorded (`generate`, `Spans`). `resolve_blocks` maps each device to its block
  instance.

### Measured (a throwaway probe in a worktree outside the repository, release build, the 4.4a line)

| What | Result |
|---|---|
| Condition `Main.count = 2` on line 41 | stops once, at 7.34 s (the second carton), on line 41 |
| Hit count `Equal(3)` on line 40 | stops at 9.34 s, the third carton; the breakpoint counts 3 hits |
| Logpoint `count={Main.count}` on line 41, 20 s | no stop, 6 messages; the values read `DInt(1)`, `DInt(2)`… |
| The same with `TO_STRING(Main.count)` | `1`, `2`… (`DINT_TO_STRING` too) |
| `TO_STRING` of a TIME (`carry.PT`) | `<error: type mismatch>`; the bare TIME reads `Time(Duration { nanos: 1500000000 })` |
| 20 s of virtual time, with the logpoint / without debug | 1.12 s / 1.05 s (+7 %, the debug control) |
| A breakpoint on a block's first statement, in the field's runtime | stops at the first field scan; the engine's mutex is held (HOLD); two frames, the field program and the block |
| Write `Main.count := 100` while held at line 40, then one step | the snapshot still reads 1 at line 41; at 9 s count is 101 (the reference 2): the write landed at the next cycle |
| The same write replayed as an engine input, no debugger | at the held step's time: count 102, another trajectory; **one step later** (5341, 5349 or 5350 ms): count 101, **fingerprint equal** to the held run (`752240475ee39f98`) |

**What these do not show:**
- the field's line mapping, worked out by hand in the probe and not through `Spans`;
- which device a block stop belongs to;
- a block shared by two devices: the 4.4a line has one chain transfer;
- the UI.

## Decisions

1. **A condition per breakpoint.**
   - It is an ST expression of the program, qualified as the monitoring's (`count = 2` reads `Main.count`).
   - A condition that does not parse is refused when it is set.
2. **A hit count per breakpoint: equal to, at least, or more than N** (truST's `HitCondition`).
   - CODESYS's "a multiple of N" is not in truST, upstream either, so it is left out.
3. **Logpoints: a message with `{expressions}`. It never stops, and its lines appear in a Log list in the PLC pane.**
   - Each expression is sent as `TO_STRING(expr)` (measured: `1`, not `DInt(1)`).
   - An expression `TO_STRING` refuses, such as a TIME (measured), is sent bare, and shows in truST's form.
4. **Breakpoints in blocks.**
   - A block's tab takes breakpoints in its glyph margin. They live in the field's runtime, at the block's line moved
     by its span in the generated source.
   - A stop there is a HOLD too (measured), and names the device whose instance it is, from the frame's instance and
     `resolve_blocks`.
   - It stops for every device that uses the block, as an unbound TIA breakpoint does. Binding it to one device (the
     selected one, as CODESYS's instance breakpoint) would mean continuing past the others on every millisecond's field
     scan, and is left out.
   - Monitoring and writing in a block stay read-only (4.8).
5. **Writing while held, for the program's variables only.**
   - The table and the code's values take a write while held.
   - The value shows at once, and reaches the program at the next cycle, as truST does (measured). The bar says so.
   - Force and Release stay refused while held.
6. **Replay of a write made while held.**
   - The write is logged as an engine input at the step after the held one (measured: equal fingerprint).
   - The debugger cannot reach the engine while held, so it queues the write. The engine takes the queue at its next
     step, into its input log only, since truST has already queued the write itself.
7. **The editing UI, as VS Code's:**
   - a right click on a breakpoint's dot, or Shift+F9 on its line, opens a bar above the code with three fields:
     condition, hit count (operator and N) and log message;
   - the dot shows what kind it is: plain, conditional, logpoint.

## Test plan

- **Engine:**
  - the condition, the hit count and the logpoint, on the 4.4a line, as measured;
  - a block breakpoint resolved through the spans, a HOLD that names `chain_transfer`;
  - a write while held, and the saved state that replays it to the same fingerprint.
- **App:** while held, `write_value` is accepted for a variable and refused for a force; logs are drained into the Log
  list.
- **Unit (vitest):** the bar's fields to the command's arguments; the dot's class for each kind.
- **E2E, in `breakpoints.mjs`:**
  - a condition (`count = 2`): the HOLD comes with the second carton;
  - a hit count of 3;
  - a logpoint: no HOLD, `count=1` and `count=2` in the Log;
  - a block breakpoint: HOLD naming the device;
  - a write of count while held: the value shows, and after Continue the count goes on from it.

## What this does not prove

- A breakpoint bound to one device.
- The hit count "a multiple of N".
- Logpoint values of every type in IEC form: a TIME shows in truST's form.
- Writing in a block, and forcing while held.
