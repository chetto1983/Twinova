import { describe, expect, it } from "vitest";
import { SPECS } from "../../materials";
import { partsBounds } from "../../meshes";
import { createGeometryBuilder } from "../geometry/builder";
import { pusherGeometry, pusherLamps } from "./pusher";

const pusher = (stroke: number, standHeight: number) =>
  pusherGeometry(createGeometryBuilder(), { stroke, standHeight });

/** OIP's footprint for the diverter (examples/parts/diverter/rig.json `geometry.bounds` at 1e7488c): length 0.73
 *  across, width 0.602 + stroke along Z about centerZ −0.802 + stroke/2 (the stroke held to 0.1..1.5), from the floor
 *  up to the hood's top at 0.66. */
function footprint(stroke: number, standHeight: number) {
  const S = Math.min(1.5, Math.max(0.1, stroke));
  const width = S + 0.602,
    centerZ = 0.5 * S - 0.802;
  return { halfLength: 0.73 / 2, z0: centerZ - width / 2, z1: centerZ + width / 2, top: 0.66, bottom: -standHeight };
}

// partsBounds reads the Float32 buffers three draws from: 6 decimals, not 9
describe("OIP's pusher", () => {
  it.each([
    [0.75, 0.3847],
    [1.2, 0.9],
    [0.05, 0.3847],
    [2, 1.5],
  ])("fills OIP's footprint at stroke %f, standing %f above the floor", (stroke, standHeight) => {
    const want = footprint(stroke, standHeight);
    const all = partsBounds(pusher(stroke, standHeight));
    expect(all.min.z).toBeCloseTo(want.z0, 6); // the blade's face
    expect(all.max.z).toBeCloseTo(want.z1, 6); // the tie-rod nuts
    expect(all.max.y).toBeCloseTo(want.top, 6);
    expect(all.min.y).toBeCloseTo(want.bottom, 6);
    expect(all.max.x).toBeCloseTo(0.364, 6); // the lens visors, on the control box's side
    expect(all.min.x).toBeCloseTo(-0.33, 6); // the stand's foot plates, on the other
    expect(Math.max(all.max.x, -all.min.x)).toBeLessThanOrEqual(want.halfLength);
  });

  it("stands on OIP's default height when given none", () => {
    expect(partsBounds(pusher(0.75, 0)).min.y).toBeCloseTo(-0.3847, 6);
  });

  it.each([0.1, 0.75, 1.5])("draws the moving stage at rest, its face at z = -1.103, at stroke %f", (stroke) => {
    const parts = pusher(stroke, 0.3847);
    const moving = parts.filter((p) => p.link === "moving");
    expect(moving.map((p) => p.material)).toEqual(["pusherRod", "pusherHood", "pusherFrame", "pusherRubber"]);
    const stage = partsBounds(moving);
    expect(stage.min.z).toBeCloseTo(-1.103, 6);
    expect(stage.max.z).toBeCloseTo(stroke - 0.516, 6); // the tie bar, a stroke behind the rear glands
    // the plate: 0.395 tall about y 0.447 and 0.47 wide, inside OIP's blade collider (0.4 tall, 0.47 wide)
    expect([stage.min.y, stage.max.y, stage.min.x, stage.max.x]).toEqual(
      [0.2495, 0.6445, -0.235, 0.235].map((v) => expect.closeTo(v, 6)),
    );
    expect(stage.min.y).toBeGreaterThanOrEqual(0.447 - 0.2);
    expect(stage.max.y).toBeLessThanOrEqual(0.447 + 0.2);
    // the block's glands and bolts stay behind the blade's plate
    const fixed = partsBounds(parts.filter((p) => !p.link));
    expect(fixed.min.z).toBeGreaterThan(-1.103 + 0.012 + 0.025);
  });

  it("lights green at home and red anywhere along the stroke", () => {
    expect(pusherLamps(0)).toEqual({ green: true, red: false });
    expect(pusherLamps(1e-4)).toEqual({ green: true, red: false });
    expect(pusherLamps(2e-4)).toEqual({ green: false, red: true });
    expect(pusherLamps(0.75)).toEqual({ green: false, red: true });
  });

  it("draws in materials that exist, the lenses in the lamps that glow", () => {
    const parts = pusher(0.75, 0.3847);
    for (const p of parts) expect(SPECS[p.material]).toBeDefined();
    const lit = parts.filter((p) => SPECS[p.material].emissive).map((p) => p.material);
    expect(lit).toEqual(["pusherLampRed", "pusherLampGreen"]);
  });
});
