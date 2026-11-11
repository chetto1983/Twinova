import { describe, expect, it } from "vitest";
import { SPECS } from "../../materials";
import { partsBounds } from "../../meshes";
import { createGeometryBuilder } from "../geometry/builder";
import type { MeshPart } from "./part";
import { stopperGeometry } from "./stopper";

// OIP's examples/parts/blade-stop/part.js at 1e7488c: native units scale by S; the blade centreline (native x 0.1036)
// lands on x = 0; the blade's native top 0.2494 rests at −0.05 and the housing's at −0.12.
const S = 0.45;
const X = (x: number) => (x - 0.1036) * S;
const BLADE_Y = -0.05 - 0.2494 * S;
const BASE_Y = -0.12 - 0.2494 * S;
const cornerZ = (width: number) => width / 2 - 0.2 * S;
const defaults = { width: 1.524, raiseHeight: 0.04 };
const r = (x: number) => Math.round(x * 1e6) / 1e6;
const draw = (p = defaults) => stopperGeometry(createGeometryBuilder(), p);
const extents = (parts: readonly MeshPart[]) => {
  const box = partsBounds(parts);
  return { x: [r(box.min.x), r(box.max.x)], y: [r(box.min.y), r(box.max.y)], z: [r(box.min.z), r(box.max.z)] };
};
/** The static cylinder blocks' lowest point at a raise height. */
const blocks = (raiseHeight: number) =>
  extents(draw({ ...defaults, raiseHeight }).filter((p) => p.material === "beltBlack")).y[0];
const moving = (parts: readonly MeshPart[]) => parts.filter((p) => p.link === "moving");

describe("OIP's blade stop", () => {
  it("draws the housing static and the blade's assembly on the moving link", () => {
    expect(draw().map((p) => [p.material, p.link])).toEqual([
      ["darkSteel", undefined],
      ["plate", undefined],
      ["beltBlack", undefined],
      ["bladeOrange", "moving"],
      ["darkSteel", "moving"],
      ["plate", "moving"],
    ]);
  });

  it("fits OIP's manifest bounds, its housing ends at ±width/2", () => {
    const all = extents(draw());
    // rig.json bounds: length 0.2, bottom −0.5295 (the slab's native −0.6606 under BASE_Y). The housing runs native
    // x 0..0.4, 0.18 m, on the +x side of the blade: OIP's footprint, centred on the origin (no centerX), leaves its
    // last 33 mm out. Its end plates reach native z ±1.2, which is cornerZ + 0.2·S = width/2.
    expect(all.x).toEqual([r(X(0)), r(X(0.4))]);
    expect(all.x[1] - all.x[0]).toBeLessThanOrEqual(0.2);
    expect(all.y[0]).toBe(-0.5295);
    expect(all.z).toEqual([-0.762, 0.762]);
    // At rest the top is the moving end posts' (native 0.2462, 3.2 native mm under the blade's 0.2494 reference); the
    // stroke at OIP's raiseHeight cap (0.5) keeps it under the manifest's top, 0.5.
    expect(all.y[1]).toBe(r(BLADE_Y + 0.2462 * S));
    expect(all.y[1] + 0.5 + 0.05).toBeLessThanOrEqual(0.5);
  });

  it("draws the blade lowered: its top at −0.05 less the model's 1.5 mm, 22.6 mm thick, spanning to the corner nodes", () => {
    const parts = draw();
    const blade = extents(parts.filter((p) => p.material === "bladeOrange"));
    expect(blade.x).toEqual([r(X(0.0786)), r(X(0.1288))]);
    expect(r(blade.x[1] - blade.x[0])).toBe(r(0.0502 * S));
    expect(blade.y).toEqual([r(BLADE_Y), r(-0.05 - (0.2494 - 0.2461) * S)]);
    expect(blade.z).toEqual([r(-(cornerZ(1.524) + 0.0843 * S)), r(cornerZ(1.524) + 0.0843 * S)]);
    // the whole moving assembly: posts and clamps across, the guide rods' feet below, the bolt heads outermost
    const all = extents(moving(parts));
    expect(all.x).toEqual(blade.x);
    expect(all.y).toEqual([r(BLADE_Y - 0.3271 * S), r(BLADE_Y + 0.2462 * S)]);
    expect(all.z).toEqual([r(-(cornerZ(1.524) + 0.1275 * S)), r(cornerZ(1.524) + 0.1275 * S)]);
  });

  it("moves the ends to ±width/2 as the width changes, the corner hardware rigid", () => {
    for (const width of [0.57, 1.524]) {
      const parts = draw({ ...defaults, width });
      expect(extents(parts).z).toEqual([r(-width / 2), r(width / 2)]);
      expect(extents(moving(parts)).z[1]).toBe(r(width / 2 - 0.2 * S + 0.1275 * S));
    }
  });

  it("lengthens the cylinders down for a long stroke, stopping 1 cm above the floor slab", () => {
    const airY = BASE_Y + 0.07;
    expect(blocks(0.04)).toBe(r(airY - 0.3385 * S));
    expect(blocks(0.06)).toBe(blocks(0.04));
    expect(blocks(0.5)).toBe(r(BASE_Y - 0.6124 * S + 0.01));
  });

  it("draws in materials the view knows", () => {
    for (const part of draw()) expect(SPECS[part.material]).toBeDefined();
  });
});
