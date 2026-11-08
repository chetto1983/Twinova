import { afterEach, describe, expect, it, vi } from "vitest";
import { mount, unmount } from "svelte";
import App from "../App.svelte";
import {
  BELT_LENGTH_M,
  formatSpeed,
  formatTime,
  framingDistance,
  isBehind,
  partCentreX,
  PART_LENGTH_M,
  SPEEDS,
} from "./snapshot";

vi.mock("./SceneView.svelte", () => import("./__fixtures__/Empty.svelte"));
vi.mock("./backend", () => ({
  backend: {
    defaultProgram: vi.fn().mockResolvedValue(""),
    build: vi.fn().mockResolvedValue(undefined),
    pause: vi.fn().mockResolvedValue(undefined),
    run: vi.fn().mockResolvedValue(undefined),
    step: vi.fn().mockResolvedValue(undefined),
    reset: vi.fn().mockResolvedValue(undefined),
    saveState: vi.fn().mockResolvedValue(undefined),
    restoreState: vi.fn().mockResolvedValue(undefined),
    press: vi.fn().mockResolvedValue(undefined),
    setOutletBlocked: vi.fn().mockResolvedValue(undefined),
  },
  onSnapshot: vi.fn().mockResolvedValue(() => {}),
  onStopped: vi.fn().mockResolvedValue(() => {}),
  onEngineError: vi.fn().mockResolvedValue(() => {}),
}));

describe("formatTime", () => {
  it("shows minutes, seconds and hundredths", () => {
    expect(formatTime(0)).toBe("0:00.00");
    expect(formatTime(1230)).toBe("0:01.23");
    expect(formatTime(61_000)).toBe("1:01.00");
    expect(formatTime(3_599_990)).toBe("59:59.99");
  });
  it("never shows negative time", () => {
    expect(formatTime(-50)).toBe("0:00.00");
  });
});

describe("partCentreX", () => {
  it("puts a part just entering at the left end of a centred belt", () => {
    expect(partCentreX(PART_LENGTH_M)).toBeCloseTo(PART_LENGTH_M / 2 - BELT_LENGTH_M / 2);
  });
  it("puts a part at the outlet at the right end", () => {
    expect(partCentreX(BELT_LENGTH_M)).toBeCloseTo(BELT_LENGTH_M / 2 - PART_LENGTH_M / 2);
  });
});

describe("isBehind", () => {
  it("ignores frame jitter and flags real lag", () => {
    expect(isBehind(0)).toBe(false);
    expect(isBehind(100)).toBe(false);
    expect(isBehind(101)).toBe(true);
  });
});

describe("framingDistance", () => {
  it("fits the width in a narrow viewport and the height in a wide one", () => {
    // 90 degrees vertical: tan(45) = 1, so distance = half-extent / aspect-scaled tangent.
    expect(framingDistance(2, 1, 90, 0.5)).toBeCloseTo(4);
    expect(framingDistance(2, 1, 90, 4)).toBeCloseTo(1);
    // 60 degrees: tan(30) = 1 / sqrt(3), where dividing and multiplying by the tangent differ.
    expect(framingDistance(2, 1, 60, 10)).toBeCloseTo(Math.sqrt(3));
  });
  it("grows as the viewport narrows", () => {
    expect(framingDistance(1.8, 1, 40, 0.8)).toBeGreaterThan(framingDistance(1.8, 1, 40, 1.6));
  });
});

describe("formatSpeed", () => {
  it("writes the multiple in the locale's number format, and names the unpaced run", () => {
    expect(formatSpeed(0.25, "en", "Max")).toBe("×0.25");
    expect(formatSpeed(0.25, "it", "Max")).toBe("×0,25");
    expect(formatSpeed(10, "it", "Max")).toBe("×10");
    expect(formatSpeed(null, "en", "Max")).toBe("Max");
  });
  it("offers real time and an unpaced run", () => {
    expect(SPEEDS).toContain(1);
    expect(SPEEDS.at(-1)).toBeNull();
  });
});

describe("Twinova brand", () => {
  let app: ReturnType<typeof mount> | undefined;

  afterEach(() => {
    if (app) unmount(app);
    document.body.replaceChildren();
    app = undefined;
  });

  it("renders the shared mark as a decorative image", () => {
    app = mount(App, { target: document.body });
    const mark = document.querySelector<HTMLImageElement>(".brand img");

    expect(mark).toBeInstanceOf(HTMLImageElement);
    expect(mark?.getAttribute("src")).toMatch(/^(?:data:image\/svg\+xml|.*twinova-mark\.svg)/);
    expect(mark?.alt).toBe("");
    expect(document.querySelector(".brand circle")).toBeNull();
  });
});
