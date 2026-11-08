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

import { backend, onEngineError, onSnapshot, onStopped, projects } from "./backend";

describe("backend bridge", () => {
  beforeEach(() => {
    invoke.mockReset().mockResolvedValue(undefined);
    listen.mockReset().mockResolvedValue(() => {});
  });

  it("calls the Rust commands registered in lib.rs with their argument names", async () => {
    await backend.defaultProgram();
    await backend.build("PROGRAM Main END_PROGRAM");
    await backend.reset();
    await backend.step();
    await backend.run(2);
    await backend.run(null);
    await backend.saveState();
    await backend.restoreState();
    await backend.pause();
    await backend.press(true, false);
    await backend.setOutletBlocked(true);
    expect(invoke.mock.calls).toEqual([
      ["default_program"],
      ["build", { source: "PROGRAM Main END_PROGRAM" }],
      ["reset"],
      ["step"],
      ["run", { speed: 2 }],
      ["run", { speed: null }],
      ["save_state"],
      ["restore_state"],
      ["pause"],
      ["press", { start: true, stop: false }],
      ["set_outlet_blocked", { blocked: true }],
    ]);
  });

  it("hands each event's payload to its listener", async () => {
    const seen: unknown[] = [];
    await onSnapshot((s) => seen.push(s));
    await onEngineError((e) => seen.push(e));
    await onStopped(() => seen.push("stopped"));
    expect(listen.mock.calls.map((c) => c[0])).toEqual(["snapshot", "engine-error", "stopped"]);
    for (const [i, payload] of [{ time_ms: 10 }, "boom", null].entries()) {
      listen.mock.calls[i][1]({ payload });
    }
    expect(seen).toEqual([{ time_ms: 10 }, "boom", "stopped"]);
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
    expect(open.mock.calls).toEqual([[{ directory: true }], [{ directory: true }], [{ filters: ARCHIVE }]]);
    expect(save.mock.calls).toEqual([[{ filters: ARCHIVE }]]);
    expect(invoke.mock.calls).toEqual([
      ["open_project", { folder: "/p/line" }],
      ["save_project", { folder: "/p/copy" }],
      ["import_project", { file: "/p/line.zip" }],
      ["export_project", { file: "/p/out.zip" }],
      ["save_project", { folder: "/p/line" }],
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
