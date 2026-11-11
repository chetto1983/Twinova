import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { drawnOf } from "./families";
import type { MeshDescription } from "./oip/geometry/types";
import { LASER } from "./oip/parts/laser";
import { PHOTOEYE } from "./oip/parts/photoeye";
import type { Definition, Instance, ProjectView } from "./project";

// The 4.4d sensors (spec 2026-11-11-sensors-products-design.md): each housing where the physics casts its beam, the
// lens 20 mm out from the bed edge and the beam 50 mm over the surface (physics world_sensors.rs); the scanner's arch
// over its conveyor.
const belt: Definition = JSON.parse(
  readFileSync(
    join(__dirname, "../../src-tauri/project/tests/fixtures/valid/sort_line/components/twinova.belt@1.json"),
    "utf8",
  ),
);
const family = (name: string): Definition => ({
  id: `twinova.${name}`,
  version: 1,
  generator: `twinova.${name}/1`,
  parameters: [],
  signals: [],
  ports: [{ name: "mount", kind: "mount_device" }],
});
const place = (id: string, definition: string): Instance => ({
  id,
  name: id,
  definition,
  position: [0, 0, 0],
  rotation: [0, 0, 0, 1],
  parameters: {},
  signals: {},
});
const NAMES = ["sensor_diffuse", "sensor_through_beam", "sensor_color", "sensor_laser", "scanner_barcode"];
const view = {
  instances: [
    { ...place("belt_a", "twinova.belt@1"), parameters: { width: 0.8, height: 0.9 } },
    ...NAMES.map((n) => place(n, `twinova.${n}@1`)),
  ],
  definitions: {
    "twinova.belt@1": belt,
    ...Object.fromEntries(NAMES.map((n) => [`twinova.${n}@1`, family(n)])),
  },
  mounts: NAMES.map((device) => ({ device, host: "belt_a", at: 1.5 })),
} as unknown as ProjectView;
const drawn = (id: string) =>
  drawnOf(
    view,
    view.instances.find((i) => i.id === id)!,
  )!;

/** The extent of a mesh along one axis. */
function extent(desc: MeshDescription, axis: 0 | 1 | 2): [number, number] {
  const values = desc.positions.filter((_, k) => k % 3 === axis);
  return [Math.min(...values), Math.max(...values)];
}

describe("the 4.4d sensors in the view", () => {
  it("seats the laser's window on the beam, 20 mm out from the bed edge", () => {
    const d = drawn("sensor_laser");
    expect(d.offset[0]).toBe(1.5);
    expect(d.offset[1] + LASER.emitY).toBeCloseTo(0.9 + 0.05, 9);
    expect(d.offset[2] + LASER.front).toBeCloseTo(-0.4 - 0.02, 9);
    expect(d.parts.map((p) => p.material)).toContain("laserWindow");
  });

  it("draws the diffuse sensor and the colour sensor in the same housing, each in its own paint", () => {
    expect(drawn("sensor_diffuse").offset).toEqual(drawn("sensor_color").offset);
    expect(
      drawn("sensor_diffuse")
        .parts.map((p) => p.material)
        .slice(0, 2),
    ).toEqual(["body", "lens"]);
  });

  it("draws the colour sensor as the photo eye in teal with a red window", () => {
    const d = drawn("sensor_color");
    expect(d.offset[1] + PHOTOEYE.emitY).toBeCloseTo(0.95, 9);
    expect(d.offset[2] + PHOTOEYE.front).toBeCloseTo(-0.42, 9);
    expect(d.parts.map((p) => p.material).slice(0, 2)).toEqual(["colorBody", "colorLens"]);
  });

  it("faces a through-beam's receiver to its emitter from the far edge", () => {
    const d = drawn("sensor_through_beam");
    const half = d.parts.length / 2;
    expect(half).toBe(5);
    const [emitter, receiver] = [d.parts[0].desc, d.parts[half].desc];
    // The receiver's lens face, at its local z min, lies 20 mm out from the far edge: 0.84 m from the emitter's.
    const lens = d.offset[2] + extent(receiver, 2)[0];
    expect(lens).toBeCloseTo(0.4 + 0.02, 9);
    expect(extent(receiver, 1)).toEqual(extent(emitter, 1));
    expect(d.offset[2] + extent(emitter, 2)[1]).toBeCloseTo(-0.42, 9);
  });

  it("stands the scanner's arch over its conveyor, its window 0.6 m over the surface", () => {
    const d = drawn("scanner_barcode");
    expect(d.offset).toEqual([1.5, 0, 0]);
    expect(d.turn![1]).toBeCloseTo(Math.SQRT1_2, 12);
    const window = d.parts.find((p) => p.material === "scannerWindow")!.desc;
    expect(extent(window, 1)[0]).toBeCloseTo(0.9 + 0.6, 9);
    // The crossbeam spans the clear width, 0.2 m past each edge of the 0.8 m conveyor, and the two 50 mm columns.
    const beam = d.parts.find((p) => p.material === "scannerBeam")!.desc;
    const [from, to] = extent(beam, 0);
    expect(from).toBeCloseTo(-(0.6 + 0.05), 9);
    expect(to).toBeCloseTo(0.6 + 0.05, 9);
  });

  it("draws nothing for a sensor that is not mounted", () => {
    const loose = { ...view, mounts: [] } as ProjectView;
    for (const id of NAMES)
      expect(
        drawnOf(
          loose,
          view.instances.find((i) => i.id === id)!,
        ),
      ).toBeNull();
  });
});
