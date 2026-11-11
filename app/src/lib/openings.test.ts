import { describe, expect, it } from "vitest";
import { guardSpans } from "./openings";

const at = (centre: number, right = true) => ({ right, centre, length: 0.75 });

describe("guard openings", () => {
  it("leave a guard whole without a pusher on its side", () => {
    expect(guardSpans(0, 3, true, [])).toEqual([[0, 3]]);
    expect(guardSpans(0, 3, false, [at(1)])).toEqual([[0, 3]]);
  });

  it("open 0.75 m about each pusher, trimmed at the ends, as the physics does", () => {
    expect(guardSpans(0, 3, true, [at(1)])).toEqual([
      [0, 0.625],
      [1.375, 3],
    ]);
    expect(guardSpans(0, 3, true, [at(2.9), at(1)])).toEqual([
      [0, 0.625],
      [1.375, 2.525],
    ]);
    expect(guardSpans(-0.085, 3.085, false, [at(0.1, false)])).toEqual([[0.475, 3.085]]);
    expect(guardSpans(0, 3, true, [at(1), at(1.2)])).toEqual([
      [0, 0.625],
      [1.575, 3],
    ]);
    // flush with either end: no sliver of guard there
    expect(guardSpans(0, 3, true, [at(0.375), at(2.625)])).toEqual([[0.75, 2.25]]);
  });
});
