import { describe, expect, it } from "vitest";
import { GROUPS, callOf, filtered, freeInstance, groupOf, grouped, template, type CatalogEntry } from "./catalog";

const pin = (name: string, type_name = "", bool = false) => ({ name, type_name, bool });
const entry = (name: string, more: Partial<CatalogEntry> = {}): CatalogEntry => ({
  name,
  block: false,
  own: false,
  inputs: [],
  outputs: [pin("OUT")],
  variadic: false,
  table: null,
  ...more,
});
const TON = entry("TON", {
  block: true,
  table: 46,
  inputs: [pin("IN", "BOOL", true), pin("PT", "TIME")],
  outputs: [pin("Q", "BOOL", true), pin("ET", "TIME")],
});
const VALVE = entry("Valve", { block: true, own: true, inputs: [pin("Open", "BOOL", true)], outputs: [] });
const EQ = entry("EQ", { table: 33, inputs: [pin("IN1"), pin("IN2")], variadic: true });
const ADD = entry("ADD", { table: 29, inputs: [pin("IN1"), pin("IN2")], variadic: true });
const ALL = [TON, entry("TON_TIME", { block: true, table: 46 }), VALVE, EQ, ADD, entry("SPLIT_DT", { table: 99 })];

describe("the box picker's catalogue", () => {
  it("finds the names a filter starts first, then the ones it is inside", () => {
    expect(filtered(ALL, "ton").map((e) => e.name)).toEqual(["TON", "TON_TIME"]);
    expect(filtered(ALL, "q").map((e) => e.name)).toEqual(["EQ"]);
    expect(filtered(ALL, "d").map((e) => e.name)).toEqual(["ADD", "SPLIT_DT"]);
    expect(filtered(ALL, "  ")).toHaveLength(ALL.length);
  });

  it("groups by IEC table, the project's own first, the rest last", () => {
    expect(ALL.map(groupOf)).toEqual(["timer", "timer", "project", "comparison", "math", "other"]);
    expect(grouped(ALL).map(([g, list]) => [g, list.length])).toEqual([
      ["project", 1],
      ["timer", 2],
      ["comparison", 1],
      ["math", 1],
      ["other", 1],
    ]);
    expect(GROUPS[0]).toBe("project");
  });

  it("names a new instance that nothing declares yet", () => {
    expect(freeInstance("TON", [])).toBe("TON_1");
    expect(freeInstance("TON", ["ton_1", "TON_2", "x"])).toBe("TON_3");
  });

  it("starts a block on its BOOL pins, a comparison by OUT, a function by EN and ENO", () => {
    expect(template(TON)).toBe("IN, Q");
    expect(template(VALVE)).toBe("Open");
    expect(template(entry("Idle", { block: true, outputs: [] }))).toBe("");
    expect(template(EQ)).toBe("IN1 := , IN2 := , OUT");
    expect(template(ADD)).toBe("EN, IN1 := , IN2 := , OUT => , ENO");
  });

  it("writes a box's call as the grammar reads it", () => {
    const pins = [
      { name: "IN", value: null, out: false },
      { name: "PT", value: "T#2s", out: false },
      { name: "ET", value: "Waited", out: true },
    ];
    expect(callOf({ kind: "box", head: "T1", pins })).toBe("T1(IN, PT := T#2s, ET => Waited)");
    expect(callOf({ kind: "box", head: null, pins: [] })).toBe("?()");
  });
});
