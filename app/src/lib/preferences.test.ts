import { afterEach, describe, expect, it, vi } from "vitest";
import { readPreference, writePreference } from "./preferences";

afterEach(() => {
  vi.unstubAllGlobals();
});

const blocked = () => {
  throw new DOMException("blocked", "SecurityError");
};

describe("preferences", () => {
  it("round-trips through localStorage", () => {
    writePreference("k", "v");
    expect(readPreference("k")).toBe("v");
  });

  it("lasts for the session only when storage is blocked", () => {
    vi.stubGlobal("localStorage", { getItem: blocked, setItem: blocked });
    expect(() => writePreference("k", "v")).not.toThrow();
    expect(readPreference("k")).toBeNull();
  });
});
