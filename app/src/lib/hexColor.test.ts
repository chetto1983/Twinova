import { describe, expect, it } from "vitest";
import { fullHex } from "./hexColor";

describe("a colour for Monaco's theme", () => {
  it("writes a short hex out in full, and leaves any other colour as it is", () => {
    expect(fullHex("#fff")).toBe("#ffffff");
    expect(fullHex(" #0b5 ")).toBe("#00bb55");
    expect(fullHex("#abcd")).toBe("#aabbccdd");
    expect(fullHex("#172026")).toBe("#172026");
    expect(fullHex("rgb(1, 2, 3)")).toBe("rgb(1, 2, 3)");
    expect(fullHex("")).toBe("");
  });
});
