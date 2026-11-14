import { describe, expect, it } from "vitest";
import { lineOf, resolver, rungErrors } from "./view";
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
});
