import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  eulerDegrees,
  family,
  formatTime,
  formatValue,
  frame,
  parameterRows,
  shapeOf,
  signalRows,
  type Definition,
  type Instance,
  type ParameterDef,
} from "./project";

// The same reference line the Rust corpus opens (src-tauri/project/tests/fixtures).
const FIXTURE = join(__dirname, "../../src-tauri/project/tests/fixtures/valid/two_belts_diverter");
const read = (path: string) => JSON.parse(readFileSync(join(FIXTURE, path), "utf8"));
const scene: { instances: Instance[] } = read("scene.json");
const definition = (name: string): Definition => read(`components/twinova.${name}@1.json`);
const instance = (id: string): Instance => scene.instances.find((i) => i.id === id)!;
const param = (over: Partial<ParameterDef>): ParameterDef => ({
  name: "p",
  type: "REAL",
  unit: null,
  default: 0,
  min: null,
  max: null,
  ...over,
});

describe("values", () => {
  it("names the family without its namespace", () => {
    expect(family("twinova.sensor_diffuse")).toBe("sensor_diffuse");
    expect(family("belt")).toBe("belt");
  });

  it("writes a TIME in the largest unit that keeps it whole", () => {
    expect(formatTime(2_000_000_000)).toBe("2 s");
    expect(formatTime(300_000_000)).toBe("300 ms");
    expect(formatTime(1_500_000_000)).toBe("1500 ms");
    expect(formatTime(7_000)).toBe("7 µs");
    expect(formatTime(1_001)).toBe("1001 ns");
    expect(formatTime(0)).toBe("0 ns");
  });

  it("writes each type with its unit", () => {
    expect(formatValue(param({ type: "REAL", unit: "m" }), 3)).toBe("3 m");
    expect(formatValue(param({ type: "DINT" }), 9)).toBe("9");
    expect(formatValue(param({ type: "BOOL" }), true)).toBe("TRUE");
    expect(formatValue(param({ type: "BOOL" }), false)).toBe("FALSE");
    expect(formatValue(param({ type: "TIME", unit: "ns" }), 10_000_000)).toBe("10 ms");
  });
});

describe("inspector rows", () => {
  it("shows defaults and marks the instance's overrides", () => {
    const rows = parameterRows(instance("belt_b"), definition("belt"));
    expect(rows).toEqual([
      { name: "length", text: "2 m", overridden: true },
      { name: "width", text: "0.6 m", overridden: false },
      { name: "height", text: "0.8 m", overridden: false },
      { name: "speed", text: "0.5 m/s", overridden: false },
    ]);
  });

  it("lists every signal with its address, or none when unbound", () => {
    expect(signalRows(instance("diverter"), definition("diverter"))).toEqual([
      { name: "divert", type: "BOOL", direction: "plc_output", address: "%QX0.2" },
      { name: "in_position", type: "BOOL", direction: "plc_input", address: "%IX0.4" },
    ]);
    const unbound = { ...instance("belt_a"), signals: {} };
    expect(signalRows(unbound, definition("belt")).map((r) => r.address)).toEqual([null, null]);
  });
});

/** The normal sink moved to depth `z`. */
function sinkAt(z: number): Instance {
  return { ...instance("sink_ok"), position: [-0.5, 0, z] };
}

/** The unit quaternion of a rotation by `degrees` about one axis. */
function about(axis: 0 | 1 | 2, degrees: number): [number, number, number, number] {
  const q: [number, number, number, number] = [0, 0, 0, Math.cos((degrees * Math.PI) / 360)];
  q[axis] = Math.sin((degrees * Math.PI) / 360);
  return q;
}

describe("geometry", () => {
  it("turns quaternions into degrees about X, Y and Z", () => {
    const h = Math.SQRT1_2;
    expect(eulerDegrees([0, 0, 0, 1])).toEqual([0, 0, 0]);
    expect(eulerDegrees([0, h, 0, h])).toEqual([0, 90, 0]);
    expect(eulerDegrees(about(1, -90))).toEqual([0, -90, 0]);
    expect(eulerDegrees(about(1, 180))).toEqual([0, 180, 0]);
    expect(eulerDegrees(about(2, 30))).toEqual([0, 0, 30]);
    expect(eulerDegrees(about(0, 45))).toEqual([45, 0, 0]);
    expect(eulerDegrees(about(0, 90))).toEqual([90, 0, 0]);
    expect(eulerDegrees(about(0, -90))).toEqual([-90, 0, 0]);
    // Y 30° then X 20°: the order YXZ gives both back.
    const [a, b] = [about(1, 30), about(0, 20)];
    const q: [number, number, number, number] = [
      a[3] * b[0] + a[0] * b[3] + a[1] * b[2] - a[2] * b[1],
      a[3] * b[1] - a[0] * b[2] + a[1] * b[3] + a[2] * b[0],
      a[3] * b[2] + a[0] * b[1] - a[1] * b[0] + a[2] * b[3],
      a[3] * b[3] - a[0] * b[0] - a[1] * b[1] - a[2] * b[2],
    ];
    expect(eulerDegrees(q)).toEqual([20, 30, 0]);
    // At the singularity (90° about X) yaw and roll merge; the yaw is kept.
    const c = about(0, 90);
    const locked: [number, number, number, number] = [
      a[3] * c[0] + a[0] * c[3] + a[1] * c[2] - a[2] * c[1],
      a[3] * c[1] - a[0] * c[2] + a[1] * c[3] + a[2] * c[0],
      a[3] * c[2] + a[0] * c[1] - a[1] * c[0] + a[2] * c[3],
      a[3] * c[3] - a[0] * c[0] - a[1] * c[1] - a[2] * c[2],
    ];
    expect(eulerDegrees(locked)).toEqual([90, 30, 0]);
  });

  it("sizes a belt from its length, width and height", () => {
    expect(shapeOf(instance("belt_b"), definition("belt"))).toEqual({
      kind: "box",
      size: [2, 0.08, 0.6],
      centre: [1, 0.76, 0],
      token: "--color-surface-3",
    });
    const diverter = shapeOf(instance("diverter"), definition("diverter"));
    expect([diverter.size, diverter.centre]).toEqual([
      [1, 0.08, 0.6],
      [0.5, 0.76, 0],
    ]);
  });

  it("gives every other family its own stand-in", () => {
    const shape = (id: string, kind: string) => shapeOf(instance(id), definition(kind));
    const sensor = shape("sensor_a", "sensor_diffuse");
    expect([sensor.kind, sensor.size, sensor.token]).toEqual(["cylinder", [0.06, 0.3, 0.06], "--color-success"]);
    expect([sensor.centre.length, sensor.centre[0], sensor.centre[2]]).toEqual([3, 0, 0]);
    expect(sensor.centre[1]).toBeCloseTo(0.95, 12);
    expect(shape("source", "source")).toEqual({
      kind: "box",
      size: [0.4, 0.4, 0.4],
      centre: [0, 0.2, 0],
      token: "--color-info",
    });
    expect(shape("sink_ok", "sink")).toEqual({
      kind: "box",
      size: [0.4, 0.4, 0.4],
      centre: [0, 0.2, 0],
      token: "--color-border-strong",
    });
    const unknown = { ...definition("sink"), generator: "acme.robot/3" };
    expect(shapeOf(instance("sink_ok"), unknown)).toEqual({
      kind: "box",
      size: [0.3, 0.3, 0.3],
      centre: [0, 0.15, 0],
      token: "--color-warning",
    });
  });

  it("takes a size from the definition's default when the instance does not set it", () => {
    const belt = definition("belt");
    const wide = structuredClone(belt);
    wide.parameters.find((p) => p.name === "width")!.default = 0.9;
    wide.parameters.find((p) => p.name === "height")!.default = 1.1;
    const shape = shapeOf(instance("belt_b"), wide);
    expect(shape.size).toEqual([2, 0.08, 0.9]);
    expect(shape.centre[1]).toBeCloseTo(1.06, 12);
  });

  it("falls back to default sizes when a parameter is missing or not a number", () => {
    const bare = { ...definition("belt"), parameters: [] };
    expect(shapeOf({ ...instance("belt_a"), parameters: {} }, bare).size).toEqual([1, 0.08, 0.6]);
    const odd = { ...instance("belt_a"), parameters: { length: true } };
    expect(shapeOf(odd, definition("belt")).size[0]).toBe(1);
  });

  it("frames the footprint of the scene", () => {
    expect(frame([])).toEqual({ centre: [0, 0, 0], radius: 1 });
    expect(frame(scene.instances)).toEqual({ centre: [2.75, 0.8, 0], radius: 4.25 });
    expect(frame([sinkAt(2), sinkAt(10)])).toEqual({ centre: [-0.5, 0.8, 6], radius: 5 });
  });
});
