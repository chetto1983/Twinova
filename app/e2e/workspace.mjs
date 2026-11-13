import {
  browser,
  button,
  check,
  click,
  dock,
  editorText,
  eventually,
  expectEqual,
  out,
  session,
  setEditorText,
  textOf,
  virtualSeconds,
} from "./helpers.mjs";

/** The dock's, the workspace's and the ST editor's rectangles, rounded: [left, top, right, bottom]. */
const rectangles = () =>
  browser.execute(() => {
    const [panel, workspace, editor] = [".dock", ".workspace", ".monaco-host"].map((selector) => {
      const r = document.querySelector(selector).getBoundingClientRect();
      return [r.left, r.top, r.right, r.bottom].map(Math.round);
    });
    return { dock: panel, workspace, editor };
  });

await session("debug", async () => {
  await check("the IDE gives the scene the centre and has no horizontal page overflow", async () => {
    await browser.$(".stage canvas").waitForExist({ timeout: 30000 });
    const layout = await browser.execute(() => {
      const left = document.querySelector(".left-sidebar").getBoundingClientRect();
      const stage = document.querySelector(".stage").getBoundingClientRect();
      const right = document.querySelector(".right-sidebar").getBoundingClientRect();
      const dockBounds = document.querySelector(".dock").getBoundingClientRect();
      return {
        overflow: document.documentElement.scrollWidth > innerWidth,
        ordered: left.right <= stage.left + 1 && stage.right <= right.left + 1,
        dockBelow: dockBounds.top >= stage.bottom - 1,
      };
    });
    expectEqual([layout.overflow, layout.ordered, layout.dockBelow], [false, true, true], "layout");
  });
  await check("catalogue previews are generated for placeable equipment", async () => {
    await browser.waitUntil(async () => (await browser.$$(".preview img")).length >= 5, { timeout: 30000 });
  });
  await browser.saveScreenshot(`${out}/workspace.png`);
  await click('.tree .item[title^="Belt A"]');
  await check("the inspector filters and edits a real project parameter", async () => {
    await browser.$('input[aria-label="Filter properties…"]').setValue("speed");
    expectEqual(await browser.$('input[aria-label="length"]').isExisting(), false, "filtered length");
    await browser.$('input[aria-label="speed"]').setValue("0.75");
    await browser.keys("Tab");
    await eventually(
      async () => expectEqual(await browser.$('input[aria-label="speed"]').getValue(), "0.75", "speed"),
      "edit",
      5000,
    );
    await browser.$('input[aria-label="Filter properties…"]').click();
    await browser.keys(["Control", "a"]);
    await browser.keys("Backspace");
    await click(button("Undo"));
    await eventually(
      async () => expectEqual(await browser.$('input[aria-label="speed"]').getValue(), "0.5", "undo speed"),
      "undo",
      5000,
    );
  });
  await check("numeric scrubbing is one undo step and supports fine keyboard increments", async () => {
    const handle = await browser.$('button[aria-label="Adjust speed"]');
    await browser
      .action("pointer")
      .move({ origin: handle })
      .down()
      .move({ origin: handle, x: 30, duration: 200 })
      .up()
      .perform();
    await eventually(
      async () => expectEqual(await browser.$('input[aria-label="speed"]').getValue(), "0.8", "scrubbed"),
      "scrub",
      5000,
    );
    await click(button("Undo"));
    await eventually(
      async () => expectEqual(await browser.$('input[aria-label="speed"]').getValue(), "0.5", "one undo"),
      "undo scrub",
      5000,
    );
    await browser.$('input[aria-label="speed"]').click();
    await browser.keys(["Shift", "ArrowUp"]);
    await eventually(
      async () => expectEqual(await browser.$('input[aria-label="speed"]').getValue(), "0.501", "fine step"),
      "fine step",
      5000,
    );
    await click(button("Undo"));
  });
  await check("locking prevents inspector edits and keyboard deletion", async () => {
    await click('button[aria-label="Lock editing Belt A"]');
    expectEqual(await browser.$('input[aria-label="speed"]').isEnabled(), false, "locked");
    await browser.keys("Delete");
    expectEqual(await browser.$('.tree .item[title^="Belt A"]').isExisting(), true, "locked object still present");
    await click('button[aria-label="Unlock editing Belt A"]');
  });
  await check("PLC sources open in the bottom dock", async () => {
    await dock("plc");
    expectEqual(await browser.$(".monaco-host .view-lines").isDisplayed(), true, "ST editor");
    expectEqual(await browser.$(".organizer").isDisplayed(), true, "organizer");
  });
  await browser.saveScreenshot(`${out}/plc.png`);
  await check("the dock maximizes over the workspace and restores its height", async () => {
    const before = await rectangles();
    await click(button("Maximize panel"));
    await eventually(
      async () => {
        const now = await rectangles();
        expectEqual(now.dock, now.workspace, "dock over the workspace");
        if (now.editor[2] - now.editor[0] <= before.editor[2] - before.editor[0]) throw new Error("editor not wider");
      },
      "maximized",
      5000,
    );
    expectEqual(await browser.$(button("Build")).isDisplayed(), true, "toolbar");
    expectEqual(await browser.$(".statusbar").isDisplayed(), true, "status bar");
    await browser.saveScreenshot(`${out}/plc-maximized.png`);
    await click(button("Restore panel"));
    await eventually(async () => expectEqual((await rectangles()).dock, before.dock, "restored"), "restored", 5000);
  });
  await click(button("Build"));
  await click(button("Run"));
  await click(button("Start"));
  await check("Watch shows measured runtime costs and simulation speed", async () => {
    await dock("watch");
    await eventually(
      async () => {
        const timing = await textOf(".timing");
        if (!timing.includes("µs") && !timing.includes(" ms")) throw new Error(timing);
        if (timing.includes("—")) throw new Error("no measured scan timing");
        const speed = await textOf(".actual");
        if (!/^~\d+\.\d+×$/.test(speed)) throw new Error(`no measured speed: ${speed}`);
      },
      "runtime telemetry",
      10000,
    );
  });
  await browser.saveScreenshot(`${out}/watch.png`);
  await check("a failed build opens Problems and the old running line can still be paused", async () => {
    const original = await editorText();
    await setEditorText("PROGRAM Main\nthis is invalid\nEND_PROGRAM");
    const before = await virtualSeconds("Virtual time");
    await click(button("Build"));
    await browser.$("#panel-problems .error-line").waitForDisplayed({ timeout: 10000 });
    expectEqual(await browser.$(button("Pause")).isEnabled(), true, "Pause after failed build");
    expectEqual(await browser.$(".statusbar .build-status").isDisplayed(), true, "build status remains visible");
    expectEqual(await textOf(".statusbar .build-status"), "Previous build loaded", "runtime build status");
    expectEqual(
      await browser.$(".statusbar .error-status").isDisplayed(),
      true,
      "error remains visible alongside build",
    );
    await browser.saveScreenshot(`${out}/failed-build.png`);
    await eventually(
      async () => {
        if ((await virtualSeconds("Virtual time")) <= before) throw new Error("old line stopped");
      },
      "old line runs",
      5000,
    );
    await click(button("Pause"));
    await setEditorText(original);
    await click(button("Build"));
  });
});
