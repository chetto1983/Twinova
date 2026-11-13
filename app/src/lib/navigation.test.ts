import { describe, expect, it } from "vitest";
import { applyChanged, declaration, isSnippet, templateText } from "./navigation";
import type { Source } from "./sources";

const tab = (file: string, text: string): Source => ({ key: file, label: file, kind: "program", text, base: text });

describe("completion and navigation (spec 2026-11-13-completion-navigation-design.md)", () => {
  it("closes a call's template unless the editor already closed the bracket", () => {
    expect(templateText("IN := ${1}, PT := ${2}", ")")).toBe("IN := ${1}, PT := ${2}");
    expect(templateText("IN := ${1}, PT := ${2}", "")).toBe("IN := ${1}, PT := ${2})");
    expect(templateText("", ";")).toBe(")");
  });

  it("tells truST's snippets from plain words", () => {
    expect(isSnippet("IN := $0")).toBe(true);
    expect(isSnippet("BOOL#${1:TRUE}")).toBe(true);
    expect(isSnippet("count")).toBe(false);
    expect(isSnippet("costs $")).toBe(false);
  });

  it("makes a table row of Auto Declare's fields, an empty one left out", () => {
    const row = declaration({ name: "speed", pou: "Main", type_name: "INT" }, "VAR", "INT", "3", "");
    expect(row).toEqual({
      pou: "Main",
      name: "speed",
      class: "VAR",
      qualifier: null,
      type_name: "INT",
      address: null,
      initial: "3",
      comment: null,
    });
  });

  it("puts a rename's texts in their tabs and leaves the others", () => {
    const sources = [tab("plc/Main.st", "a"), tab("plc/Other.st", "b")];
    applyChanged(sources, [
      ["plc/Main.st", "A"],
      ["plc/Gone.st", "x"],
    ]);
    expect(sources.map((s) => s.text)).toEqual(["A", "b"]);
  });
});
