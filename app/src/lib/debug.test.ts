import { describe, expect, it } from "vitest";
import { breakpointDecorations, holdLabel, keyAction, toggleLine } from "./debug";

const key = (k: string, mods: { shift?: boolean; ctrl?: boolean; meta?: boolean } = {}) => ({
  key: k,
  shiftKey: !!mods.shift,
  ctrlKey: !!mods.ctrl,
  metaKey: !!mods.meta,
});

describe("breakpoints and HOLD in the editor", () => {
  it("toggles a line in or out, the lines kept in order", () => {
    expect(toggleLine([40, 37], 38)).toEqual([37, 38, 40]);
    expect(toggleLine([37, 38, 40], 38)).toEqual([37, 40]);
    expect(toggleLine([], 5)).toEqual([5]);
  });

  it("takes VS Code's keys", () => {
    expect(keyAction(key("F9"))).toBe("toggle");
    expect(keyAction(key("F5"))).toBe("continue");
    expect(keyAction(key("F10"))).toBe("over");
    expect(keyAction(key("F10", { ctrl: true }))).toBe("cursor");
    expect(keyAction(key("F10", { meta: true }))).toBe("cursor");
    expect(keyAction(key("F11"))).toBe("into");
    expect(keyAction(key("F11", { shift: true }))).toBe("out");
    expect(keyAction(key("F12"))).toBeNull();
    expect(keyAction(key("F5", { shift: true }))).toBeNull();
  });

  it("marks each breakpoint in the glyph margin, and the held line across", () => {
    const marks = breakpointDecorations([37, 40], 40);
    expect(marks.map((d) => [d.range.startLineNumber, d.options.glyphMarginClassName, d.options.className])).toEqual([
      [37, "debug-breakpoint", undefined],
      [40, "debug-breakpoint", undefined],
      [40, "debug-held", "debug-held-line"],
    ]);
    expect(marks[2].options.isWholeLine).toBe(true);
    expect(breakpointDecorations([], null)).toEqual([]);
  });

  it("names where the program is held", () => {
    expect(holdLabel({ file: "plc/Main.st", line: 40, frames: [] })).toBe("plc/Main.st:40");
  });
});
