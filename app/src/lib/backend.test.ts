import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, listen } = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));

import { backend, onEngineError, onSnapshot, onStopped } from "./backend";

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
