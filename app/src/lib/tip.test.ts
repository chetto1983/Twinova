import { describe, expect, it } from "vitest";
import { tip } from "./tip";

describe("a control's tooltip", () => {
  it("names the control, and says why when it is disabled", () => {
    expect(tip("Run", null)).toBe("Run");
    expect(tip("Run", "build the project first")).toBe("Run: build the project first");
  });
});
