import { describe, expect, it } from "vitest";
import {
  decorations,
  deviceOf,
  labelOf,
  newRow,
  split,
  spotAt,
  toggled,
  valueClass,
  type Spot,
  type Target,
} from "./monitor";
import type { Source } from "./sources";

const count: Target = { kind: "variable", program: "Main", path: "count" };
const eye: Target = { kind: "address", address: "%IX0.0" };
const spot = (line: number, column: number, condition = false, target: Target | null = null): Spot => ({
  line,
  column,
  condition,
  target,
});

describe("online monitoring in the editor", () => {
  it("labels a target as the Watch pane lists it", () => {
    expect(labelOf(count)).toBe("Main.count");
    expect(labelOf(eye)).toBe("%IX0.0");
  });

  it("splits the values into the spots' and the rows'", () => {
    const watched = {
      spots: [spot(1, 2), spot(2, 3)],
      rows: [{ name: "a", declaration: 0, target: null, instance: false }],
    };
    expect(split(watched, ["1", "TRUE", "7"])).toEqual({ spots: ["1", "TRUE"], rows: ["7"] });
    expect(split(watched, [])).toEqual({ spots: [], rows: [] });
  });

  it("colours a condition and marks a forced value", () => {
    const forced = new Set(["Main.count"]);
    expect(valueClass(spot(1, 1), "3", forced)).toBe("monitor-value");
    expect(valueClass(spot(1, 1, true), "TRUE", forced)).toBe("monitor-value monitor-true");
    expect(valueClass(spot(1, 1, true), "FALSE", forced)).toBe("monitor-value monitor-false");
    expect(valueClass(spot(1, 1, false, count), "3", forced)).toBe("monitor-value monitor-forced");
    expect(valueClass(spot(1, 1, false, eye), "3", forced)).toBe("monitor-value");
  });

  it("draws each value after its spot, and nothing without a value", () => {
    const found = decorations([spot(40, 10), spot(41, 5), spot(42, 7)], ["3", null], new Set());
    expect(found).toEqual([
      {
        range: { startLineNumber: 40, startColumn: 10, endLineNumber: 40, endColumn: 10 },
        options: { after: { content: " 3", inlineClassName: "monitor-value" } },
      },
    ]);
  });

  it("finds the spot a click hits", () => {
    const spots = [spot(40, 10), spot(42, 7)];
    expect(spotAt(spots, 42, 7)).toBe(1);
    expect(spotAt(spots, 42, 8)).toBe(-1);
    expect(spotAt(spots, 41, 7)).toBe(-1);
  });

  it("toggles a BOOL and leaves any other value to typing", () => {
    expect(toggled("TRUE")).toBe("FALSE");
    expect(toggled("FALSE")).toBe("TRUE");
    expect(toggled("3")).toBeNull();
    expect(toggled(null)).toBeNull();
  });

  it("monitors a block's tab on the selected instance of its family", () => {
    const belt: Source = { key: "twinova.belt@1", label: "belt", kind: "behaviour", text: "", base: "" };
    const main: Source = { key: "plc/Main.st", label: "Main", kind: "program", text: "", base: "" };
    const instances = [
      { id: "belt_a", definition: "twinova.belt@1" },
      { id: "sink", definition: "twinova.sink@1" },
    ];
    expect(deviceOf(belt, "belt_a", instances)).toBe("belt_a");
    expect(deviceOf(belt, "sink", instances)).toBeNull();
    expect(deviceOf(belt, null, instances)).toBeNull();
    expect(deviceOf(main, "belt_a", instances)).toBeNull();
  });

  it("names a new row so that it does not clash", () => {
    expect(newRow("Main", ["var1", "var2"]).name).toBe("var3");
    expect(newRow("Main", [])).toEqual({
      pou: "Main",
      name: "var1",
      class: "VAR",
      qualifier: null,
      type_name: "BOOL",
      address: null,
      initial: null,
      comment: null,
    });
  });
});
