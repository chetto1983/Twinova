# Verification: gap control on the 4.4a line (2026-11-13)

- **Spec:** `2026-11-10-diverter-line-design.md`, amendment 2026-11-13.
- **Why:** the Linux E2E `line.mjs` ("both sinks count, two to one") failed on `main` in run 43283896219 (239 cartons
  to `sink_ok`, 471 to reject), and again on pull request #43.
- **Machine:** the cloud container with 4 cores, Linux.

## The cause, measured on the engine alone

- **Deterministic.** The same timeline in 5 separate processes, debug and release, with fingerprint
  `401bceb9e82902f9`. A warm restart at any time from 0.5 to 30 s left the ratio at 1:2.
- **Two in three rejected for good.**
  - Each raise holds belt A for about 1.6 s while the source keeps its 2 s interval, so cartons bunch up.
  - From 31 s they reach `sensor_a` about 1.2 s apart. When the third one raises the chains, the carton before it is
    still on the deck and is carried out too: two rejects per raise (31.65 and 32.12 s).
  - At 200 s: 34 to `sink_ok`, 60 to reject.
- **Why the E2E sometimes passed.** It read the counts at Max speed, and passed only when a read fell between about 28
  and 38 s of virtual time.

## The change

- **The line:**
  - `sensor_in`, a diffuse photocell on belt A at 2.95 m (`%IX0.6`);
  - the roller conveyor at 0.8 m/s.
- **The program:**
  - it counts at `sensor_in`;
  - it holds belt A while the third carton is there, until the zone between the photocells is empty and `sensor_a`
    has been clear for 0.8 s;
  - it raises on that carton's arrival at `sensor_a`;
  - the zone's count is RETAIN, so an online change keeps it.

Each part was measured before it was kept:

| Variant | Result |
|---|---|
| `sensor_in` at 2.7 m | the line stopped at 3 / 1: a carton past the photocell was still on the held belt A |
| `F_TRIG` for the zone's count | an extra carton from the first scan (IEC: `F_TRIG` on a FALSE input fires once), so the zone never emptied |
| `sensor_in` at 2.95 m, rollers at 0.5 m/s | stopped at 10 / 4: cartons held on belt A queue touching, and `sensor_a` saw two as one (covered 1.2 s) |
| rollers at 0.8 m/s | 170 / 86 at 600 s, 259 counted (86 = ⌊259 / 3⌋), 7 to 9 cartons on the line |
| zone count not RETAIN | an online change at 8 s left the line stopped: the count restarted at 0 with cartons in the zone |
| zone count RETAIN | after that change, the second carton counted is rejected under MOD 2 at scan 1,443 |

## The tests

- **New: `engine_tests.rs`, `the_line_with_its_chain_transfer_rejects_one_carton_in_three_as_they_bunch_up`.**
  - At 200 s: 54 to `sink_ok` and 27 to reject;
  - the rejects are every third carton counted, the last one possibly still on its way;
  - at most 9 cartons are on the line.
  - The first program fails it (34 / 60).
- **Changed, because the fixture changed:**
  - **`project_commands_tests.rs`:** 10 instances, and `sensor_in`'s mount on belt A.
  - **`engine_change_tests.rs`, the online change:**
    - `count` is 2 at 8 s, since cartons are now counted at belt A's end;
    - under MOD 2 the reject comes at scans 1,400–1,500, without the change at 1,200–1,300 (measured 1,443 and
      1,243).
  - **Comments with the new times:** `line_tests.rs`, the lift test in `engine_tests.rs`, and `line.mjs`.
- **`e2e/snapping.mjs`:** it now places the infeed photocell at belt A's end, mounted at 2.95 m, and sets the roller
  conveyor's speed. 11/11.
- **The gates:**
  - `cargo test --workspace` green;
  - `cargo fmt` and `cargo clippy -D warnings` clean;
  - vitest 195;
  - the whole E2E suite on Linux under CI's screen: 107/107, `line.mjs` 4/4.
  - No production Rust or TypeScript changed, so there is no new mutation scope.

## What this does not prove

- **Windows,** until CI's windows-2022 E2E on the pull request.
- **Any spacing.** Only the fixture's source interval (2 s) was measured; another interval may need other times.
