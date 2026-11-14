import { describe, expect, it } from "vitest";
import { keyAction, lineOf, onlineOf, powered, resolver, rungErrors, selectionOf } from "./view";
import type { Lowering } from "./model";

const lowering: Lowering = {
  power: [],
  boxes: [
    [
      { rung: 1, path: [0, 1, 2] },
      { function: false, type_name: "TON", input: "IN", output: "Q" },
    ],
  ],
  errors: [
    { element: { rung: 2, path: [0] }, message: "This coil has no variable yet: the ? on this rung" },
    { element: { rung: 2, path: [] }, message: "there is no rung labelled 'X' to jump to" },
  ],
};

const spot = (rung: number, path: number[], power: boolean, pin: string | null = null) => ({
  element: { rung, path },
  pin,
  power,
  target: null,
});

const key = (k: string, mods: Partial<{ ctrlKey: boolean; metaKey: boolean; shiftKey: boolean }> = {}) =>
  keyAction({ key: k, ctrlKey: false, metaKey: false, shiftKey: false, ...mods });

describe("the Ladder view's reading of a file", () => {
  it("names the line a byte offset is on, counting bytes as Rust does", () => {
    expect(lineOf("VAR\nEND_VAR\n|A|", 0)).toBe(1);
    expect(lineOf("VAR\nEND_VAR\n|A|", 12)).toBe(3);
    // "è" is two bytes in UTF-8: offset 4 is still on line 1
    expect(lineOf("(*è*)\n|A|", 4)).toBe(1);
    expect(lineOf("(*è*)\n|A|", 7)).toBe(2);
  });

  it("finds a box's pins by its rung and path", () => {
    expect(resolver(lowering)(1)([0, 1, 2])?.type_name).toBe("TON");
    expect(resolver(lowering)(0)([0, 1, 2])).toBeUndefined();
    expect(resolver(null)(1)([0, 1, 2])).toBeUndefined();
  });

  it("lists what keeps a rung from lowering", () => {
    expect(rungErrors(lowering, 2)).toEqual([
      "This coil has no variable yet: the ? on this rung",
      "there is no rung labelled 'X' to jump to",
    ]);
    expect(rungErrors(lowering, 0)).toEqual([]);
    expect(rungErrors(null, 2)).toEqual([]);
  });

  it("reads a click as the element, the branch path or the rung it landed on", () => {
    expect(selectionOf("2", "1.0.3")).toEqual({ rung: 2, path: [1, 0, 3] });
    expect(selectionOf("0", undefined)).toEqual({ rung: 0, path: [] });
    expect(selectionOf("0", "")).toEqual({ rung: 0, path: [] });
    expect(selectionOf(undefined, "1")).toBeNull();
  });

  it("reads the keys the view acts on", () => {
    expect([key("Delete"), key("Backspace"), key("ArrowLeft"), key("ArrowRight"), key("Escape")]).toEqual([
      "delete",
      "delete",
      "left",
      "right",
      "escape",
    ]);
    expect([
      key("z", { ctrlKey: true }),
      key("Z", { metaKey: true, shiftKey: true }),
      key("y", { ctrlKey: true }),
    ]).toEqual(["undo", "redo", "redo"]);
    expect([key("Enter"), key("F2"), key("b", { ctrlKey: true })]).toEqual(["edit", "edit", "box"]);
    expect([key("c", { ctrlKey: true }), key("X", { metaKey: true }), key("v", { ctrlKey: true })]).toEqual([
      "copy",
      "cut",
      "paste",
    ]);
    expect([key("a"), key("Delete", { ctrlKey: true }), key("q", { ctrlKey: true })]).toEqual([null, null, null]);
  });

  it("reads the values online: each element's power out, each operand's and pin's value, by rung", () => {
    const spots = [
      spot(0, [0], true),
      spot(0, [1, 0, 0], true),
      spot(1, [0], true),
      spot(0, [0], false),
      spot(0, [2], false, "ET"),
    ];
    const online = onlineOf(spots, ["TRUE", "FALSE", "TRUE", "TRUE", null], 0);
    expect([...online.power]).toEqual([
      ["0.0", true],
      ["0.1.0.0", false],
    ]);
    expect([...online.values]).toEqual([["0.0", { value: "TRUE", spot: 3 }]]);
    expect(onlineOf(spots, ["TRUE", "TRUE", "TRUE", "1", "T#2s"], 0).values.get("0.2#ET")).toEqual({
      value: "T#2s",
      spot: 4,
    });
    expect([
      powered(online, 0, null),
      powered(online, 0, [0]),
      powered(online, 0, [1, 0, 0]),
      powered(online, 0, [5]),
    ]).toEqual([true, true, false, null]);
    expect(powered(null, 0, null)).toBeNull();
  });
});
