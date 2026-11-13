// End-to-end run of the PLC pane in a window of its own (spec 2026-11-13-plc-window-design.md), on the 4.4a line:
// opened from the dock, edited, built, run, stepped, monitored and forced from the window, and brought back. WebDriver
// reaches each window by its handle (measured on WebKitWebDriver). The window is closed through the app: WebDriver's
// closeWindow and the page's window.close() are not a user's close and fire no Destroyed (decision 8). Closing the main
// window is therefore measured once, with a window manager's close, in the verification.
//
//   npx tauri build --debug --no-bundle && node e2e/plc_window.mjs

import {
  BAR,
  app,
  browser,
  button,
  check,
  click,
  dock,
  drawn,
  editorText,
  eventually,
  expectEqual,
  fixtureCopy,
  out,
  pickRow,
  selectSpeed,
  sessions,
  setInput,
  textOf,
  virtualSeconds,
} from "./helpers.mjs";

const folder = fixtureCopy("two_belts_diverter");
const EDIT = "(* edited in the PLC window *)";

const built = () =>
  eventually(
    async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
    "built",
    15_000,
  );

/** The handle of the window that is not `main`, once there are two. */
const otherWindow = (main) =>
  eventually(
    async () => {
      const handles = await browser.getWindowHandles();
      const other = handles.find((h) => h !== main);
      if (handles.length !== 2 || !other) throw new Error(`${handles.length} windows`);
      return other;
    },
    "the PLC window",
    15_000,
  );

const plcText = () => browser.execute(() => document.querySelector(".monaco-host")?.editor?.getValue() ?? null);

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("The PLC pane in a window of its own");
    await built();
    const main = await browser.getWindowHandle();
    let plc;
    await check("the dock opens the PLC in a window and says where it went", async () => {
      await dock("plc");
      await click(button("Open the PLC in a window"));
      plc = await otherWindow(main);
      expectEqual(await textOf("#panel-plc strong"), "The PLC is in its own window.", "dock placeholder");
      expectEqual(await browser.$("#panel-plc .monaco-host").isExisting(), false, "one editor at a time");
      await browser.switchToWindow(plc);
      await eventually(
        async () => {
          if (!(await plcText())?.includes("belt_b_run := TRUE;")) throw new Error("no program in the window");
        },
        "program shown",
        15_000,
      );
      await browser.saveScreenshot(`${out}/plc-window.png`);
    });
    await check("an edit in the window is built by its own Build", async () => {
      await browser.switchToWindow(plc);
      await browser.execute((edit) => {
        const editor = document.querySelector(".monaco-host").editor;
        editor.setValue(editor.getValue().replace("belt_b_run := TRUE;", `belt_b_run := TRUE; ${edit}`));
      }, EDIT);
      await eventually(
        async () => {
          if (!(await browser.$(".tab.edited").isExisting())) throw new Error("the tab is not marked edited");
        },
        "edited",
        5_000,
      );
      await click(button("Build"));
      await eventually(
        async () => {
          if (await browser.$(".tab.edited").isExisting()) throw new Error("still marked edited");
        },
        "built from the window",
        15_000,
      );
      await browser.switchToWindow(main);
      await built();
    });
    await check("Run, Pause and Step from the window drive the line", async () => {
      await browser.switchToWindow(main);
      await selectSpeed("Max");
      const before = await virtualSeconds("Virtual time");
      await browser.switchToWindow(plc);
      await click(button("Run"));
      await eventually(
        async () => expectEqual(await browser.$(button("Pause")).isExisting(), true, "running"),
        "run",
        5_000,
      );
      await browser.pause(1_000);
      await click(button("Pause"));
      await eventually(
        async () => expectEqual(await browser.$(button("Run")).isExisting(), true, "paused"),
        "pause",
        5_000,
      );
      await browser.switchToWindow(main);
      const ran = await virtualSeconds("Virtual time");
      if (ran <= before) throw new Error(`virtual time ${ran} after ${before}`);
      await browser.switchToWindow(plc);
      await click(button("Step"));
      await browser.switchToWindow(main);
      await eventually(
        async () => {
          if ((await virtualSeconds("Virtual time")) <= ran) throw new Error("no step");
        },
        "step",
        5_000,
      );
    });
    await check("Monitoring in the window draws the values while the line runs", async () => {
      await browser.switchToWindow(plc);
      await click(button("Monitoring (Ctrl+T)"));
      await browser.execute(() => document.querySelector(".monaco-host").editor.revealLineInCenter(40));
      await click(button("Run"));
      await eventually(
        async () => {
          const values = await drawn();
          if (!values.some((v) => /^40 monitor-value \d+$/.test(v))) throw new Error(values.join(" | "));
        },
        "drawn",
        10_000,
      );
      await click(button("Pause"));
      await browser.saveScreenshot(`${out}/plc-window-monitor.png`);
    });
    await check("a force from the window is listed in the main window's Watch", async () => {
      await browser.switchToWindow(plc);
      await pickRow("belt_b_run");
      await setInput(`${BAR}//input`, "FALSE", "input");
      await click(`${BAR}//button[normalize-space()="Force"]`);
      await browser.switchToWindow(main);
      await dock("watch");
      await eventually(
        async () => {
          const listed = await textOf(".forces li");
          if (!listed.startsWith("Main.belt_b_run = FALSE")) throw new Error(`forces: ${listed}`);
        },
        "listed",
        5_000,
      );
      await click(`//ul[contains(@class,"forces")]//button[normalize-space()="Release"]`);
    });
    await check("back to the dock: one window, and the dock's editor holds the edit", async () => {
      await browser.switchToWindow(plc);
      // The click closes the window it was sent to, so it is made from the page after the call has returned: a
      // WebDriver click would answer "no such window" (measured).
      await browser.execute(() =>
        setTimeout(() =>
          [...document.querySelectorAll("button")].find((b) => b.textContent.trim() === "Back to the dock").click(),
        ),
      );
      await browser.switchToWindow(main);
      await eventually(
        async () => expectEqual((await browser.getWindowHandles()).length, 1, "windows"),
        "closed",
        10_000,
      );
      await eventually(
        async () => {
          if (!(await editorText()).includes(EDIT)) throw new Error("the edit is not in the dock");
        },
        "back",
        10_000,
      );
    });
  }),
);
