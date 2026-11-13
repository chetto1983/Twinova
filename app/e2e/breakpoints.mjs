// End-to-end run of breakpoints as HOLD (spec 2026-11-13-breakpoints-hold-design.md) on the 4.4a line, at top-level
// lines, and in an IF's body through the fork's matching rule (plan, Task 6): a breakpoint from the glyph margin and F9,
// HOLD with virtual time standing, the value at the stop, F10, F5, Run to cursor, a breakpoint removed, and Reset
// while held.
//
//   npx tauri build --debug --no-bundle && node e2e/breakpoints.mjs

import {
  BAR,
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
  pickRow,
  selectSpeed,
  sessions,
  setInput,
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

/** Inserts `text` at the start of `line`, as typing does: through the model, so the decorations move with it. */
const insertAt = (line, text) =>
  browser.execute(
    (l, t) =>
      document
        .querySelector(".monaco-host")
        .editor.executeEdits("e2e", [
          { range: { startLineNumber: l, startColumn: 1, endLineNumber: l, endColumn: 1 }, text: t },
        ]),
    line,
    text,
  );

const built = () =>
  eventually(
    async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
    "built",
    15_000,
  );

/** The virtual time Run makes in `ms` of wall time. */
async function virtualIn(ms) {
  const before = await statusSeconds();
  await click(button("Run"));
  await browser.pause(ms);
  await click(button("Pause"));
  return (await statusSeconds()) - before;
}

/** The breakpoint bar's field `label` (spec 2026-11-13-breakpoints-complete-design.md, decision 7). */
const field = (label) => `//div[contains(@class,"breakpoint-bar")]//input[@aria-label="${label}"]`;

/** Opens line `line`'s breakpoint bar with Shift+F9, sets its fields, and applies them. */
async function editBreakpoint(line, fields) {
  await cursorAt(line);
  await browser.keys(["Shift", "F9"]);
  await browser.$(field("Condition")).waitForExist({ timeout: 5_000 });
  for (const [label, text] of Object.entries(fields)) await setInput(field(label), text, "input");
  await click(`//div[contains(@class,"breakpoint-bar")]//button[normalize-space()="Apply"]`);
}

/** Whether the code shows `value` after line 40's `count`, brought into view: values are drawn on visible lines. */
const countShows = (value) =>
  eventually(
    async () => {
      await editor(() => document.querySelector(".monaco-host").editor.revealLineInCenter(40));
      const values = await drawn();
      if (!values.includes(`40 monitor-value ${value}`)) throw new Error(values.join(" | "));
    },
    `count ${value}`,
    10_000,
  );

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
    await built();
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
      // Monitoring turned on while held: the engine, in its scan, cannot read the table, which comes at Continue.
      await click(button("Monitoring (Ctrl+T)"));
      await click(button("Monitoring (Ctrl+T)"));
      await clickGutter(37);
      await eventually(async () => expectEqual(await marks(), ["39 debug-held"], "marks"), "off", 5_000);
      await click(button("Continue (F5)"));
      await eventually(
        async () => {
          const names = await browser.execute(() =>
            [...document.querySelectorAll('[aria-label="Variables"] tbody input[aria-label="Name"]')].map(
              (i) => i.value,
            ),
          );
          if (!names.includes("count")) throw new Error(names.join(" | ") || "no row");
        },
        "the table after Continue",
        10_000,
      );
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
    await check("a condition stops only when it holds: count = 2, the second carton", async () => {
      await clickGutter(40);
      await editBreakpoint(41, { Condition: "count = 2" });
      await eventually(
        async () => expectEqual(await marks(), ["41 debug-breakpoint-conditional"], "marks"),
        "conditional",
        5_000,
      );
      await selectSpeed("Max");
      await click(button("Run"));
      await heldAt(41);
      await countShows(2);
      await click(button("Reset"));
    });
    await check("a hit count stops at its hit: the third carton", async () => {
      await editBreakpoint(41, { Condition: "", Hits: "3" });
      await click(button("Run"));
      await heldAt(41);
      await countShows(3);
      await click(button("Reset"));
    });
    await check("a logpoint never stops and writes its lines to the Log", async () => {
      await editBreakpoint(41, { Hits: "", "Log message": "count={count}" });
      await eventually(async () => expectEqual(await marks(), ["41 debug-logpoint"], "marks"), "logpoint", 5_000);
      await click(button("Run"));
      await eventually(
        async () => {
          const lines = await browser.execute(() =>
            [...document.querySelectorAll('section[aria-label="Log"] li')].map((li) => li.textContent.trim()),
          );
          if (!lines.some((l) => l.endsWith("count=1")) || !lines.some((l) => l.endsWith("count=2")))
            throw new Error(lines.join(" | "));
        },
        "logged",
        15_000,
      );
      expectEqual(await browser.$(".statusbar .hold").isExisting(), false, "never held");
      await click(button("Pause"));
      await cursorAt(41);
      await browser.keys(["Shift", "F9"]);
      await click(`//div[contains(@class,"breakpoint-bar")]//button[normalize-space()="Remove"]`);
      await eventually(async () => expectEqual(await marks(), [], "removed"), "removed", 5_000);
      await click(`//section[@aria-label="Log"]//button[normalize-space()="Clear"]`);
      await click(button("Reset"));
    });
    await check("a block's breakpoint holds the line and names the device", async () => {
      await click(`//button[@role="tab"][contains(normalize-space(), "chain_transfer")]`);
      await clickGutter(20);
      await click(button("Run"));
      await eventually(
        async () =>
          expectEqual(await holdText(), "HOLD at components/twinova.chain_transfer@1.st:20 (chain_transfer)", "hold"),
        "held in the block",
        10_000,
      );
      await browser.saveScreenshot(`${out}/breakpoint-block.png`);
      // Taken off while held, it lets the field's debug go at Continue: on, the line runs 7.7 times slower.
      await clickGutter(20);
      await click(button("Continue (F5)"));
      await click(button("Reset"));
      await click(`//button[@role="tab"][contains(normalize-space(), "plc/Main.st")]`);
    });
    await check("a write while held shows at once and reaches the program at the next cycle", async () => {
      await clickGutter(40);
      await click(button("Run"));
      await heldAt(40);
      await pickRow("count");
      await setInput(`${BAR}//input`, "100", "input");
      await click(`${BAR}//button[normalize-space()="Write"]`);
      expectEqual(await browser.$(".held-note").isExisting(), true, "the bar says when it lands");
      // The next carton stops at line 40 again, before count goes up: count is the 100 written, not 1.
      await click(button("Continue (F5)"));
      await heldAt(40);
      await countShows(100);
      await click(button("Reset"));
    });
    await check("a breakpoint moves with its line when a line goes in above, and stops there after Build", async () => {
      await insertAt(30, "(* a line above *)\n");
      await eventually(async () => expectEqual(await marks(), ["41 debug-breakpoint"], "moved"), "moved", 5_000);
      await click(button("Build"));
      await built();
      await click(button("Run"));
      await heldAt(41);
      await click(button("Reset"));
      await clickGutter(41);
      await eventually(async () => expectEqual(await marks(), [], "cleared"), "cleared", 5_000);
    });
    await check("a breakpoint on a line that never runs never holds the line", async () => {
      // `count` only counts up, so this body never runs; a constant condition might be compiled away.
      const end = await editor(
        () =>
          document
            .querySelector(".monaco-host")
            .editor.getModel()
            .findMatches("END_PROGRAM", false, false, true, null, false)[0].range.startLineNumber,
      );
      await insertAt(end, "IF count < 0 THEN\n    count := 0;\nEND_IF;\n");
      await click(button("Build"));
      await built();
      await selectSpeed("Max");
      const free = await virtualIn(4_000);
      await clickGutter(end + 1);
      await eventually(async () => expectEqual(await marks(), [`${end + 1} debug-breakpoint`], "set"), "set", 5_000);
      const armed = await virtualIn(4_000);
      expectEqual(await browser.$(".statusbar .hold").isExisting(), false, "never held");
      if (armed <= 0) throw new Error(`the line did not run: ${armed} s`);
      // A measurement, not a pass mark: the debug build, on whatever machine runs it.
      console.log(
        `      as fast as possible for 4 s: ${free.toFixed(1)} s of virtual time free, ${armed.toFixed(1)} s with the breakpoint`,
      );
    });
  }),
);
