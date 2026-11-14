import { describe, expect, it } from "vitest";
import { History } from "./history";

describe("the Ladder view's undo and redo", () => {
  it("goes back and forth over the texts the edits replaced", () => {
    const h = new History();
    expect([h.canUndo, h.canRedo, h.undo("a"), h.redo("a")]).toEqual([false, false, null, null]);
    h.record("a");
    h.record("b");
    expect(h.undo("c")).toBe("b");
    expect(h.undo("b")).toBe("a");
    expect([h.canUndo, h.canRedo]).toEqual([false, true]);
    expect(h.redo("a")).toBe("b");
    expect(h.redo("b")).toBe("c");
    expect(h.canRedo).toBe(false);
  });

  it("forgets the redo on a new edit, and keeps the last hundred edits", () => {
    const h = new History();
    h.record("a");
    h.undo("b");
    h.record("a");
    expect(h.canRedo).toBe(false);
    for (let i = 0; i < 150; i++) h.record(String(i));
    let oldest = "";
    for (let text = h.undo("x"); text !== null; text = h.undo(text)) oldest = text;
    expect(oldest).toBe("50");
  });
});
