import { afterEach, describe, expect, it, vi } from "vitest";
import { completionKind, debounce, fileOf, listed, marker, tally, type Problem, type Problems } from "./analysis";
import type { Source } from "./sources";

const source = (key: string, kind: Source["kind"]): Source => ({ key, label: key, kind, text: "", base: "" });
const main = source("plc/Main.st", "program");
const belt = source("twinova.belt@1", "behaviour");

const problem = (line: number, column: number, code = "W001", severity = "warning"): Problem => ({
  severity,
  code,
  message: `${code} at ${line}:${column}`,
  line,
  column,
  end_line: line,
  end_column: column + 4,
});

describe("the editor's analysis", () => {
  afterEach(() => vi.useRealTimers());

  it("names a program tab by its path and a behaviour by its block file", () => {
    expect(fileOf(main)).toBe("plc/Main.st");
    expect(fileOf(belt)).toBe("components/twinova.belt@1.st");
  });

  it("turns a problem into a Monaco marker with its severity", () => {
    expect(marker(problem(21, 1, "E203", "error"))).toEqual({
      severity: 8,
      code: "E203",
      message: "E203 at 21:1",
      source: "truST",
      startLineNumber: 21,
      startColumn: 1,
      endLineNumber: 21,
      endColumn: 5,
    });
    expect([marker(problem(1, 1, "W", "warning")).severity, marker(problem(1, 1, "I", "info")).severity]).toEqual([
      4, 2,
    ]);
    expect(marker(problem(1, 1, "H", "hint")).severity).toBe(1);
    expect(marker(problem(1, 1, "X", "unknown")).severity).toBe(2);
  });

  it("lists the PLC files' problems in the tabs' order, by line and column, and no behaviour's", () => {
    const other = source("plc/Other.st", "program");
    const problems: Problems = {
      "components/twinova.belt@1.st": [problem(34, 3, "W013")],
      "plc/Main.st": [problem(16, 5), problem(9, 7), problem(9, 5)],
      "plc/Other.st": [problem(2, 1)],
      "plc/Gone.st": [problem(1, 1)],
    };
    const rows = listed(problems, [main, belt, other]);
    expect(rows.map((r) => `${r.key} ${r.line}:${r.column}`)).toEqual([
      "plc/Main.st 9:5",
      "plc/Main.st 9:7",
      "plc/Main.st 16:5",
      "plc/Other.st 2:1",
    ]);
    expect(rows[3].file).toBe("plc/Other.st");
    expect(listed({}, [main])).toEqual([]);
  });

  it("counts the errors and the hints apart from the warnings", () => {
    const rows = listed(
      { "plc/Main.st": [problem(1, 1, "E101", "error"), problem(2, 1, "W001", "hint"), problem(3, 1), problem(4, 1)] },
      [main],
    );
    expect(tally(rows)).toEqual({ errors: 1, hints: 1 });
    expect(tally([])).toEqual({ errors: 0, hints: 0 });
  });

  it("maps the engine's completion kinds onto Monaco's", () => {
    const kinds = { Keyword: 17, Function: 1, Class: 5, Method: 0, Property: 9, Variable: 4, Constant: 14 };
    const more = { Struct: 6, EnumMember: 16, Module: 8, Snippet: 27, Text: 18 };
    const all = { ...kinds, ...more };
    const names = ["keyword", "function", "function_block", "method", "property", "variable", "constant"];
    expect(names.map((k) => completionKind(k, all))).toEqual([17, 1, 5, 0, 9, 4, 14]);
    expect(["type", "enum_value", "namespace", "snippet", "other"].map((k) => completionKind(k, all))).toEqual([
      6, 16, 8, 27, 18,
    ]);
  });

  it("runs once, after the last call of a burst, and not after cancel", () => {
    vi.useFakeTimers();
    const run = vi.fn();
    const later = debounce(run, 300);
    later.call(1);
    vi.advanceTimersByTime(200);
    later.call(2);
    vi.advanceTimersByTime(299);
    expect(run).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(run).toHaveBeenCalledExactlyOnceWith(2);
    later.call(3);
    later.cancel();
    vi.advanceTimersByTime(1000);
    expect(run).toHaveBeenCalledTimes(1);
  });
});
