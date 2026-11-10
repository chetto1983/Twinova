import { describe, expect, it } from "vitest";
import { partsBounds } from "./meshes";
import { createGeometryBuilder } from "./oip/geometry/builder";
import { sweepProfileAlongPath } from "./oip/geometry/sweep";
import { beltGeometry } from "./oip/parts/belt";

const r = (x: number) => Math.round(x * 1e4) / 1e4;
const belt = { length: 3, width: 0.6, frameHeight: 0.17, standHeight: 0.8, guards: [true, true] as const };

describe("OIP's sweep", () => {
  it("carries a section along a path, level, with a hard edge at each corner and caps that face out", () => {
    const square: [number, number][] = [
      [0, 0],
      [0, 1],
      [1, 1],
      [1, 0],
    ];
    const open = sweepProfileAlongPath(square, [
      [0, 0, 0],
      [2, 0, 0],
    ]);
    // Four sharp corners: two vertices each per ring, two rings; a quad per side.
    expect(open.positions.length / 3).toBe(16);
    expect(open.indices.length).toBe(4 * 6);
    // The section's up is the world's up and its width the path's right (T × up): [u, w] → (x, u, w).
    expect(open.positions.slice(0, 3)).toEqual([0, 0, 0]);
    expect(open.positions.slice(12, 15)).toEqual([0, 1, 1]);
    // u runs along the path, 0 to 1; v around the section, by its perimeter.
    expect([open.uvs[0], open.uvs[1], open.uvs[open.uvs.length - 2]]).toEqual([0, 0, 1]);
    const capped = sweepProfileAlongPath(
      square,
      [
        [0, 0, 0],
        [1, 0, 0],
        [3, 0, 0],
      ],
      { caps: true },
    );
    expect(capped.positions.length / 3).toBe(3 * 8 + 2 * 4);
    expect(capped.indices.length).toBe(2 * 4 * 6 + 2 * 2 * 3);
    // The start cap faces back along the path (−x), whichever way the section winds.
    for (const section of [square, square.toReversed()]) {
      const ends = sweepProfileAlongPath(
        section,
        [
          [0, 0, 0],
          [1, 0, 0],
        ],
        { caps: true },
      );
      const [a, b, c] = ends.indices.slice(-12, -9).map((i) => ends.positions.slice(i * 3, i * 3 + 3));
      const n = (b![1] - a![1]) * (c![2] - a![2]) - (b![2] - a![2]) * (c![1] - a![1]);
      expect(n).toBeLessThan(0);
    }
    expect(() =>
      sweepProfileAlongPath(square.slice(0, 2), [
        [0, 0, 0],
        [1, 0, 0],
      ]),
    ).toThrow("≥3 points");
    expect(() => sweepProfileAlongPath(square, [[0, 0, 0]])).toThrow("≥2 stations");
  });

  it("follows a turning path with its width level", () => {
    const desc = sweepProfileAlongPath(
      [
        [0, -0.1],
        [0, 0.1],
        [0.01, 0],
      ],
      [
        [0, 0, 0],
        [1, 0, 0],
        [1, 0, -1],
      ],
    );
    const ys = desc.positions.filter((_, i) => i % 3 === 1);
    expect(Math.max(...ys)).toBeCloseTo(0.01, 9);
    expect(Math.min(...ys)).toBeCloseTo(0, 9);
  });
});

/** The stand boxes a belt draws, if any. */
const stands = (length: number, standHeight: number) =>
  beltGeometry(createGeometryBuilder(), { ...belt, length, standHeight }).find((p) => p.material === "leg")?.instances
    ?.length;

describe("OIP's belt conveyor", () => {
  const parts = beltGeometry(createGeometryBuilder(), belt);

  it("draws the band, the rails and guards a side, its stands and its web bolts", () => {
    // Two stands (0.15 m in from each end), telescoping at a 0.625 m frame bottom: 10 boxes and 14 bolts each
    // (legs.js); and round(3 / 1.2) = 3 web-bolt pairs a side (JavaScript rounds 2.5 up).
    expect(parts.map((p) => [p.material, p.instances?.length, p.scroll])).toEqual([
      ["belt", undefined, 1],
      ["frame", 2, undefined],
      ["guard", 2, undefined],
      ["leg", 2 * 10, undefined],
      ["bolt", 2 * 14, undefined],
      ["bolt", 3 * 2 * 2, undefined],
    ]);
  });

  it("spans the belt and its end drums, its rails and guards, and stands on the floor", () => {
    const all = partsBounds(parts);
    // The frame and guards reach the drum radius (height / 2) past each end; the band wraps the drums exactly.
    expect([r(all.min.x), r(all.max.x)]).toEqual([-0.085, 3.085]);
    // part.js: the guard tops out at 0.15; the foot plates stand 3 mm above the floor (legs.js).
    expect([r(all.min.y), r(all.max.y)]).toEqual([-0.797, 0.15]);
    // Across, the guard's return reaches hw + 0.01 + 0.025; the foot plates under the uprights pass it.
    const foot = 0.3 + 0.01 - 0.0445 / 2 + 0.0508;
    expect(all.min.z).toBeCloseTo(-foot, 6);
    expect(all.max.z).toBeCloseTo(foot, 6);
    const band = partsBounds([parts[0]!]);
    expect([r(band.min.x), r(band.max.x), r(band.min.y), r(band.max.y)]).toEqual([-0.085, 3.085, -0.17, 0]);
    expect([r(band.min.z), r(band.max.z)]).toEqual([-0.295, 0.295]);
    const guards = partsBounds([parts[2]!]);
    expect([r(guards.min.z), r(guards.max.z)]).toEqual([-0.335, 0.335]);
  });

  it("runs its fabric in metres round the loop, and draws only the guards it has", () => {
    const uvs = parts[0]!.desc.uvs;
    const loop = uvs[uvs.length - 2]!;
    // Two 3 m runs and two drums of 8 chords over a 0.085 m radius.
    expect(loop).toBeCloseTo(6 + 2 * 16 * 0.085 * Math.sin(Math.PI / 16), 9);
    const left = beltGeometry(createGeometryBuilder(), { ...belt, guards: [true, false] });
    const guard = left.find((p) => p.material === "guard")!;
    expect(guard.instances).toHaveLength(1);
    expect(partsBounds([guard]).max.z).toBeLessThan(0);
    const none = beltGeometry(createGeometryBuilder(), { ...belt, guards: [false, false] });
    expect(none.some((p) => p.material === "guard")).toBe(false);
  });

  it("stands a short belt on one post and a tall one on knee-braced stands, none when too low", () => {
    expect(stands(0.35, 0.8)).toBe(10);
    expect(stands(3, 1.6)).toBe(2 * 12);
    expect(stands(3, 0.2)).toBeUndefined();
  });
});
