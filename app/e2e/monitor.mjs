// End-to-end run of online monitoring, forcing and the variables table (spec 2026-11-13-online-monitoring-design.md,
// acceptance 1 to 4): the 4.4a line built at start, Monitoring on, values in the code and in the table, a variable and
// an input forced and released, and a variable added and retyped through the table, then built. Text goes in through
// the DOM's values (helpers' note: WebKitWebDriver drops typed spaces).
//
//   npx tauri build --debug --no-bundle && node e2e/monitor.mjs

import {
  app,
  bit,
  browser,
  button,
  check,
  click,
  dock,
  editorText,
  eventually,
  expectEqual,
  fixtureCopy,
  out,
  selectSpeed,
  sessions,
  textOf,
} from "./helpers.mjs";

const folder = fixtureCopy("two_belts_diverter");
const BAR = `//div[@role="group"][@aria-label="Value to write or force"]`;

/** The value cell of the variables table's row `name`: a declaration's input or a member's text. */
const tableValue = (name) =>
  browser.execute((n) => {
    const rows = [...document.querySelectorAll('[aria-label="Variables"] tbody tr')];
    const row = rows.find(
      (r) => (r.querySelector('input[aria-label="Name"]')?.value ?? r.querySelector(".name")?.innerText.trim()) === n,
    );
    return row?.querySelector(".value")?.innerText.trim() ?? null;
  }, name);

/** Whether the variables table has a row named `name`. */
const hasRow = (name) =>
  browser.execute(
    (n) =>
      [...document.querySelectorAll('[aria-label="Variables"] input[aria-label="Name"]')].some((i) => i.value === n),
    name,
  );

/** Clicks row `name`'s value, which opens the write and force bar. */
const pickRow = (name) =>
  browser.execute((n) => {
    const rows = [...document.querySelectorAll('[aria-label="Variables"] tbody tr')];
    const row = rows.find((r) => r.querySelector('input[aria-label="Name"]')?.value === n);
    row.querySelector(".value button").click();
  }, name);

/** Sets an input and tells Svelte, as typing would: `input` for a bound value, `change` for a committed one. */
const setInput = (selector, text, event) =>
  browser.execute(
    (s, t, e) => {
      const input = document.evaluate(s, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
      input.value = t;
      input.dispatchEvent(new Event(e, { bubbles: true }));
    },
    selector,
    text,
    event,
  );

/** Changes row `name`'s field `label` to `text`, committed as leaving the cell does. */
const setRowField = (name, label, text) =>
  browser.execute(
    (n, l, t) => {
      const rows = [...document.querySelectorAll('[aria-label="Variables"] tbody tr')];
      const row = rows.find((r) => r.querySelector('input[aria-label="Name"]')?.value === n);
      const input = row.querySelector(`input[aria-label="${l}"]`);
      input.value = t;
      input.dispatchEvent(new Event("change", { bubbles: true }));
    },
    name,
    label,
    text,
  );

/** The values drawn in the code: each as `line class text`. */
const drawn = () =>
  browser.execute(() =>
    document
      .querySelector(".monaco-host")
      .editor.getModel()
      .getAllDecorations()
      .filter((d) => d.options.after?.content)
      .map((d) => `${d.range.startLineNumber} ${d.options.after.inlineClassName} ${d.options.after.content.trim()}`),
  );

/** The number drawn after line 40's `count`. */
const countDrawn = async () => {
  const value = (await drawn()).find((v) => /^40 monitor-value \d+$/.test(v));
  if (!value) throw new Error("no value at line 40");
  return Number(value.split(" ").at(-1));
};

const built = () =>
  eventually(
    async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
    "built",
    15_000,
  );

const runFor = async (ms) => {
  await click(button("Run"));
  await browser.pause(ms);
  await click(button("Pause"));
};

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("Online monitoring, forcing and the variables table");
    await check("Monitoring on shows the program's variables in the table", async () => {
      await built();
      await dock("plc");
      await click(button("Monitoring (Ctrl+T)"));
      await eventually(
        async () => {
          const count = await tableValue("count");
          if (!/^\d+$/.test(count ?? "")) throw new Error(`count reads "${count}"`);
        },
        "monitored",
        10_000,
      );
    });
    await check("the code shows each value, and TRUE or FALSE after a condition", async () => {
      await selectSpeed("Max");
      await runFor(1_500);
      await dock("plc");
      // Only the lines in view are monitored (spec decision 4).
      await browser.execute(() => document.querySelector(".monaco-host").editor.revealLineInCenter(40));
      await eventually(
        async () => {
          const values = await drawn();
          if (!values.some((v) => /^39 monitor-value monitor-(true|false) (TRUE|FALSE)$/.test(v)))
            throw new Error(values.join(" | "));
          if (!values.some((v) => /^40 monitor-value \d+$/.test(v))) throw new Error(values.join(" | "));
        },
        "drawn",
        10_000,
      );
      await browser.saveScreenshot(`${out}/monitor-values.png`);
    });
    await check("the values are drawn while the dock is maximized", async () => {
      await click(button("Maximize panel"));
      await browser.execute(() => document.querySelector(".monaco-host").editor.revealLineInCenter(40));
      // Line 40 counts the cartons that entered, so a run as fast as possible raises it (selectSpeed above).
      const before = await eventually(countDrawn, "drawn", 10_000);
      await runFor(1_500);
      await eventually(
        async () => {
          const now = await countDrawn();
          if (now <= before) throw new Error(`count ${now} after ${before}`);
        },
        "counted",
        10_000,
      );
      await browser.saveScreenshot(`${out}/monitor-maximized.png`);
      await click(button("Restore panel"));
    });
    await check("the table's values equal the Watch pane's", async () => {
      const [belt, eye] = [await tableValue("belt_b_run"), await tableValue("detected_a")];
      expectEqual([belt, eye], [await bit("belt_b.run"), await bit("sensor_a.detected")], "values");
    });
    await check("a forced variable holds against the program until released", async () => {
      await dock("plc");
      await pickRow("belt_b_run");
      await setInput(`${BAR}//input`, "FALSE", "input");
      await click(`${BAR}//button[normalize-space()="Force"]`);
      await runFor(1_500);
      expectEqual(await bit("belt_b.run"), "FALSE", "forced");
      const listed = await textOf(".forces li");
      if (!listed.startsWith("Main.belt_b_run = FALSE")) throw new Error(`forces: ${listed}`);
      await click(`//ul[contains(@class,"forces")]//button[normalize-space()="Release"]`);
      await click(button("Step"));
      await eventually(async () => expectEqual(await bit("belt_b.run"), "TRUE", "released"), "released", 5_000);
    });
    await check("a forced input is read by the program, and the plant takes it back", async () => {
      // From time zero no carton reaches the photocell for 5 s, so the plant's value is known: FALSE.
      await click(button("Reset"));
      await dock("watch");
      await click(button("Force this address %IX0.6"));
      // The field offers the opposite of what the photocell reads now, which a carton may be covering.
      await setInput(`//div[contains(@class,"forcing")]//input`, "TRUE", "input");
      await click(`//div[contains(@class,"forcing")]//button[normalize-space()="Force"]`);
      await click(button("Step"));
      await eventually(async () => expectEqual(await bit("sensor_in.detected"), "TRUE", "forced"), "forced", 5_000);
      await dock("plc");
      await eventually(async () => expectEqual(await tableValue("detected_in"), "TRUE", "table"), "table", 5_000);
      await click(`//ul[contains(@class,"forces")]//button[normalize-space()="Release"]`);
      await click(button("Step"));
      await eventually(async () => expectEqual(await bit("sensor_in.detected"), "FALSE", "plant"), "plant", 5_000);
    });
    await check("a variable added and retyped in the table builds and is monitored", async () => {
      await dock("plc");
      await click(button("Add variable"));
      await eventually(
        async () => {
          if (!(await editorText()).includes("var1 : BOOL;")) throw new Error("not added");
        },
        "added",
        5_000,
      );
      await eventually(
        async () => {
          if (!(await hasRow("var1"))) throw new Error("no row var1 yet");
        },
        "row",
        5_000,
      );
      await setRowField("var1", "Type", "INT");
      await eventually(
        async () => {
          if (!(await editorText()).includes("var1 : INT;")) throw new Error(await editorText());
        },
        "retyped",
        5_000,
      );
      await click(button("Build"));
      await built();
      await eventually(async () => expectEqual(await tableValue("var1"), "0", "var1"), "monitored", 10_000);
      await dock("watch");
      expectEqual(await textOf(".forces li"), "No value is forced", "no force after Build");
      await browser.saveScreenshot(`${out}/monitor-table.png`);
    });
  }),
);
