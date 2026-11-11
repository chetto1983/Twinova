# Plan: products with attributes, and the through-beam, colour, laser and barcode sensors (ROADMAP 4.4d)

## Goal

The source makes products with a size, a mass, a colour and a code, from a seed. Four new sensors read them:
- a through-beam;
- a colour sensor that classifies;
- a laser that measures distance;
- a barcode scanner that reads the code once.

A line sorts by code and by colour, in the app and through the agent server.

## Architecture

- **Format (`twinova-project`):** two list parameter types, `DINT_LIST` and `COLOR_LIST`.
- **Physics (`twinova-physics`):**
  - each body is a `Product` with half extents, mass, colour, code and a read mark;
  - the source draws products from a seeded mulberry32;
  - every carton-size constant becomes the product's own;
  - the beams return the product they hit;
  - the scanner's read logic lives here.
  - `world.rs` splits into `world_products.rs` (source, products, draws) and `world_sensors.rs` (beams and scanners),
    to stay under 600 lines.
- **Engine (`twinova-engine`):** the line writes each sensor's signal: BOOL, DINT or REAL.
- **View:**
  - each product is a unit box scaled to its size, in its colour;
  - the four sensors' housings are copied from OIP.
- **Fixture `code_sort_line`:** sort by code (barcode → pusher) and by colour (colour sensor → turntable).

## Tech Stack

Rust 2024, Rapier 0.36.1 / parry 0.31.1, truST `6405915`, Svelte 5, three.js; OIP `1e7488c` (MIT) for the draw order,
the classification, the scanner and the housings.

## Spec

`docs/superpowers/specs/2026-11-11-sensors-products-design.md`: approved 2026-11-11, and amended while planning with
the native sensors (user), the carton's real mass, unpainted products and the lens origin.

## Global Constraints

- **Determinism:** the same seed gives the same products, bit for bit. The draws are mulberry32 on u32, mapped to f64,
  then cast once to f32.
- **Fingerprints:** they now cover each product's attributes, and the carton's mass is set, so the four values pinned
  in the physics tests ("equal bits on every platform") are pinned again, equal in dev and release, with the reason
  beside each; Windows CI checks the same bits.
- **Files:** none over 600 lines.
- **Fixtures:** written by an independent script (`scratchpad/migrate_fixtures_4_4d.py`), never by the code under test.
- **Gates:**
  - fmt, clippy;
  - coverage ≥ 85 %;
  - cargo mutants with no survivor on `world_products.rs`, `world_sensors.rs`, the classification and the codec;
  - Stryker ≥ 70 % on the TS helpers;
  - the E2E on Linux and Windows.

## Review Focus

- The draw order per product is OIP's: one draw for colour and code together (random order), then three for the size.
  Twinova has no jitter.
- Every former `CARTON_HALF` use takes the product's half extents:
  - a sink's reach;
  - a side exit's reach;
  - the source's entry probe and spawn height.
- Read-once: a product is read once, even while it stands blocked under the scanner.
- The colour classification: Chebyshev distance on 0–255; the earliest colour wins a tie; tolerance 0 is exact.

## Decisions taken while writing this plan

1. **List values.**
   - `DINT_LIST` is a JSON array of integers, each in DINT range.
   - `COLOR_LIST` is a JSON array of `#rrggbb` strings, lowercase: the format refuses `#FF0000`, so one colour has one
     spelling and the codec writes it as it is.
   - `ParamValue` gains `List(Vec<Item>)`, where `Item` is an integer or a text. `ParamValue` is no longer `Copy`, and
     `Instance::parameter` gives the override or the default, which the physics, the drive and the field shared as copies.
   - A list parameter cannot be an optional signal's fallback; the existing type check already refuses it.
2. **Source parameters:**
   - `length`, `width`, `height`: REAL, m, default 0.3, min 0.1;
   - `mass`: REAL, kg, 2.7, min 0.01;
   - `random_size`: BOOL, false;
   - `colors`: COLOR_LIST, [];
   - `codes`: DINT_LIST, [];
   - `random_order`: BOOL, false;
   - `seed`: DINT, 1.

   Random size draws L 0.4–0.8, W 0.3–0.5, H 0.3–0.5 (OIP's `SIZE_MIN`/`SIZE_MAX`, order L, H, W as OIP draws).
3. **Sensor parameters and signals:**

   | Family | Parameters | Signal |
   |---|---|---|
   | `twinova.sensor_through_beam@1` | none | `detected` BOOL |
   | `twinova.sensor_color@1` | `colors` COLOR_LIST (OIP's red, green, blue), `tolerance` DINT 80 (0–255), `range` REAL 0.5 | `color` DINT |
   | `twinova.sensor_laser@1` | `max_range` REAL 4 (0.1–10) | `distance` REAL |
   | `twinova.scanner_barcode@1` | `read_depth` REAL 0.5 | `read` BOOL, `count` DINT, `code` DINT |

   - All four mount on a conveyor's rail, like the diffuse sensor.
   - The barcode scanner only on a straight conveyor: Build refuses a curve, with its name.
4. **The pulse is 250 ms from the last read.** A read during a pulse extends it, as OIP's merged pulses.
5. **The snapshot's `Carton`** gains `size` (full extents, m) and `color` (`[r, g, b]` or null). The code stays
   physics-side: the view does not draw it.

## File structure

| File | Change |
|---|---|
| `project/src/model.rs`, `validate.rs`, `codec.rs` (+ tests) | list types and values |
| `physics/src/world.rs` | the world, slimmed |
| `physics/src/world_products.rs` | new: `Product`, the source, mulberry32 |
| `physics/src/world_sensors.rs` | new: beams, the classification, scanners |
| `physics/src/layout.rs`, `layout_sensors.rs` | the new families in the layout |
| `physics/src/world_products_tests.rs`, `world_sensors_tests.rs` | new tests |
| `engine/src/line.rs` (+ tests) | sensor signals by kind |
| `agent/src/edit.rs` | `set_parameter` takes lists |
| `app/src/lib/snapshot.ts`, `ProjectScene.svelte`, `project.ts`, `families.ts` | products and sensors drawn |
| `app/src/lib/oip/parts/{laser,colorSensor,receiver,barcodeScanner}.ts` | OIP's housings, copied |
| `project/tests/fixtures/valid/code_sort_line/` | new fixture (script) |
| `app/e2e/code_sort.mjs` | new E2E |
| docs | verification, ROADMAP 4.4d, CHANGELOG |

## Tasks

- [x] **Task 1: spec amendment and plan** (docs commit).
- [x] **Task 2: list parameter types.**
  - Tests first:
    - parse, validate and encode both types;
    - refuse a bad colour, an out-of-range DINT and a list where a scalar belongs;
    - a property test of the codec round trip for lists;
    - the agent's `set_parameter` with a list.
  - TS: `project.ts` parameter types; a list shows as its values joined.
  - Commit `feat(format): list parameters, DINT_LIST and COLOR_LIST`.
- [x] **Task 3: products with attributes.**
  - Tests first:
    - mulberry32 against OIP's reference sequence for a seed;
    - colour and code lists cycle together (item n takes colour n and code n);
    - random order uses one draw for both;
    - random size stays in range;
    - the same seed gives the same products;
    - the mass is the product's (rigid body mass within 1e-6);
    - OIP's box (0.6 × 0.4 × 0.4) and the largest random box (0.8 × 0.5 × 0.5) pass the 4.4b–c families with the probe's
      bounds: turntable, pusher, stopper, belt curve, roller curve, sinks and side exits reached by their own size.
  - Then the split and the per-product sizes. The snapshot gains `size` and `color`.
  - The source definition of every valid fixture gains the product parameters (`migrate_fixtures_4_4d.py`), defaults
    M3's carton; a source pinned without them makes the carton too.
  - Commit `feat(physics): products with size, mass, colour and code`.
- [ ] **Task 4: the four sensors in physics and the line.**
  - Tests first:
    - through-beam: in between seen, past the receiver not;
    - colour: OIP's `colorSensorBehavior.test.ts` cases;
    - laser: the distance to the face within 1 mm, and `max_range` when clear;
    - barcode: each product read once with its code, the count rising by one, merged pulses for products closer than
      0.25 s, and no double read while blocked;
    - the line writes each signal at its address.
  - Commit `feat(physics): through-beam, colour, laser and barcode sensors`.
- [ ] **Task 5: the view.**
  - Products scaled and coloured; the four housings copied from OIP with the notice; family shapes; vitest and
    Stryker.
  - Commit `feat(view): products at their size and colour, and the new sensors`.
- [ ] **Task 6: the code-and-colour sort line.**
  - Fixture by script; its program; line tests (no product lost; the fingerprint equal across run modes); the agent
    drives it; `e2e/code_sort.mjs` on Linux, Windows in CI.
  - Commit `feat(fixtures): a line sorting by code and colour`.
- [ ] **Task 7: gates and documents.**
  - Mutation and coverage; the verification file; ROADMAP 4.4d; CHANGELOG.
  - PR; merge when green.
