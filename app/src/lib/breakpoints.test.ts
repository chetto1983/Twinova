import { beforeEach, describe, expect, it, vi } from "vitest";

// A plain stub, not vi.fn: vitest reports a rejected promise a vi.fn returned as unhandled, though the code catches it.
const backend = vi.hoisted(() => ({
  calls: [] as unknown[][],
  answer: (): Promise<unknown> => Promise.resolve([]),
}));
vi.mock("./backend", () => ({
  debug: {
    setBreakpoints: (...args: unknown[]) => {
      backend.calls.push(args);
      return backend.answer();
    },
  },
}));

import { Breakpoints } from "./breakpoints.svelte";

const settled = () => new Promise((resolve) => setTimeout(resolve));

describe("the session's breakpoints", () => {
  beforeEach(() => {
    backend.calls = [];
  });

  it("keeps a file's breakpoints as the debugger placed them, and says whether any stops or logs", async () => {
    backend.answer = () => Promise.resolve([{ line: 37 }, { line: 41, log: "c={count}" }]);
    const kept = new Breakpoints(
      () => false,
      () => {},
    );
    expect([kept.armed, kept.logging, kept.of(null), kept.of("plc/Main.st")]).toEqual([false, false, [], []]);
    kept.set("plc/Main.st", [{ line: 34 }, { line: 41, log: "c={count}" }]);
    await settled();
    expect(backend.calls).toEqual([["plc/Main.st", [{ line: 34 }, { line: 41, log: "c={count}" }]]]);
    expect(kept.of("plc/Main.st")).toEqual([{ line: 37 }, { line: 41, log: "c={count}" }]);
    expect([kept.armed, kept.logging]).toEqual([true, true]);
    kept.move("plc/Main.st", [{ line: 38 }]);
    expect([kept.of("plc/Main.st"), kept.logging]).toEqual([[{ line: 38 }], false]);
  });

  it("keeps a tab's breakpoints in the editor while its edits are not built, and sends them all after a Build", async () => {
    let edited = true;
    const kept = new Breakpoints(
      () => edited,
      () => {},
    );
    kept.set("plc/Main.st", [{ line: 40 }]);
    expect(backend.calls).toEqual([]);
    expect(kept.of("plc/Main.st")).toEqual([{ line: 40 }]);
    edited = false;
    backend.answer = () => Promise.resolve([{ line: 40 }]);
    kept.resend();
    await settled();
    expect(backend.calls).toEqual([["plc/Main.st", [{ line: 40 }]]]);
  });

  it("shows a refusal", async () => {
    backend.answer = () => Promise.reject("line 41: the condition 'x =' does not parse");
    const failed: string[] = [];
    const kept = new Breakpoints(
      () => false,
      (e) => failed.push(e),
    );
    kept.set("plc/Main.st", [{ line: 41, condition: "x =" }]);
    await settled();
    expect(failed).toEqual(["line 41: the condition 'x =' does not parse"]);
    expect(kept.armed).toBe(false);
  });
});
