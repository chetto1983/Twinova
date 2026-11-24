import { beforeEach, describe, expect, it, vi } from "vitest";

// The backend's control bridge, as backend.test.ts checks it against Tauri.
const { invoke, listen } = vi.hoisted(() => ({ invoke: vi.fn(), listen: vi.fn() }));
vi.mock("../backend", () => ({
  control: {
    onRequest: (f: (r: unknown) => void) => listen("control-request", (e: { payload: unknown }) => f(e.payload)),
    reply: (id: number, ok: boolean, value: unknown) => invoke("control_reply", { id, ok, value }),
    ready: () => invoke("control_ready"),
    commands: (commands: unknown) => invoke("control_commands", { commands }),
  },
}));

import {
  described,
  dispatch,
  listed,
  listenControl,
  none,
  object,
  oneOf,
  register,
  string,
  stringOf,
} from "./registry";

const settled = () => new Promise((resolve) => setTimeout(resolve));
const byId = (call: unknown[]) => (call[1] as { id: number }).id;

describe("the command registry", () => {
  beforeEach(() => {
    invoke.mockReset().mockResolvedValue(true);
    listen.mockReset().mockResolvedValue(() => {});
  });

  it("lists, describes and runs what is registered, and forgets it when its component goes", async () => {
    const off = register(
      { name: "b", description: "second", params: none, run: () => 2 },
      { name: "a", description: "first", params: object({ x: string("x") }), run: (p) => p.x },
    );
    expect(listed()).toEqual([
      { name: "a", description: "first" },
      { name: "b", description: "second" },
    ]);
    expect(await dispatch("list_commands", undefined)).toEqual(listed());
    expect(await dispatch("describe_command", { name: "a" })).toEqual({
      name: "a",
      description: "first",
      params: {
        type: "object",
        properties: { x: { type: "string", description: "x" } },
        required: ["x"],
        additionalProperties: false,
      },
    });
    expect(await dispatch("a", { x: "y" })).toBe("y");
    expect(await dispatch("b", null)).toBe(2);
    await expect(dispatch("b", { ms: 500 })).rejects.toThrow("b takes no ms: describe_command gives its parameters");
    await expect(dispatch("a", {})).rejects.toThrow("a needs x");
    expect(invoke.mock.calls).toEqual([["control_commands", { commands: listed() }]]);
    off();
    expect(listed()).toEqual([]);
    expect(invoke.mock.calls.at(-1)).toEqual(["control_commands", { commands: [] }]);
    await expect(dispatch("a", {})).rejects.toThrow("no command a: list_commands names them");
    expect(() => described("a")).toThrow("no command a");
  });

  it("keeps a command registered again by another component when the first goes", () => {
    const first = register({ name: "c", description: "1", params: none, run: () => 1 });
    const second = register({ name: "c", description: "2", params: none, run: () => 2 });
    first();
    expect(listed()).toEqual([{ name: "c", description: "2" }]);
    second();
  });

  it("answers the channel's requests through control_reply, a failure with its text", async () => {
    const off = register(
      { name: "ok", description: "", params: none, run: () => undefined },
      {
        name: "fails",
        description: "",
        params: none,
        run: () => {
          throw new Error("build the project first");
        },
      },
      { name: "rejects", description: "", params: none, run: () => Promise.reject("plain") },
    );
    await listenControl();
    // the registry told the app its commands as they came, then that it listens
    expect(invoke.mock.calls).toEqual([
      [
        "control_commands",
        {
          commands: [
            { name: "fails", description: "" },
            { name: "ok", description: "" },
            { name: "rejects", description: "" },
          ],
        },
      ],
      ["control_ready"],
    ]);
    invoke.mockClear();
    const handler = listen.mock.calls[0]?.[1] as (e: { payload: unknown }) => void;
    expect(listen.mock.calls[0]?.[0]).toBe("control-request");
    handler({ payload: { id: 1, method: "ok", params: {} } });
    handler({ payload: { id: 2, method: "fails", params: {} } });
    handler({ payload: { id: 3, method: "rejects", params: {} } });
    await settled();
    // each request is answered on its own, in whatever order they settle
    expect(invoke.mock.calls.toSorted((a, b) => byId(a) - byId(b))).toEqual([
      ["control_reply", { id: 1, ok: true, value: null }],
      ["control_reply", { id: 2, ok: false, value: "build the project first" }],
      ["control_reply", { id: 3, ok: false, value: "plain" }],
    ]);
    off();
  });

  it("builds schemas and reads string parameters", () => {
    expect(object({ a: string("a"), b: string("b") }, ["b"]).required).toEqual(["a"]);
    expect(oneOf(["x"], "d")).toEqual({ type: "string", enum: ["x"], description: "d" });
    expect(stringOf({ a: "v" }, "a")).toBe("v");
    expect(() => stringOf({ a: 1 }, "a")).toThrow("a must be a string");
  });
});
