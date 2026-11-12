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

import { analysis, backend, onDiskSources, onEngineError, onSnapshot, onStopped, projects } from "./backend";

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

  it("hands each event's payload to its listener", async () => {
    const seen: unknown[] = [];
    await onSnapshot((s) => seen.push(s));
    await onEngineError((e) => seen.push(e));
    await onStopped(() => seen.push("stopped"));
    await onDiskSources((d) => seen.push(d));
    expect(listen.mock.calls.map((c) => c[0])).toEqual(["snapshot", "engine-error", "stopped", "disk-sources"]);
    const disk = { kind: "program", program: [], behaviours: {} };
    for (const [i, payload] of [{ time_ms: 10 }, "boom", null, disk].entries()) {
      listen.mock.calls[i][1]({ payload });
    }
    expect(seen).toEqual([{ time_ms: 10 }, "boom", "stopped", disk]);
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
