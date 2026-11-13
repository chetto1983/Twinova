// End-to-end run of the in-app ST editor (spec 2026-11-12-st-editor-design.md, acceptance 2 and 3): Monaco in the PLC
// pane, with truST's diagnostics before any Build, a Problems row that opens its line, hover and completion. The text is
// set through the editor (helpers `setEditorText`): WebKitWebDriver drops typed spaces.
//
//   npx tauri build --debug --no-bundle && node e2e/editor_st.mjs

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
  setEditorText,
  textOf,
} from "./helpers.mjs";

const folder = fixtureCopy("two_belts_diverter");
const rows = async () => {
  await dock("problems");
  return browser.execute(() =>
    [...document.querySelectorAll("#panel-problems .problem.analysis")].map((r) => r.innerText.replace(/\s+/g, " ")),
  );
};
/** The PLC editor's model decorations of a class: Monaco draws a squiggle only for the lines in view, so the test reads
 *  the decorations instead of the DOM. */
const decorated = (cls) =>
  browser.execute(
    (c) =>
      document
        .querySelector(".monaco-host")
        .editor.getModel()
        .getAllDecorations()
        .filter((d) => String(d.options.className).includes(c))
        .map((d) => d.range.startLineNumber),
    cls,
  );
const cursorLine = () => browser.execute(() => document.querySelector(".monaco-host").editor.getPosition().lineNumber);
/** Puts the cursor on the first line holding `text`, `offset` characters into `text`, in view, and runs editor `action`. */
const actAt = (text, offset, action) =>
  browser.execute(
    (t, o, a) => {
      const editor = document.querySelector(".monaco-host").editor;
      const model = editor.getModel();
      const line = model.getLinesContent().findIndex((l) => l.includes(t)) + 1;
      editor.setPosition({ lineNumber: line, column: model.getLineContent(line).indexOf(t) + 1 + o });
      // Monaco shows a hover only at a position in view.
      editor.revealLineInCenter(line);
      editor.focus();
      editor.trigger("e2e", a, {});
      return line;
    },
    text,
    offset,
    action,
  );

sessions(() =>
  app("debug", [`--project=${folder}`], async () => {
    console.log("The in-app ST editor: Monaco with truST's analysis");
    let original = "";
    await check("the program opens in Monaco, and its problems are VS Code's without the false E202", async () => {
      await dock("plc");
      await browser.$(".monaco-host .view-lines").waitForDisplayed({ timeout: 15_000 });
      original = await editorText();
      expectEqual(original.startsWith("(* The 4.4a line"), true, "the program's text");
      await eventually(
        async () => {
          const found = await rows();
          if (!found.some((r) => r.includes("plc/Main.st:14:5") && r.includes("W001")))
            throw new Error(found.join(" | "));
          if (found.some((r) => / E202 | W01[01] /.test(` ${r} `))) throw new Error(`hidden ones shown: ${found}`);
        },
        "analysed",
        15_000,
      );
    });
    await check("a broken program shows its errors as markers and in Problems before any Build", async () => {
      await setEditorText(
        original.replace("belt_b_run := TRUE;", "belt_b_run := 5;\nnot_declared := 1;\ncarry(IN := popup, PT := 15);"),
      );
      await eventually(
        async () => {
          const errors = (await rows()).filter((r) => /E2\d\d|E1\d\d/.test(r)).map((r) => r.split(" ")[0]);
          expectEqual(errors, ["plc/Main.st:37:1", "plc/Main.st:38:1", "plc/Main.st:39:19"], "errors");
        },
        "errors listed",
        15_000,
      );
      await dock("plc");
      await eventually(
        async () => expectEqual(await decorated("squiggly-error"), [37, 38, 39], "error squiggles"),
        "squiggles",
        10_000,
      );
      await browser.saveScreenshot(`${out}/editor-st-errors.png`);
    });
    await check("a Problems row opens its line", async () => {
      await dock("problems");
      await click(`//button[contains(@class,"analysis")][contains(., "plc/Main.st:38:1")]`);
      await eventually(async () => expectEqual(await cursorLine(), 38, "line"), "revealed", 5_000);
    });
    await check("hover tells a variable's type", async () => {
      await dock("plc");
      const at = await actAt("arrived(CLK := detected_a", 16, "editor.action.showHover");
      if (!(at > 0)) throw new Error("no line with detected_a");
      await eventually(
        async () => {
          const shown = await textOf(".monaco-hover-content");
          if (!shown.includes("BOOL")) throw new Error(shown);
        },
        "hover",
        10_000,
      );
    });
    await check("completion offers the declared variables", async () => {
      // A line no other holds: `detec` alone would first match the declaration of `detected_a`.
      const typed = "count := detec";
      await setEditorText(original.replace("belt_b_run := TRUE;", `belt_b_run := TRUE;\n${typed}`));
      await dock("plc");
      expectEqual(await actAt(typed, typed.length, "editor.action.triggerSuggest"), 38, "the typed line");
      await eventually(
        async () => {
          const offered = await textOf(".suggest-widget");
          if (!offered.includes("detected_a") || !offered.includes("detected_b")) throw new Error(offered);
        },
        "suggestions",
        10_000,
      );
      await browser.saveScreenshot(`${out}/editor-st-completion.png`);
    });
    await check("the restored program builds, and the blocks show no E202", async () => {
      await setEditorText(original);
      await click(button("Build"));
      await eventually(
        async () => expectEqual(await textOf(".statusbar .build-status"), "Built: ready to run", "built"),
        "built",
        15_000,
      );
      const found = await rows();
      expectEqual(
        found.filter((r) => r.startsWith("components/") && r.includes("E202")),
        [],
        "no false E202 in the blocks",
      );
    });
  }),
);
