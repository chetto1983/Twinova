import { describe, expect, it } from "vitest";
import {
  flowPorts,
  headingOf,
  headingQuat,
  hostsFor,
  mountOf,
  mountsOnAHost,
  parseTime,
  parseValue,
  valueText,
} from "./inspector";
import type { Definition, Instance, ParameterDef, ProjectView } from "./project";

const param = (type: ParameterDef["type"]): ParameterDef => ({
  name: "p",
  type,
  unit: null,
  default: 0,
  min: null,
  max: null,
});

describe("a parameter typed in the inspector", () => {
  it("reads a duration in its unit, or nanoseconds alone", () => {
    expect(parseTime("2 s")).toBe(2e9);
    expect(parseTime("300ms")).toBe(3e8);
    expect(parseTime("1,5 s")).toBe(1.5e9);
    expect(parseTime("5 µs")).toBe(5000);
    expect(parseTime("5 us")).toBe(5000);
    expect(parseTime("250")).toBe(250);
    expect(parseTime("0.5 ns")).toBeNull();
    expect(parseTime("2 h")).toBeNull();
    expect(parseTime("-1 s")).toBeNull();
  });

  it("is a value of its type, the default back when empty, or refused", () => {
    expect(parseValue(param("REAL"), "0,75")).toBe(0.75);
    expect(parseValue(param("REAL"), "fast")).toBeUndefined();
    expect(parseValue(param("DINT"), "12")).toBe(12);
    expect(parseValue(param("DINT"), "1.5")).toBeUndefined();
    expect(parseValue(param("TIME"), "150 ms")).toBe(150_000_000);
    expect(parseValue(param("TIME"), "soon")).toBeUndefined();
    expect(parseValue(param("DINT_LIST"), "3, 1,2")).toEqual([3, 1, 2]);
    expect(parseValue(param("DINT_LIST"), "1, x")).toBeUndefined();
    expect(parseValue(param("COLOR_LIST"), "#FF0000, #00ff7f")).toEqual(["#ff0000", "#00ff7f"]);
    expect(parseValue(param("COLOR_LIST"), "red")).toBeUndefined();
    expect(parseValue(param("BOOL"), "true")).toBeUndefined();
    expect(parseValue(param("REAL"), "  ")).toBeNull();
  });

  it("shows a duration in its unit and a list as its items", () => {
    expect(valueText(param("TIME"), 2e9)).toBe("2 s");
    expect(valueText(param("DINT_LIST"), [1, 2])).toBe("1, 2");
    expect(valueText(param("REAL"), 0.5)).toBe("0.5");
  });
});

describe("the heading about +Y", () => {
  it("turns degrees into a rotation and back", () => {
    for (const degrees of [0, 15, 90, -45, 180, -135]) expect(headingOf(headingQuat(degrees))).toBe(degrees);
    const [x, y, z, w] = headingQuat(90);
    expect([x, z]).toEqual([0, 0]);
    expect(y).toBeCloseTo(Math.SQRT1_2, 12);
    expect(w).toBeCloseTo(Math.SQRT1_2, 12);
    expect(headingOf([0, 0, 0, 1])).toBe(0);
  });
});

const def = (id: string, ports: Definition["ports"]): Definition => ({
  id,
  version: 1,
  generator: `${id}/1`,
  parameters: [],
  signals: [],
  ports,
});
const inst = (id: string, definition: string): Instance => ({
  id,
  name: id,
  definition,
  position: [0, 0, 0],
  rotation: [0, 0, 0, 1],
  parameters: {},
  signals: {},
});
const view = {
  instances: [
    inst("source_1", "twinova.source@1"),
    inst("belt_1", "twinova.belt@1"),
    inst("belt_2", "twinova.belt@1"),
    inst("eye_1", "twinova.sensor_diffuse@1"),
  ],
  definitions: {
    "twinova.source@1": def("twinova.source", [{ name: "out", kind: "flow_out" }]),
    "twinova.belt@1": def("twinova.belt", [
      { name: "in", kind: "flow_in" },
      { name: "out", kind: "flow_out" },
      { name: "rail", kind: "mount_host" },
    ]),
    "twinova.sensor_diffuse@1": def("twinova.sensor_diffuse", [{ name: "mount", kind: "mount_device" }]),
  },
  flows: [{ from: ["source_1", "out"], to: ["belt_1", "in"] }],
  mounts: [{ device: "eye_1", host: "belt_2", at: 1.5 }],
} as unknown as ProjectView;

describe("ports and mounts in the inspector", () => {
  it("gives each flow port its peer and the ports of the opposite kind elsewhere", () => {
    expect(flowPorts(view, view.instances[1])).toEqual([
      {
        port: "in",
        kind: "flow_in",
        peer: ["source_1", "out"],
        candidates: [
          ["source_1", "out"],
          ["belt_2", "out"],
        ],
      },
      { port: "out", kind: "flow_out", peer: null, candidates: [["belt_2", "in"]] },
    ]);
    expect(flowPorts(view, view.instances[0])[0].peer).toEqual(["belt_1", "in"]);
    expect(flowPorts(view, view.instances[3])).toEqual([]);
  });

  it("offers a device the instances with a rail, and tells where it is mounted", () => {
    expect(mountsOnAHost(view, view.instances[3])).toBe(true);
    expect(mountsOnAHost(view, view.instances[1])).toBe(false);
    expect(hostsFor(view, view.instances[3])).toEqual(["belt_1", "belt_2"]);
    expect(mountOf(view, "eye_1")).toEqual({ host: "belt_2", at: 1.5 });
    expect(mountOf(view, "belt_1")).toBeNull();
  });
});
