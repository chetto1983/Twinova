import { readFileSync } from "node:fs";
import { join } from "node:path";
import * as THREE from "three";
import { describe, expect, it } from "vitest";
import { drawnOf } from "./families";
import { partsBounds } from "./meshes";
import type { Definition, Instance, ProjectView } from "./project";

const FIXTURE = join(__dirname, "../../src-tauri/project/tests/fixtures/valid/curve_line");
const read = (path: string) => JSON.parse(readFileSync(join(FIXTURE, path), "utf8"));
const scene: { instances: Instance[] } = read("scene.json");
const definitions: Record<string, Definition> = Object.fromEntries(
  ["belt", "belt_curve", "roller_curve", "sensor_diffuse", "source", "sink"].map((f) => [
    `twinova.${f}@1`,
    read(`components/twinova.${f}@1.json`),
  ]),
);
const instance = (id: string) => scene.instances.find((i) => i.id === id)!;
const project = (mounts: ProjectView["mounts"] = []) =>
  ({ instances: scene.instances, definitions, mounts }) as ProjectView;

/** Where a point of the drawing lands in its instance's frame. */
function placed(id: string, point: THREE.Vector3, edit: Partial<Instance> = {}): THREE.Vector3 {
  const d = drawnOf(project(), { ...instance(id), ...edit })!;
  const q = new THREE.Quaternion(...(d.turn ?? [0, 0, 0, 1]));
  return point
    .clone()
    .applyQuaternion(q)
    .add(new THREE.Vector3(...d.offset));
}

const near = (a: THREE.Vector3, b: [number, number, number]) =>
  expect(a.distanceTo(new THREE.Vector3(...b))).toBeLessThan(1e-6);

/** The bounds of an instance's guards with `parameters` set, or null without any. */
const guardReach = (id: string, parameters: Record<string, boolean>) => {
  const d = drawnOf(project(), { ...instance(id), parameters: { ...instance(id).parameters, ...parameters } })!;
  const guard = d.parts.find((p) => p.material === "guard");
  return guard ? partsBounds([guard]) : null;
};

describe("the curves OIP draws, where the physics carries the load", () => {
  // OIP's inlet stub tip is (rc, 0, stub) about the centre, heading −Z; the outlet's tip a stub past P(sweep).
  const rc = 0.9;
  const inlet = new THREE.Vector3(rc, 0, 0.085);
  const outlet = new THREE.Vector3(-0.085, 0, -rc);

  it("enters a left curve at its origin and leaves it a stub past the arc, at its height", () => {
    near(placed("curve_a", inlet), [0, 0.8, 0]);
    // physics path.rs: a default 90° left curve hands over at (0.985, 0, −0.985)
    near(placed("curve_a", outlet), [0.985, 0.8, -0.985]);
    near(placed("curve_a", new THREE.Vector3(0, 0, 0)), [0.085, 0.8, -rc]);
  });

  it("runs a right curve's drawing backwards, from its outlet, and its fabric with it", () => {
    near(placed("curve_b", outlet), [0, 0.8, 0]);
    near(placed("curve_b", inlet), [0.985, 0.8, 0.985]);
    near(placed("curve_b", new THREE.Vector3(0, 0, 0)), [0.085, 0.8, rc]);
    expect(drawnOf(project(), instance("curve_b"))!.parts.every((p) => p.scroll === undefined)).toBe(true);
    const belt = drawnOf(project(), { ...instance("curve_a"), parameters: { turn_right: true } })!;
    expect(belt.parts[0]).toMatchObject({ material: "belt", scroll: -1 });
  });

  it("puts each guard on its side of the flow: the inner one is the left of a left curve, the right of a right", () => {
    for (const id of ["curve_a", "curve_b"]) {
      const right = id === "curve_b";
      expect(guardReach(id, { left_guard: false, right_guard: false })).toBeNull();
      // OIP's frame: the inner guard reaches in to the inner radius, the outer one out past the outer.
      const onlyLeft = guardReach(id, { left_guard: true, right_guard: false })!;
      const inner = Math.hypot(onlyLeft.max.x, onlyLeft.max.z) < rc;
      expect(inner).toBe(!right);
    }
  });

  it("hangs a photo eye on a curve at its angle, facing across the arc", () => {
    const view = project([{ device: "sensor_c", host: "curve_a", at: 45 }]);
    const eye = drawnOf(view, instance("sensor_c"))!;
    expect(eye.position).toEqual(instance("curve_a").position);
    const a = Math.PI / 4;
    // On the left curve's frame at 45°: the centreline point, and the lens across to the left (−z of the frame).
    const frame = new THREE.Vector3(0.085 + rc * Math.sin(a), 0, -rc * (1 - Math.cos(a)));
    const lens = new THREE.Vector3(...eye.offset).sub(frame);
    lens.y = 0;
    const left = new THREE.Vector3(0, 0, -1).applyQuaternion(new THREE.Quaternion(...eye.turn!));
    expect(lens.normalize().dot(left)).toBeCloseTo(1, 6);
    const straight = drawnOf(project([{ device: "sensor_c", host: "belt_c", at: 1 }]), instance("sensor_c"))!;
    expect(straight.turn).toBeUndefined();
    expect(straight.offset[0]).toBe(1);
  });

  it("draws a straight belt from its tail, its guards from its parameters", () => {
    const belt = drawnOf(project(), instance("belt_a"))!;
    expect(belt.offset).toEqual([0, 0.8, 0]);
    expect(belt.parts.find((p) => p.material === "guard")!.instances).toHaveLength(2);
    const bare = drawnOf(project(), { ...instance("belt_a"), parameters: { left_guard: false, right_guard: false } })!;
    expect(bare.parts.some((p) => p.material === "guard")).toBe(false);
  });
});
