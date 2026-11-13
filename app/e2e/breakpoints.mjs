// End-to-end run of breakpoints as HOLD (spec 2026-11-13-breakpoints-hold-design.md) on the 4.4a line, at top-level
// lines, and in an IF's body through the fork's matching rule (plan, Task 6): a breakpoint from the glyph margin and F9,
// HOLD with virtual time standing, the value at the stop, F10, F5, Run to cursor, a breakpoint removed, and Reset
// while held.
//
//   npx tauri build --debug --no-bundle && node e2e/breakpoints.mjs

import {
  app,
  browser,
  button,
  check,
  click,
  dock,
  drawn,
  eventually,
  expectEqual,
  fixtureCopy,
  out,
  sessions,
  statusSeconds,
  textOf,
} from "./helpers.mjs";

const folder = fixtureCopy("two_belts_diverter");
const editor = (f) => browser.execute(f);

/** Clicks line `line`'s glyph margin, where a breakpoint is set or taken off. */
async function clickGutter(line) {
  const at = await browser.execute((l) => {
    const e = document.querySelector(".monaco-host").editor;
    e.revealLineInCenter(l);
    const box = e.getDomNode().getBoundingClientRect();
    const layout = e.getLayoutInfo();
    return {
      x: Math.round(box.left + layout.glyphMarginLeft + layout.glyphMarginWidth / 2),
      y: Math.round(box.top + e.getTopForLineNumber(l) - e.getScrollTop() + 8),
    };
  }, line);
  await browser.action("pointer").move({ origin: "viewport", x: at.x, y: at.y }).down().up().perform();
}

/** The lines marked in the glyph margin: breakpoints, and the held line. */
const marks = () =>
  editor(() =>
    document
      .querySelector(".monaco-host")
      .editor.getModel()
      .getAllDecorations()
      .filter((d) => d.options.glyphMarginClassName)
      .map((d) => `${d.range.startLineNumber} ${d.options.glyphMarginClassName}`),
  );

const holdText = () => textOf(".statusbar .hold");
const heldAt = (where) =>
  eventually(async () => expectEqual(await holdText(), `HOLD at plc/Main.st:${where}`, "hold"), "held", 10_000);

/** Puts the cursor on `line` and gives the editor the focus. */
const cursorAt = (line) =>
  browser.execute((l) => {
    const e = document.querySelector(".monaco-host").editor;
    e.setPosition({ lineNumber: l, column: 1 });
    e.focus();
  }, line);

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("Breakpoints as HOLD");
    await eventually(
      async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
      "built",
      15_000,
    );
    await dock("plc");
    await check("a click in the glyph margin and F9 set breakpoints", async () => {
      await clickGutter(37);
      await cursorAt(38);
      await browser.keys(["F9"]);
      await eventually(
        async () => expectEqual(await marks(), ["37 debug-breakpoint", "38 debug-breakpoint"], "marks"),
        "set",
        5_000,
      );
      await cursorAt(38);
      await browser.keys(["F9"]);
      await eventually(async () => expectEqual(await marks(), ["37 debug-breakpoint"], "marks"), "unset", 5_000);
    });
    await check("Run stops at the breakpoint: HOLD, and virtual time stands", async () => {
      await click(button("Monitoring (Ctrl+T)"));
      await click(button("Run"));
      await heldAt(37);
      const at = await statusSeconds();
      await browser.pause(700);
      expectEqual(await statusSeconds(), at, "virtual time");
      await browser.saveScreenshot(`${out}/breakpoint-hold.png`);
    });
    await check("the code shows the values the stopped statement sees", async () => {
      await editor(() => document.querySelector(".monaco-host").editor.revealLineInCenter(40));
      await eventually(
        async () => {
          const values = await drawn();
          if (!values.includes("40 monitor-value 0")) throw new Error(values.join(" | "));
        },
        "count at the stop",
        10_000,
      );
    });
    await check("F10 steps over to the next line, and F5 continues to the next scan", async () => {
      await browser.keys(["F10"]);
      await heldAt(38);
      expectEqual((await marks()).includes("38 debug-held"), true, "the held line is marked");
      // Line 37 comes before 38 in the scan: stopping there again is the next scan. The status bar's time does not
      // show it, since the Run loop sends a snapshot only at the end of its tick, and the tick stops again first.
      await browser.keys(["F5"]);
      await heldAt(37);
    });
    await check("Run to cursor stops on the cursor's line", async () => {
      await cursorAt(39);
      await browser.keys(["Control", "F10"]);
      await heldAt(39);
    });
    await check("with the breakpoint off, Continue lets the line run", async () => {
      await clickGutter(37);
      await eventually(async () => expectEqual(await marks(), ["39 debug-held"], "marks"), "off", 5_000);
      await click(button("Continue (F5)"));
      const at = await statusSeconds();
      await eventually(
        async () => {
          if ((await statusSeconds()) <= at + 0.5) throw new Error("not running");
        },
        "running",
        10_000,
      );
      expectEqual(await browser.$(".statusbar .hold").isExisting(), false, "no HOLD");
      await click(button("Pause"));
    });
    await check("Reset while held lets the line go back to time zero", async () => {
      await clickGutter(37);
      await click(button("Run"));
      await heldAt(37);
      await click(button("Reset"));
      await eventually(
        async () => {
          if (await browser.$(".statusbar .hold").isExisting()) throw new Error("still held");
          expectEqual(await statusSeconds(), 0, "time zero");
        },
        "reset",
        10_000,
      );
    });
    await check("a breakpoint in an IF's body stops when the body runs, with its values", async () => {
      // truST c9635b1c, ported through the fork (spec decision 1): at the pin this stopped at the IF, at time zero.
      await clickGutter(37);
      await clickGutter(40);
      await eventually(async () => expectEqual(await marks(), ["40 debug-breakpoint"], "marks"), "set", 5_000);
      await click(button("Run"));
      await heldAt(40);
      const at = await statusSeconds();
      if (at < 5) throw new Error(`stopped at ${at} s, before the first carton enters`);
      await eventually(
        async () => {
          if (!(await drawn()).includes("40 monitor-value 0")) throw new Error((await drawn()).join(" | "));
        },
        "count before the line",
        10_000,
      );
      await browser.keys(["F11"]);
      await heldAt(41);
      await eventually(
        async () => {
          if (!(await drawn()).includes("40 monitor-value 1")) throw new Error((await drawn()).join(" | "));
        },
        "count after the line",
        10_000,
      );
      await browser.saveScreenshot(`${out}/breakpoint-body.png`);
      await click(button("Reset"));
    });
  }),
);
