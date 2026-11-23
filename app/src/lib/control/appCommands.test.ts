import { beforeEach, describe, expect, it, vi } from "vitest";

const backend = vi.hoisted(() => {
  const state = {
    calls: [] as unknown[][],
    view: { name: "line", folder: "/p/line", instances: [], can_undo: false, can_redo: false },
  };
  const answer =
    (name: string) =>
    (...args: unknown[]) => {
      state.calls.push([name, ...args]);
      return Promise.resolve(state.view);
    };
  return Object.assign(state, { answer });
});
vi.mock("../backend", () => ({
  projects: {
    create: backend.answer("create"),
    examples: () => Promise.resolve([{ id: "demo_m3", name: "M3 demo" }]),
    example: backend.answer("example"),
    openFolder: backend.answer("openFolder"),
    save: backend.answer("save"),
    importFrom: backend.answer("importFrom"),
    exportTo: (file: string) => {
      backend.calls.push(["exportTo", file]);
      return Promise.resolve(file);
    },
  },
}));

import { appCommands, stateOf, type AppContext } from "./appCommands";
import { currentLocale } from "../i18n.svelte";
import type { Source } from "../sources";
import type { ProjectView } from "../project";

const MAIN: Source = { key: "plc/Main.st", label: "plc/Main.st", kind: "program", text: "A", base: "A" };

/** A window whose actions are logged, its error set by `fail`. */
function windowOf() {
  const log: unknown[][] = [];
  let error = "";
  let sources = [MAIN];
  const did =
    (name: string) =>
    (...args: unknown[]) => {
      log.push([name, ...args]);
      return Promise.resolve();
    };
  const ctx: AppContext = {
    project: () => backend.view as unknown as ProjectView,
    snapshot: () => null,
    sources: () => sources,
    setSources: (next) => (sources = next),
    status: () => "ok",
    running: () => false,
    warnings: () => ["w"],
    notice: () => "n",
    error: () => error,
    show: did("show"),
    build: did("build"),
    run: did("run"),
    pause: did("pause"),
    step: did("step"),
    reset: did("reset"),
    saveState: did("saveState"),
    restoreState: did("restoreState"),
    edit: (op) => {
      log.push(["edit", op]);
      return Promise.resolve(!error);
    },
    undo: did("undo"),
    redo: did("redo"),
    select: (id) => void log.push(["select", id]),
    press: did("press"),
    blockSink: did("blockSink"),
    theme: (t) => void log.push(["theme", t]),
  };
  const commands = new Map(appCommands(ctx).map((c) => [c.name, c]));
  const run = async (name: string, params: Record<string, unknown> = {}) => commands.get(name)!.run(params);
  return { ctx, log, run, fail: (text: string) => (error = text), sources: () => sources };
}

describe("the main window's commands", () => {
  beforeEach(() => {
    backend.calls = [];
  });

  it("runs the window's own actions for the line and the scene", async () => {
    const w = windowOf();
    expect(await w.run("build")).toEqual({ build: "ok", warnings: ["w"] });
    await w.run("run", { speed: null });
    await w.run("run");
    expect(await w.run("step")).toEqual({ time_ms: null });
    w.ctx.snapshot = () => ({ time_ms: 40 }) as never;
    expect(await w.run("reset")).toEqual({ time_ms: 40 });
    expect(await w.run("restore_state")).toEqual({ time_ms: 40 });
    await ["pause", "step", "reset", "save_state", "restore_state", "undo", "redo"].reduce<Promise<unknown>>(
      (done, name) => done.then(() => w.run(name)),
      Promise.resolve(),
    );
    await w.run("edit", { op: { op: "remove", id: "belt" } });
    await w.run("select", { id: "belt" });
    await w.run("select", {});
    await w.run("press", { address: "%IX1.0" });
    await w.run("block_sink", { sink: 1, blocked: true });
    await w.run("theme", { theme: "light" });
    await w.run("theme", { theme: "other" });
    await w.run("language", { language: "it" });
    expect(currentLocale()).toBe("it");
    await w.run("language", { language: "en" });
    expect(currentLocale()).toBe("en");
    await expect(w.run("language", { language: "fr" })).rejects.toThrow("no language fr: en, it");
    expect(w.log).toEqual([
      ["build"],
      ["run", null],
      ["run", 1],
      ["step"],
      ["reset"],
      ["restoreState"],
      ["pause"],
      ["step"],
      ["reset"],
      ["saveState"],
      ["restoreState"],
      ["undo"],
      ["redo"],
      ["edit", { op: "remove", id: "belt" }],
      ["select", "belt"],
      ["select", null],
      ["press", "%IX1.0"],
      ["blockSink", 1, true],
      ["theme", "light"],
      ["theme", "dark"],
    ]);
  });

  it("fails a command with the error the window shows, and an edit refused with its reason", async () => {
    const w = windowOf();
    w.fail("build the project first");
    await expect(w.run("step")).rejects.toThrow("build the project first");
    await expect(w.run("edit", { op: { op: "remove", id: "x" } })).rejects.toThrow("build the project first");
    const refused = windowOf();
    refused.ctx.edit = () => Promise.resolve(false);
    const commands = new Map(appCommands(refused.ctx).map((c) => [c.name, c]));
    await expect(commands.get("edit")!.run({ op: {} })).rejects.toThrow("the edit was refused (a locked instance?)");
  });

  it("opens projects as the project menu does, without its dialogs, and shows each", async () => {
    const w = windowOf();
    await w.run("new_project");
    expect(await w.run("list_examples")).toEqual([{ id: "demo_m3", name: "M3 demo" }]);
    await w.run("open_example", { id: "sort_line" });
    await w.run("open_project", { folder: "/p/line" });
    await w.run("save_project", { folder: "/p/copy" });
    await w.run("save_project");
    await w.run("import_archive", { file: "/p/line.zip" });
    expect(await w.run("export_archive", { file: "/p/out.zip" })).toBe("/p/out.zip");
    expect(backend.calls).toEqual([
      ["create"],
      ["example", "sort_line"],
      ["openFolder", "/p/line"],
      ["save", "/p/copy"],
      ["save", "/p/line"],
      ["importFrom", "/p/line.zip"],
      ["exportTo", "/p/out.zip"],
    ]);
    expect(w.log.filter(([name]) => name === "show")).toHaveLength(6);
    backend.view = { ...backend.view, folder: null as unknown as string };
    await expect(w.run("save_project")).rejects.toThrow("the project has no folder yet: give one");
    backend.view = { ...backend.view, folder: "/p/line" };
  });

  it("reads and writes the editor's sources, which run at the next build", async () => {
    const w = windowOf();
    expect(await w.run("read_source", { file: "plc/Main.st" })).toBe("A");
    await w.run("write_source", { file: "plc/Main.st", text: "B" });
    expect(w.sources()).toEqual([{ ...MAIN, text: "B" }]);
    expect(stateOf(w.ctx).sources).toEqual([{ file: "plc/Main.st", kind: "program", edited: true }]);
    await expect(w.run("read_source", { file: "plc/Nope.st" })).rejects.toThrow(
      "plc/Nope.st is not a source of the project",
    );
    await expect(w.run("write_source", { file: "plc/Main.st" })).rejects.toThrow("text must be a string");
  });

  it("answers the window's state", async () => {
    const w = windowOf();
    expect(await w.run("state")).toEqual({
      project: { name: "line", folder: "/p/line", instances: [], can_undo: false, can_redo: false },
      build: "ok",
      running: false,
      snapshot: null,
      sources: [{ file: "plc/Main.st", kind: "program", edited: false }],
      warnings: ["w"],
      notice: "n",
      error: "",
    });
    w.ctx.project = () => null;
    expect(stateOf(w.ctx).project).toBeNull();
  });
});
