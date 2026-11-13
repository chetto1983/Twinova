import { beforeEach, describe, expect, it, vi } from "vitest";

const { invoke, listen, emitTo } = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn(), emitTo: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen, emitTo }));

import { Inbox, Outbox, hostPlcWindow, plcWindow, type PlcState } from "./plcWindow";
import type { Source } from "./sources";

const text = (t: string): Source[] => [{ key: "plc/Main.st", label: "plc/Main.st", kind: "program", text: t, base: t }];
const state = (ack: number, t = "a"): PlcState => ({
  sources: text(t),
  status: "ok",
  problems: {},
  selected: null,
  instances: [],
  running: false,
  ready: true,
  ack,
});

describe("the numbering between the windows", () => {
  it("lets the window take main's texts only once main has taken its last edit", () => {
    const out = new Outbox();
    expect(out.takes(state(0)), "nothing typed yet").toBe(true);
    expect(out.next(text("1")).seq).toBe(1);
    expect(out.next(text("2")).seq).toBe(2);
    expect(out.takes(state(1, "1")), "the echo of edit 1, while edit 2 is on its way").toBe(false);
    expect(out.takes(state(2, "2"))).toBe(true);
  });

  it("lets main take each edit once and in order", () => {
    const inbox = new Inbox();
    expect(inbox.take({ seq: 2, sources: text("2") })).toBe(true);
    expect(inbox.ack).toBe(2);
    expect(inbox.take({ seq: 1, sources: text("1") }), "an older edit arriving late").toBe(false);
    expect(inbox.take({ seq: 2, sources: text("2") })).toBe(false);
    expect(inbox.ack).toBe(2);
    inbox.reset();
    expect(inbox.ack).toBe(0);
    expect(inbox.take({ seq: 1, sources: text("1") }), "a new window counts from 1").toBe(true);
  });
});

describe("the PLC window's bridge", () => {
  beforeEach(() => {
    invoke.mockReset().mockResolvedValue(undefined);
    emitTo.mockReset().mockResolvedValue(undefined);
    listen.mockReset().mockResolvedValue(() => {});
  });

  it("calls plc_window.rs and sends each event to its window", async () => {
    await plcWindow.open();
    await plcWindow.close();
    expect(invoke.mock.calls).toEqual([["open_plc_window"], ["close_plc_window"]]);
    await plcWindow.send(state(3));
    await plcWindow.ready();
    await plcWindow.edit({ seq: 1, sources: text("x") });
    await plcWindow.ask({ seq: 2, sources: text("x"), request: "build" });
    expect(emitTo.mock.calls).toEqual([
      ["plc", "plc-state", state(3)],
      ["main", "plc-ready", null],
      ["main", "plc-edit", { seq: 1, sources: text("x") }],
      ["main", "plc-ask", { seq: 2, sources: text("x"), request: "build" }],
    ]);
  });

  it("hands each event's payload to its handler", async () => {
    const seen: unknown[] = [];
    plcWindow.onState((s) => seen.push(["state", s.ack]));
    const inbox = new Inbox();
    hostPlcWindow(inbox, {
      take: (sources) => seen.push(["take", sources[0].text]),
      request: (r) => seen.push(["request", r]),
      ready: () => seen.push(["ready"]),
      closed: () => seen.push(["closed"]),
    });
    const handlers = new Map(listen.mock.calls.map(([name, f]) => [name, f]));
    expect([...handlers.keys()]).toEqual(["plc-state", "plc-edit", "plc-ask", "plc-ready", "plc-closed"]);
    handlers.get("plc-state")({ payload: state(4) });
    handlers.get("plc-edit")({ payload: { seq: 1, sources: text("e1") } });
    handlers.get("plc-edit")({ payload: { seq: 1, sources: text("again") } });
    handlers.get("plc-ask")({ payload: { seq: 2, sources: text("e2"), request: "run" } });
    handlers.get("plc-ask")({ payload: { seq: 1, sources: text("old"), request: "step" } });
    handlers.get("plc-ready")({ payload: null });
    handlers.get("plc-closed")({ payload: null });
    expect(seen).toEqual([
      ["state", 4],
      ["take", "e1"],
      ["take", "e2"],
      ["request", "run"],
      ["request", "step"],
      ["ready"],
      ["closed"],
    ]);
  });
});
