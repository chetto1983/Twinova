import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { drawnOf, TRANSFER_STROKE } from "./families";
import type { Definition, Instance, ProjectView } from "./project";

const FIXTURE = join(__dirname, "../../src-tauri/project/tests/fixtures/valid/two_belts_diverter");
const read = (path: string) => JSON.parse(readFileSync(join(FIXTURE, path), "utf8"));
const scene: {
  instances: Instance[];
  connections: { kind: string; device?: string[]; host?: string[]; position?: number[] }[];
} = read("scene.json");
const definitions: Record<string, Definition> = Object.fromEntries(
  ["belt", "roller", "chain_transfer", "sensor_diffuse", "source", "sink"].map((f) => [
    `twinova.${f}@1`,
    read(`components/twinova.${f}@1.json`),
  ]),
);
const project = {
  instances: scene.instances,
  definitions,
  mounts: scene.connections
    .filter((c) => c.kind === "mount")
    .map((c) => ({ device: c.device![0], host: c.host![0], at: c.position![0] })),
  merges: [] as ProjectView["merges"],
} as ProjectView;
const drawn = (id: string) =>
  drawnOf(
    project,
    scene.instances.find((i) => i.id === id)!,
  );

describe("the families OIP's generators draw", () => {
  it("reads each family's own parameters, not the defaults", () => {
    const withParams = (id: string, parameters: Record<string, number>) => ({
      ...scene.instances.find((i) => i.id === id)!,
      parameters,
    });
    const roller = withParams("roller_r", { length: 3, width: 0.8, height: 0.9, frame_height: 0.25 });
    const changed = { ...project, instances: project.instances.map((i) => (i.id === "roller_r" ? roller : i)) };
    expect(drawnOf(changed, roller)!.offset).toEqual([1.5, 0.9, 0]);
    // the frame wall hangs frame_height under the rollers; its 20 mm flange, centred 0.02 past the edge, reaches 0.03 out
    const frameWalls = drawnOf(changed, roller)!.parts.find((p) => p.material === "frame")!.desc.positions;
    const ys = frameWalls.filter((_, k) => k % 3 === 1);
    const zs = frameWalls.filter((_, k) => k % 3 === 2);
    expect(Math.min(...ys)).toBeCloseTo(-0.255, 9);
    expect(Math.max(...zs)).toBeCloseTo(0.4 + 0.03, 9);
    const transfer = withParams("chain_transfer", { chains: 3, distance: 0.25, lane_length: 0.8 });
    const t = drawnOf(changed, transfer)!;
    expect(t.offset[1]).toBeCloseTo(0.87, 9);
    const chains = t.parts.find((p) => p.material === "chain")!.desc.positions;
    const xs = chains.filter((_, k) => k % 3 === 0);
    const zt = chains.filter((_, k) => k % 3 === 2);
    // three lanes 0.25 apart; the chain loop reaches 0.001 past the lane length's half (meshes.test.ts)
    expect(Math.max(...xs) - Math.min(...xs)).toBeCloseTo(0.5 + 2 * 0.0148 * (0.04 / 0.087), 6);
    expect(Math.max(...zt)).toBeCloseTo(0.401, 6);
    const eye = drawnOf(
      changed,
      scene.instances.find((i) => i.id === "sensor_a")!,
    )!;
    expect(eye.offset[1]).toBeCloseTo(0.9 + 0.05 - 0.1, 9);
    expect(eye.offset[2]).toBeCloseTo(-0.4 - 0.02 - 0.037, 9);
  });

  it("draws nothing for a device whose host is gone, or an instance of an unknown definition", () => {
    const orphan = { ...project, instances: project.instances.filter((i) => i.id !== "roller_r") };
    expect(
      drawnOf(
        orphan,
        scene.instances.find((i) => i.id === "chain_transfer")!,
      ),
    ).toBeNull();
    expect(drawnOf(project, { ...scene.instances[0], definition: "acme.robot@3" })).toBeNull();
  });

  it("centres a roller conveyor on its length, the rollers' top at its height", () => {
    const d = drawn("roller_r")!;
    expect([d.position, d.offset, d.motion]).toEqual([[3.085, 0, 0], [1, 0.8, 0], undefined]);
    expect(d.parts[0].material).toBe("roller");
  });

  it("hangs a chain transfer from its host, 3 cm under the rollers at its mount, lifted by its stroke", () => {
    const d = drawn("chain_transfer")!;
    expect([d.position, d.offset[0], d.offset[2], d.motion]).toEqual([
      [3.085, 0, 0],
      1,
      0,
      { slide: [0, TRANSFER_STROKE, 0] },
    ]);
    expect(d.parts.every((part) => part.link === "moving")).toBe(true);
    expect(d.offset[1]).toBeCloseTo(0.77, 9);
  });

  it("seats a photo eye beside its host's edge, the lens on the physics' beam", () => {
    const d = drawn("sensor_a")!;
    expect(d.position).toEqual([3.085, 0, 0]);
    // the beam 0.05 above the rollers, the lens at emitY 0.1 above the eye's origin, its face 20 mm out from the edge
    expect(d.offset[0]).toBe(1);
    expect(d.offset[1]).toBeCloseTo(0.8 + 0.05 - 0.1, 9);
    expect(d.offset[2]).toBeCloseTo(-0.3 - 0.02 - 0.037, 9);
  });

  it("finds each device's own mount", () => {
    const d = drawn("sensor_b")!;
    expect([d.position, d.offset[0]]).toEqual([[5.17, 0, 0], 1.5]);
  });

  it("leaves the other families, and an unmounted device, to their stand-ins", () => {
    expect(drawn("sink_ok")).toBeNull();
    expect(drawn("source")).toBeNull();
    const loose = { ...project, mounts: [] };
    const transfer = scene.instances.find((i) => i.id === "chain_transfer")!;
    expect(drawnOf(loose, transfer)).toBeNull();
    expect(
      drawnOf(
        loose,
        scene.instances.find((i) => i.id === "sensor_a")!,
      ),
    ).toBeNull();
  });
});
