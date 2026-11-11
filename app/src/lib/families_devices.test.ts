import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { drawnOf } from "./families";
import { rollerGap } from "./oip/parts/roller";
import type { Definition, Instance, ProjectView } from "./project";

// The 4.4c sort line (spec 2026-11-11-stopper-pusher-turntable-design.md): the view places each device where the
// physics does (physics layout_devices.rs, world_devices.rs).
const FIXTURE = join(__dirname, "../../src-tauri/project/tests/fixtures/valid/sort_line");
const read = (path: string) => JSON.parse(readFileSync(join(FIXTURE, path), "utf8"));
const scene: {
  instances: Instance[];
  connections: { kind: string; device?: string[]; host?: string[]; position?: number[] }[];
} = read("scene.json");
const definitions: Record<string, Definition> = Object.fromEntries(
  readdirSync(join(FIXTURE, "components"))
    .filter((f) => f.endsWith(".json"))
    .map((f) => [f.replace(/\.json$/, ""), read(`components/${f}`)]),
);
const project = {
  instances: scene.instances,
  definitions,
  mounts: scene.connections
    .filter((c) => c.kind === "mount")
    .map((c) => ({ device: c.device![0], host: c.host![0], at: c.position![0] })),
} as ProjectView;
const instance = (id: string) => scene.instances.find((i) => i.id === id)!;
const drawn = (id: string, view: ProjectView = project) => drawnOf(view, instance(id))!;
const moving = (id: string) => drawn(id).parts.filter((p) => p.link === "moving").length;

describe("the 4.4c devices in the view", () => {
  it("snaps a stopper's mount to the gap between rollers nearest it, as the physics does", () => {
    for (const [length, at, gap] of [
      [2, 1.64, 1.6375],
      [2, 1.6, 1.6375],
      [2, 0, 0.0625],
      [2, 2, 1.9375],
      [4, 2, 2.0375],
      [4, 1.99, 1.9625],
      [0.2, 0.1, 0.1375],
      [0.2, 0.05, 0.0625],
      [0.1, 0, 0.05],
    ] as const)
      expect(rollerGap(length, at)).toBeCloseTo(gap, 9);
  });

  it("raises the stopper's blade in its gap by its raise and 5 cm", () => {
    const d = drawn("stopper");
    expect([d.position, d.offset, d.motion]).toEqual([[2, 0, 0], [1.6375, 0.8, 0], { slide: [0, 0.09, 0] }]);
    expect(moving("stopper")).toBeGreaterThan(0);
  });

  it("stands the pusher beside belt B with its paddle face 2 cm outside the belt, pushing across it", () => {
    const d = drawn("pusher");
    // OIP's face is at −1.103 along its push: 0.3 + 0.0205 from belt B's centreline, as the physics's paddle.
    expect(d.position).toEqual([4.9, 0, 0]);
    expect(d.offset[0]).toBe(1.5);
    expect(d.offset[1]).toBeCloseTo(0.6, 9);
    expect(d.offset[2] - 1.103).toBeCloseTo(0.3205, 9);
    expect([d.turn, d.motion]).toEqual([undefined, { slide: [0, 0, -0.75] }]);
    expect(moving("pusher")).toBeGreaterThan(0);
    const pushesRight = { ...instance("pusher"), parameters: { out_right: true } };
    const mirrored = drawnOf({ ...project, instances: [...project.instances, pushesRight] }, pushesRight)!;
    expect(mirrored.offset[2] + 1.103).toBeCloseTo(-0.3205, 9);
    expect(mirrored.turn![1]).toBeCloseTo(1, 9);
  });

  it("opens belt B's guard on the pusher's side, 0.75 m about it", () => {
    const guards = drawn("belt_b").parts.find((p) => p.material === "guard")!;
    const right = guards.instances!.filter((i) => i.position[2] > 0);
    expect(guards.instances!.length).toBe(right.length);
    expect(right.map((i) => [i.position[0], i.position[0] + i.scale![0]].map((x) => +x.toFixed(4)))).toEqual([
      [-0.085, 1.125],
      [1.875, 3.085],
    ]);
  });

  it("turns the turntable's deck about its centre, and stands its photocell still across its start lane", () => {
    const d = drawn("turntable");
    expect([d.position, d.offset, d.motion]).toEqual([[4.45, 0, 0], [0, 0.8, 0], { turn: true }]);
    expect(moving("turntable")).toBeGreaterThan(0);
    expect(drawn("turntable").parts.some((p) => p.material === "leg" && !p.link)).toBe(true);
    // The beam 0.6 m along the lane from its entry, 0.45 m behind the centre: 0.15 m past the centre.
    expect(drawn("s_turn").offset[0]).toBeCloseTo(0.15, 9);
  });
});
