import { describe, expect, it } from "vitest";
import { instanceOf, isClick, picked, pose, type Node } from "./picking";

const node = (userData: Record<string, unknown>, parent: Node | null = null): Node => ({ userData, parent });

describe("picking", () => {
  const placed = node({ instance: "belt_1" }, node({}));
  const mesh = node({}, node({}, placed));

  it("finds the instance a hit mesh belongs to through its ancestors", () => {
    expect(instanceOf(mesh)).toBe("belt_1");
    expect(instanceOf(placed)).toBe("belt_1");
    expect(instanceOf(node({ instance: 7 }))).toBeNull();
    expect(instanceOf(null)).toBeNull();
  });

  it("takes the first hit that is an instance's, and none on the floor", () => {
    const floor = node({});
    expect(picked([{ object: floor }, { object: mesh }])).toBe("belt_1");
    expect(picked([{ object: floor }])).toBeNull();
    expect(picked([])).toBeNull();
  });

  it("rounds a gizmo's pose to the millimetre", () => {
    const rounded = Math.round(Math.SQRT1_2 * 1e9) / 1e9;
    expect(pose([1.00049, -0.0001, 2.5], [0, Math.SQRT1_2, 0, Math.SQRT1_2])).toEqual({
      position: [1, 0, 2.5],
      rotation: [0, rounded, 0, rounded],
    });
    expect(rounded).not.toBe(Math.SQRT1_2);
  });

  it("tells a click from a drag", () => {
    expect(isClick({ x: 10, y: 10 }, { x: 13, y: 12 })).toBe(true);
    expect(isClick({ x: 10, y: 10 }, { x: 15, y: 10 })).toBe(false);
  });
});
