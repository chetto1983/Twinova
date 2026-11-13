// End-to-end run of navigation, call templates and Auto Declare in the ST editor (spec
// 2026-11-13-completion-navigation-design.md) on the 4.4a line's program: F12, Shift+F12, F2, a call's parameters at
// its `(`, and an undeclared name declared from its quick fix.
//
//   npx tauri build --debug --no-bundle && node e2e/navigation.mjs

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
  sessions,
  textOf,
} from "./helpers.mjs";

const folder = fixtureCopy("two_belts_diverter");
const editor = (f, ...args) => browser.execute(f, ...args);

/** Puts the cursor on `line` and `column` and gives the editor the focus. */
const cursorAt = (line, column) =>
  editor(
    (l, c) => {
      const e = document.querySelector(".monaco-host").editor;
      e.setPosition({ lineNumber: l, column: c });
      e.revealLineInCenter(l);
      e.focus();
    },
    line,
    column,
  );
const cursor = () =>
  editor(() => {
    const p = document.querySelector(".monaco-host").editor.getPosition();
    return [p.lineNumber, p.column];
  });
/** Types `text` as the keyboard does: brackets close, and trigger characters open the suggestions. */
const type = (text) =>
  editor((t) => document.querySelector(".monaco-host").editor.trigger("keyboard", "type", { text: t }), text);
const lineText = (line) =>
  editor((l) => document.querySelector(".monaco-host").editor.getModel().getLineContent(l), line);
/** The line holding `needle`, from 1. */
const lineOf = async (needle) => (await editorText()).split("\n").findIndex((l) => l.includes(needle)) + 1;
/** A new line with `text` before END_PROGRAM, and the cursor at its end. */
async function newLine(text) {
  const end = await lineOf("END_PROGRAM");
  await editor(
    (l, t) => {
      const e = document.querySelector(".monaco-host").editor;
      e.executeEdits("e2e", [
        { range: { startLineNumber: l, startColumn: 1, endLineNumber: l, endColumn: 1 }, text: t },
      ]);
    },
    end,
    `${text}\n`,
  );
  await cursorAt(end, text.length + 1);
  return end;
}
const suggestion = (label) =>
  `(//div[contains(@class,"suggest-widget")]//div[contains(@class,"monaco-list-row")][.//span[normalize-space()="${label}"]])[1]`;
/** The Problems pane's analysis rows. */
const problems = () =>
  editor(() => [...document.querySelectorAll("#panel-problems .problem.analysis")].map((r) => r.innerText).join(" | "));
const built = () =>
  eventually(
    async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
    "built",
    15_000,
  );

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("Navigation, call templates and Auto Declare");
    await built();
    await dock("plc");
    await click(button("Maximize panel"));
    await click(button("Variables"));
    const count = await lineOf("count := count + 1");
    const declared = await lineOf("count : DINT;");

    await check("F12 on count goes to its declaration", async () => {
      await cursorAt(count, 5);
      await browser.keys(["F12"]);
      await eventually(async () => expectEqual(await cursor(), [declared, 5], "cursor"), "definition", 5_000);
    });

    await check("Shift+F12 lists count's four references", async () => {
      await cursorAt(count, 5);
      await browser.keys(["Shift", "F12"]);
      await eventually(
        async () => {
          const title = await textOf(".peekview-title");
          if (!/4/.test(title)) throw new Error(title);
        },
        "references",
        5_000,
      );
      await browser.saveScreenshot(`${out}/navigation-references.png`);
      await browser.keys(["Escape"]);
    });

    await check("F2 renames count everywhere, and the line builds", async () => {
      await cursorAt(count, 5);
      await browser.keys(["F2"]);
      const input = await browser.$(".rename-box input");
      await input.waitForExist({ timeout: 5_000 });
      await editor(() => {
        const i = document.querySelector(".rename-box input");
        i.value = "cartons";
        i.dispatchEvent(new Event("input", { bubbles: true }));
      });
      await browser.keys(["Enter"]);
      await eventually(
        async () => expectEqual(await lineText(count), "    cartons := cartons + 1;", "renamed"),
        "renamed",
        5_000,
      );
      // The comment's "their count" is prose, not the variable: a rename leaves it.
      expectEqual(await lineText(declared), "    cartons : DINT;", "the declaration");
      expectEqual((await lineText(count + 2)).includes("cartons MOD 3"), true, "the read on line 42");
      await click(button("Build"));
      await built();
    });

    await check("typing carry( offers its parameters, and Enter writes them all", async () => {
      const at = await newLine("carry");
      await type("(");
      await (await browser.$(suggestion("All parameters"))).waitForExist({ timeout: 10_000 });
      await browser.keys(["Enter"]);
      await eventually(
        async () => expectEqual(await lineText(at), "carry(IN := , PT := , Q => , ET => )", "template"),
        "template",
        5_000,
      );
      expectEqual(await cursor(), [at, 13], "the cursor on IN's value");
    });

    await check("the mandatory parameters of an FB are none", async () => {
      const at = await newLine("carry");
      await type("(");
      await (await browser.$(suggestion("Mandatory parameters"))).waitForExist({ timeout: 10_000 });
      await browser.keys(["ArrowDown", "Enter"]);
      await eventually(async () => expectEqual(await lineText(at), "carry()", "template"), "mandatory", 5_000);
    });

    await check("an undeclared speed is declared from its quick fix as INT, and the line builds", async () => {
      for (const needle of ["carry(IN := , PT", "carry()"]) {
        const l = await lineOf(needle);
        await editor(
          (n) =>
            document
              .querySelector(".monaco-host")
              .editor.executeEdits("e2e", [
                { range: { startLineNumber: n, startColumn: 1, endLineNumber: n + 1, endColumn: 1 }, text: "" },
              ]),
          l,
        );
      }
      const at = await newLine("speed := 3;");
      await dock("problems");
      await eventually(
        async () => {
          if (!(await problems()).includes("undefined identifier 'speed'")) throw new Error(await problems());
        },
        "E101",
        10_000,
      );
      await dock("plc");
      await cursorAt(at, 1);
      await browser.keys(["Control", "."]);
      await (
        await browser.$(`//*[contains(@class,"action-widget")]//*[contains(text(),"Declare speed")]`)
      ).waitForExist({
        timeout: 10_000,
      });
      await browser.keys(["Enter"]);
      const typeField = `//div[contains(@class,"declare-bar")]//input[@aria-label="Type"]`;
      await (await browser.$(typeField)).waitForExist({ timeout: 5_000 });
      expectEqual(await (await browser.$(typeField)).getValue(), "INT", "proposed type");
      await browser.saveScreenshot(`${out}/navigation-declare.png`);
      await click(`//div[contains(@class,"declare-bar")]//button[normalize-space()="Apply"]`);
      await eventually(
        async () => expectEqual((await editorText()).includes("speed : INT;"), true, "declared"),
        "declared",
        5_000,
      );
      await click(button("Build"));
      await built();
    });
  }),
);
