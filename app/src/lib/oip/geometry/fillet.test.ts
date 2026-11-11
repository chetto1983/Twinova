import { describe, expect, it } from "vitest";
import { createGeometryBuilder } from "./builder";
import { filletCorner, filletPolyline } from "./fillet";
import type { Vec3 } from "./mesh";

const r = (x: number) => Math.round(x * 1e9) / 1e9 + 0; // + 0 turns −0 into 0
const corner: Vec3[] = [
  [0, 1, 0.5],
  [0, 0, 0.5],
  [1, 0, 0.5],
];

describe("OIP's fillet", () => {
  it("rounds a right-angle corner with a tangent arc, the endpoints and z kept", () => {
    const path = createGeometryBuilder().fillet(corner, 0.25);
    expect(path).toHaveLength(2 + 7); // the ends, and the default 6 segments' 7 points
    expect(path[0]).toEqual([0, 1, 0.5]);
    expect(path.at(-1)).toEqual([1, 0, 0.5]);
    const arc = path.slice(1, -1);
    expect(arc[0]!.map(r)).toEqual([0, 0.25, 0.5]);
    expect(arc.at(-1)!.map(r)).toEqual([0.25, 0, 0.5]);
    for (const p of arc) expect(r(Math.hypot(p[0] - 0.25, p[1] - 0.25))).toBe(0.25);
  });

  it("turns the other way round a clockwise corner", () => {
    const path = filletPolyline(corner.toReversed(), 0.25, 2);
    expect(path.map((p) => p.map(r))).toEqual([
      [1, 0, 0.5],
      [0.25, 0, 0.5],
      [r(0.25 - 0.25 * Math.SQRT1_2), r(0.25 - 0.25 * Math.SQRT1_2), 0.5],
      [0, 0.25, 0.5],
      [0, 1, 0.5],
    ]);
  });

  it("caps the inset at half the shorter edge, tightening the radius", () => {
    expect(filletCorner(1, Math.PI / 2, 0.2, 3)).toEqual({ inset: 0.1, radius: 0.1 / Math.tan(Math.PI / 4) });
    expect(filletCorner(0.25, Math.PI / 3, 1, 1)).toEqual({ inset: 0.25 * Math.tan(Math.PI / 6), radius: 0.25 });
    const tight = filletPolyline(
      [
        [0, 0.2, 0],
        [0, 0, 0],
        [3, 0, 0],
      ],
      1,
      4,
    );
    expect(tight[1]!.map(r)).toEqual([0, 0.1, 0]);
    expect(tight[5]!.map(r)).toEqual([0.1, 0, 0]);
  });

  it("keeps a straight run, a repeated point, a short polyline and a zero radius as they are", () => {
    const straight: Vec3[] = [
      [0, 0, 0],
      [1, 0, 0],
      [2, 0, 0],
    ];
    expect(filletPolyline(straight, 0.3, 6)).toEqual(straight);
    const repeated: Vec3[] = [
      [0, 0, 0],
      [0, 0, 0],
      [1, 1, 0],
    ];
    expect(filletPolyline(repeated, 0.3, 6)).toEqual(repeated);
    expect(filletPolyline(corner.slice(0, 2), 0.3, 6)).toEqual(corner.slice(0, 2));
    expect(filletPolyline(corner, 0, 6)).toEqual(corner);
    expect(filletPolyline(corner, 0.3, 0)).toHaveLength(2 + 2); // at least one segment per corner
  });
});
