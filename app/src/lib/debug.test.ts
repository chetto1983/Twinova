import { describe, expect, it } from "vitest";
import {
  breakpointMarks,
  heldMark,
  holdLabel,
  keyAction,
  kindOf,
  moved,
  replaced,
  sameLines,
  toggle,
  trimmed,
} from "./debug";

const key = (k: string, mods: { shift?: boolean; ctrl?: boolean; meta?: boolean } = {}) => ({
  key: k,
  shiftKey: !!mods.shift,
  ctrlKey: !!mods.ctrl,
  metaKey: !!mods.meta,
});

describe("breakpoints and HOLD in the editor", () => {
  it("toggles a plain breakpoint in or out, in order", () => {
    expect(toggle([{ line: 40 }, { line: 37 }], 38)).toEqual([{ line: 37 }, { line: 38 }, { line: 40 }]);
    expect(toggle([{ line: 37 }, { line: 38, condition: "x" }], 38)).toEqual([{ line: 37 }]);
    expect(toggle([], 5)).toEqual([{ line: 5 }]);
  });

  it("replaces the breakpoint on a line, and sends only the fields set", () => {
    expect(replaced([{ line: 40 }, { line: 37 }], { line: 40, log: "x" })).toEqual([
      { line: 37 },
      { line: 40, log: "x" },
    ]);
    expect(trimmed({ line: 3, condition: "  ", hit: { op: "equal", count: 0 }, log: "" })).toEqual({ line: 3 });
    expect(trimmed({ line: 3, condition: " a > 1 ", hit: { op: "at_least", count: 2 }, log: "v={a}" })).toEqual({
      line: 3,
      condition: "a > 1",
      hit: { op: "at_least", count: 2 },
      log: "v={a}",
    });
  });

  it("takes VS Code's keys", () => {
    expect(keyAction(key("F9"))).toBe("toggle");
    expect(keyAction(key("F9", { shift: true }))).toBe("edit");
    expect(keyAction(key("F5"))).toBe("continue");
    expect(keyAction(key("F10"))).toBe("over");
    expect(keyAction(key("F10", { ctrl: true }))).toBe("cursor");
    expect(keyAction(key("F10", { meta: true }))).toBe("cursor");
    expect(keyAction(key("F11"))).toBe("into");
    expect(keyAction(key("F11", { shift: true }))).toBe("out");
    expect(keyAction(key("F12"))).toBeNull();
    expect(keyAction(key("F5", { shift: true }))).toBeNull();
  });

  it("marks each breakpoint by its kind, and the held line across", () => {
    expect(kindOf({ line: 1 })).toBe("debug-breakpoint");
    expect(kindOf({ line: 1, condition: "x" })).toBe("debug-breakpoint-conditional");
    expect(kindOf({ line: 1, hit: { op: "equal", count: 3 } })).toBe("debug-breakpoint-conditional");
    expect(kindOf({ line: 1, log: "x", condition: "y" })).toBe("debug-logpoint");
    const marks = breakpointMarks([{ line: 37 }, { line: 40, log: "x" }], 3);
    expect(marks.map((d) => [d.range.startLineNumber, d.options.glyphMarginClassName, d.options.stickiness])).toEqual([
      [37, "debug-breakpoint", 3],
      [40, "debug-logpoint", 3],
    ]);
    expect(heldMark(40)).toEqual([
      {
        range: { startLineNumber: 40, startColumn: 1, endLineNumber: 40, endColumn: 1 },
        options: { glyphMarginClassName: "debug-held", className: "debug-held-line", isWholeLine: true },
      },
    ]);
    expect(heldMark(null)).toEqual([]);
  });

  it("moves each breakpoint with its mark, keeping its kind", () => {
    const list = [{ line: 37 }, { line: 40, condition: "c" }];
    expect(moved(list, [{ startLineNumber: 38 }, { startLineNumber: 41 }])).toEqual([
      { line: 38 },
      { line: 41, condition: "c" },
    ]);
    expect(moved(list, [{ startLineNumber: 38 }, { startLineNumber: 38 }]), "joined lines keep the first").toEqual([
      { line: 38 },
    ]);
    expect(moved(list, [{ startLineNumber: 39 }]), "a mark gone leaves its line").toEqual([
      { line: 39 },
      { line: 40, condition: "c" },
    ]);
  });

  it("tells lists apart by their lines", () => {
    expect(sameLines([{ line: 1 }, { line: 2, log: "x" }], [{ line: 1, condition: "c" }, { line: 2 }])).toBe(true);
    expect(sameLines([{ line: 1 }], [{ line: 2 }])).toBe(false);
    expect(sameLines([{ line: 1 }], [{ line: 1 }, { line: 2 }])).toBe(false);
  });

  it("names where the program is held, and the device in a block", () => {
    expect(holdLabel({ file: "plc/Main.st", line: 40, frames: [] })).toBe("plc/Main.st:40");
    expect(holdLabel({ file: "components/b.st", line: 20, frames: [], device: "chain_transfer" })).toBe(
      "components/b.st:20 (chain_transfer)",
    );
  });
});
