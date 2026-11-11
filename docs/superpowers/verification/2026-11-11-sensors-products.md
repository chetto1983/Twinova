# Verification: products with attributes, and the through-beam, colour, laser and barcode sensors (ROADMAP 4.4d, 2026-11-11)

Spec: `2026-11-11-sensors-products-design.md` (approved, amended while planning). Plan: `2026-11-11-sensors-products.md`.
Machine: the cloud container with 4 cores. CI: pull request #38.

## What runs

- **Format:** two list parameter types, `DINT_LIST` and `COLOR_LIST` (lowercase `#rrggbb`).
  - An error names the item.
  - A list never reaches an ST block.
- **Products:** a source makes each product with a size, a mass, a colour and a code.
  - The colour and code lists are walked in step, or drawn at random.
  - The size is the one set, or drawn from OIP's range.
  - Everything comes from a seed: OIP's mulberry32 and its draw order, without its spawn jitter.
  - A source pinned before 4.4d makes M3's carton.
- **Sensors,** native like the diffuse one (user, 2026-11-11):
  - `twinova.sensor_through_beam@1`: `detected`;
  - `twinova.sensor_color@1`: `color`, a DINT index, 0 for none;
  - `twinova.sensor_laser@1`: `distance` in metres, `max_range` when clear;
  - `twinova.scanner_barcode@1`: `read`, a 250 ms pulse; `count`; `code`. It reads each product once.
- **View:** products at their size and colour; the four housings copied from OIP.
- **Fixture `code_sort_line`:** red, green and blue cartons with codes 1, 2 and 3.
  - The colour sensor tells the turntable to turn green.
  - The scanner tells the pusher to reject code 3.

## The acceptance

| # | Criterion | Evidence | Result |
|---|---|---|---|
| 1 | OIP's box (0.6 × 0.4 × 0.4 m) and the random range's largest (0.8 × 0.5 × 0.5 m), 10 kg, through every family | `physics/src/world_sizes_tests.rs`, each fed by a source, see the table below | ✅ |
| 2 | Seeded sources: the same seed, the same products, bit for bit; the lists cycle in OIP's order | `world_products_tests.rs`: mulberry32 against OIP's `rng.ts` run in Node (four seeds, five values each); walked lists in step; one draw for colour and code; size after colour as length, height, width; a product waiting for the entry keeps its draw; equal fingerprints for a seed, different for another | ✅ |
| 3 | Through-beam: seen between the heads, not past the receiver | `world_sensors_tests.rs`: a carton breaks the beam at 2 m for 0.6 s at 0.5 m/s; a carton 10 mm inside the receiver's lens is seen, one past it is not | ✅ |
| 4 | Colour: OIP's classification cases | white, grey and yellow read 0; near red reads 1; an empty list reads 0; tolerance 0 is exact; the bound is inclusive; the nearer of two wins, the earlier on a tie; an unpainted product reads 0; on the belt: green, none, red in turn | ✅ |
| 5 | Laser: the distance to the face within 1 mm, `max_range` when clear | 0.17 m to a carton on the centreline, 0.27 m to one 0.1 m further, within 1 mm and 2 mm; 4.0 when clear | ✅ |
| 6 | Barcode: read once with its code, the count by one; reads closer than 0.25 s give one long pulse; a blocked line does not read twice | three products read in turn as 41, 42 and 43; the pulse lasts 250 ms and the code holds; a product held 3 s under the arch counts once; two products read about 0.1 s apart give one pulse from the first read to 250 ms after the second, and a count of 2 | ✅ |
| 7 | A line sorting by code and colour in the app on Linux and Windows; no product lost; the fingerprint equal across run modes | `engine/src/line_sensors_tests.rs`: 90 s, conservation every millisecond; red, green and blue each delivered to its own sink or still on the line, every route taken at least 3 times. Step, ×1 and as fast as possible reach the same snapshot and fingerprint. `e2e/code_sort.mjs` on Linux (here) and windows-2022 (CI): each sink counts, and the scanner's count and code and the laser's distance read in the I/O panel | ✅ Linux 4/4 checks (here, debug build under Xvfb); windows-2022 on the PR |
| 8 | The agent server drives it; the gates are 4.4c's | `agent/tests/code_sort.rs`: the agent runs the line over HTTP until the reject sink counts, reads the scanner's code 3, and sees each carton's size and colour | ✅ |

### Acceptance 1, measured (2026-11-11, dev and release give the same bits)

| Family | OIP's box | The largest | Bound in the test |
|---|---|---|---|
| Belt curve (grip 0.7) between belts, 0.5 m/s: yaw 1.5 m along the exit belt | 2.09° | 2.44° | 3° |
| Roller curve between roller conveyors (0.42) | 2.26° | 2.31° | 3° |
| Roller curve fed and emptied by belts | 5.43° | 6.97° | 7.5° (physical, as in the spec) |
| Off the centreline after any curve | ≤ 3.3 mm | ≤ 5.6 mm | 10 mm |
| Turntable 1.0 m, 90° at 20°/s: off its carried place, yaw | 0.1 mm, 0.06° | 0.1 mm, 0.05° | 2 mm, 0.5° |
| Stopper: a queue of three, front against the blade, worst tilt | 0.49° | 0.36° | 2 mm, 2° |
| Pusher onto a takeaway: worst tilt, then yaw | 0.21°, −0.07° | 0.27°, 0.01° | 5°, 3° |
| Chain transfer into its side sink: worst tilt | 0.04° | 0.04° | 3° |

Every product reached its sink.

- **Probe vs tests:** the spec's probe placed boxes by hand and found up to 3.1° on a roller curve between rollers. Fed
  by a source, the same curve turns them within 2.3°, so the test holds 4.4b's 3°.

The screenshot after the run shows red, green and blue cartons on belt In and roller A, a red one on the turntable,
the scanner's arch over belt B and the I/O panel with the new signals.

## Found by measuring

- **The carton weighed 1.76 kg, not 2.7 kg.**
  - parry 0.31.1 gives a round cuboid the mass properties of its inner cuboid (`shape/shape.rs`,
    `impl_shape_for_round_shape`), and the carton's core is 0.26 m: 0.26³ × 100 kg/m³ = 1.76 kg.
  - Each product's mass is now set (`ColliderBuilder::mass`), 2.7 kg by default.
  - Every test of the transport passed unchanged, as the spec's probe found (mass does not change the transport).
  - The four fingerprints pinned in the physics tests are pinned again, for this and for the attributes the fingerprint
    now covers; windows-2022 checks the same bits.
- **A colour read at the moment a carton reaches the stopper's photocell needs its beam behind that photocell.** The
  fixture's colour sensor stands at 1.4 m, before the photocell at 1.45 m. At 1.5 m it read the carton before the
  carton reached it, and every carton went straight.
- **The test helpers of the agent's end-to-end tests were duplicated** in two files; a third would have copied them
  again. They are in `agent/tests/common/mod.rs` now.

## Mutation and coverage

- **cargo mutants:**
  - The diff's 189 mutants were started in place here.
  - The engine's and the agent's mutants take 10 s each, and the run was stopped after 34: 23 caught, 10 unviable, no
    survivor. The one timeout was a test waiting without a bound for a product the mutant removed; those waits are now
    bounded.
  - The gate is CI's in-diff run on the pull request (three shards, no survivor allowed).
  - CI's first run found three survivors, each now killed by a test and checked locally:
    - OIP's rounding share below 2 cm (a 6 cm product);
    - the scanner's read window (a long product read when its centre enters it);
    - `Instance::parameter`: only other crates' tests exercised it, and cargo-mutants runs only the mutated crate's
      tests.
- **Stryker:** 90.0 % over `src/lib`. The one survivor in the new code was the diffuse sensor's `case`; it is killed by
  a test of its paint.
- **Coverage:** frontend 99.0 % of lines. Rust: CI's llvm-cov gate (≥ 85 %) passes on the pull request.

## What this does not prove

- **Pallets, totes, unit loads and the light curtain:** out of 4.4d.
- **Products that are not boxes:** the product is a rounded box; CAD products are ROADMAP 9.1.
- **A colour read under lighting:** the colour is the product's, as in OIP.
- **A barcode's orientation:** the scanner reads any face, as OIP's does.
- **The scanner's lamps:** the scan window and the good-read beacon are drawn, not lit.
- **A through-beam receiver of its own:** the receiver is the emitter's housing turned round, as the spec's notes
  planned; OIP has no through-beam part.
