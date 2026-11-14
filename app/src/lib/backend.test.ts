import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, listen, open, save } = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
  open: vi.fn(),
  save: vi.fn(),
}));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open, save }));

import {
  analysis,
  backend,
  debug,
  declarations,
  ladder,
  monitor,
  navigation,
  onDiskSources,
  onEngineError,
  onMonitor,
  onSnapshot,
  onStopped,
  projects,
} from "./backend";

describe("backend bridge", () => {
  beforeEach(() => {
    invoke.mockReset().mockResolvedValue(undefined);
    listen.mockReset().mockResolvedValue(() => {});
  });

  it("calls the Rust commands registered in lib.rs with their argument names", async () => {
    await backend.build([["plc/Main.st", "PROGRAM Main END_PROGRAM"]], { "twinova.belt@1": "FUNCTION_BLOCK B" });
    await backend.changeProgram([["plc/Main.st", "PROGRAM Main END_PROGRAM"]]);
    await backend.reset();
    await backend.step();
    await backend.run(2);
    await backend.run(null);
    await backend.saveState();
    await backend.restoreState();
    await backend.pause();
    await backend.press("%IX1.0");
    await backend.setSinkBlocked(0, true);
    await backend.scanTiming();
    expect(invoke.mock.calls).toEqual([
      [
        "build",
        {
          program: [["plc/Main.st", "PROGRAM Main END_PROGRAM"]],
          behaviours: { "twinova.belt@1": "FUNCTION_BLOCK B" },
        },
      ],
      ["change_program", { program: [["plc/Main.st", "PROGRAM Main END_PROGRAM"]] }],
      ["reset"],
      ["step"],
      ["run", { speed: 2 }],
      ["run", { speed: null }],
      ["save_state"],
      ["restore_state"],
      ["pause"],
      ["press", { address: "%IX1.0" }],
      ["set_sink_blocked", { sink: 0, blocked: true }],
      ["scan_timing"],
    ]);
  });

  it("calls the navigation commands with their argument names", async () => {
    const files: [string, string][] = [["plc/Main.st", "PROGRAM Main"]];
    await navigation.definition("plc/Main.st", "PROGRAM Main", 3, 5);
    await navigation.references("plc/Main.st", "PROGRAM Main", 3, 5);
    await navigation.rename(files, "plc/Main.st", 3, 5, "cartons");
    await navigation.callTemplate(files, "plc/Main.st", 4, 7);
    await navigation.declareProposal(files, "plc/Main.st", 5, 1);
    expect(invoke.mock.calls).toEqual([
      ["definition", { file: "plc/Main.st", text: "PROGRAM Main", line: 3, column: 5 }],
      ["references", { file: "plc/Main.st", text: "PROGRAM Main", line: 3, column: 5 }],
      ["rename_symbol", { files, file: "plc/Main.st", line: 3, column: 5, newName: "cartons" }],
      ["call_template", { files, file: "plc/Main.st", line: 4, column: 7 }],
      ["declare_proposal", { files, file: "plc/Main.st", line: 5, column: 1 }],
    ]);
  });

  it("calls the analysis commands of analysis.rs with their argument names", async () => {
    await analysis.analyze([["plc/Main.st", "PROGRAM Main END_PROGRAM"]], { "twinova.belt@1": "FUNCTION_BLOCK B" });
    await analysis.hover("plc/Main.st", "PROGRAM Main", 3, 5);
    await analysis.completion("plc/Main.st", "PROGRAM M", 4, 9);
    expect(invoke.mock.calls).toEqual([
      [
        "analyze",
        {
          program: [["plc/Main.st", "PROGRAM Main END_PROGRAM"]],
          behaviours: { "twinova.belt@1": "FUNCTION_BLOCK B" },
        },
      ],
      ["hover", { file: "plc/Main.st", text: "PROGRAM Main", line: 3, column: 5 }],
      ["completion", { file: "plc/Main.st", text: "PROGRAM M", line: 4, column: 9 }],
    ]);
  });

  it("calls the monitoring and declaration commands with their argument names", async () => {
    const target = { kind: "variable", program: "Main", path: "count" } as const;
    const row = {
      pou: "Main",
      name: "x",
      class: "VAR",
      qualifier: null,
      type_name: "INT",
      address: null,
      initial: null,
      comment: null,
    };
    await monitor.watchRange("plc/Main.st", 3, 9, null, ["carry"]);
    await monitor.unwatch();
    await monitor.watched();
    await monitor.forces();
    await monitor.force(target, "5");
    await monitor.release(target);
    await monitor.writeValue(target, "6");
    await declarations.of("plc/Main.st", "PROGRAM Main END_PROGRAM");
    await declarations.edit("plc/Main.ld", "T", { op: "add", row });
    await declarations.rename([["plc/Main.st", "T"]], "plc/Main.st", 2, "y");
    await debug.setBreakpoints("plc/Main.st", [{ line: 37, condition: "count = 2" }]);
    await debug.runToCursor("plc/Main.st", 40);
    await debug.resume("over");
    await debug.hold();
    await debug.logs();
    await ladder.parse("|A| (B);");
    await ladder.print({ declarations: "", rungs: [] });
    await ladder.complete("plc/Main.ld", "|A| (B);", "St");
    await ladder.lowering([["plc/Main.ld", "|A| (B);"]], "plc/Main.ld");
    expect(invoke.mock.calls).toEqual([
      ["watch_range", { file: "plc/Main.st", first: 3, last: 9, device: null, unfolded: ["carry"] }],
      ["unwatch"],
      ["watched"],
      ["forces"],
      ["force", { target, text: "5" }],
      ["release", { target }],
      ["write_value", { target, text: "6" }],
      ["declarations_of", { file: "plc/Main.st", text: "PROGRAM Main END_PROGRAM" }],
      ["edit_declaration", { file: "plc/Main.ld", text: "T", change: { op: "add", row } }],
      ["rename_declaration", { files: [["plc/Main.st", "T"]], file: "plc/Main.st", index: 2, newName: "y" }],
      ["set_breakpoints", { file: "plc/Main.st", breakpoints: [{ line: 37, condition: "count = 2" }] }],
      ["run_to_cursor", { file: "plc/Main.st", line: 40 }],
      ["resume", { how: "over" }],
      ["hold"],
      ["debug_logs"],
      ["ladder_parse", { text: "|A| (B);" }],
      ["ladder_print", { program: { declarations: "", rungs: [] } }],
      ["ladder_completion", { file: "plc/Main.ld", text: "|A| (B);", typed: "St" }],
      ["ladder_lowering", { program: [["plc/Main.ld", "|A| (B);"]], file: "plc/Main.ld" }],
    ]);
  });

  it("hands each event's payload to its listener", async () => {
    const seen: unknown[] = [];
    await onSnapshot((s) => seen.push(s));
    await onEngineError((e) => seen.push(e));
    await onStopped(() => seen.push("stopped"));
    await onDiskSources((d) => seen.push(d));
    await onMonitor((v) => seen.push(v));
    expect(listen.mock.calls.map((c) => c[0])).toEqual([
      "snapshot",
      "engine-error",
      "stopped",
      "disk-sources",
      "monitor",
    ]);
    const disk = { kind: "program", program: [], behaviours: {} };
    for (const [i, payload] of [{ time_ms: 10 }, "boom", null, disk, ["3", null]].entries()) {
      listen.mock.calls[i][1]({ payload });
    }
    expect(seen).toEqual([{ time_ms: 10 }, "boom", "stopped", disk, ["3", null]]);
  });
});

describe("project bridge", () => {
  const ARCHIVE = [{ name: "Twinova project", extensions: ["zip"] }];
  beforeEach(() => {
    invoke.mockReset().mockResolvedValue({ name: "view" });
    open.mockReset();
    save.mockReset();
  });

  it("asks for a folder or a file, then calls the project commands with it", async () => {
    open.mockResolvedValueOnce("/p/line").mockResolvedValueOnce("/p/copy").mockResolvedValueOnce("/p/line.zip");
    save.mockResolvedValueOnce("/p/out.zip");
    expect(await projects.open()).toEqual({ name: "view" });
    expect(await projects.saveAs()).toEqual({ name: "view" });
    expect(await projects.importArchive()).toEqual({ name: "view" });
    expect(await projects.exportArchive()).toBe("/p/out.zip");
    await projects.save("/p/line");
    expect(await projects.launch()).toEqual({ name: "view" });
    expect(await projects.current()).toEqual({ name: "view" });
    expect(await projects.demo()).toEqual({ name: "view" });
    expect(open.mock.calls).toEqual([[{ directory: true }], [{ directory: true }], [{ filters: ARCHIVE }]]);
    expect(save.mock.calls).toEqual([[{ filters: ARCHIVE }]]);
    expect(invoke.mock.calls).toEqual([
      ["open_project", { folder: "/p/line" }],
      ["save_project", { folder: "/p/copy" }],
      ["import_project", { file: "/p/line.zip" }],
      ["export_project", { file: "/p/out.zip" }],
      ["save_project", { folder: "/p/line" }],
      ["open_launch_project"],
      ["current_project"],
      ["open_demo"],
    ]);
  });

  it("moves the line only once the breakpoints set before have reached the app, even refused", async () => {
    const pending: { refuse?: (e: string) => void } = {};
    invoke.mockImplementation((cmd: string) =>
      cmd === "set_breakpoints" ? new Promise((_, reject) => (pending.refuse = reject)) : Promise.resolve(undefined),
    );
    const set = debug.setBreakpoints("plc/Main.st", []);
    const moves = [backend.run(null), backend.step(), debug.resume("continue"), debug.runToCursor("plc/Main.st", 3)];
    await Promise.resolve();
    expect(invoke.mock.calls.map((c) => c[0])).toEqual(["set_breakpoints"]);
    pending.refuse?.("line 3: the condition 'x' is not a variable");
    await expect(set).rejects.toBe("line 3: the condition 'x' is not a variable");
    await Promise.all(moves);
    expect(invoke.mock.calls.map((c) => c[0])).toEqual(["set_breakpoints", "run", "step", "resume", "run_to_cursor"]);
  });

  it("does nothing when a dialog is cancelled", async () => {
    open.mockResolvedValue(null);
    save.mockResolvedValue(null);
    expect(await projects.open()).toBeNull();
    expect(await projects.saveAs()).toBeNull();
    expect(await projects.importArchive()).toBeNull();
    expect(await projects.exportArchive()).toBeNull();
    expect(invoke).not.toHaveBeenCalled();
  });
});
