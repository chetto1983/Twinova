import { describe, expect, it } from "vitest";
import { SPECS } from "../../materials";
import { partsBounds } from "../../meshes";
import { createGeometryBuilder } from "../geometry/builder";
import type { Instance, MeshPart } from "./part";
import { turntableGeometry } from "./turntable";

const r = (x: number) => Math.round(x * 1e4) / 1e4;
const mm = (x: number) => Math.round(x * 1e3) / 1e3;
const turntable = (width: number, standHeight = 0.95) =>
  turntableGeometry(createGeometryBuilder(), { width, standHeight });
/** The roller tubes: rollerBed's body bank, the only roller-steel part of a stubless bed. */
const tubes = (parts: MeshPart[]) => parts.filter((p) => p.material === "roller");
/** A tube's body length: its scale over the template's rim-to-rim body, 2 × 0.9371. */
const length = (i: Instance) => mm(i.scale![2] * 2 * 0.9371);

describe("OIP's turntable conveyor", () => {
  it.each([0.6, 1.524])("spans its deck disc and its bolt ring, from the floor to the rollers' top (width %s)", (w) => {
    const all = partsBounds(turntable(w));
    // part.js: the disc's radius is width × PLATE / 2; geometry()'s bolt ring sits at deckR + 0.005 with studs 0.01
    // deep pointing out, the one at angle 0 on +x.
    const R = (w * 1.5) / 2;
    expect(all.max.x).toBeCloseTo(R + 0.01, 6);
    for (const v of [-all.min.x, all.max.z, -all.min.z]) {
      expect(v).toBeGreaterThanOrEqual(R - 1e-6);
      expect(v).toBeLessThanOrEqual(R + 0.01 + 1e-6);
    }
    // rig(): the foot plates, 0.02 thick, centred on the floor at −standHeight; the roller tops are the datum.
    expect(all.min.y).toBeCloseTo(-0.96, 6);
    expect(all.max.y).toBeCloseTo(0, 6);
  });

  it("fans OIP's rollers across the plate, tapered to its rim, their tops on the datum", () => {
    // geometry(): k from −⌊(R − 0.06) / 0.13⌋ to +⌊…⌋, each 2·min(width/2, √(rBed² − (0.13k)²)) long.
    const wide = tubes(turntable(1.524));
    expect(wide.length).toBe(1);
    const lane = Array<number>(11).fill(1.524);
    expect(wide[0].instances!.map(length)).toEqual([0.604, 1.174, 1.503, ...lane, 1.503, 1.174, 0.604]);
    expect(wide[0].instances!.map((i) => r(i.position[0] / 0.13))).toEqual([
      -8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8,
    ]);
    expect(wide[0].instances![0].position[1]).toBe(-0.04);
    expect(partsBounds(wide).max.y).toBeCloseTo(0, 6);
    // At 0.6 m, rBed / 0.13 is 2.9999… in floating point, so OIP lays five rollers, not seven.
    const narrow = tubes(turntable(0.6))[0].instances!;
    expect(narrow.map(length)).toEqual([0.581, 0.6, 0.6, 0.6, 0.581]);
  });

  it("turns the deck, its bed and its gearmotors' moving halves; the stand and the slew motor stay put", () => {
    const parts = turntable(1.524);
    expect(parts.filter((p) => p.material === "leg").every((p) => p.link === undefined)).toBe(true);
    for (const m of ["roller", "bolt", "frame", "turntableGear", "turntableNameplate"])
      expect(parts.filter((p) => p.material === m).every((p) => p.link === "moving")).toBe(true);
    // rig.json: the static part reaches the foot plates (the leg ring's 0.62 at 45°, plus half of 0.26) and the slew
    // gearbox (0.1 wide at x 0.541), up to the poles' tops at −0.1; the moving group hangs to the ring gear's web, 0.027
    // tall at −0.5125.
    const fixed = partsBounds(parts.filter((p) => !p.link));
    expect(r(fixed.max.z)).toBe(r(0.62 * Math.SQRT1_2 + 0.13));
    expect(r(fixed.max.x)).toBe(0.591);
    expect(r(fixed.max.y)).toBe(-0.1);
    expect(r(partsBounds(parts.filter((p) => p.link)).min.y)).toBe(-0.526);
  });

  it.each([0.6, 1.524])("keeps the side gearmotor in its pocket, 5 mm under the rollers' top (width %s)", (w) => {
    const motor = turntable(w).filter(
      (p) => p.link && p.material.startsWith("turntable") && p.material !== "turntableGear",
    );
    expect(partsBounds(motor).max.y).toBeCloseTo(-0.005, 6);
  });

  it("draws in materials Twinova defines", () => {
    for (const p of turntable(1.524)) expect(SPECS[p.material]).toBeDefined();
  });
});
