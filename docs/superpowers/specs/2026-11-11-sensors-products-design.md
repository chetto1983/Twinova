# Spec: products with attributes, and the through-beam, colour and barcode sensors (ROADMAP 4.4d)

Date: 2026-11-11. Status: **approved** (user, 2026-11-11), with OIP's laser sensor added to the scope, and amended
before the plan (below, "Amended while planning").

4.4a split ROADMAP 4.4 into sub-phases. 4.4d is the through-beam, colour and barcode sensors, with product attributes.
The user asked to go in order, after the agent server (2.7). The method is 4.4a–c's:
- OIP's geometry copied;
- its physics translated into Rust;
- its `step()` re-expressed in ST;
- its Jolt workarounds kept only where Rapier needs them.

## What OIP does (read at `1e7488c`; notes in the scratchpad, `notes-4.4d.md`)

**Products** (`box-spawner/behavior.ts`, `src/sim/cargoRegistry.ts`, `bodyFactory.ts`):
- **Size:** a box 0.6 × 0.4 × 0.4 m by default (each side at least 0.1). Its edges are rounded by
  `min(0.02, 0.4 × smallest half)`, and its friction is 0.5.
- **Mass:** an explicit mass, 10 kg by default, the same whatever the size.
- **Random size:** `randomSize` draws L 0.4–0.8 m and W, H 0.3–0.5 m, ignoring the set size.
- **Colour:** each box has an sRGB colour (white by default) from `boxColor`, or from the list `boxColors`, walked in
  order or drawn at random.
- **Code:** each box carries `marks`, a map of name to number, bool or text.
  - The spawner sets one mark, `mixAttribute`, from `mixValues`; item n gets colour n and value n.
  - Machines add marks during the run (`scanned`).
- **Randomness:** mulberry32, seeded by a hash of the spawner's parameters. Each item draws, in order, its jitter, its
  colour/value, then its size.

**Through-beam:** OIP has none.
- Its photo eye has a `retro` mode, which only draws a reflector. Its beam is the same ray as the diffuse eye's, and
  hits only cargo within `maxRange`.
- A through-beam is the same physics with the range fixed to the gap between its two heads: a receiver housing instead
  of the reflector.

**Colour sensor** (`color-sensor/part.js`):
- **Housing:** the photo eye's, in other materials.
- **Parameters:** a list of colours (default red, green, blue) and a `tolerance` (default 80 on 0–255).
- **Output:** an INT, the 1-based index of the nearest listed colour within tolerance, or 0 for none, or with nothing
  in range.
- **Match:** the Chebyshev distance per channel (the largest channel difference). The earliest colour wins a tie.

**Barcode scanner** (`barcode-scanner/behavior.ts`):
- **Shape:** an arch over the conveyor, placed by hand: width 1.0, clearance 0.6 and read depth 0.5 m. A scan line runs
  across the conveyor 5 cm above the surface.
- **Read:** an unread box whose centre is in the read zone and which crosses the line is read once and marked
  `scanned`.
- **Outputs:**
  - `read`: a BOOL pulse of 0.25 s;
  - `count`: a DINT;
  - `code`: a DINT, the box's code mark — a number, a bool as 1/0, a text of digits, else 0.

## Measured in Rapier (2026-11-11, throwaway worktree, release)

The setup is the 4.4c world. Boxes are placed by hand: the 0.3 m carton (2.7 kg), OIP's 0.6 × 0.4 × 0.4 box at 10 and
2.7 kg, and the random range's largest, 0.8 × 0.5 × 0.5 at 10 kg.

| Where | 0.3 m carton | 0.6 m box | 0.8 m box |
|---|---|---|---|
| Turntable 90°, 0.6 and 1.0 m lanes | 0.0 mm, 0.00° | 0.1 mm, 0.00° | 0.1 mm, 0.00° |
| Pusher onto a takeaway, 0.25 / 0.5 s | upright, max 2.6° | upright, max 0.3° | upright, max 0.8° |
| Stopper, queue of three, raise 0.04 / 0.15 | max tilt 1.7° | max 0.5° | max 0.3° |
| Belt curve (grip 0.7), 0.5 m/s | 1.1°, 1 mm | 2.0–2.1°, 2 mm | 2.4°, 2 mm |
| Roller curve with roller conveyors before and after | 1.2° | 2.1–2.7° | 2.2–3.1° |
| Roller curve fed and emptied by **belts** | 2.8° | **5.3–5.4°** | **6.6°** |
| Guarded belt curve at 1 m/s | 4 mm off | 4 mm off | against the guard, 52 mm off, square |

- **Mass does not matter:** 10 against 2.7 kg changes nothing beyond 0.2°.
- **Solver iterations do not help:** one iteration gives the smallest yaw, 4 the largest (2.3 → 2.7° for the 0.6 m box).
  4.4b2's choice holds.
- **The 5–7° is the belt-to-roller-curve handover.** A long box straddles the grip 0.7 belt and the grip 0.42 roller
  curve, one end pushed straight and the other turned. It is physical; 4.4b's 3° acceptance holds for a carton, or for a
  roller curve between roller conveyors.
- The probe's chain-transfer cases had no side exit, so the box fell off. They show nothing about size, and Task 1
  measures them again with a sink.

## Decisions of the user (2026-11-11)

1. **Scope.**
   - In: products with size, mass, colour and code, made by the source; the through-beam, colour and barcode sensors,
     and OIP's laser distance sensor.
   - Out: pallets, totes and unit loads (compound colliders, OIP's heavy-solver override to measure in Rapier), and the
     light curtain.
2. **Colour to the PLC as OIP does:** the sensor classifies and writes a DINT index; the PLC never sees RGB.
3. **Randomness:** an explicit `seed` parameter on the source, not a hash of its parameters, which would change when the
   format serialises differently. The draw order per product is OIP's.
4. **The default product stays the 0.3 m, 2.7 kg carton.** Existing projects and fingerprints are unchanged; OIP's
   box is the source's parameters away.
5. **The barcode code is a DINT,** as OIP's; no STRING on the IEC image.

## The families (proposed)

- **`twinova.source@1` gains parameters:**
  - size, as length, width and height;
  - mass;
  - `random_size`, with OIP's range;
  - `colors`, a list of `#rrggbb`;
  - `codes`, a list of DINT;
  - `random_order`;
  - `seed`.

  The defaults reproduce today's carton, white and with no code. **The format needs a list-of-colours and a
  list-of-DINT parameter type**, a format addition.
- **`twinova.sensor_through_beam@1`:**
  - mounted like the diffuse sensor;
  - its range is the host's width plus the heads' clearance;
  - `detected` BOOL, as the diffuse one;
  - view: OIP's photo-eye housing, plus a receiver composed from it.
- **`twinova.sensor_color@1`:**
  - mounted like the diffuse sensor;
  - `colors` and `tolerance` parameters;
  - `color` DINT out (0 none);
  - the classification native (amended above).
- **`twinova.sensor_laser@1`** (OIP `laser-sensor/part.js`):
  - mounted like the diffuse sensor;
  - `max_range` 4 m;
  - `distance` REAL out, in metres to the nearest product, reading `max_range` when clear, as OIP's;
  - no scaling to an analog integer: the PLC scales, as it would for an analog card;
  - view: OIP's laser housing, copied.
- **`twinova.scanner_barcode@1`:**
  - an arch mounted on a conveyor at a position, spanning its width (Twinova snaps it; OIP places it by hand);
  - `read` BOOL pulse, `count` DINT and `code` DINT;
  - read-once is a mark on the product, as OIP's;
  - all native (amended above).
- **The physics:** each product carries its half extents, mass, colour and marks.
  - Every place that uses `CARTON_HALF` (sinks' reach, side exits, the source's entry probe, the beam heights' checks)
    takes the product's own size.
  - The beam returns the product it hits, with its distance.
  - The fingerprint covers the attributes.
- **The view:** each product drawn at its size in its colour (a unit box scaled per product), and the three sensors'
  generators copied.

## Amended while planning (2026-11-11)

1. **The four sensors are native, like the diffuse one (user, 2026-11-11).** The colour classification, the scanner's
   pulse, count and code are Rust, not ST.
   - The field has one direction only: the native model reads block variables (`surface_speed`, `lift`), and nothing
     writes a block's inputs every millisecond.
   - A list of colours in ST would be an ARRAY parameter, which the field does not pass.
   - `twinova.sensor_diffuse@1` is native, without a block, so the new sensors follow it, and the mutation gate covers
     their logic.
2. **The carton weighs 1.76 kg in Rapier, not 2.7 kg** (read in parry 0.31.1, `shape/shape.rs`,
   `impl_shape_for_round_shape`). A round cuboid's mass properties are its inner cuboid's, and the carton's core is
   0.26 m: 0.26³ × 100 kg/m³. 4.4d gives each product its mass explicitly, 2.7 kg by default as specified. The
   fingerprints of existing scenes change; the measurements above show the mass does not change the transport.
3. **A product with no colour** (the source's `colors` empty) is unpainted: the view draws it in the theme's product
   colour, and a colour sensor reads 0, as OIP's white box does against its default colours.
4. **The new sensors' beams start at the lens,** 20 mm out from the bed edge, as OIP's snap seats the eye. The laser
   measures from there, and the through-beam's range is the host's width plus 40 mm. The diffuse sensor keeps its beam
   from the bed edge, unchanged.

## Acceptance

1. **Sizes and masses:** OIP's box and the random range's largest run through every family as measured above. The
   4.4b–c criteria hold, and the belt-to-roller-curve handover is recorded as above.
2. **Seeded sources:** the same seed gives the same products, bit for bit and on every platform; colour and code lists
   cycle in OIP's order.
3. **Through-beam:** a product between the heads is seen; one past the receiver is not.
4. **Colour:** OIP's classification cases (`colorSensorBehavior.test.ts`): white, grey and yellow read 0, near-red reads
   1, an empty list reads 0, tolerance 0 is exact.
5. **Laser:** the distance to the nearest product's face within 1 mm, and `max_range` with nothing in range.
6. **Barcode:** each product is read once, with its code, the count rising by one. Products closer than 0.25 s give one
   long pulse and a count that tells them apart (OIP's own caveat, now specified). A blocked line does not double-read.
7. **A sorting line by code and colour,** in the app on Linux and Windows: the barcode decides the pusher, the colour
   the turntable. No product is lost, and the fingerprint is equal across run modes and platforms.
8. **The agent server drives it,** and the gates are 4.4c's.

## What this will not prove

- Pallets, totes, unit loads.
- Products that are not boxes.
- A colour read under lighting: the colour is the product's, as in OIP.
- A barcode's orientation: OIP reads any face.
