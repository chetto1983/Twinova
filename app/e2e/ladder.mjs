// End-to-end run of the Ladder editor (spec 2026-11-13-ladder-editor-design.md, test plan; plan Task 8): the 4.4a line
// with its program as Main.ld, drawn; a start/stop rung with a seal-in drawn with the toolbox and the keys, its
// variables declared by Auto Declare; built and run with Monitoring on, Start written from the drawing energises the
// coil and the wires up to it, the rung holds itself once Start is let go, Stop drops it, and a force from the drawing
// marks its value. Text goes in through the DOM's values (helpers' note: WebKitWebDriver drops typed spaces).
//
//   npx tauri build --debug --no-bundle && node e2e/ladder.mjs

import {
  app,
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
  BAR,
  sessions,
  setInput,
  textOf,
} from "./helpers.mjs";

const folder = fixtureCopy("ladder_line");
const RUNG = 13;
const SEAL_IN = "[|start_pb|, |motor|] |/stop_pb| (motor);";

const built = () =>
  eventually(
    async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
    "built",
    30_000,
  );

/** Clicks the toolbox's button `title` once it is enabled: for a moment after an edit the drawing is read again, and
 *  its buttons are disabled, as a person sees them. */
const tool = async (title) => {
  const found = await browser.$(`[aria-label="Ladder elements"] button[title^="${title}"]`);
  await found.waitForEnabled({ timeout: 10_000 });
  await found.click();
};

/** Types `text` as the element just placed: once it shows selected (the insert is printed and read back in Rust),
 *  Enter opens its field, and Enter takes it. */
const operand = async (text) => {
  await browser.$(`[data-rung="${RUNG}"] rect.picked`).waitForExist({ timeout: 10_000 });
  await browser.keys("Enter");
  await browser.$(".operand input").waitForExist({ timeout: 5_000 });
  await setInput('//div[contains(@class,"operand")]//input', text, "input");
  await browser.keys("Enter");
  await browser.pause(400);
};

/** Takes Auto Declare's offer for `name`. */
const declare = async (name) => {
  await click(button(`Declare ${name} : BOOL`));
  await browser.pause(400);
};

/** The new rung's values, in drawing order: start_pb, motor (the seal-in), stop_pb, the coil motor. */
const readings = () =>
  browser.execute(
    (r) => [...document.querySelectorAll(`[data-rung="${r}"] .reading text`)].map((t) => t.textContent),
    RUNG,
  );

/** Whether the new rung's coil is drawn energised. */
const coilOn = () => browser.execute((r) => !!document.querySelector(`[data-rung="${r}"] .coil .symbol.on`), RUNG);

/** Writes, or forces, `text` into the reading at `index`, through the value bar a click on it opens. */
const prepare = async (index, text, action) => {
  await browser.execute(
    (r, i) => {
      const value = document.querySelectorAll(`[data-rung="${r}"] .reading text`).item(i);
      value.dispatchEvent(new MouseEvent("click", { bubbles: true }));
    },
    RUNG,
    index,
  );
  await setInput(`${BAR}//input`, text, "input");
  await click(`${BAR}//button[normalize-space()="${action}"]`);
};

/** A screenshot of the new rung, scrolled into view. */
const shot = async (name) => {
  await browser.execute((r) => document.querySelector(`[data-rung="${r}"]`)?.scrollIntoView({ block: "center" }), RUNG);
  await browser.pause(300);
  await browser.saveScreenshot(`${out}/${name}.png`);
};

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("The Ladder editor");
    await check("the 4.4a line's Main.ld is drawn, rung by rung", async () => {
      await built();
      await dock("plc");
      await click(button("Maximize panel"));
      await eventually(
        async () =>
          expectEqual(
            await browser.execute(() => document.querySelectorAll("[role=application] section.rung").length),
            13,
            "rungs",
          ),
        "drawn",
        10_000,
      );
    });

    await check("a start/stop rung is drawn with the toolbox and the keys, its variables declared", async () => {
      await click(`[data-rung="${RUNG - 1}"] header`);
      await tool("New rung");
      await tool("Coil");
      await operand("motor");
      await declare("motor");
      await click(`[data-rung="${RUNG}"] header`);
      await tool("Contact");
      await operand("stop_pb");
      await declare("stop_pb");
      await tool("Negated");
      await click(`[data-rung="${RUNG}"] header`);
      await tool("Parallel branch");
      await tool("Contact");
      await operand("motor");
      // a branch path is picked on its wire, a line WebDriver will not click (no height): the click goes to it as the
      // mouse's would, and the view reads its data-path
      await browser.execute(
        (r) =>
          document
            .querySelector(`[data-rung="${r}"] line.leg[data-path="0.0"]`)
            .dispatchEvent(new MouseEvent("click", { bubbles: true })),
        RUNG,
      );
      await tool("Contact");
      await operand("start_pb");
      await declare("start_pb");
      await eventually(
        async () => {
          const text = await editorText();
          for (const wanted of [SEAL_IN, "start_pb : BOOL", "stop_pb : BOOL", "motor : BOOL"])
            if (!text.includes(wanted)) throw new Error(`no "${wanted}" in:\n${text}`);
        },
        "the file",
        10_000,
      );
      await shot("ladder-drawn");
    });

    await check("built and run, Start written from the drawing energises the coil and its wires", async () => {
      await click(button("Build"));
      await built();
      await click(button("Run"));
      await click(button("Monitoring (Ctrl+T)"));
      await eventually(
        async () => expectEqual(await readings(), ["FALSE", "FALSE", "FALSE", "FALSE"], "off"),
        "read",
        10_000,
      );
      await prepare(0, "TRUE", "Write");
      await eventually(
        async () => {
          expectEqual(await readings(), ["TRUE", "TRUE", "FALSE", "TRUE"], "started");
          expectEqual(await coilOn(), true, "the coil energised");
        },
        "started",
        10_000,
      );
      await shot("ladder-started");
    });

    await check("Start let go, the rung holds itself through the seal-in", async () => {
      await prepare(0, "FALSE", "Write");
      await browser.pause(500);
      expectEqual(await readings(), ["FALSE", "TRUE", "FALSE", "TRUE"], "held");
      expectEqual(await coilOn(), true, "the coil still energised");
    });

    await check("Stop drops the coil", async () => {
      await prepare(2, "TRUE", "Write");
      await eventually(
        async () => {
          expectEqual(await readings(), ["FALSE", "FALSE", "TRUE", "FALSE"], "stopped");
          expectEqual(await coilOn(), false, "the coil dropped");
        },
        "stopped",
        10_000,
      );
      await prepare(2, "FALSE", "Write");
    });

    await check("a force from the drawing starts it and marks its value", async () => {
      await prepare(0, "TRUE", "Force");
      await eventually(
        async () => {
          expectEqual(await coilOn(), true, "forced on");
          const forced = await browser.execute(
            (r) => document.querySelectorAll(`[data-rung="${r}"] .reading.forced`).length,
            RUNG,
          );
          expectEqual(forced, 1, "start_pb marked forced");
        },
        "forced",
        10_000,
      );
      await shot("ladder-forced");
      await click(`${BAR}//button[normalize-space()="Release"]`);
    });
  }),
);
