import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

// A plain stub, not vi.fn: vitest reports a rejected promise a vi.fn returned as unhandled, though the code catches it.
const backend = vi.hoisted(() => ({
  asked: 0,
  answer: (): Promise<unknown> => Promise.resolve(null),
}));
vi.mock("./backend", () => ({
  debug: {
    hold: () => {
      backend.asked += 1;
      return backend.answer();
    },
  },
}));

import { plcHold, POLL_MS } from "./held.svelte";
import type { Hold } from "./debug";

const AT_40: Hold = { file: "plc/Main.st", line: 40, frames: [["plc/Main.st", 40]] };

describe("where the PLC is held, one answer for the window", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    backend.asked = 0;
    backend.answer = () => Promise.resolve(null);
  });
  afterEach(() => vi.useRealTimers());

  it("asks the debugger while one watcher at least is mounted, once a period for all of them", async () => {
    const statusBar = plcHold.watch();
    const debugBar = plcHold.watch();
    backend.answer = () => Promise.resolve(AT_40);
    await vi.advanceTimersByTimeAsync(POLL_MS);
    expect([backend.asked, plcHold.current]).toEqual([1, AT_40]);
    statusBar();
    await vi.advanceTimersByTimeAsync(POLL_MS);
    expect(backend.asked).toBe(2);
    debugBar();
    await vi.advanceTimersByTimeAsync(POLL_MS * 3);
    expect(backend.asked).toBe(2);
  });

  it("is not held when the debugger cannot answer", async () => {
    plcHold.current = AT_40;
    backend.answer = () => Promise.reject(new Error("a lock is poisoned"));
    await plcHold.ask();
    expect(plcHold.current).toBeNull();
  });
});
