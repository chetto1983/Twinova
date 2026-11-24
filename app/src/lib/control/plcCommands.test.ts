import { describe, expect, it, vi } from "vitest";

const backend = vi.hoisted(() => {
  const calls: unknown[][] = [];
  const log =
    (name: string, answer: unknown = undefined) =>
    (...args: unknown[]) => {
      calls.push([name, ...args]);
      return Promise.resolve(answer);
    };
  return { calls, log };
});
vi.mock("../backend", () => {
  const { log } = backend;
  return {
    debug: { resume: log("resume"), runToCursor: log("runToCursor") },
    ladder: {
      exportPlcopenTo: log("exportPlcopenTo", "/p/out.xml"),
      importPlcopenFrom: log("importPlcopenFrom", { files: [["plc/Pump.ld", "|A| (X);\n"]], notes: ["n"] }),
    },
    monitor: {
      writeValue: log("writeValue"),
      force: log("force", [{ label: "Main.run", value: "TRUE" }]),
      release: log("release", []),
    },
  };
});

import { dockCommands, plcCommands, valuesOf, type PlcContext } from "./plcCommands";
import type { Breakpoint } from "../debug";
import type { Source } from "../sources";

const MAIN: Source = { key: "plc/Main.st", label: "plc/Main.st", kind: "program", text: "A", base: "A" };
const BELT: Source = { key: "twinova.belt@1", label: "twinova.belt@1", kind: "behaviour", text: "B", base: "B" };

function pane() {
  const log: unknown[][] = [];
  let sources = [MAIN, BELT];
  const breakpoints = new Map<string, Breakpoint[]>();
  const ctx: PlcContext = {
    sources: () => sources,
    setSources: (next) => (sources = next),
    current: () => "plc/Main.st",
    open: (key) => log.push(["open", key]),
    setView: (ladder) => log.push(["view", ladder]),
    setMonitoring: (on) => log.push(["monitoring", on]),
    setTable: (on) => log.push(["table", on]),
    table: () =>
      Promise.resolve({
        declarations: [{ pou: "Main", name: "run", class: "VAR", type_name: "BOOL", address: "%QX0.0" }] as never,
        rows: null,
        values: [],
      }),
    setForces: (forces) => void log.push(["forces", forces]),
    pathOf: (key) => (key === BELT.key ? "components/twinova.belt@1.st" : key),
    setBreakpoints: (file, list) => {
      const bad = list.find((b) => b.condition === "x =");
      if (bad) return Promise.resolve(`line ${bad.line}: the condition 'x =' does not parse`);
      breakpoints.set(
        file,
        list.map((b) => ({ ...b, line: b.line + 1 })),
      );
      return Promise.resolve(null);
    },
    breakpointsOf: (file) => breakpoints.get(file) ?? [],
    hold: () => null,
  };
  const commands = new Map(plcCommands(ctx).map((c) => [c.name, c]));
  const run = async (name: string, params: Record<string, unknown> = {}) => commands.get(name)!.run(params);
  return { ctx, log, run, sources: () => sources };
}

describe("the PLC pane's commands", () => {
  it("shows a file, a view, monitoring and the table as the pane's controls do", async () => {
    const p = pane();
    const answers = [
      await p.run("open_file", { file: "twinova.belt@1" }),
      await p.run("view", { view: "ladder" }),
      await p.run("view", { view: "text" }),
      await p.run("monitoring", { on: true }),
      await p.run("variables_table", { on: false }),
    ];
    expect(answers, "a setting answers nothing, not the value it set").toEqual(Array(5).fill(undefined));
    expect(p.log).toEqual([
      ["open", "twinova.belt@1"],
      ["view", true],
      ["view", false],
      ["monitoring", true],
      ["table", false],
    ]);
    await expect(p.run("open_file", { file: "plc/Nope.st" })).rejects.toThrow("no file plc/Nope.st in the PLC");
  });

  it("reads the table, its values while monitoring, and writes, forces and releases through the value bar's way", async () => {
    const p = pane();
    expect(await p.run("values")).toEqual([{ name: "run", type: "BOOL", address: "%QX0.0", value: null }]);
    p.ctx.table = async () => ({
      declarations: [{ name: "run", type_name: "BOOL", address: "%QX0.0" }] as never,
      rows: [
        { name: "run", declaration: 0, target: null, instance: false },
        { name: "T1.Q", declaration: null, target: null, instance: false },
      ],
      values: ["TRUE"],
    });
    expect(await valuesOf(p.ctx)).toEqual([
      { name: "run", type: "BOOL", address: "%QX0.0", value: "TRUE" },
      { name: "T1.Q", type: null, address: null, value: null },
    ]);
    const run = { program: "Main", path: "run" };
    expect(await p.run("write", { target: run, value: "TRUE" })).toBeNull();
    expect(await p.run("force", { target: { address: "%QX0.0" }, value: "FALSE" })).toEqual([
      { label: "Main.run", value: "TRUE" },
    ]);
    await p.run("release", { target: run });
    expect(backend.calls.splice(0)).toEqual([
      ["writeValue", { kind: "variable", program: "Main", path: "run" }, "TRUE"],
      ["force", { kind: "address", address: "%QX0.0" }, "FALSE"],
      ["release", { kind: "variable", program: "Main", path: "run" }],
    ]);
    expect(p.log).toEqual([
      ["forces", [{ label: "Main.run", value: "TRUE" }]],
      ["forces", []],
    ]);
    await expect(p.run("write", { target: { program: "Main" }, value: "1" })).rejects.toThrow(
      "target must be { program, path } or { address }",
    );
    await expect(p.run("force", { target: run })).rejects.toThrow("value must be a string");
  });

  it("sets breakpoints in a block's file and answers them as placed; steps and runs to a line", async () => {
    const p = pane();
    expect(await p.run("breakpoints", { file: "twinova.belt@1", lines: [3, { line: 7, condition: "x" }] })).toEqual([
      { line: 4 },
      { line: 8, condition: "x" },
    ]);
    expect(await p.run("breakpoints", { lines: [] })).toEqual([]);
    await expect(p.run("breakpoints", { lines: [{ line: 5, condition: "x =" }] })).rejects.toThrow(
      "line 5: the condition 'x =' does not parse",
    );
    expect(await p.run("hold")).toBeNull();
    await p.run("resume", { how: "over" });
    await p.run("run_to_cursor", { file: "twinova.belt@1", line: 12 });
    expect(backend.calls.splice(0)).toEqual([
      ["resume", "over"],
      ["runToCursor", "components/twinova.belt@1.st", 12],
    ]);
  });

  it("exports and imports PLCopen as the pane does, without its dialogs", async () => {
    const p = pane();
    expect(await p.run("plcopen_export", { target: "/p/out.xml" })).toBe("/p/out.xml");
    expect(await p.run("plcopen_import", { source: "/p/in.xml" })).toEqual({
      replaced: [],
      added: ["plc/Pump.ld"],
      notes: ["n"],
    });
    expect(p.sources().map((s) => s.key)).toEqual(["plc/Main.st", "plc/Pump.ld", "twinova.belt@1"]);
    expect(backend.calls.splice(0)).toEqual([
      ["exportPlcopenTo", [["plc/Main.st", "A"]], "plc/Main.st", "/p/out.xml"],
      ["importPlcopenFrom", "/p/in.xml"],
    ]);
  });
});

describe("the dock's commands", () => {
  it("shows a tab, maximized or not, and moves the PLC pane to its window and back", async () => {
    const log: unknown[][] = [];
    const [show, plcWindow] = dockCommands({
      show: (panel) => log.push(["show", panel]),
      maximize: (on) => log.push(["max", on]),
      plcWindow: (open) => Promise.resolve(log.push(["window", open])),
    });
    await show!.run({ panel: "plc", maximized: true });
    await show!.run({ panel: "parts" });
    expect([await plcWindow!.run({ open: true }), await plcWindow!.run({ open: false })]).toEqual([
      undefined,
      undefined,
    ]);
    expect(log).toEqual([
      ["show", "plc"],
      ["max", true],
      ["show", "parts"],
      ["window", true],
      ["window", false],
    ]);
  });
});
