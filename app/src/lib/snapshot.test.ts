import { afterEach, describe, expect, it, vi } from "vitest";
import { mount, unmount } from "svelte";
import App from "../App.svelte";
import {
  deliveredOf,
  formatSpeed,
  formatTime,
  framingDistance,
  isBehind,
  paintOf,
  scaled,
  scrollOffset,
  SPEEDS,
  type Snapshot,
} from "./snapshot";

vi.mock("./ProjectScene.svelte", () => ({ default: () => {} }));
// The App hosts the PLC window's events (plcWindow.ts); here no window and no Tauri event bus.
vi.mock("./plcWindow", async (original) => ({
  ...(await original<typeof import("./plcWindow")>()),
  hostPlcWindow: () => [],
}));
vi.mock("./backend", () => ({
  backend: {
    build: vi.fn().mockResolvedValue({ snapshot: null, warnings: [] }),
    pause: vi.fn().mockResolvedValue(undefined),
    run: vi.fn().mockResolvedValue(undefined),
    step: vi.fn().mockResolvedValue(undefined),
    reset: vi.fn().mockResolvedValue(undefined),
    saveState: vi.fn().mockResolvedValue(undefined),
    restoreState: vi.fn().mockResolvedValue(undefined),
    press: vi.fn().mockResolvedValue(undefined),
    setSinkBlocked: vi.fn().mockResolvedValue(undefined),
  },
  projects: {
    current: vi.fn().mockResolvedValue({
      name: "M3 demo",
      folder: null,
      instances: [],
      definitions: {},
      changed_outside: [],
      recovered: false,
      program: [["plc/Main.st", "PROGRAM Main END_PROGRAM"]],
      behaviours: {},
      panel: [{ name: "Start", address: "%IX1.0" }],
    }),
    launch: vi.fn().mockResolvedValue(null),
  },
  catalogue: vi.fn().mockResolvedValue([{ key: "twinova.belt@1", group: "conveyors" }]),
  onSnapshot: vi.fn().mockResolvedValue(() => {}),
  onStopped: vi.fn().mockResolvedValue(() => {}),
  onEngineError: vi.fn().mockResolvedValue(() => {}),
  onDiskSources: vi.fn().mockResolvedValue(() => {}),
  debug: {
    hold: vi.fn().mockResolvedValue(null),
    setBreakpoints: vi.fn().mockResolvedValue([]),
    runToCursor: vi.fn().mockResolvedValue(undefined),
    resume: vi.fn().mockResolvedValue(undefined),
  },
  onMonitor: vi.fn().mockResolvedValue(() => {}),
  monitor: {
    watchRange: vi.fn().mockResolvedValue({ spots: [], rows: [] }),
    unwatch: vi.fn().mockResolvedValue(undefined),
    watched: vi.fn().mockResolvedValue([]),
    forces: vi.fn().mockResolvedValue([]),
  },
  declarations: { of: vi.fn().mockResolvedValue([]) },
}));

describe("paintOf", () => {
  it("keys a painted product by its colour, two hex digits a channel", () => {
    expect(paintOf([255, 0, 127])).toBe("#ff007f");
    expect(paintOf([0, 10, 1])).toBe("#000a01");
  });

  it("keys an unpainted product apart, for the theme's product colour", () => {
    expect(paintOf(null)).toBe("unpainted");
  });
});

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

describe("deliveredOf", () => {
  it("adds the deliveries of every sink, and none before a build", () => {
    const snapshot = {
      sinks: [
        { id: "ok", delivered: 3, blocked: false },
        { id: "reject", delivered: 2, blocked: true },
      ],
    } as Snapshot;
    expect(deliveredOf(snapshot)).toBe(5);
    expect(deliveredOf(null)).toBe(0);
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

  it("keeps the industrial brand lockup stable across themes", () => {
    app = mount(App, { target: document.body });
    const lockup = document.querySelector<HTMLElement>(".brand-lockup");
    const name = lockup?.querySelector<HTMLElement>(".name");

    expect(name?.textContent).toBe("Twinova");
    document.documentElement.dataset.theme = "dark";
    const darkColors = [getComputedStyle(lockup!).backgroundColor, getComputedStyle(name!).color];
    document.documentElement.dataset.theme = "light";
    const lightColors = [getComputedStyle(lockup!).backgroundColor, getComputedStyle(name!).color];

    expect(lightColors).toEqual(darkColors);
    expect(getComputedStyle(name!).fontFamily).toContain("Atkinson Hyperlegible Next");
  });
});

describe("a scrolling fabric", () => {
  it("walks its offset back by the travel, in tiles, within one tile", () => {
    expect(scrollOffset(0, 1)).toBe(-0);
    expect(scrollOffset(0.25, 1)).toBe(-0.25);
    expect(scrollOffset(2.75, 1)).toBeCloseTo(-0.75, 12);
    expect(scrollOffset(0.25, 2)).toBe(-0.5);
    // a right curve's fabric runs backwards: a negative rate walks the offset forward
    expect(scrollOffset(0.25, -1)).toBe(0.25);
  });
});

describe("a sliding link", () => {
  it("stands at the device's position along its stroke", () => {
    expect(scaled([0, 0.095, 0], 0)).toEqual([0, 0, 0]);
    expect(scaled([0, 0, -0.75], 0.5)).toEqual([0, 0, -0.375]);
    expect(scaled([1, 2, 3], 1)).toEqual([1, 2, 3]);
  });
});
