import { describe, expect, it } from "vitest";
import { shortcutOf, typing } from "./shortcuts";

const key = (k: string, init: KeyboardEventInit = {}, target?: HTMLElement): KeyboardEvent => {
  const event = new KeyboardEvent("keydown", { key: k, ...init });
  if (target) Object.defineProperty(event, "target", { value: target });
  return event;
};

describe("the editor's keys", () => {
  it("undo, redo, remove, move and turn", () => {
    expect(shortcutOf(key("z", { ctrlKey: true }))).toBe("undo");
    expect(shortcutOf(key("Z", { ctrlKey: true, shiftKey: true }))).toBe("redo");
    expect(shortcutOf(key("y", { ctrlKey: true }))).toBe("redo");
    expect(shortcutOf(key("z", { metaKey: true }))).toBe("undo");
    expect(shortcutOf(key("Delete"))).toBe("remove");
    expect(shortcutOf(key("Backspace"))).toBe("remove");
    expect(shortcutOf(key("w"))).toBe("translate");
    expect(shortcutOf(key("E"))).toBe("rotate");
  });

  it("nothing else, and nothing with Alt or a modifier on a plain key", () => {
    expect(shortcutOf(key("x"))).toBeNull();
    expect(shortcutOf(key("s", { ctrlKey: true }))).toBeNull();
    expect(shortcutOf(key("w", { altKey: true }))).toBeNull();
    expect(shortcutOf(key("Delete", { altKey: true }))).toBeNull();
  });

  it("nothing while the focus is where text is typed", () => {
    for (const tag of ["input", "textarea", "select"]) {
      const field = document.createElement(tag);
      expect(typing(field)).toBe(true);
      expect(shortcutOf(key("z", { ctrlKey: true }, field))).toBeNull();
      expect(shortcutOf(key("Delete", {}, field))).toBeNull();
    }
    const editable = document.createElement("div");
    Object.defineProperty(editable, "isContentEditable", { value: true });
    expect(typing(editable)).toBe(true);
    expect(typing(document.createElement("button"))).toBe(false);
    expect(typing(null)).toBe(false);
  });
});
